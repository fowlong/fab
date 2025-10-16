//! Intermediate representation extraction for the MVP transform workflow.
//!
//! The extractor focuses on page 0 and recognises text runs (BT … ET) and
//! image XObject invocations (Do). It produces a simplified IR consumed by the
//! frontend overlay along with a in-memory cache that retains byte spans for
//! patching.

use std::{collections::HashMap, ops::Range};

use anyhow::{anyhow, Context, Result};
use lopdf::{Dictionary, Document, Object, ObjectId};

use crate::{
    pdf::content::{self, Operand, Operator, Token},
    types::{DocumentIR, FontSummary, ImageObject, PageIR, PageObject, Span, TextObject},
};

#[derive(Debug, Clone)]
pub struct StreamSegment {
    pub id: ObjectId,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct TextCache {
    pub stream_id: ObjectId,
    pub bt_range: Range<usize>,
    pub et_range: Range<usize>,
    pub tm_range: Option<Range<usize>>,
    pub tm: [f64; 6],
}

#[derive(Debug, Clone)]
pub struct ImageCache {
    pub stream_id: ObjectId,
    pub cm_range: Range<usize>,
    pub matrix: [f64; 6],
}

#[derive(Debug, Clone)]
pub struct PageCache {
    pub page_index: usize,
    pub page_id: ObjectId,
    pub width_pt: f64,
    pub height_pt: f64,
    pub segments: Vec<StreamSegment>,
    pub text_map: HashMap<String, TextCache>,
    pub image_map: HashMap<String, ImageCache>,
}

#[derive(Debug, Clone)]
pub struct ExtractionResult {
    pub ir: DocumentIR,
    pub page_cache: PageCache,
}
pub fn extract_page_zero(doc: &Document) -> Result<ExtractionResult> {
    let pages = doc.get_pages();
    let (&page_number, &page_id) = pages
        .iter()
        .min_by_key(|(index, _)| *index)
        .ok_or_else(|| anyhow!("PDF has no pages"))?;
    let page_index = page_number.saturating_sub(1) as usize;

    let page_obj = doc
        .get_object(page_id)
        .with_context(|| format!("page object {page_id:?} missing"))?;
    let page_dict = page_obj
        .as_dict()
        .cloned()
        .ok_or_else(|| anyhow!("page object is not a dictionary"))?;

    let (width_pt, height_pt) = resolve_page_dimensions(doc, &page_dict)?;

    let resources_dict = resolve_dict(doc, page_dict.get(b"Resources"))?;
    let image_targets = resolve_image_xobjects(doc, &resources_dict)?;

    let contents_obj = page_dict
        .get(b"Contents")
        .context("page has no /Contents entry")?;
    let mut segments_with_tokens = resolve_streams(doc, contents_obj)?;

    let mut objects = Vec::new();
    let mut text_map: HashMap<String, TextCache> = HashMap::new();
    let mut image_map: HashMap<String, ImageCache> = HashMap::new();

    let mut text_counter = 0usize;
    let mut image_counter = 0usize;

    let mut ctm = [1.0, 0.0, 0.0, 1.0, 0.0, 0.0];
    let mut ctm_stack: Vec<[f64; 6]> = Vec::new();
    let mut text_state = TextState::default();
    let mut current_cm: Option<CmRecord> = None;

    for segment in &segments_with_tokens {
        for token in &segment.tokens {
            match token.operator {
                Operator::q => {
                    ctm_stack.push(ctm);
                    current_cm = None;
                }
                Operator::Q => {
                    ctm = ctm_stack.pop().unwrap_or([1.0, 0.0, 0.0, 1.0, 0.0, 0.0]);
                    current_cm = None;
                }
                Operator::Cm => {
                    if let Some(matrix) = operands_to_matrix(&token.operands) {
                        let record = CmRecord {
                            stream_id: segment.id,
                            range: token.range.clone(),
                            matrix,
                            ctm_before: ctm,
                        };
                        ctm = multiply(ctm, matrix);
                        current_cm = Some(record);
                    }
                }
                Operator::Bt => {
                    text_state.begin(segment.id, token.range.clone());
                }
                Operator::Et => {
                    if let Some(run) = text_state.end(token.range.clone()) {
                        let span = Span {
                            stream_obj: run.stream_id.0,
                            start: run.bt_range.start.try_into().unwrap_or(u32::MAX),
                            end: run.et_range.end.try_into().unwrap_or(u32::MAX),
                        };
                        let width = approx_text_width(run.font.size, run.glyph_bytes);
                        let height = run.font.size;
                        let x = run.tm[4];
                        let y = run.tm[5];
                        let bbox = [x, y, x + width, y + height];
                        let text_object = TextObject {
                            id: run.id.clone(),
                            bt_span: span,
                            tm: run.tm,
                            font: run.font.clone(),
                            bbox,
                        };
                        text_map.insert(
                            run.id.clone(),
                            TextCache {
                                stream_id: run.stream_id,
                                bt_range: run.bt_range.clone(),
                                et_range: run.et_range.clone(),
                                tm_range: run.tm_range.clone(),
                                tm: run.tm,
                            },
                        );
                        objects.push(PageObject::Text(text_object));
                    }
                }
                Operator::Tf => {
                    if token.operands.len() >= 2 {
                        if let (Some(font_name), Some(size)) = (
                            operand_name(&token.operands[0]),
                            operand_number(&token.operands[1]),
                        ) {
                            text_state.set_font(font_name.to_string(), size);
                        }
                    }
                }
                Operator::Tm => {
                    if let Some(matrix) = operands_to_matrix(&token.operands) {
                        text_state.set_tm(matrix, token.range.clone());
                    }
                }
                Operator::Td => {
                    if token.operands.len() >= 2 {
                        if let (Some(tx), Some(ty)) = (
                            operand_number(&token.operands[0]),
                            operand_number(&token.operands[1]),
                        ) {
                            text_state.translate(tx, ty);
                        }
                    }
                }
                Operator::TStar => {
                    text_state.new_line();
                }
                Operator::Tj | Operator::TJ => {
                    if let Some(len) = text_length(token) {
                        text_state.ensure_run(segment.id, ctm, &mut text_counter);
                        text_state.add_glyphs(len);
                    }
                }
                Operator::Do => {
                    if let Some(name) = token.operands.first().and_then(operand_name) {
                        if image_targets.contains_key(name) {
                            let id = format!("img:{image_counter}");
                            image_counter += 1;
                            if let Some(record) = current_cm.take() {
                                let effective = multiply(record.ctm_before, record.matrix);
                                let width = record.matrix[0].abs();
                                let height = record.matrix[3].abs();
                                let tx = effective[4];
                                let ty = effective[5];
                                let bbox = [tx, ty, tx + width, ty + height];
                                let image_object = ImageObject {
                                    id: id.clone(),
                                    x_object: name.to_string(),
                                    cm: effective,
                                    bbox,
                                };
                                image_map.insert(
                                    id.clone(),
                                    ImageCache {
                                        stream_id: record.stream_id,
                                        cm_range: record.range.clone(),
                                        matrix: record.matrix,
                                    },
                                );
                                objects.push(PageObject::Image(image_object));
                            }
                        }
                    }
                    current_cm = None;
                }
                _ => {}
            }
        }
    }

    let segments: Vec<StreamSegment> = segments_with_tokens
        .into_iter()
        .map(|segment| StreamSegment {
            id: segment.id,
            bytes: segment.bytes,
        })
        .collect();

    let page_ir = PageIR {
        index: page_index,
        width_pt,
        height_pt,
        objects,
    };

    let ir = DocumentIR {
        pages: vec![page_ir],
    };

    let page_cache = PageCache {
        page_index,
        page_id,
        width_pt,
        height_pt,
        segments,
        text_map,
        image_map,
    };

    Ok(ExtractionResult { ir, page_cache })
}
#[derive(Debug)]
struct StreamWithTokens {
    id: ObjectId,
    bytes: Vec<u8>,
    tokens: Vec<Token>,
}

#[derive(Debug, Clone)]
struct ActiveRun {
    id: String,
    stream_id: ObjectId,
    bt_range: Range<usize>,
    et_range: Range<usize>,
    tm_range: Option<Range<usize>>,
    tm: [f64; 6],
    font: FontSummary,
    glyph_bytes: usize,
}

#[derive(Debug, Default)]
struct TextState {
    in_text: bool,
    current_stream: Option<ObjectId>,
    bt_range: Option<Range<usize>>,
    tm_range: Option<Range<usize>>,
    tm_matrix: [f64; 6],
    line_matrix: [f64; 6],
    font_name: Option<String>,
    font_size: f64,
    leading: f64,
    run: Option<ActiveRun>,
}

impl TextState {
    fn begin(&mut self, stream_id: ObjectId, range: Range<usize>) {
        self.in_text = true;
        self.current_stream = Some(stream_id);
        self.bt_range = Some(range);
        self.tm_range = None;
        self.tm_matrix = [1.0, 0.0, 0.0, 1.0, 0.0, 0.0];
        self.line_matrix = self.tm_matrix;
        self.run = None;
    }

    fn end(&mut self, et_range: Range<usize>) -> Option<ActiveRun> {
        if let Some(mut run) = self.run.take() {
            run.et_range = et_range;
            self.reset_after_text();
            Some(run)
        } else {
            self.reset_after_text();
            None
        }
    }

    fn reset_after_text(&mut self) {
        self.in_text = false;
        self.current_stream = None;
        self.bt_range = None;
        self.tm_range = None;
        self.run = None;
    }

    fn set_font(&mut self, name: String, size: f64) {
        self.font_name = Some(name);
        self.font_size = size;
        if self.leading.abs() < f64::EPSILON {
            self.leading = size;
        }
    }

    fn set_tm(&mut self, matrix: [f64; 6], range: Range<usize>) {
        self.tm_matrix = matrix;
        self.line_matrix = matrix;
        self.tm_range = Some(range);
    }

    fn translate(&mut self, tx: f64, ty: f64) {
        let translation = [1.0, 0.0, 0.0, 1.0, tx, ty];
        self.line_matrix = multiply(self.line_matrix, translation);
        self.tm_matrix = self.line_matrix;
        self.tm_range = None;
    }

    fn new_line(&mut self) {
        let shift = [1.0, 0.0, 0.0, 1.0, 0.0, -self.leading];
        self.line_matrix = multiply(self.line_matrix, shift);
        self.tm_matrix = self.line_matrix;
        self.tm_range = None;
    }

    fn ensure_run(&mut self, stream_id: ObjectId, ctm: [f64; 6], counter: &mut usize) {
        if self.run.is_some() || !self.in_text {
            return;
        }
        let bt_range = self
            .bt_range
            .clone()
            .unwrap_or_else(|| Range { start: 0, end: 0 });
        let id = format!("t:{}", *counter);
        *counter += 1;
        let font_size = if self.font_size.abs() < f64::EPSILON {
            12.0
        } else {
            self.font_size
        };
        let font = FontSummary {
            res_name: self.font_name.clone().unwrap_or_else(|| "F1".to_string()),
            size: font_size,
        };
        let tm = multiply(ctm, self.tm_matrix);
        let run = ActiveRun {
            id,
            stream_id,
            bt_range,
            et_range: bt_range.clone(),
            tm_range: self.tm_range.clone(),
            tm,
            font,
            glyph_bytes: 0,
        };
        self.run = Some(run);
    }

    fn add_glyphs(&mut self, glyph_bytes: usize) {
        if let Some(run) = &mut self.run {
            run.glyph_bytes += glyph_bytes;
            if run.tm_range.is_none() {
                run.tm_range = self.tm_range.clone();
            }
        }
        let font_size = if self.font_size.abs() < f64::EPSILON {
            12.0
        } else {
            self.font_size
        };
        let advance = approx_text_width(font_size, glyph_bytes);
        let translation = [1.0, 0.0, 0.0, 1.0, advance, 0.0];
        self.tm_matrix = multiply(self.tm_matrix, translation);
        self.line_matrix = self.tm_matrix;
        self.tm_range = None;
    }
}

#[derive(Debug, Clone)]
struct CmRecord {
    stream_id: ObjectId,
    range: Range<usize>,
    matrix: [f64; 6],
    ctm_before: [f64; 6],
}

fn resolve_page_dimensions(doc: &Document, dict: &Dictionary) -> Result<(f64, f64)> {
    let media_box = dict.get(b"MediaBox").context("page is missing MediaBox")?;
    let array = match media_box {
        Object::Array(items) => items.clone(),
        Object::Reference(id) => doc
            .get_object(*id)?
            .as_array()
            .cloned()
            .ok_or_else(|| anyhow!("MediaBox reference is not an array"))?,
        other => return Err(anyhow!("unexpected MediaBox object: {other:?}")),
    };
    if array.len() < 4 {
        return Err(anyhow!("MediaBox must contain four numbers"));
    }
    let x0 = array[0].clone().as_f64()?;
    let y0 = array[1].clone().as_f64()?;
    let x1 = array[2].clone().as_f64()?;
    let y1 = array[3].clone().as_f64()?;
    Ok((x1 - x0, y1 - y0))
}

fn resolve_dict(doc: &Document, object: Option<&Object>) -> Result<Dictionary> {
    match object {
        Some(Object::Dictionary(dict)) => Ok(dict.clone()),
        Some(Object::Reference(id)) => doc
            .get_object(*id)?
            .as_dict()
            .cloned()
            .ok_or_else(|| anyhow!("referenced object is not a dictionary")),
        Some(_) => Err(anyhow!("expected dictionary reference")),
        None => Err(anyhow!("dictionary entry missing")),
    }
}

fn resolve_image_xobjects(
    doc: &Document,
    resources: &Dictionary,
) -> Result<HashMap<String, ObjectId>> {
    let mut out = HashMap::new();
    if let Some(xobj) = resources.get(b"XObject") {
        let dict = resolve_dict(doc, Some(xobj))?;
        for (key, value) in dict.iter() {
            if let Object::Reference(id) = value {
                if let Some(subtype) = doc
                    .get_object(*id)?
                    .as_dict()
                    .and_then(|d| d.get(b"Subtype"))
                {
                    if matches!(subtype, Object::Name(name) if name == b"Image") {
                        let name = String::from_utf8_lossy(key).to_string();
                        out.insert(name, *id);
                    }
                }
            }
        }
    }
    Ok(out)
}

fn resolve_streams(doc: &Document, contents: &Object) -> Result<Vec<StreamWithTokens>> {
    match contents {
        Object::Reference(id) => Ok(vec![load_stream(doc, *id)?]),
        Object::Array(items) => {
            let mut out = Vec::new();
            for item in items {
                match item {
                    Object::Reference(id) => out.push(load_stream(doc, *id)?),
                    other => {
                        return Err(anyhow!("unsupported inline content stream: {other:?}"));
                    }
                }
            }
            Ok(out)
        }
        other => Err(anyhow!("unsupported /Contents object: {other:?}")),
    }
}

fn load_stream(doc: &Document, id: ObjectId) -> Result<StreamWithTokens> {
    let stream = doc
        .get_object(id)?
        .as_stream()
        .cloned()
        .ok_or_else(|| anyhow!("content stream {id:?} is not a stream"))?;
    let bytes = stream.decompressed_content()?;
    let tokens = content::tokenize_stream(&bytes)?;
    Ok(StreamWithTokens { id, bytes, tokens })
}

fn operands_to_matrix(operands: &[Operand]) -> Option<[f64; 6]> {
    if operands.len() < 6 {
        return None;
    }
    let mut values = [0.0; 6];
    for (idx, slot) in values.iter_mut().enumerate() {
        *slot = operand_number(&operands[idx])?;
    }
    Some(values)
}

fn operand_number(operand: &Operand) -> Option<f64> {
    match operand {
        Operand::Number(value) => Some(*value),
        _ => None,
    }
}

fn operand_name(operand: &Operand) -> Option<&str> {
    match operand {
        Operand::Name(name) => Some(name.as_str()),
        _ => None,
    }
}

fn text_length(token: &Token) -> Option<usize> {
    match token.operands.first()? {
        Operand::String(bytes) => Some(bytes.len()),
        Operand::Array(items) => Some(
            items
                .iter()
                .filter_map(|operand| match operand {
                    Operand::String(bytes) => Some(bytes.len()),
                    _ => None,
                })
                .sum(),
        ),
        _ => None,
    }
}

fn approx_text_width(font_size: f64, glyph_bytes: usize) -> f64 {
    font_size * glyph_bytes as f64 * 0.6
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
