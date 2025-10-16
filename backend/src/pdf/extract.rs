use std::{
    collections::{HashMap, HashSet},
    ops::Range,
};

use anyhow::{anyhow, Context, Result};
use lopdf::{Dictionary, Document, Object, ObjectId};

use crate::{
    types::{DocumentIR, FontInfo, ImageObject, PageIR, PageObject, StreamSpan, TextObject},
    util::matrix::Matrix2D,
};

use super::content::{self, Operand, Operator};

#[derive(Debug, Clone, Default)]
pub struct DocumentCache {
    pub ir: DocumentIR,
    pub page0: Option<PageState>,
}

#[derive(Debug, Clone)]
pub struct PageState {
    pub context: PageContext,
    pub content_bytes: Vec<u8>,
    pub text_objects: HashMap<String, TextPatch>,
    pub image_objects: HashMap<String, ImagePatch>,
}

#[derive(Debug, Clone)]
pub struct PageContext {
    pub page_id: ObjectId,
    pub width_pt: f64,
    pub height_pt: f64,
    pub primary_stream_obj: u32,
    pub image_xobjects: HashSet<String>,
}

#[derive(Debug, Clone)]
pub struct TextPatch {
    pub span: Range<u32>,
    pub tm: [f64; 6],
    pub tm_range: Option<Range<u32>>,
    pub insert_after_bt: u32,
}

#[derive(Debug, Clone)]
pub struct ImagePatch {
    pub cm: [f64; 6],
    pub cm_range: Range<u32>,
}

pub fn extract_document(doc: &Document) -> Result<DocumentCache> {
    let mut cache = DocumentCache::default();
    let mut pages = Vec::new();

    if let Some((_, &page_id)) = doc.get_pages().iter().next() {
        let (page_ir, state) = extract_page_state(doc, page_id)?;
        pages.push(page_ir.clone());
        cache.ir = DocumentIR { pages };
        cache.page0 = Some(state);
    } else {
        cache.ir = DocumentIR { pages: Vec::new() };
        cache.page0 = None;
    }

    Ok(cache)
}

pub fn inspect_page_with_context(
    context: &PageContext,
    content_bytes: &[u8],
) -> Result<(PageIR, PageState)> {
    let tokens = content::tokenize_stream(context.primary_stream_obj, content_bytes)?;
    let mut text_counter = 0usize;
    let mut image_counter = 0usize;
    let mut objects: Vec<PageObject> = Vec::new();
    let mut text_map: HashMap<String, TextPatch> = HashMap::new();
    let mut image_map: HashMap<String, ImagePatch> = HashMap::new();

    let mut graphics = GraphicsState::default();
    let mut stack: Vec<GraphicsState> = Vec::new();
    let mut text_state: Option<TextRunState> = None;

    for token in tokens.iter() {
        match token.operator {
            Operator::q => {
                stack.push(graphics.clone());
            }
            Operator::Q => {
                graphics = stack.pop().unwrap_or_default();
            }
            Operator::Cm => {
                if let Some(matrix) = matrix_from_operands(&token.operands) {
                    graphics.ctm = graphics.ctm.multiply(matrix);
                    graphics.last_cm = Some(CmRecord {
                        matrix,
                        range: token.range.clone(),
                    });
                }
            }
            Operator::Bt => {
                text_state = Some(TextRunState {
                    id: None,
                    bt_start: token.range.start,
                    insert_after_bt: token.range.end,
                    stream_obj: token.stream_obj,
                    tm: Matrix2D::identity(),
                    tm_range: None,
                    base_tm: None,
                    font: None,
                    glyph_estimate: 0,
                    has_text: false,
                    ctm_at_bt: graphics.ctm,
                    leading: None,
                });
            }
            Operator::Tf => {
                if let Some(state) = text_state.as_mut() {
                    if token.operands.len() >= 2 {
                        let name = match &token.operands[0] {
                            Operand::Name(name) => name.clone(),
                            _ => "F0".to_string(),
                        };
                        let size = match token.operands[1] {
                            Operand::Number(value) => value,
                            _ => 12.0,
                        };
                        state.font = Some(FontInfo {
                            res_name: name,
                            size,
                        });
                        state.leading.get_or_insert(size);
                    }
                }
            }
            Operator::Tm => {
                if let Some(state) = text_state.as_mut() {
                    if let Some(matrix) = matrix_from_operands(&token.operands) {
                        state.tm = matrix;
                        state.tm_range = Some(token.range.clone());
                        if state.base_tm.is_none() {
                            state.base_tm = Some(matrix);
                        }
                    }
                }
            }
            Operator::Td => {
                if let Some(state) = text_state.as_mut() {
                    if token.operands.len() >= 2 {
                        let tx = match token.operands[0] {
                            Operand::Number(value) => value,
                            _ => 0.0,
                        };
                        let ty = match token.operands[1] {
                            Operand::Number(value) => value,
                            _ => 0.0,
                        };
                        state.tm = state.tm.multiply(translation_matrix(tx, ty));
                        if state.base_tm.is_none() {
                            state.base_tm = Some(state.tm);
                        }
                    }
                }
            }
            Operator::TStar => {
                if let Some(state) = text_state.as_mut() {
                    let leading = state
                        .leading
                        .or_else(|| state.font.as_ref().map(|f| f.size))
                        .unwrap_or(0.0);
                    state.tm = state.tm.multiply(translation_matrix(0.0, -leading));
                    if state.base_tm.is_none() {
                        state.base_tm = Some(state.tm);
                    }
                }
            }
            Operator::Tj | Operator::TJ => {
                if let Some(state) = text_state.as_mut() {
                    if state.id.is_none() {
                        let id = format!("t:{text_counter}");
                        text_counter += 1;
                        state.id = Some(id);
                    }
                    state.has_text = true;
                    let glyphs = estimate_glyphs(&token.operands);
                    state.glyph_estimate = state.glyph_estimate.saturating_add(glyphs);
                    if state.base_tm.is_none() {
                        state.base_tm = Some(state.tm);
                    }
                }
            }
            Operator::Et => {
                if let Some(state) = text_state.take() {
                    if state.has_text {
                        let font = state.font.unwrap_or(FontInfo {
                            res_name: "F0".into(),
                            size: 12.0,
                        });
                        let id = state.id.unwrap_or_else(|| {
                            let id = format!("t:{text_counter}");
                            text_counter += 1;
                            id
                        });
                        let base_tm = state.base_tm.unwrap_or(state.tm);
                        let glyph_estimate = state.glyph_estimate.max(1);
                        let tm_array = matrix_to_array(base_tm);
                        let bbox = bbox_for_text(base_tm, state.ctm_at_bt, &font, glyph_estimate);
                        let span = StreamSpan {
                            stream_obj: state.stream_obj,
                            start: state.bt_start,
                            end: token.range.end,
                        };
                        let text_object = TextObject {
                            id: id.clone(),
                            bt_span: span,
                            tm: tm_array,
                            font: font.clone(),
                            bbox,
                        };
                        objects.push(PageObject::Text(text_object));
                        text_map.insert(
                            id,
                            TextPatch {
                                span: state.bt_start..token.range.end,
                                tm: tm_array,
                                tm_range: state.tm_range.clone(),
                                insert_after_bt: state.insert_after_bt,
                            },
                        );
                    }
                }
            }
            Operator::Do => {
                if let Some(name) = token.operands.first() {
                    if let Operand::Name(resource) = name {
                        if context.image_xobjects.contains(resource) {
                            let id = format!("img:{image_counter}");
                            image_counter += 1;
                            let cm_record = graphics.last_cm.clone().unwrap_or(CmRecord {
                                matrix: Matrix2D::identity(),
                                range: token.range.clone(),
                            });
                            let cm_array = matrix_to_array(cm_record.matrix);
                            let bbox = bbox_from_matrix(graphics.ctm);
                            let image_object = ImageObject {
                                id: id.clone(),
                                x_object: resource.clone(),
                                cm: cm_array,
                                bbox,
                            };
                            objects.push(PageObject::Image(image_object));
                            image_map.insert(
                                id,
                                ImagePatch {
                                    cm: cm_array,
                                    cm_range: cm_record.range,
                                },
                            );
                        }
                    }
                }
            }
            _ => {}
        }
    }

    let page_ir = PageIR {
        index: 0,
        width_pt: context.width_pt,
        height_pt: context.height_pt,
        objects,
    };

    let state = PageState {
        context: context.clone(),
        content_bytes: content_bytes.to_vec(),
        text_objects: text_map,
        image_objects: image_map,
    };

    Ok((page_ir, state))
}

fn extract_page_state(doc: &Document, page_id: ObjectId) -> Result<(PageIR, PageState)> {
    let page_dict = doc
        .get_dictionary(page_id)
        .context("page dictionary missing")?;
    let (width_pt, height_pt) = page_dimensions(page_dict)?;
    let stream_ids = doc.get_page_contents(page_id);
    let primary_stream_obj = stream_ids.first().map(|id| id.0).unwrap_or(0);

    let mut combined = Vec::new();
    for (idx, stream_id) in stream_ids.iter().enumerate() {
        let stream = doc
            .get_object(*stream_id)
            .and_then(Object::as_stream)
            .context("content stream missing")?;
        let mut data = match stream.decompressed_content() {
            Ok(bytes) => bytes,
            Err(_) => stream.content.clone(),
        };
        combined.append(&mut data);
        if idx + 1 != stream_ids.len() && !combined.ends_with(b"\n") {
            combined.push(b'\n');
        }
    }

    let image_xobjects = gather_image_xobjects(doc, page_id)?;
    let context = PageContext {
        page_id,
        width_pt,
        height_pt,
        primary_stream_obj,
        image_xobjects,
    };

    inspect_page_with_context(&context, &combined)
}

fn gather_image_xobjects(doc: &Document, page_id: ObjectId) -> Result<HashSet<String>> {
    let mut names = HashSet::new();
    let (maybe_dict, resource_ids) = doc.get_page_resources(page_id);
    if let Some(dict) = maybe_dict {
        collect_xobjects(doc, dict, &mut names)?;
    }
    for res_id in resource_ids {
        let dict = doc.get_dictionary(res_id)?;
        collect_xobjects(doc, dict, &mut names)?;
    }
    Ok(names)
}

fn collect_xobjects(doc: &Document, dict: &Dictionary, names: &mut HashSet<String>) -> Result<()> {
    if let Ok(object) = dict.get(b"XObject") {
        match object {
            Object::Reference(id) => {
                let referenced = doc.get_dictionary(*id)?;
                collect_xobjects(doc, referenced, names)?;
            }
            Object::Dictionary(map) => {
                for (key, value) in map.iter() {
                    let target_dict = match value {
                        Object::Reference(id) => doc.get_dictionary(*id).ok(),
                        Object::Dictionary(inner) => Some(inner),
                        _ => None,
                    };
                    if let Some(target) = target_dict {
                        if let Ok(Object::Name(subtype)) = target.get(b"Subtype") {
                            if subtype == b"Image" {
                                names.insert(String::from_utf8_lossy(key).to_string());
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
    Ok(())
}

fn page_dimensions(page: &Dictionary) -> Result<(f64, f64)> {
    let media_box = page
        .get(b"MediaBox")
        .or_else(|_| page.get(b"CropBox"))
        .context("page has no MediaBox or CropBox")?;
    if let Object::Array(values) = media_box {
        if values.len() >= 4 {
            let width = value_as_number(&values[2])? - value_as_number(&values[0])?;
            let height = value_as_number(&values[3])? - value_as_number(&values[1])?;
            return Ok((width, height));
        }
    }
    Err(anyhow!("invalid MediaBox"))
}

fn value_as_number(object: &Object) -> Result<f64> {
    match object {
        Object::Integer(value) => Ok(*value as f64),
        Object::Real(value) => Ok(*value),
        _ => Err(anyhow!("expected numeric value")),
    }
}

fn translation_matrix(tx: f64, ty: f64) -> Matrix2D {
    Matrix2D {
        a: 1.0,
        b: 0.0,
        c: 0.0,
        d: 1.0,
        e: tx,
        f: ty,
    }
}

fn matrix_from_operands(operands: &[Operand]) -> Option<Matrix2D> {
    if operands.len() != 6 {
        return None;
    }
    let mut values = [0.0; 6];
    for (idx, operand) in operands.iter().enumerate() {
        if let Operand::Number(value) = operand {
            values[idx] = *value;
        } else {
            return None;
        }
    }
    Some(Matrix2D {
        a: values[0],
        b: values[1],
        c: values[2],
        d: values[3],
        e: values[4],
        f: values[5],
    })
}

fn matrix_to_array(matrix: Matrix2D) -> [f64; 6] {
    [matrix.a, matrix.b, matrix.c, matrix.d, matrix.e, matrix.f]
}

fn bbox_from_matrix(matrix: Matrix2D) -> [f64; 4] {
    let points = [
        (0.0_f64, 0.0_f64),
        (1.0_f64, 0.0_f64),
        (0.0_f64, 1.0_f64),
        (1.0_f64, 1.0_f64),
    ];
    let mut min_x = f64::INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut max_y = f64::NEG_INFINITY;
    for (x, y) in points {
        let px = matrix.a * x + matrix.c * y + matrix.e;
        let py = matrix.b * x + matrix.d * y + matrix.f;
        min_x = min_x.min(px);
        max_x = max_x.max(px);
        min_y = min_y.min(py);
        max_y = max_y.max(py);
    }
    if !min_x.is_finite() || !min_y.is_finite() || !max_x.is_finite() || !max_y.is_finite() {
        [0.0, 0.0, 0.0, 0.0]
    } else {
        [min_x, min_y, max_x, max_y]
    }
}

fn bbox_for_text(
    base_tm: Matrix2D,
    ctm: Matrix2D,
    font: &FontInfo,
    glyph_estimate: usize,
) -> [f64; 4] {
    let effective = ctm.multiply(base_tm);
    let width = font.size * glyph_estimate as f64 * 0.6;
    let height = font.size * 1.2;
    let x0 = effective.e;
    let y_baseline = effective.f;
    let x1 = x0 + width;
    let y0 = y_baseline - height * 0.8;
    let y1 = y_baseline + height * 0.2;
    [x0, y0, x1, y1]
}

fn estimate_glyphs(operands: &[Operand]) -> usize {
    operands
        .iter()
        .map(|operand| match operand {
            Operand::String(bytes) => estimate_text_units(bytes),
            Operand::Array(entries) => entries
                .iter()
                .map(|item| match item {
                    Operand::String(bytes) => estimate_text_units(bytes),
                    _ => 0,
                })
                .sum(),
            _ => 0,
        })
        .sum()
}

fn estimate_text_units(bytes: &[u8]) -> usize {
    if bytes.is_empty() {
        return 0;
    }
    let zero_count = bytes.iter().filter(|&&b| b == 0).count();
    if zero_count * 2 == bytes.len() && bytes.len() % 2 == 0 {
        bytes.len() / 2
    } else {
        bytes.len()
    }
}

#[derive(Debug, Clone)]
struct GraphicsState {
    ctm: Matrix2D,
    last_cm: Option<CmRecord>,
}

impl Default for GraphicsState {
    fn default() -> Self {
        Self {
            ctm: Matrix2D::identity(),
            last_cm: None,
        }
    }
}

#[derive(Debug, Clone)]
struct CmRecord {
    matrix: Matrix2D,
    range: Range<u32>,
}

#[derive(Debug, Clone)]
struct TextRunState {
    id: Option<String>,
    bt_start: u32,
    insert_after_bt: u32,
    stream_obj: u32,
    tm: Matrix2D,
    tm_range: Option<Range<u32>>,
    base_tm: Option<Matrix2D>,
    font: Option<FontInfo>,
    glyph_estimate: usize,
    has_text: bool,
    ctm_at_bt: Matrix2D,
    leading: Option<f64>,
}
