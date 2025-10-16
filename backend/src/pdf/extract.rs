//! Extract a light-weight IR for page 0 (text runs and image XObjects).

use std::collections::{HashMap, HashSet};

use anyhow::{anyhow, Result};
use lopdf::{Dictionary, Document, Object, ObjectId};

use super::content::{tokenize_stream, Operand, Operator};
use crate::types::{DocumentIR, FontInfo, ImageObject, PageIR, PageObject, Span, TextObject};

use super::loader::PageInfo;

#[derive(Clone)]
pub struct TokenPosition {
    pub stream_index: usize,
    pub token_index: usize,
    pub start: usize,
    pub end: usize,
}

#[derive(Clone)]
pub struct TextMeta {
    pub stream_index: usize,
    pub tm: [f64; 6],
    pub bt_span: Span,
    pub font: FontInfo,
    pub bbox: [f64; 4],
    pub tm_token: Option<TokenPosition>,
    pub bt_token: TokenPosition,
}

#[derive(Clone)]
pub struct ImageMeta {
    pub stream_index: usize,
    pub cm: [f64; 6],
    pub bbox: [f64; 4],
    pub cm_token: Option<TokenPosition>,
    pub x_object: String,
}

#[derive(Clone)]
pub struct StreamCache {
    pub object_id: ObjectId,
    pub bytes: Vec<u8>,
}

#[derive(Clone)]
pub struct PageCache {
    pub page_ir: PageIR,
    pub text: HashMap<String, TextMeta>,
    pub images: HashMap<String, ImageMeta>,
    pub streams: Vec<StreamCache>,
}

pub fn extract_page0(
    document: &Document,
    page_info: &PageInfo,
    overrides: &HashMap<u32, Vec<u8>>,
) -> Result<PageCache> {
    let page_dict = document
        .get_dictionary(page_info.page_id)
        .map_err(|_| anyhow!("failed to resolve page 0"))?;
    let (width_pt, height_pt) = page_media_box(&page_dict)?;
    let mut streams = Vec::new();
    for object_id in &page_info.contents {
        let stream_bytes = overrides
            .get(&object_id.0)
            .cloned()
            .unwrap_or_else(|| load_stream_bytes(document, *object_id));
        streams.push(StreamCache {
            object_id: *object_id,
            bytes: stream_bytes,
        });
    }

    let image_names = collect_image_xobjects(document, page_info.page_id)?;
    let mut text_counter = 0usize;
    let mut image_counter = 0usize;
    let mut objects = Vec::new();
    let mut text_meta = HashMap::new();
    let mut image_meta = HashMap::new();

    let mut graphics_state = GraphicsState::default();
    let mut graphics_stack: Vec<GraphicsState> = Vec::new();
    let mut text_state: Option<TextState> = None;
    let mut current_font = FontInfo {
        res_name: String::from("F0"),
        size: 12.0,
    };

    for (stream_index, stream) in streams.iter().enumerate() {
        graphics_state = GraphicsState::default();
        graphics_stack.clear();
        text_state = None;
        current_font = FontInfo {
            res_name: String::from("F0"),
            size: 12.0,
        };
        let tokens = tokenize_stream(&stream.bytes)?;
        for (token_index, token) in tokens.iter().enumerate() {
            let position = TokenPosition {
                stream_index,
                token_index,
                start: token.start,
                end: token.end,
            };
            match token.operator {
                Operator::q => {
                    graphics_stack.push(graphics_state.clone());
                }
                Operator::Q => {
                    graphics_state = graphics_stack.pop().unwrap_or_default();
                }
                Operator::Cm => {
                    if let Some(matrix) = parse_matrix(&token.operands) {
                        graphics_state.ctm = multiply(graphics_state.ctm, matrix);
                        graphics_state.last_cm_token = Some(position);
                    }
                }
                Operator::BT => {
                    text_state = Some(TextState {
                        bt_token: position.clone(),
                        current_tm: IDENTITY,
                        last_tm_token: None,
                        run: None,
                    });
                }
                Operator::ET => {
                    if let Some(state) = &mut text_state {
                        if let Some(run) = state.run.take() {
                            let bbox = estimate_text_bbox(&run.tm, run.font.size, run.text_len);
                            let span = Span {
                                stream_obj: streams[run.stream_index].object_id.0,
                                start: run.span_start as u32,
                                end: run.span_end as u32,
                            };
                            let text_id = format!("t:{}", text_counter);
                            text_counter += 1;
                            let text_obj = TextObject {
                                id: text_id.clone(),
                                bt_span: span.clone(),
                                tm: run.tm,
                                font: run.font.clone(),
                                bbox,
                            };
                            objects.push(PageObject::Text(text_obj.clone()));
                            text_meta.insert(
                                text_id,
                                TextMeta {
                                    stream_index: run.stream_index,
                                    tm: run.tm,
                                    bt_span: span,
                                    font: run.font.clone(),
                                    bbox,
                                    tm_token: run.tm_token,
                                    bt_token: state.bt_token.clone(),
                                },
                            );
                        }
                    }
                    text_state = None;
                }
                Operator::Tf => {
                    if let Some((name, size)) = parse_tf(&token.operands) {
                        current_font = FontInfo {
                            res_name: name,
                            size,
                        };
                    }
                }
                Operator::Tm => {
                    if let Some(matrix) = parse_matrix(&token.operands) {
                        if let Some(state) = &mut text_state {
                            state.current_tm = matrix;
                            state.last_tm_token = Some(position.clone());
                        }
                    }
                }
                Operator::Td => {
                    if let Some([tx, ty]) = parse_td(&token.operands) {
                        if let Some(state) = &mut text_state {
                            let translation = [1.0, 0.0, 0.0, 1.0, tx, ty];
                            state.current_tm = multiply(state.current_tm, translation);
                            state.last_tm_token = None;
                        }
                    }
                }
                Operator::Tj | Operator::TJ => {
                    if let Some(state) = &mut text_state {
                        let text_len = estimate_text_len(&token.operands);
                        if text_len == 0 {
                            continue;
                        }
                        let span_start = token.start;
                        let span_end = token.end;
                        let run_entry = state.run.get_or_insert_with(|| TextRunBuilder {
                            stream_index,
                            tm: state.current_tm,
                            tm_token: state.last_tm_token.clone(),
                            font: current_font.clone(),
                            span_start,
                            span_end,
                            text_len: 0,
                        });
                        run_entry.text_len += text_len;
                        run_entry.span_end = span_end;
                    }
                }
                Operator::Do => {
                    if let Some(name) = parse_do(&token.operands) {
                        if image_names.contains(&name) {
                            let cm = graphics_state.ctm;
                            let bbox = estimate_image_bbox(&cm);
                            let image_id = format!("img:{}", image_counter);
                            image_counter += 1;
                            objects.push(PageObject::Image(ImageObject {
                                id: image_id.clone(),
                                x_object: name.clone(),
                                cm,
                                bbox,
                            }));
                            image_meta.insert(
                                image_id,
                                ImageMeta {
                                    stream_index,
                                    cm,
                                    bbox,
                                    cm_token: graphics_state.last_cm_token.clone(),
                                    x_object: name,
                                },
                            );
                        }
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

    Ok(PageCache {
        page_ir,
        text: text_meta,
        images: image_meta,
        streams,
    })
}

pub fn build_document_ir(cache: &PageCache) -> DocumentIR {
    DocumentIR {
        pages: vec![cache.page_ir.clone()],
    }
}

fn page_media_box(page_dict: &Dictionary) -> Result<(f64, f64)> {
    let media_box = page_dict
        .get(b"MediaBox")
        .and_then(Object::as_array)
        .map_err(|_| anyhow!("page missing MediaBox"))?;
    if media_box.len() != 4 {
        return Err(anyhow!("unexpected MediaBox length"));
    }
    let llx = media_box[0].as_f64().unwrap_or(0.0);
    let lly = media_box[1].as_f64().unwrap_or(0.0);
    let urx = media_box[2].as_f64().unwrap_or(0.0);
    let ury = media_box[3].as_f64().unwrap_or(0.0);
    Ok((urx - llx, ury - lly))
}

fn load_stream_bytes(document: &Document, object_id: ObjectId) -> Vec<u8> {
    match document.get_object(object_id).and_then(Object::as_stream) {
        Ok(stream) => stream
            .decompressed_content()
            .unwrap_or_else(|_| stream.content.clone()),
        Err(_) => Vec::new(),
    }
}

fn collect_image_xobjects(document: &Document, page_id: ObjectId) -> Result<HashSet<String>> {
    fn gather_from_dict(
        doc: &Document,
        dict: &Dictionary,
        out: &mut HashSet<String>,
    ) -> Result<()> {
        if let Ok(xobj) = dict.get(b"XObject") {
            match xobj {
                Object::Dictionary(map) => {
                    for (name, value) in map.iter() {
                        if is_image_xobject(doc, value)? {
                            out.insert(String::from_utf8_lossy(name).into_owned());
                        }
                    }
                }
                Object::Reference(id) => {
                    if let Ok(Object::Dictionary(map)) = doc.get_object(*id) {
                        for (name, value) in map.iter() {
                            if is_image_xobject(doc, value)? {
                                out.insert(String::from_utf8_lossy(name).into_owned());
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }

    let mut names = HashSet::new();
    if let Ok(page_dict) = document.get_dictionary(page_id) {
        gather_from_dict(document, &page_dict, &mut names)?;
    }
    let (_, resource_ids) = document.get_page_resources(page_id);
    for id in resource_ids {
        if let Ok(dict) = document.get_dictionary(id) {
            gather_from_dict(document, dict, &mut names)?;
        }
    }
    Ok(names)
}

fn is_image_xobject(document: &Document, object: &Object) -> Result<bool> {
    let resolved = resolve_object(document, object)?;
    if let Object::Stream(stream) = resolved {
        if let Ok(subtype) = stream.dict.get(b"Subtype").and_then(Object::as_name_str) {
            return Ok(subtype == "Image");
        }
    }
    Ok(false)
}

fn resolve_object<'a>(document: &'a Document, mut object: &'a Object) -> Result<&'a Object> {
    loop {
        match object {
            Object::Reference(id) => {
                object = document.get_object(*id)?;
            }
            _ => return Ok(object),
        }
    }
}

#[derive(Clone)]
struct GraphicsState {
    ctm: [f64; 6],
    last_cm_token: Option<TokenPosition>,
}

impl Default for GraphicsState {
    fn default() -> Self {
        Self {
            ctm: IDENTITY,
            last_cm_token: None,
        }
    }
}

struct TextRunBuilder {
    stream_index: usize,
    tm: [f64; 6],
    tm_token: Option<TokenPosition>,
    font: FontInfo,
    span_start: usize,
    span_end: usize,
    text_len: usize,
}

struct TextState {
    bt_token: TokenPosition,
    current_tm: [f64; 6],
    last_tm_token: Option<TokenPosition>,
    run: Option<TextRunBuilder>,
}

const IDENTITY: [f64; 6] = [1.0, 0.0, 0.0, 1.0, 0.0, 0.0];

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

fn parse_matrix(operands: &[Operand]) -> Option<[f64; 6]> {
    if operands.len() < 6 {
        return None;
    }
    let mut values = [0.0; 6];
    for (i, operand) in operands.iter().take(6).enumerate() {
        if let Operand::Number(value) = operand {
            values[i] = *value;
        } else {
            return None;
        }
    }
    Some(values)
}

fn parse_tf(operands: &[Operand]) -> Option<(String, f64)> {
    if operands.len() != 2 {
        return None;
    }
    let name = match &operands[0] {
        Operand::Name(n) => n.clone(),
        _ => return None,
    };
    let size = match operands[1] {
        Operand::Number(v) => v,
        _ => return None,
    };
    Some((name, size))
}

fn parse_td(operands: &[Operand]) -> Option<[f64; 2]> {
    if operands.len() != 2 {
        return None;
    }
    match (&operands[0], &operands[1]) {
        (Operand::Number(tx), Operand::Number(ty)) => Some([*tx, *ty]),
        _ => None,
    }
}

fn parse_do(operands: &[Operand]) -> Option<String> {
    operands.get(0).and_then(|operand| match operand {
        Operand::Name(name) => Some(name.clone()),
        _ => None,
    })
}

fn estimate_text_len(operands: &[Operand]) -> usize {
    if operands.is_empty() {
        return 0;
    }
    match &operands[0] {
        Operand::String(bytes) => bytes.len(),
        Operand::Array(items) => items
            .iter()
            .map(|item| match item {
                Operand::String(bytes) => bytes.len(),
                _ => 0,
            })
            .sum(),
        _ => 0,
    }
}

fn estimate_text_bbox(tm: &[f64; 6], font_size: f64, text_len: usize) -> [f64; 4] {
    let width = (text_len as f64 * font_size * 0.5).max(font_size * 0.5);
    let height = font_size;
    let x = tm[4];
    let y = tm[5];
    [x, y - height, x + width, y]
}

fn estimate_image_bbox(cm: &[f64; 6]) -> [f64; 4] {
    let width = (cm[0] * cm[0] + cm[1] * cm[1]).sqrt();
    let height = (cm[2] * cm[2] + cm[3] * cm[3]).sqrt();
    let x = cm[4];
    let y = cm[5];
    [x, y - height, x + width, y]
}
