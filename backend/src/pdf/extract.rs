//! Intermediate representation extraction for the MVP editor.

use std::collections::HashMap;

use anyhow::{anyhow, Context, Result};
use lopdf::{Dictionary, Document, Object, ObjectId};

use crate::types::{BtSpan, DocumentIR, FontInfo, ImageObject, PageIR, PageObject, TextObject};

use super::content::{tokenize_stream, ContentToken, Operand, Operator};

#[derive(Debug, Clone)]
pub struct StreamCache {
    pub object_id: ObjectId,
    pub dict: Dictionary,
    pub bytes: Vec<u8>,
    pub tokens: Vec<ContentToken>,
}

#[derive(Debug, Clone)]
pub struct TextCache {
    pub stream_index: usize,
    pub bt_index: usize,
    pub et_index: usize,
    pub tm_index: Option<usize>,
    pub tm: [f64; 6],
    pub span: BtSpan,
    pub has_explicit_tm: bool,
}

#[derive(Debug, Clone)]
pub struct ImageCache {
    pub stream_index: usize,
    pub cm_index: usize,
    pub do_index: usize,
    pub cm: [f64; 6],
    pub name: String,
}

#[derive(Debug, Clone)]
pub enum CachedObject {
    Text(TextCache),
    Image(ImageCache),
}

#[derive(Debug, Clone)]
pub struct PageCache {
    pub page_id: ObjectId,
    pub width_pt: f64,
    pub height_pt: f64,
    pub streams: Vec<StreamCache>,
    pub objects: HashMap<String, CachedObject>,
}

#[derive(Debug, Clone)]
pub struct ExtractedDocument {
    pub ir: DocumentIR,
    pub cache: PageCache,
}

pub fn extract_page_zero(doc: &Document) -> Result<ExtractedDocument> {
    let pages = doc.get_pages();
    let (&page_number, &(page_id, ref page_dict)) = pages
        .iter()
        .next()
        .ok_or_else(|| anyhow!("document has no pages"))?;
    let page_dict = page_dict.clone();

    let (width_pt, height_pt) = media_box_size(&page_dict)?;

    let resources = resolve_dict(doc, page_dict.get(b"Resources")?)?;
    let image_map = discover_images(doc, &resources)?;

    let stream_refs = resolve_contents(doc, &page_dict)?;
    let mut streams = Vec::new();
    for object_id in stream_refs {
        let stream = doc
            .get_object(object_id)
            .with_context(|| format!("content stream {object_id:?} missing"))?;
        let stream = stream
            .as_stream()
            .cloned()
            .map_err(|_| anyhow!("content entry was not a stream"))?;
        let dict = stream.dict.clone();
        let bytes = stream
            .decompressed_content()
            .unwrap_or_else(|_| stream.content.clone());
        let tokens = tokenize_stream(&bytes)?;
        streams.push(StreamCache {
            object_id,
            dict,
            bytes,
            tokens,
        });
    }

    let mut objects = Vec::new();
    let mut cache_map = HashMap::new();
    let mut text_counter = 0usize;
    let mut image_counter = 0usize;

    for (stream_index, stream) in streams.iter().enumerate() {
        let mut state = GraphicsState::default();
        let mut text_state = TextState::default();
        for (token_index, token) in stream.tokens.iter().enumerate() {
            match token.operator {
                Operator::q => state.push(),
                Operator::Q => state.pop(),
                Operator::Cm => {
                    let m = extract_matrix(&token.operands)?;
                    state.apply_cm(&m, token_index);
                }
                Operator::BT => {
                    text_state.begin(token.span.start, token_index);
                }
                Operator::ET => {
                    if let Some(result) = text_state.end(token.span.end, token_index) {
                        let id = format!("t:{text_counter}");
                        text_counter += 1;
                        let bbox = bbox_from_tm(&result.tm, result.glyphs, result.font_size);
                        let text_object = TextObject {
                            id: id.clone(),
                            bt_span: BtSpan {
                                stream_obj: stream.object_id.0,
                                start: result.bt_span_start as u32,
                                end: token.span.end as u32,
                            },
                            tm: result.tm,
                            font: FontInfo {
                                res_name: result.font_name,
                                size: result.font_size,
                            },
                            bbox,
                        };
                        objects.push(PageObject::Text(text_object.clone()));
                        cache_map.insert(
                            id,
                            CachedObject::Text(TextCache {
                                stream_index,
                                bt_index: result.bt_token_index,
                                et_index: token_index,
                                tm_index: result.tm_token_index,
                                tm: result.tm,
                                span: BtSpan {
                                    stream_obj: stream.object_id.0,
                                    start: result.bt_span_start as u32,
                                    end: token.span.end as u32,
                                },
                                has_explicit_tm: result.tm_token_index.is_some(),
                            }),
                        );
                    }
                }
                Operator::Tf => {
                    if let Some(name) = token.operands.first().and_then(|op| match op {
                        Operand::Name(name) => Some(name.clone()),
                        _ => None,
                    }) {
                        let size = token
                            .operands
                            .get(1)
                            .and_then(|op| match op {
                                Operand::Number(n) => Some(*n),
                                _ => None,
                            })
                            .unwrap_or(12.0);
                        text_state.font = Some((name, size));
                    }
                }
                Operator::Tm => {
                    let tm = extract_matrix(&token.operands)?;
                    text_state.set_tm(tm, token_index);
                }
                Operator::Td => {
                    let tx = match token.operands.get(0) {
                        Some(Operand::Number(n)) => *n,
                        _ => 0.0,
                    };
                    let ty = match token.operands.get(1) {
                        Some(Operand::Number(n)) => *n,
                        _ => 0.0,
                    };
                    text_state.translate(tx, ty);
                }
                Operator::TStar => {
                    text_state.translate(0.0, -text_state.leading());
                }
                Operator::Tj | Operator::TJ => {
                    let glyphs = count_glyphs(&token.operands);
                    if glyphs > 0 {
                        text_state.register_text(token.span.end, glyphs, token_index);
                    }
                }
                Operator::Do => {
                    if let Some(name) = token.operands.first().and_then(|op| match op {
                        Operand::Name(name) => Some(name.clone()),
                        _ => None,
                    }) {
                        if image_map.contains_key(&name) {
                            let cm = state.current_ctm;
                            let bbox = bbox_from_cm(&cm);
                            let id = format!("img:{image_counter}");
                            image_counter += 1;
                            let image_object = ImageObject {
                                id: id.clone(),
                                x_object: name.clone(),
                                cm,
                                bbox,
                            };
                            objects.push(PageObject::Image(image_object));
                            cache_map.insert(
                                id,
                                CachedObject::Image(ImageCache {
                                    stream_index,
                                    cm_index: state.last_cm_index.unwrap_or(token_index),
                                    do_index: token_index,
                                    cm,
                                    name,
                                }),
                            );
                        }
                    }
                }
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

    Ok(ExtractedDocument {
        ir: DocumentIR {
            pages: vec![page_ir],
        },
        cache: PageCache {
            page_id,
            width_pt,
            height_pt,
            streams,
            objects: cache_map,
        },
    })
}

fn media_box_size(page_dict: &Dictionary) -> Result<(f64, f64)> {
    let media_box = page_dict
        .get(b"MediaBox")
        .or_else(|_| page_dict.get(b"CropBox"))?
        .as_array()
        .map_err(|_| anyhow!("MediaBox not an array"))?;
    if media_box.len() < 4 {
        return Err(anyhow!("MediaBox missing coordinates"));
    }
    let llx = media_box[0].as_f64()?;
    let lly = media_box[1].as_f64()?;
    let urx = media_box[2].as_f64()?;
    let ury = media_box[3].as_f64()?;
    Ok((urx - llx, ury - lly))
}

fn resolve_dict(doc: &Document, object: &Object) -> Result<Dictionary> {
    match object {
        Object::Dictionary(dict) => Ok(dict.clone()),
        Object::Reference(id) => doc
            .get_object(*id)
            .context("referenced object missing")?
            .as_dict()
            .map_err(|_| anyhow!("referenced object was not a dictionary"))
            .map(|d| d.clone()),
        _ => Err(anyhow!("expected dictionary")),
    }
}

fn discover_images(doc: &Document, resources: &Dictionary) -> Result<HashMap<String, ObjectId>> {
    let mut images = HashMap::new();
    if let Ok(xobjects) = resources.get(b"XObject") {
        let xobjects = resolve_dict(doc, xobjects)?;
        for (name, value) in xobjects.iter() {
            if let Ok(reference) = value.as_reference() {
                if let Ok(object) = doc.get_object(reference) {
                    if let Ok(stream) = object.as_stream() {
                        if stream
                            .dict
                            .get(b"Subtype")
                            .and_then(Object::as_name_str)
                            .ok()
                            == Some("Image")
                        {
                            images.insert(String::from_utf8_lossy(name).to_string(), reference);
                        }
                    }
                }
            }
        }
    }
    Ok(images)
}

fn resolve_contents(doc: &Document, page_dict: &Dictionary) -> Result<Vec<ObjectId>> {
    let contents = page_dict.get(b"Contents")?;
    let mut out = Vec::new();
    match contents {
        Object::Reference(id) => out.push(*id),
        Object::Array(arr) => {
            for entry in arr {
                out.push(entry.as_reference()?);
            }
        }
        Object::Stream(_) => return Err(anyhow!("inline content streams are not supported")),
        _ => return Err(anyhow!("unexpected contents object")),
    }
    Ok(out)
}

#[derive(Debug, Clone, Copy)]
struct GraphicsState {
    current_ctm: [f64; 6],
    last_cm_index: Option<usize>,
    stack: Vec<([f64; 6], Option<usize>)>,
}

impl Default for GraphicsState {
    fn default() -> Self {
        Self {
            current_ctm: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
            last_cm_index: None,
            stack: Vec::new(),
        }
    }
}

impl GraphicsState {
    fn push(&mut self) {
        self.stack.push((self.current_ctm, self.last_cm_index));
    }

    fn pop(&mut self) {
        if let Some((ctm, last_index)) = self.stack.pop() {
            self.current_ctm = ctm;
            self.last_cm_index = last_index;
        } else {
            self.current_ctm = [1.0, 0.0, 0.0, 1.0, 0.0, 0.0];
            self.last_cm_index = None;
        }
    }

    fn apply_cm(&mut self, m: &[f64; 6], token_index: usize) {
        self.current_ctm = multiply(self.current_ctm, *m);
        self.last_cm_index = Some(token_index);
    }
}

#[derive(Debug, Clone)]
struct TextState {
    active: bool,
    font: Option<(String, f64)>,
    tm: [f64; 6],
    tm_token_index: Option<usize>,
    bt_span_start: usize,
    bt_token_index: usize,
    glyphs: usize,
    tm_at_first_glyph: Option<[f64; 6]>,
    font_size_at_first_glyph: f64,
}

impl Default for TextState {
    fn default() -> Self {
        Self {
            active: false,
            font: None,
            tm: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
            tm_token_index: None,
            bt_span_start: 0,
            bt_token_index: 0,
            glyphs: 0,
            tm_at_first_glyph: None,
            font_size_at_first_glyph: 12.0,
        }
    }
}

struct TextRunResult {
    bt_span_start: usize,
    bt_token_index: usize,
    tm_token_index: Option<usize>,
    tm: [f64; 6],
    font_name: String,
    font_size: f64,
    glyphs: usize,
}

impl TextState {
    fn begin(&mut self, span_start: usize, token_index: usize) {
        self.active = true;
        self.tm = [1.0, 0.0, 0.0, 1.0, 0.0, 0.0];
        self.tm_token_index = None;
        self.bt_span_start = span_start;
        self.bt_token_index = token_index;
        self.glyphs = 0;
        self.tm_at_first_glyph = None;
        self.font_size_at_first_glyph = self.font.as_ref().map(|(_, size)| *size).unwrap_or(12.0);
    }

    fn end(&mut self, _span_end: usize, _token_index: usize) -> Option<TextRunResult> {
        if !self.active {
            return None;
        }
        self.active = false;
        if self.glyphs == 0 {
            return None;
        }
        let (font_name, font_size) = self
            .font
            .clone()
            .unwrap_or_else(|| ("F0".to_string(), self.font_size_at_first_glyph));
        let tm = self.tm_at_first_glyph.unwrap_or(self.tm);
        Some(TextRunResult {
            bt_span_start: self.bt_span_start,
            bt_token_index: self.bt_token_index,
            tm_token_index: self.tm_token_index,
            tm,
            font_name,
            font_size,
            glyphs: self.glyphs,
        })
    }

    fn set_tm(&mut self, tm: [f64; 6], token_index: usize) {
        if !self.active {
            return;
        }
        self.tm = tm;
        self.tm_token_index = Some(token_index);
    }

    fn translate(&mut self, tx: f64, ty: f64) {
        if !self.active {
            return;
        }
        let translation = [1.0, 0.0, 0.0, 1.0, tx, ty];
        self.tm = multiply(self.tm, translation);
        if self.tm_token_index.is_none() {
            self.tm_at_first_glyph = Some(self.tm);
        }
    }

    fn register_text(&mut self, _span_end: usize, glyphs: usize, _token_index: usize) {
        if !self.active {
            return;
        }
        if self.tm_at_first_glyph.is_none() {
            self.tm_at_first_glyph = Some(self.tm);
            self.font_size_at_first_glyph =
                self.font.as_ref().map(|(_, size)| *size).unwrap_or(12.0);
        }
        self.glyphs += glyphs;
    }

    fn leading(&self) -> f64 {
        self.font
            .as_ref()
            .map(|(_, size)| size * 1.2)
            .unwrap_or(12.0)
    }
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

fn extract_matrix(operands: &[Operand]) -> Result<[f64; 6]> {
    if operands.len() < 6 {
        return Err(anyhow!("matrix operand missing components"));
    }
    let mut values = [0.0; 6];
    for (idx, op) in operands.iter().take(6).enumerate() {
        values[idx] = match op {
            Operand::Number(n) => *n,
            _ => return Err(anyhow!("matrix operand was not numeric")),
        };
    }
    Ok(values)
}

fn count_glyphs(operands: &[Operand]) -> usize {
    operands
        .iter()
        .map(|op| match op {
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

fn bbox_from_tm(tm: &[f64; 6], glyphs: usize, font_size: f64) -> [f64; 4] {
    let width = glyphs as f64 * font_size * 0.6;
    [tm[4], tm[5], tm[4] + width, tm[5] + font_size]
}

fn bbox_from_cm(cm: &[f64; 6]) -> [f64; 4] {
    let width = cm[0].abs();
    let height = cm[3].abs();
    [cm[4], cm[5], cm[4] + width, cm[5] + height]
}
