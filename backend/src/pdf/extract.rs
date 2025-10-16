//! Extract a minimal intermediate representation for page 0.

use std::collections::HashMap;

use anyhow::{anyhow, Context, Result};
use lopdf::{Dictionary, Document, Object, ObjectId};

use crate::pdf::content::{tokenize_stream, Operand, Operator};
use crate::types::{
    DocumentIR, FontInfo, ImageObject, PageIR, PageObject, StreamSpan, TextObject, TextSpans,
};

#[derive(Debug, Clone)]
pub struct PageStream {
    pub object_id: ObjectId,
    pub data: Vec<u8>,
}

pub fn extract_ir(doc: &Document) -> Result<DocumentIR> {
    let (page_number, page_id) = doc
        .get_pages()
        .into_iter()
        .next()
        .ok_or_else(|| anyhow!("document contains no pages"))?;

    let page_dict = doc.get_dictionary(page_id)?;
    let (width_pt, height_pt) = page_dimensions(doc, page_dict)?;

    let streams = page_streams(doc, page_id)?;
    let resources = resolve_resources(doc, page_dict).unwrap_or_else(|_| Dictionary::new());
    let image_names = collect_image_xobjects(doc, &resources);

    let mut objects = Vec::new();
    let mut text_counter = 0usize;
    let mut image_counter = 0usize;

    for stream in &streams {
        let tokens =
            tokenize_stream(&stream.data).with_context(|| "failed to tokenise content stream")?;
        let mut state = GraphicsState::new(stream.object_id.0);

        for token in tokens {
            match token.operator {
                Operator::q => state.push(),
                Operator::Q => state.pop(),
                Operator::Cm => state.apply_cm(&token)?,
                Operator::Bt => state.begin_text(&token),
                Operator::Et => {
                    if let Some(text_object) = state.end_text(&token, &mut text_counter) {
                        objects.push(PageObject::Text(text_object));
                    }
                }
                Operator::Tf => state.set_font(&token),
                Operator::Tm => state.set_tm(&token)?,
                Operator::Td => state.translate_text(&token)?,
                Operator::TStar => state.next_line(),
                Operator::Tj | Operator::TJ => state.consume_text(&token),
                Operator::Do => {
                    if let Some(obj) = state.consume_image(&token, &image_names, &mut image_counter)
                    {
                        objects.push(PageObject::Image(obj));
                    }
                }
                _ => {}
            }
        }

        if let Some(text_object) = state.flush_text_at_stream_end(&mut text_counter) {
            objects.push(PageObject::Text(text_object));
        }
    }

    let page = PageIR {
        index: page_number as usize - 1,
        width_pt,
        height_pt,
        objects,
    };

    Ok(DocumentIR { pages: vec![page] })
}

pub fn page_streams(doc: &Document, page_id: ObjectId) -> Result<Vec<PageStream>> {
    let page_dict = doc.get_dictionary(page_id)?;
    let contents = page_dict
        .get(b"Contents")
        .ok_or_else(|| anyhow!("page missing Contents"))?;
    resolve_content_streams(doc, contents)
}

fn resolve_content_streams(doc: &Document, object: &Object) -> Result<Vec<PageStream>> {
    match object {
        Object::Reference(id) => vec![load_stream(doc, *id)?].pipe(Ok),
        Object::Array(entries) => {
            let mut streams = Vec::new();
            for entry in entries {
                match entry {
                    Object::Reference(id) => streams.push(load_stream(doc, *id)?),
                    _ => return Err(anyhow!("unsupported inline stream in Contents")),
                }
            }
            Ok(streams)
        }
        _ => Err(anyhow!("unsupported Contents object type")),
    }
}

fn load_stream(doc: &Document, id: ObjectId) -> Result<PageStream> {
    let stream = doc
        .get_object(id)?
        .as_stream()
        .map_err(|_| anyhow!("expected stream object"))?;
    let data = stream.decompressed_content()?;
    Ok(PageStream {
        object_id: id,
        data,
    })
}

fn page_dimensions(doc: &Document, page_dict: &Dictionary) -> Result<(f64, f64)> {
    if let Ok(object) = page_dict.get(b"MediaBox") {
        if let Ok(mediabox) = resolve_dictionary_value(doc, Some(object)) {
            if let Object::Array(values) = mediabox {
                if values.len() == 4 {
                    let llx = to_number(&values[0])?;
                    let lly = to_number(&values[1])?;
                    let urx = to_number(&values[2])?;
                    let ury = to_number(&values[3])?;
                    return Ok((urx - llx, ury - lly));
                }
            }
        }
    }
    Ok((595.0, 842.0))
}

fn resolve_resources(doc: &Document, page_dict: &Dictionary) -> Result<Dictionary> {
    let resources_obj = page_dict.get(b"Resources")?;
    match resolve_dictionary_value(doc, Some(resources_obj))? {
        Object::Dictionary(dict) => Ok(dict),
        _ => Err(anyhow!("Resources is not a dictionary")),
    }
}

fn resolve_dictionary_value(doc: &Document, object: Option<&Object>) -> Result<Object> {
    let Some(obj) = object else {
        return Err(anyhow!("missing object"));
    };
    match obj {
        Object::Reference(id) => Ok(doc.get_object(*id)?.clone()),
        other => Ok(other.clone()),
    }
}

fn to_number(object: &Object) -> Result<f64> {
    match object {
        Object::Integer(value) => Ok(*value as f64),
        Object::Real(value) => Ok(*value),
        _ => Err(anyhow!("expected numeric value")),
    }
}

fn collect_image_xobjects(doc: &Document, resources: &Dictionary) -> HashMap<String, ObjectId> {
    let mut map = HashMap::new();
    if let Ok(xobjects_obj) = resources.get(b"XObject") {
        if let Ok(Object::Dictionary(dict)) = resolve_dictionary_value(doc, Some(xobjects_obj)) {
            for (name, value) in dict.iter() {
                if let Object::Reference(id) = value {
                    if let Ok(dict_ref) = doc.get_object(*id).and_then(Object::as_dict) {
                        let xobj_dict = dict_ref.clone();
                        if let Ok(subtype) = xobj_dict.get(b"Subtype").and_then(Object::as_name_str)
                        {
                            if subtype == "Image" {
                                map.insert(String::from_utf8_lossy(name).into_owned(), *id);
                            }
                        }
                    }
                }
            }
        }
    }
    map
}

#[derive(Clone)]
struct GraphicsState {
    stream_obj: u32,
    ctm: [f64; 6],
    stack: Vec<StackEntry>,
    last_cm: Option<(RangeHolder, [f64; 6])>,
    text: Option<TextState>,
}

#[derive(Clone)]
struct StackEntry {
    ctm: [f64; 6],
    last_cm: Option<(RangeHolder, [f64; 6])>,
}

#[derive(Clone)]
struct TextState {
    bt_span: RangeHolder,
    tm_span: Option<RangeHolder>,
    tm: [f64; 6],
    tlm: [f64; 6],
    font: Option<FontInfo>,
    run: Option<TextRunBuilder>,
}

#[derive(Clone)]
struct TextRunBuilder {
    stream_obj: u32,
    block_start: RangeHolder,
    bt_op_span: RangeHolder,
    tm_span: Option<RangeHolder>,
    tm: [f64; 6],
    font: FontInfo,
    glyph_count: usize,
    last_end: usize,
}

#[derive(Clone)]
struct RangeHolder {
    start: usize,
    end: usize,
}

impl RangeHolder {
    fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    fn to_span(&self, stream_obj: u32) -> StreamSpan {
        StreamSpan {
            start: self.start as u32,
            end: self.end as u32,
            stream_obj,
        }
    }
}

impl GraphicsState {
    fn new(stream_obj: u32) -> Self {
        Self {
            stream_obj,
            ctm: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
            stack: Vec::new(),
            last_cm: None,
            text: None,
        }
    }

    fn push(&mut self) {
        self.stack.push(StackEntry {
            ctm: self.ctm,
            last_cm: self.last_cm.clone(),
        });
    }

    fn pop(&mut self) {
        if let Some(entry) = self.stack.pop() {
            self.ctm = entry.ctm;
            self.last_cm = entry.last_cm;
        }
    }

    fn apply_cm(&mut self, token: &crate::pdf::content::Token) -> Result<()> {
        let matrix = parse_six_numbers(&token.operands)?;
        self.ctm = multiply(self.ctm, matrix);
        self.last_cm = Some((RangeHolder::new(token.span.start, token.span.end), matrix));
        Ok(())
    }

    fn begin_text(&mut self, token: &crate::pdf::content::Token) {
        let holder = RangeHolder::new(token.span.start, token.span.end);
        self.text = Some(TextState {
            bt_span: holder.clone(),
            tm_span: None,
            tm: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
            tlm: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
            font: None,
            run: None,
        });
    }

    fn end_text(
        &mut self,
        token: &crate::pdf::content::Token,
        counter: &mut usize,
    ) -> Option<TextObject> {
        if let Some(mut text_state) = self.text.take() {
            if let Some(run) = text_state.run.take() {
                let block_span = StreamSpan {
                    start: run.block_start.start as u32,
                    end: token.span.end as u32,
                    stream_obj: run.stream_obj,
                };
                let spans = TextSpans {
                    block: block_span,
                    bt_op: run.bt_op_span.to_span(run.stream_obj),
                    tm_op: run
                        .tm_span
                        .as_ref()
                        .map(|holder| holder.to_span(run.stream_obj)),
                };
                let bbox = text_bbox(&run.tm, run.font.size, run.glyph_count);
                let text_object = TextObject {
                    id: format!("t:{}", *counter),
                    tm: run.tm,
                    font: run.font,
                    spans,
                    bbox,
                };
                *counter += 1;
                return Some(text_object);
            }
        }
        None
    }

    fn flush_text_at_stream_end(&mut self, counter: &mut usize) -> Option<TextObject> {
        if let Some(text_state) = self.text.take() {
            if let Some(run) = text_state.run {
                let spans = TextSpans {
                    block: StreamSpan {
                        start: run.block_start.start as u32,
                        end: run.last_end as u32,
                        stream_obj: run.stream_obj,
                    },
                    bt_op: run.bt_op_span.to_span(run.stream_obj),
                    tm_op: run
                        .tm_span
                        .as_ref()
                        .map(|holder| holder.to_span(run.stream_obj)),
                };
                let bbox = text_bbox(&run.tm, run.font.size, run.glyph_count);
                let text_object = TextObject {
                    id: format!("t:{}", *counter),
                    tm: run.tm,
                    font: run.font,
                    spans,
                    bbox,
                };
                *counter += 1;
                return Some(text_object);
            }
        }
        None
    }

    fn set_font(&mut self, token: &crate::pdf::content::Token) {
        if let Some(text_state) = &mut self.text {
            if token.operands.len() == 2 {
                if let Operand::Name(name) = &token.operands[0] {
                    if let Operand::Number(size) = token.operands[1] {
                        text_state.font = Some(FontInfo {
                            res_name: name.clone(),
                            size,
                        });
                    }
                }
            }
        }
    }

    fn set_tm(&mut self, token: &crate::pdf::content::Token) -> Result<()> {
        if let Some(text_state) = &mut self.text {
            let matrix = parse_six_numbers(&token.operands)?;
            text_state.tm = matrix;
            text_state.tlm = matrix;
            text_state.tm_span = Some(RangeHolder::new(token.span.start, token.span.end));
        }
        Ok(())
    }

    fn translate_text(&mut self, token: &crate::pdf::content::Token) -> Result<()> {
        if let Some(text_state) = &mut self.text {
            if token.operands.len() >= 2 {
                if let (Operand::Number(tx), Operand::Number(ty)) =
                    (&token.operands[0], &token.operands[1])
                {
                    let translation = [1.0, 0.0, 0.0, 1.0, *tx, *ty];
                    text_state.tlm = multiply(text_state.tlm, translation);
                    text_state.tm = text_state.tlm;
                    text_state.tm_span = None;
                }
            }
        }
        Ok(())
    }

    fn next_line(&mut self) {
        if let Some(text_state) = &mut self.text {
            let size = text_state
                .font
                .as_ref()
                .map(|font| font.size)
                .unwrap_or(12.0);
            let translation = [1.0, 0.0, 0.0, 1.0, 0.0, -size];
            text_state.tlm = multiply(text_state.tlm, translation);
            text_state.tm = text_state.tlm;
            text_state.tm_span = None;
        }
    }

    fn consume_text(&mut self, token: &crate::pdf::content::Token) {
        if let Some(text_state) = &mut self.text {
            let font = text_state.font.clone().unwrap_or(FontInfo {
                res_name: String::from("F1"),
                size: 12.0,
            });
            if text_state.run.is_none() {
                let builder = TextRunBuilder {
                    stream_obj: self.stream_obj,
                    block_start: text_state.bt_span.clone(),
                    bt_op_span: text_state.bt_span.clone(),
                    tm_span: text_state.tm_span.clone(),
                    tm: text_state.tm,
                    font,
                    glyph_count: 0,
                    last_end: token.span.end,
                };
                text_state.run = Some(builder);
            }
            if let Some(run) = &mut text_state.run {
                run.glyph_count += glyph_count(&token.operands);
                run.last_end = token.span.end;
            }
        }
    }

    fn consume_image(
        &mut self,
        token: &crate::pdf::content::Token,
        image_names: &HashMap<String, ObjectId>,
        counter: &mut usize,
    ) -> Option<ImageObject> {
        let Operand::Name(name) = token.operands.get(0)? else {
            return None;
        };
        if !image_names.contains_key(name) {
            return None;
        }
        let (span_holder, matrix) = self.last_cm.clone()?;
        let bbox = image_bbox(&matrix);
        let image = ImageObject {
            id: format!("img:{}", *counter),
            x_object: name.clone(),
            cm: matrix,
            span: span_holder.to_span(self.stream_obj),
            bbox,
        };
        *counter += 1;
        Some(image)
    }
}

fn parse_six_numbers(operands: &[Operand]) -> Result<[f64; 6]> {
    if operands.len() < 6 {
        return Err(anyhow!("expected six numeric operands"));
    }
    let mut values = [0.0; 6];
    for (idx, operand) in operands.iter().take(6).enumerate() {
        match operand {
            Operand::Number(value) => values[idx] = *value,
            _ => return Err(anyhow!("matrix operand is not numeric")),
        }
    }
    Ok(values)
}

fn multiply(a: [f64; 6], b: [f64; 6]) -> [f64; 6] {
    [
        a[0] * b[0] + a[2] * b[1],
        a[1] * b[0] + a[3] * b[1],
        a[0] * b[2] + a[2] * b[3],
        a[1] * b[2] + a[3] * b[3],
        a[0] * b[4] + a[2] * b[5] + a[4],
        a[1] * b[4] + a[3] * b[5] + a[5],
    ]
}

fn glyph_count(operands: &[Operand]) -> usize {
    operands
        .iter()
        .map(|operand| match operand {
            Operand::String(bytes) => bytes.len(),
            Operand::Array(items) => items
                .iter()
                .map(|item| match item {
                    Operand::String(bytes) => bytes.len(),
                    _ => 0,
                })
                .sum(),
            _ => 0,
        })
        .sum()
}

fn text_bbox(tm: &[f64; 6], font_size: f64, glyph_count: usize) -> [f64; 4] {
    let width = font_size * glyph_count as f64 * 0.5;
    let height = font_size;
    let x0 = tm[4];
    let y0 = tm[5];
    [x0, y0, x0 + width, y0 + height]
}

fn image_bbox(matrix: &[f64; 6]) -> [f64; 4] {
    let width = matrix[0];
    let height = matrix[3];
    let x0 = matrix[4];
    let y0 = matrix[5];
    [x0, y0, x0 + width, y0 + height]
}

trait Pipe: Sized {
    fn pipe<F, R>(self, f: F) -> R
    where
        F: FnOnce(Self) -> R,
    {
        f(self)
    }
}

impl<T> Pipe for T {}
