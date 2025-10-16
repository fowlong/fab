use std::collections::HashMap;

use anyhow::{anyhow, Context, Result};
use lopdf::{Dictionary, Document, Object};

use crate::types::{DocumentIR, FontInfo, ImageObject, PageIR, PageObject, StreamSpan, TextObject};

use super::content::{
    matrix_from_operands, multiply, rect_from_matrix, serialize_operations, tokenize_stream,
    Matrix, Operand, Operation, Operator,
};

#[derive(Debug, Clone)]
pub struct StreamState {
    pub object_id: u32,
    pub decoded: Vec<u8>,
    pub operations: Vec<Operation>,
}

#[derive(Debug, Clone)]
pub enum PageContents {
    Single(u32),
    Array(Vec<u32>),
}

#[derive(Debug, Clone)]
pub struct PageZeroState {
    pub page_id: u32,
    pub contents: PageContents,
    pub width_pt: f64,
    pub height_pt: f64,
}

#[derive(Debug, Clone)]
pub struct TextCache {
    pub stream_obj: u32,
    pub bt_token_index: usize,
    pub et_token_index: usize,
    pub tm_token_index: Option<usize>,
    pub has_explicit_tm: bool,
    pub tm: Matrix,
}

#[derive(Debug, Clone)]
pub struct ImageCache {
    pub stream_obj: u32,
    pub cm_token_index: Option<usize>,
    pub do_token_index: usize,
    pub cm: Matrix,
    pub name: String,
}

#[derive(Debug, Clone)]
pub enum CachedObject {
    Text(TextCache),
    Image(ImageCache),
}

#[derive(Debug, Clone)]
pub struct ExtractResult {
    pub ir: DocumentIR,
    pub page_zero: PageZeroState,
    pub streams: HashMap<u32, StreamState>,
    pub lookup: HashMap<String, CachedObject>,
}

const IDENTITY: Matrix = [1.0, 0.0, 0.0, 1.0, 0.0, 0.0];

pub fn extract_document(doc: &Document) -> Result<ExtractResult> {
    let (page_id, page_dict) = first_page(doc)?;
    let (width_pt, height_pt) = page_dimensions(&page_dict)?;
    let contents = page_contents(doc, &page_dict)?;

    let mut stream_states = HashMap::new();
    let mut stream_order = Vec::new();
    match &contents {
        PageContents::Single(id) => {
            let state = load_stream(doc, *id)?;
            stream_order.push(*id);
            stream_states.insert(*id, state);
        }
        PageContents::Array(ids) => {
            for id in ids {
                let state = load_stream(doc, *id)?;
                stream_order.push(*id);
                stream_states.insert(*id, state);
            }
        }
    }

    let mut objects = Vec::new();
    let mut lookup = HashMap::new();
    let mut text_index = 0usize;
    let mut image_index = 0usize;

    for stream_id in &stream_order {
        let state = stream_states
            .get(stream_id)
            .context("missing stream state")?;
        let mut graphics_stack = vec![GraphicsState::default()];
        let mut current_index = 0usize;
        let mut text_state: Option<TextRunState> = None;

        for (op_index, operation) in state.operations.iter().enumerate() {
            match &operation.operator {
                Operator::q => {
                    let cloned = graphics_stack[current_index].clone();
                    graphics_stack.push(cloned);
                    current_index = graphics_stack.len() - 1;
                }
                Operator::Q => {
                    graphics_stack.pop();
                    if graphics_stack.is_empty() {
                        graphics_stack.push(GraphicsState::default());
                    }
                    current_index = graphics_stack.len() - 1;
                }
                Operator::Cm => {
                    if let Some(matrix) = operation.matrix() {
                        let state_entry = graphics_stack
                            .get_mut(current_index)
                            .expect("graphics state entry");
                        state_entry.ctm = multiply(&state_entry.ctm, &matrix);
                        state_entry.last_cm_index = Some(op_index);
                    }
                }
                Operator::BT => {
                    let ctm = graphics_stack[current_index].ctm;
                    text_state = Some(TextRunState::new(
                        *stream_id,
                        op_index,
                        operation.range.start,
                        ctm,
                    ));
                }
                Operator::ET => {
                    if let Some(state_run) = text_state.take() {
                        if !state_run.seen_text {
                            continue;
                        }
                        let tm = state_run.first_tm.unwrap_or(state_run.text_matrix);
                        let font = state_run.font.unwrap_or(FontInfo {
                            res_name: "Unknown".into(),
                            size: 12.0,
                        });
                        let bbox =
                            text_bbox(&tm, &state_run.ctm_at_bt, font.size, state_run.glyph_count);
                        let id = format!("t:{}", text_index);
                        text_index += 1;
                        let span = StreamSpan {
                            stream_obj: *stream_id,
                            start: state_run.bt_start.try_into().unwrap_or(u32::MAX),
                            end: operation.range.end.try_into().unwrap_or(u32::MAX),
                        };
                        objects.push(PageObject::Text(TextObject {
                            id: id.clone(),
                            bt_span: span,
                            tm,
                            font: font.clone(),
                            bbox,
                        }));
                        lookup.insert(
                            id,
                            CachedObject::Text(TextCache {
                                stream_obj: *stream_id,
                                bt_token_index: state_run.bt_token_index,
                                et_token_index: op_index,
                                tm_token_index: state_run.tm_token_index,
                                has_explicit_tm: state_run.tm_token_index.is_some(),
                                tm,
                            }),
                        );
                    }
                }
                Operator::Tf => {
                    if let Some(text_run) = text_state.as_mut() {
                        if let Some((res, size)) = parse_tf(&operation.operands) {
                            text_run.font = Some(FontInfo {
                                res_name: res,
                                size,
                            });
                            text_run.leading = size;
                        }
                    }
                }
                Operator::Tm => {
                    if let Some(text_run) = text_state.as_mut() {
                        if let Some(matrix) = matrix_from_operands(&operation.operands) {
                            text_run.text_matrix = matrix;
                            text_run.line_matrix = matrix;
                            if !text_run.seen_text {
                                text_run.tm_token_index = Some(op_index);
                                text_run.first_tm = Some(matrix);
                            }
                        }
                    }
                }
                Operator::Td => {
                    if let Some(text_run) = text_state.as_mut() {
                        if let Some((tx, ty)) = parse_td(&operation.operands) {
                            let translation = [1.0, 0.0, 0.0, 1.0, tx, ty];
                            text_run.text_matrix = multiply(&text_run.text_matrix, &translation);
                            text_run.line_matrix = text_run.text_matrix;
                        }
                    }
                }
                Operator::TStar => {
                    if let Some(text_run) = text_state.as_mut() {
                        let translation = [1.0, 0.0, 0.0, 1.0, 0.0, -text_run.leading];
                        text_run.text_matrix = multiply(&text_run.text_matrix, &translation);
                        text_run.line_matrix = text_run.text_matrix;
                    }
                }
                Operator::Tj | Operator::TJ => {
                    if let Some(text_run) = text_state.as_mut() {
                        let glyphs = count_glyphs(&operation.operands);
                        if !text_run.seen_text {
                            text_run.first_tm = Some(text_run.text_matrix);
                            text_run.seen_text = true;
                        }
                        text_run.glyph_count += glyphs.max(1);
                    }
                }
                Operator::Do => {
                    if let Some(name) = operation.operands.first().and_then(Operand::as_name) {
                        let state_entry = &graphics_stack[current_index];
                        let bbox = rect_from_matrix(&state_entry.ctm, 1.0, 1.0);
                        let id = format!("img:{}", image_index);
                        image_index += 1;
                        objects.push(PageObject::Image(ImageObject {
                            id: id.clone(),
                            x_object: name.to_string(),
                            cm: state_entry.ctm,
                            bbox,
                        }));
                        lookup.insert(
                            id,
                            CachedObject::Image(ImageCache {
                                stream_obj: *stream_id,
                                cm_token_index: state_entry.last_cm_index,
                                do_token_index: op_index,
                                cm: state_entry.ctm,
                                name: name.to_string(),
                            }),
                        );
                    }
                }
                _ => {}
            }
        }
    }

    let page_ir = PageIR {
        index: 0,
        width_pt,
        height_pt,
        objects,
    };

    Ok(ExtractResult {
        ir: DocumentIR {
            pages: vec![page_ir],
        },
        page_zero: PageZeroState {
            page_id,
            contents,
            width_pt,
            height_pt,
        },
        streams: stream_states,
        lookup,
    })
}

fn first_page(doc: &Document) -> Result<(u32, Dictionary)> {
    let pages = doc.get_pages();
    let (&page_number, &object_id) = pages.iter().next().ok_or_else(|| anyhow!("empty PDF"))?;
    let id = object_id.0;
    let dict = doc.get_dictionary(object_id)?.clone();
    let _ = page_number;
    Ok((id, dict))
}

fn page_dimensions(dict: &Dictionary) -> Result<(f64, f64)> {
    let media_box = dict
        .get(b"MediaBox")
        .or_else(|_| dict.get(b"CropBox"))?
        .as_array()?;
    if media_box.len() < 4 {
        return Err(anyhow!("MediaBox missing dimensions"));
    }
    let x0 = media_box[0].as_f64()?;
    let y0 = media_box[1].as_f64()?;
    let x1 = media_box[2].as_f64()?;
    let y1 = media_box[3].as_f64()?;
    Ok((x1 - x0, y1 - y0))
}

fn page_contents(doc: &Document, dict: &Dictionary) -> Result<PageContents> {
    let contents = dict.get(b"Contents")?;
    match contents {
        Object::Reference(id) => Ok(PageContents::Single(id.0)),
        Object::Array(items) => {
            let mut ids = Vec::new();
            for item in items {
                if let Ok(reference) = item.as_reference() {
                    ids.push(reference.0);
                }
            }
            Ok(PageContents::Array(ids))
        }
        Object::Stream(_) => Err(anyhow!("inline content streams are unsupported")),
        _ => Err(anyhow!("unsupported /Contents form")),
    }
}

fn load_stream(doc: &Document, object_id: u32) -> Result<StreamState> {
    let object = doc.get_object((object_id, 0))?;
    let stream = object.as_stream()?;
    let decoded = stream.decode_content()?;
    let operations = tokenize_stream(&decoded)?;
    Ok(StreamState {
        object_id,
        decoded,
        operations,
    })
}

#[derive(Debug, Clone)]
struct GraphicsState {
    ctm: Matrix,
    last_cm_index: Option<usize>,
}

impl Default for GraphicsState {
    fn default() -> Self {
        Self {
            ctm: IDENTITY,
            last_cm_index: None,
        }
    }
}

#[derive(Debug, Clone)]
struct TextRunState {
    stream_obj: u32,
    bt_token_index: usize,
    bt_start: usize,
    tm_token_index: Option<usize>,
    text_matrix: Matrix,
    line_matrix: Matrix,
    first_tm: Option<Matrix>,
    font: Option<FontInfo>,
    glyph_count: usize,
    seen_text: bool,
    leading: f64,
    ctm_at_bt: Matrix,
}

impl TextRunState {
    fn new(stream_obj: u32, bt_token_index: usize, start: usize, ctm: Matrix) -> Self {
        Self {
            stream_obj,
            bt_token_index,
            bt_start: start,
            tm_token_index: None,
            text_matrix: IDENTITY,
            line_matrix: IDENTITY,
            first_tm: None,
            font: None,
            glyph_count: 0,
            seen_text: false,
            leading: 0.0,
            ctm_at_bt: ctm,
        }
    }
}

fn parse_tf(operands: &[Operand]) -> Option<(String, f64)> {
    if operands.len() != 2 {
        return None;
    }
    let res = operands[0].as_name()?.to_string();
    let size = operands[1].as_number()?;
    Some((res, size))
}

fn parse_td(operands: &[Operand]) -> Option<(f64, f64)> {
    if operands.len() != 2 {
        return None;
    }
    let tx = operands[0].as_number()?;
    let ty = operands[1].as_number()?;
    Some((tx, ty))
}

fn count_glyphs(operands: &[Operand]) -> usize {
    operands
        .iter()
        .map(|operand| match operand {
            Operand::String(bytes) => bytes.len().max(1),
            Operand::Array(items) => items
                .iter()
                .map(|item| match item {
                    Operand::String(bytes) => bytes.len().max(1),
                    _ => 0,
                })
                .sum(),
            _ => 0,
        })
        .sum()
}

fn text_bbox(tm: &Matrix, ctm: &Matrix, font_size: f64, glyph_count: usize) -> [f64; 4] {
    let glyphs = glyph_count.max(1) as f64;
    let width = font_size * glyphs * 0.5;
    let height = font_size.abs();
    let combined = multiply(ctm, tm);
    rect_from_matrix(&combined, width, height)
}

pub fn rebuild_stream_bytes(state: &StreamState) -> Vec<u8> {
    serialize_operations(&state.operations)
}
