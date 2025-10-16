//! Extraction pipeline for producing an editing IR for page 0.

use std::{collections::HashMap, ops::Range};

use anyhow::{anyhow, Result};
use lopdf::{Dictionary, Document, Object, ObjectId};

use crate::types::{BtSpan, DocumentIR, FontInfo, ImageObject, PageIR, PageObject, TextObject};

use super::content::{tokenize_stream, Operand, Operator, Token};

#[derive(Debug, Clone)]
pub struct StreamCache {
    pub object_id: u32,
    pub bytes: Vec<u8>,
    pub tokens: Vec<Token>,
}

#[derive(Debug, Clone)]
pub struct TextMeta {
    pub stream_obj: u32,
    pub bt_range: Range<usize>,
    pub tm_range: Option<Range<usize>>,
    pub insert_after_bt: usize,
}

#[derive(Debug, Clone)]
pub struct ImageMeta {
    pub stream_obj: u32,
    pub cm_range: Range<usize>,
}

#[derive(Debug, Clone)]
pub enum ObjectMeta {
    Text(TextMeta),
    Image(ImageMeta),
}

#[derive(Debug, Clone)]
pub struct Page0Cache {
    pub ir: DocumentIR,
    pub page_id: ObjectId,
    pub streams: HashMap<u32, StreamCache>,
    pub objects: HashMap<String, ObjectMeta>,
}

pub fn extract_page0(doc: &Document) -> Result<Page0Cache> {
    let (page_number, page_id) = doc
        .get_pages()
        .into_iter()
        .next()
        .ok_or_else(|| anyhow!("document is empty"))?;
    let page_dict = doc.get_dictionary(page_id)?;
    let (width_pt, height_pt) = page_size(&page_dict)?;

    let stream_entries = load_page_streams(doc, page_id)?;
    let mut streams = HashMap::new();
    for entry in &stream_entries {
        streams.insert(entry.object_id, entry.clone());
    }

    let image_xobjects = collect_image_xobjects(doc, page_id)?;

    let mut objects = Vec::new();
    let mut meta_map = HashMap::new();
    let mut text_counter = 0;
    let mut image_counter = 0;

    for stream in &stream_entries {
        let mut graphics_stack = vec![GraphicsState::default()];
        let mut text_state = TextState::default();
        for token in &stream.tokens {
            match &token.operator {
                Operator::q => graphics_stack.push(*graphics_stack.last().unwrap()),
                Operator::Q => {
                    graphics_stack.pop();
                    if graphics_stack.is_empty() {
                        graphics_stack.push(GraphicsState::default());
                    }
                }
                Operator::Cm => {
                    let matrix = operands_to_matrix(&token.operands)?;
                    if let Some(state) = graphics_stack.last_mut() {
                        state.ctm = multiply(state.ctm, matrix);
                        state.last_cm = Some((token.range.clone(), matrix));
                    }
                }
                Operator::BT => {
                    text_state = TextState::new(token.range.clone());
                }
                Operator::ET => {
                    if let Some((text_obj, text_meta)) =
                        text_state.finish(token.range.clone(), stream.object_id)?
                    {
                        let id = format!("t:{text_counter}");
                        text_counter += 1;
                        objects.push(PageObject::Text(TextObject {
                            id: id.clone(),
                            ..text_obj
                        }));
                        meta_map.insert(id, ObjectMeta::Text(text_meta));
                    }
                }
                Operator::Tf => text_state.set_font(&token.operands),
                Operator::Tm => {
                    let matrix = operands_to_matrix(&token.operands)?;
                    text_state.set_tm(matrix, token.range.clone());
                }
                Operator::Td => {
                    let matrix = operands_to_translation(&token.operands)?;
                    text_state.translate(matrix);
                }
                Operator::TStar => text_state.next_line(),
                Operator::Tj | Operator::TJ => {
                    text_state.push_run(token)?;
                }
                Operator::Do => {
                    if let Some(image) =
                        build_image(token, stream.object_id, &graphics_stack, &image_xobjects)?
                    {
                        let id = format!("img:{image_counter}");
                        image_counter += 1;
                        meta_map.insert(id.clone(), ObjectMeta::Image(image.meta));
                        objects.push(PageObject::Image(ImageObject { id, ..image.object }));
                    }
                }
                Operator::Other(name) if name == "TL" => text_state.set_leading(&token.operands),
                _ => {}
            }
        }
    }

    let page_ir = PageIR {
        index: page_number as usize - 1,
        width_pt,
        height_pt,
        objects,
    };

    Ok(Page0Cache {
        ir: DocumentIR {
            pages: vec![page_ir],
        },
        page_id,
        streams,
        objects: meta_map,
    })
}

fn page_size(page_dict: &Dictionary) -> Result<(f64, f64)> {
    let media_box = page_dict
        .get(b"MediaBox")
        .or_else(|_| page_dict.get(b"CropBox"))?
        .as_array()?;
    if media_box.len() < 4 {
        return Err(anyhow!("invalid MediaBox"));
    }
    let x0 = get_number(&media_box[0])?;
    let y0 = get_number(&media_box[1])?;
    let x1 = get_number(&media_box[2])?;
    let y1 = get_number(&media_box[3])?;
    Ok((x1 - x0, y1 - y0))
}

fn get_number(object: &Object) -> Result<f64> {
    match object {
        Object::Integer(v) => Ok(*v as f64),
        Object::Real(v) => Ok(f64::from(*v)),
        _ => Err(anyhow!("expected numeric value")),
    }
}

fn load_page_streams(doc: &Document, page_id: ObjectId) -> Result<Vec<StreamCache>> {
    let mut entries = Vec::new();
    for object_id in doc.get_page_contents(page_id) {
        if let Ok(stream) = doc.get_object(object_id).and_then(Object::as_stream) {
            let bytes = stream
                .decompressed_content()
                .unwrap_or_else(|_| stream.content.clone());
            let tokens = tokenize_stream(&bytes)?;
            entries.push(StreamCache {
                object_id: object_id.0,
                bytes,
                tokens,
            });
        }
    }
    Ok(entries)
}

fn collect_image_xobjects(doc: &Document, page_id: ObjectId) -> Result<HashMap<String, ObjectId>> {
    let (direct, inherited) = doc.get_page_resources(page_id);
    let mut map = HashMap::new();
    if let Some(dict) = direct {
        collect_xobjects_from_dict(doc, dict, &mut map)?;
    }
    for res_id in inherited {
        let dict = doc.get_dictionary(res_id)?;
        collect_xobjects_from_dict(doc, &dict, &mut map)?;
    }
    Ok(map)
}

fn collect_xobjects_from_dict(
    doc: &Document,
    dict: &Dictionary,
    map: &mut HashMap<String, ObjectId>,
) -> Result<()> {
    if let Ok(xobj) = dict.get(b"XObject") {
        match xobj {
            Object::Dictionary(inner) => {
                for (name, value) in inner.iter() {
                    if let Ok(id) = value.as_reference() {
                        if is_image(doc, id)? {
                            map.insert(String::from_utf8_lossy(name).into_owned(), id);
                        }
                    }
                }
            }
            Object::Reference(id) => {
                let dict = doc.get_dictionary(*id)?;
                collect_xobjects_from_dict(doc, &dict, map)?;
            }
            _ => {}
        }
    }
    Ok(())
}

fn is_image(doc: &Document, id: ObjectId) -> Result<bool> {
    let dict = doc.get_dictionary(id)?;
    match dict.get(b"Subtype") {
        Ok(Object::Name(name)) => Ok(name == b"Image"),
        _ => Ok(false),
    }
}

#[derive(Debug)]
struct BuiltImage {
    object: ImageObject,
    meta: ImageMeta,
}

fn build_image(
    token: &Token,
    stream_obj: u32,
    graphics_stack: &[GraphicsState],
    image_xobjects: &HashMap<String, ObjectId>,
) -> Result<Option<BuiltImage>> {
    if token.operands.len() != 1 {
        return Ok(None);
    }
    let name = match &token.operands[0] {
        Operand::Name(value) => value.clone(),
        _ => return Ok(None),
    };
    if !image_xobjects.contains_key(&name) {
        return Ok(None);
    }
    let state = graphics_stack.last().unwrap();
    let (cm_range, matrix) = match &state.last_cm {
        Some(value) => value.clone(),
        None => return Ok(None),
    };
    let bbox = image_bbox(matrix);
    Ok(Some(BuiltImage {
        object: ImageObject {
            id: String::new(),
            x_object: name,
            cm: matrix,
            bbox,
        },
        meta: ImageMeta {
            stream_obj,
            cm_range,
        },
    }))
}

fn image_bbox(matrix: [f64; 6]) -> [f64; 4] {
    let [a, _b, _c, d, e, f] = matrix;
    let width = a.abs();
    let height = d.abs();
    [e, f, e + width, f + height]
}

#[derive(Clone, Copy, Debug)]
struct GraphicsState {
    ctm: [f64; 6],
    last_cm: Option<(Range<usize>, [f64; 6])>,
}

impl Default for GraphicsState {
    fn default() -> Self {
        Self {
            ctm: IDENTITY,
            last_cm: None,
        }
    }
}

#[derive(Debug, Clone)]
struct TextRun {
    tm: [f64; 6],
    tm_range: Option<Range<usize>>,
    glyphs: usize,
    bt_range: Range<usize>,
    insert_after_bt: usize,
}

#[derive(Debug, Clone)]
struct TextState {
    active: bool,
    text_matrix: [f64; 6],
    text_line_matrix: [f64; 6],
    leading: f64,
    font: Option<FontInfo>,
    current_run: Option<TextRun>,
    pending_tm: Option<(Range<usize>, [f64; 6])>,
    bt_range: Option<Range<usize>>,
}

impl Default for TextState {
    fn default() -> Self {
        Self {
            active: false,
            text_matrix: IDENTITY,
            text_line_matrix: IDENTITY,
            leading: 0.0,
            font: None,
            current_run: None,
            pending_tm: None,
            bt_range: None,
        }
    }
}

impl TextState {
    fn new(bt_range: Range<usize>) -> Self {
        Self {
            active: true,
            bt_range: Some(bt_range.clone()),
            ..Default::default()
        }
    }

    fn set_font(&mut self, operands: &[Operand]) {
        if operands.len() != 2 {
            return;
        }
        let res_name = match &operands[0] {
            Operand::Name(value) => value.clone(),
            _ => return,
        };
        let size = match operands[1] {
            Operand::Number(value) => value,
            _ => return,
        };
        self.font = Some(FontInfo { res_name, size });
    }

    fn set_tm(&mut self, matrix: [f64; 6], range: Range<usize>) {
        self.text_matrix = matrix;
        self.text_line_matrix = matrix;
        self.pending_tm = Some((range, matrix));
    }

    fn translate(&mut self, matrix: [f64; 6]) {
        self.text_line_matrix = multiply(self.text_line_matrix, matrix);
        self.text_matrix = self.text_line_matrix;
        self.pending_tm = None;
    }

    fn next_line(&mut self) {
        let translation = [1.0, 0.0, 0.0, 1.0, 0.0, -self.leading];
        self.translate(translation);
    }

    fn set_leading(&mut self, operands: &[Operand]) {
        if operands.len() != 1 {
            return;
        }
        if let Operand::Number(value) = operands[0] {
            self.leading = value;
        }
    }

    fn push_run(&mut self, token: &Token) -> Result<()> {
        if !self.active {
            return Ok(());
        }
        let glyphs = count_glyphs(&token.operands);
        if glyphs == 0 {
            return Ok(());
        }
        if self.current_run.is_none() {
            let (tm_range, tm_matrix) = match self.pending_tm.take() {
                Some((range, matrix)) => (Some(range), matrix),
                None => (None, self.text_matrix),
            };
            let bt_range = self
                .bt_range
                .clone()
                .ok_or_else(|| anyhow!("BT without range"))?;
            let insert_after_bt = bt_range.end;
            self.current_run = Some(TextRun {
                tm: tm_matrix,
                tm_range,
                glyphs: 0,
                bt_range,
                insert_after_bt,
            });
        }
        if let Some(run) = self.current_run.as_mut() {
            run.glyphs += glyphs;
        }
        Ok(())
    }

    fn finish(
        &mut self,
        et_range: Range<usize>,
        stream_obj: u32,
    ) -> Result<Option<(TextObject, TextMeta)>> {
        self.active = false;
        if let Some(run) = self.current_run.take() {
            let font = self.font.clone().unwrap_or(FontInfo {
                res_name: String::from("F0"),
                size: 12.0,
            });
            let width = font.size * run.glyphs as f64;
            let height = font.size;
            let tm = run.tm;
            let bbox = [tm[4], tm[5], tm[4] + width, tm[5] + height];
            let text = TextObject {
                id: String::new(),
                bt_span: BtSpan {
                    stream_obj,
                    start: run.bt_range.start as u32,
                    end: et_range.end as u32,
                },
                tm,
                font,
                bbox,
            };
            let meta = TextMeta {
                stream_obj,
                bt_range: run.bt_range,
                tm_range: run.tm_range,
                insert_after_bt: run.insert_after_bt,
            };
            return Ok(Some((text, meta)));
        }
        Ok(None)
    }
}

fn count_glyphs(operands: &[Operand]) -> usize {
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

const IDENTITY: [f64; 6] = [1.0, 0.0, 0.0, 1.0, 0.0, 0.0];

fn operands_to_matrix(operands: &[Operand]) -> Result<[f64; 6]> {
    if operands.len() != 6 {
        return Err(anyhow!("matrix requires six operands"));
    }
    let mut values = [0.0; 6];
    for (index, operand) in operands.iter().enumerate() {
        values[index] = match operand {
            Operand::Number(value) => *value,
            _ => return Err(anyhow!("matrix operands must be numbers")),
        };
    }
    Ok(values)
}

fn operands_to_translation(operands: &[Operand]) -> Result<[f64; 6]> {
    if operands.len() != 2 {
        return Err(anyhow!("translation requires two operands"));
    }
    let tx = match operands[0] {
        Operand::Number(v) => v,
        _ => return Err(anyhow!("translation operands must be numbers")),
    };
    let ty = match operands[1] {
        Operand::Number(v) => v,
        _ => return Err(anyhow!("translation operands must be numbers")),
    };
    Ok([1.0, 0.0, 0.0, 1.0, tx, ty])
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
