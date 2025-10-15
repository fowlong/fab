//! Extraction pipeline for building the editing IR.

use std::collections::HashMap;
use std::convert::TryFrom;

use anyhow::{anyhow, Context, Result};
use lopdf::{Document, Object, ObjectId, Stream};

use crate::pdf::content::{tokenize_stream, ByteRange, ContentToken, Operand, Operator};
use crate::types::{
    DocumentIR, FontInfo, ImageObject, Matrix6, PageIR, PageObject, Span, TextObject,
};

#[derive(Debug, Clone)]
pub struct StreamCache {
    pub id: u32,
    pub stream: Stream,
    pub content: Vec<u8>,
    pub tokens: Vec<ContentToken>,
}

#[derive(Debug, Clone)]
pub struct TextMeta {
    pub stream_obj: u32,
    pub bt_span: Span,
    pub bt_op_end: u32,
    pub tm_span: Option<ByteRange>,
    pub tm_matrix: Matrix6,
    pub font: FontInfo,
    pub bbox: Option<[f64; 4]>,
}

#[derive(Debug, Clone)]
pub struct ImageMeta {
    pub stream_obj: u32,
    pub cm_span: ByteRange,
    pub cm_matrix: Matrix6,
    pub name: String,
}

#[derive(Debug, Clone)]
pub enum CachedObject {
    Text(TextMeta),
    Image(ImageMeta),
}

#[derive(Debug, Clone)]
pub struct PageCache {
    pub page_ir: PageIR,
    pub page_id: ObjectId,
    pub streams: Vec<StreamCache>,
    pub objects: HashMap<String, CachedObject>,
}

#[derive(Debug, Clone)]
pub struct ExtractionResult {
    pub document_ir: DocumentIR,
    pub page_cache: PageCache,
}

pub fn extract_page_zero(doc: &Document) -> Result<ExtractionResult> {
    let (&page_number, &page_id) = doc
        .get_pages()
        .iter()
        .next()
        .context("document has no pages")?;
    anyhow::ensure!(page_number == 1, "expected first page to have index 1");

    let page_dict = doc.get_dictionary(page_id)?;
    let media_box = resolve_media_box(doc, page_dict)?;
    let width_pt = media_box[2] - media_box[0];
    let height_pt = media_box[3] - media_box[1];

    let content_ids = doc.get_page_contents(page_id);
    let mut streams = Vec::new();
    for id in &content_ids {
        let raw_stream = doc.get_object(*id)?.as_stream()?.clone();
        let content = raw_stream
            .decompressed_content()
            .unwrap_or_else(|_| raw_stream.content.clone());
        let tokens = tokenize_stream(&content)?;
        streams.push(StreamCache {
            id: id.0,
            stream: raw_stream,
            content,
            tokens,
        });
    }

    let resource_dicts = collect_resource_dicts(doc, page_id)?;
    let image_map = collect_image_xobjects(doc, &resource_dicts);

    let mut objects = Vec::new();
    let mut object_meta = HashMap::new();
    let mut text_counter = 0usize;
    let mut image_counter = 0usize;

    for stream in &streams {
        process_stream(
            stream,
            &mut objects,
            &mut object_meta,
            &mut text_counter,
            &mut image_counter,
            &image_map,
        )?;
    }

    let page_ir = PageIR {
        index: 0,
        width_pt,
        height_pt,
        objects,
    };

    let document_ir = DocumentIR {
        pages: vec![page_ir.clone()],
    };

    Ok(ExtractionResult {
        document_ir,
        page_cache: PageCache {
            page_ir,
            page_id,
            streams,
            objects: object_meta,
        },
    })
}

fn resolve_media_box(doc: &Document, page_dict: &lopdf::Dictionary) -> Result<[f64; 4]> {
    let media = page_dict
        .get(b"MediaBox")
        .or_else(|_| page_dict.get(b"CropBox"))?;
    let resolved = resolve_object(doc, media)?;
    let array = resolved.as_array().context("MediaBox must be an array")?;
    anyhow::ensure!(array.len() == 4, "MediaBox must contain four numbers");
    let mut values = [0.0; 4];
    for (idx, entry) in array.iter().enumerate() {
        let number = resolve_object(doc, entry)?;
        values[idx] = match number {
            Object::Integer(v) => v as f64,
            Object::Real(v) => v as f64,
            _ => anyhow::bail!("MediaBox entry must be numeric"),
        };
    }
    Ok(values)
}

fn collect_resource_dicts(doc: &Document, page_id: ObjectId) -> Result<Vec<lopdf::Dictionary>> {
    let mut dicts = Vec::new();
    let (direct, inherited) = doc.get_page_resources(page_id);
    if let Some(dict) = direct {
        dicts.push(dict.clone());
    }
    for id in inherited {
        if let Ok(dict) = doc.get_dictionary(id) {
            dicts.push(dict.clone());
        }
    }
    Ok(dicts)
}

fn collect_image_xobjects(
    doc: &Document,
    dicts: &[lopdf::Dictionary],
) -> HashMap<String, ObjectId> {
    let mut images = HashMap::new();
    for dict in dicts {
        if let Ok(xobj) = dict.get(b"XObject") {
            match xobj {
                Object::Dictionary(map) => {
                    for (name, value) in map.iter() {
                        if let Ok(id) = value.as_reference() {
                            if is_image_xobject(doc, id) {
                                images.insert(String::from_utf8_lossy(name).to_string(), id);
                            }
                        }
                    }
                }
                Object::Reference(id) => {
                    if let Ok(map) = doc.get_dictionary(*id) {
                        for (name, value) in map.iter() {
                            if let Ok(target) = value.as_reference() {
                                if is_image_xobject(doc, target) {
                                    images
                                        .insert(String::from_utf8_lossy(name).to_string(), target);
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }
    images
}

fn is_image_xobject(doc: &Document, id: ObjectId) -> bool {
    doc.get_dictionary(id)
        .ok()
        .and_then(|dict| dict.get(b"Subtype").ok())
        .and_then(|object| object.as_name_str().ok())
        .map(|name| name == "Image")
        .unwrap_or(false)
}

fn process_stream(
    stream: &StreamCache,
    objects: &mut Vec<PageObject>,
    meta: &mut HashMap<String, CachedObject>,
    text_counter: &mut usize,
    image_counter: &mut usize,
    image_map: &HashMap<String, ObjectId>,
) -> Result<()> {
    let mut ctm_stack: Vec<(Matrix6, Option<(Matrix6, ByteRange)>)> = Vec::new();
    let mut current_ctm = identity_matrix();
    let mut last_cm: Option<(Matrix6, ByteRange)> = None;

    let mut text_state = TextState::default();

    for token in &stream.tokens {
        match token.operator {
            Operator::q => {
                ctm_stack.push((current_ctm, last_cm.clone()));
            }
            Operator::Q => {
                if let Some((matrix, previous)) = ctm_stack.pop() {
                    current_ctm = matrix;
                    last_cm = previous;
                } else {
                    current_ctm = identity_matrix();
                    last_cm = None;
                }
            }
            Operator::Cm => {
                if let Some(matrix) = operands_to_matrix(&token.operands) {
                    current_ctm = multiply_matrices(current_ctm, matrix);
                    last_cm = Some((matrix, token.span));
                }
            }
            Operator::BT => {
                text_state.begin(token.span);
            }
            Operator::ET => {
                if let Some(object) = text_state.finish(token.span.end, stream.id)? {
                    let id = format!("t:{}", *text_counter);
                    *text_counter += 1;
                    objects.push(PageObject::Text(TextObject {
                        id: id.clone(),
                        bt_span: object.bt_span.clone(),
                        tm: object.tm_matrix,
                        font: object.font.clone(),
                        bbox: object.bbox,
                    }));
                    meta.insert(id, CachedObject::Text(object));
                }
            }
            Operator::Tf => {
                if let Some(font) = parse_font(&token.operands) {
                    text_state.font = Some(font);
                }
            }
            Operator::Tm => {
                if let Some(matrix) = operands_to_matrix(&token.operands) {
                    text_state.tm_matrix = matrix;
                    text_state.tm_span = Some(token.span);
                }
            }
            Operator::Td => {
                if token.operands.len() >= 2 {
                    let tx = operand_number(&token.operands[0]).unwrap_or(0.0);
                    let ty = operand_number(&token.operands[1]).unwrap_or(0.0);
                    let translation = [1.0, 0.0, 0.0, 1.0, tx, ty];
                    text_state.tm_matrix = multiply_matrices(text_state.tm_matrix, translation);
                }
            }
            Operator::TStar => {
                // Ignored in MVP; acts like Td(0, -leading). We do not track leading yet.
            }
            Operator::Tj => {
                text_state.register_text(&token.operands, &current_ctm);
            }
            Operator::TJ => {
                text_state.register_text(&token.operands, &current_ctm);
            }
            Operator::Do => {
                if let Some(name) = token.operands.get(0).and_then(|op| match op {
                    Operand::Name(name) => Some(name.clone()),
                    _ => None,
                }) {
                    if image_map.contains_key(&name) {
                        if let Some((cm_matrix, cm_span)) = last_cm.clone() {
                            let id = format!("img:{}", *image_counter);
                            *image_counter += 1;
                            let bbox = approximate_image_bbox(&current_ctm);
                            objects.push(PageObject::Image(ImageObject {
                                id: id.clone(),
                                x_object: name.clone(),
                                cm: current_ctm,
                                bbox: Some(bbox),
                            }));
                            meta.insert(
                                id,
                                CachedObject::Image(ImageMeta {
                                    stream_obj: stream.id,
                                    cm_span,
                                    cm_matrix,
                                    name,
                                }),
                            );
                        }
                        last_cm = None;
                    }
                }
            }
            _ => {}
        }
    }

    Ok(())
}

#[derive(Debug, Default)]
struct TextState {
    bt_start: Option<usize>,
    bt_op_end: Option<usize>,
    tm_matrix: Matrix6,
    tm_span: Option<ByteRange>,
    font: Option<FontInfo>,
    glyphs: usize,
    base_matrix: Option<Matrix6>,
}

impl TextState {
    fn begin(&mut self, span: ByteRange) {
        *self = TextState {
            bt_start: Some(span.start),
            bt_op_end: Some(span.end),
            tm_matrix: identity_matrix(),
            tm_span: None,
            font: None,
            glyphs: 0,
            base_matrix: None,
        };
    }

    fn register_text(&mut self, operands: &[Operand], current_ctm: &Matrix6) {
        if self.bt_start.is_none() {
            return;
        }
        if self.base_matrix.is_none() {
            self.base_matrix = Some(self.tm_matrix);
        }
        self.glyphs += estimate_glyph_count(operands);
        let _ = current_ctm; // reserved for future use when combining CTM and Tm
    }

    fn finish(&mut self, end: usize, stream_obj: u32) -> Result<Option<TextMeta>> {
        if self.bt_start.is_none() {
            return Ok(None);
        }
        if self.glyphs == 0 {
            return Ok(None);
        }
        let start = self.bt_start.unwrap();
        let tm_matrix = self.base_matrix.unwrap_or(self.tm_matrix);
        let font = self.font.clone().unwrap_or(FontInfo {
            res_name: String::from("F1"),
            size: 12.0,
        });
        let bbox = approximate_text_bbox(&tm_matrix, &font, self.glyphs);
        self.bt_start = None;
        self.glyphs = 0;
        self.base_matrix = None;
        let bt_op_end = self.bt_op_end.unwrap_or(start);
        let bt_span = Span {
            stream_obj,
            start: u32::try_from(start).context("BT span start overflow")?,
            end: u32::try_from(end).context("BT span end overflow")?,
        };
        Ok(Some(TextMeta {
            stream_obj,
            bt_span,
            bt_op_end: u32::try_from(bt_op_end).context("BT token end overflow")?,
            tm_span: self.tm_span,
            tm_matrix,
            font,
            bbox: Some(bbox),
        }))
    }
}

fn estimate_glyph_count(operands: &[Operand]) -> usize {
    operands
        .iter()
        .map(|operand| match operand {
            Operand::String(bytes) => bytes.len(),
            Operand::Array(items) => estimate_glyph_count(items),
            _ => 0,
        })
        .sum()
}

fn parse_font(operands: &[Operand]) -> Option<FontInfo> {
    if operands.len() < 2 {
        return None;
    }
    let name = match &operands[0] {
        Operand::Name(name) => name.clone(),
        _ => return None,
    };
    let size = operand_number(&operands[1])?;
    Some(FontInfo {
        res_name: name,
        size,
    })
}

fn operand_number(operand: &Operand) -> Option<f64> {
    match operand {
        Operand::Number(value) => Some(*value),
        _ => None,
    }
}

fn operands_to_matrix(operands: &[Operand]) -> Option<Matrix6> {
    if operands.len() < 6 {
        return None;
    }
    let mut matrix = [0.0; 6];
    for (idx, operand) in operands.iter().take(6).enumerate() {
        matrix[idx] = operand_number(operand)?;
    }
    Some(matrix)
}

fn multiply_matrices(lhs: Matrix6, rhs: Matrix6) -> Matrix6 {
    [
        lhs[0] * rhs[0] + lhs[2] * rhs[1],
        lhs[1] * rhs[0] + lhs[3] * rhs[1],
        lhs[0] * rhs[2] + lhs[2] * rhs[3],
        lhs[1] * rhs[2] + lhs[3] * rhs[3],
        lhs[0] * rhs[4] + lhs[2] * rhs[5] + lhs[4],
        lhs[1] * rhs[4] + lhs[3] * rhs[5] + lhs[5],
    ]
}

fn identity_matrix() -> Matrix6 {
    [1.0, 0.0, 0.0, 1.0, 0.0, 0.0]
}

fn approximate_text_bbox(tm: &Matrix6, font: &FontInfo, glyphs: usize) -> [f64; 4] {
    let width = font.size * glyphs as f64 * 0.6;
    let height = font.size;
    normalise_box(tm[4], tm[5], tm[4] + width, tm[5] + height)
}

fn approximate_image_bbox(matrix: &Matrix6) -> [f64; 4] {
    let width = matrix[0];
    let height = matrix[3];
    normalise_box(matrix[4], matrix[5], matrix[4] + width, matrix[5] + height)
}

fn normalise_box(x0: f64, y0: f64, x1: f64, y1: f64) -> [f64; 4] {
    [x0.min(x1), y0.min(y1), x0.max(x1), y0.max(y1)]
}

fn resolve_object<'a>(doc: &'a Document, object: &'a Object) -> Result<Object> {
    Ok(match object {
        Object::Reference(id) => doc.get_object(*id)?.clone(),
        _ => object.clone(),
    })
}
