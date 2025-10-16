use std::collections::HashMap;
use std::ops::Range;

use anyhow::{anyhow, Context, Result};
use lopdf::{Dictionary, Document, Object, ObjectId};

use crate::types::{BtSpan, DocumentIR, FontInfo, PageIR, PageObject};

use super::content::{tokenize_stream, ContentToken, Operand, Operator};

#[derive(Clone)]
pub struct StreamContent {
    pub object_id: ObjectId,
    pub bytes: Vec<u8>,
    pub tokens: Vec<ContentToken>,
}

#[derive(Clone)]
pub struct TextCache {
    pub stream_index: usize,
    pub bt_span: Range<usize>,
    pub tm_span: Option<Range<usize>>,
    pub tm_token_index: Option<usize>,
    pub bt_token_index: usize,
    pub et_token_index: usize,
    pub tm: [f64; 6],
    pub font: FontInfo,
}

#[derive(Clone)]
pub struct ImageCache {
    pub stream_index: usize,
    pub cm_span: Range<usize>,
    pub cm_token_index: usize,
    pub cm: [f64; 6],
    pub name: String,
}

#[derive(Clone)]
pub enum CachedObject {
    Text(TextCache),
    Image(ImageCache),
}

#[derive(Clone)]
pub struct PageCache {
    pub page_id: ObjectId,
    pub streams: Vec<StreamContent>,
    pub objects: HashMap<String, CachedObject>,
}

#[derive(Clone)]
pub struct PageExtraction {
    pub ir: PageIR,
    pub cache: PageCache,
}

pub fn extract_page_zero(document: &Document) -> Result<PageExtraction> {
    let page_id = *document
        .get_pages()
        .values()
        .next()
        .ok_or_else(|| anyhow!("document has no pages"))?;

    let page_dict = document
        .get_dictionary(page_id)
        .context("unable to read page dictionary")?;

    let (width, height) = page_dimensions(document, &page_dict)?;

    let content_ids = document.get_page_contents(page_id);
    let mut streams = Vec::new();
    for object_id in &content_ids {
        let stream = document
            .get_object(*object_id)
            .and_then(Object::as_stream)
            .with_context(|| format!("content stream {object_id:?} missing"))?;
        let bytes = stream
            .decompressed_content()
            .unwrap_or_else(|_| stream.content.clone());
        let tokens =
            tokenize_stream(&bytes).with_context(|| format!("tokenising stream {object_id:?}"))?;
        streams.push(StreamContent {
            object_id: *object_id,
            bytes,
            tokens,
        });
    }

    let image_resources = collect_image_resources(document, page_id)?;

    let mut state = GraphicsState::default();
    let mut stack: Vec<GraphicsState> = Vec::new();
    let mut text_counter = 0usize;
    let mut image_counter = 0usize;
    let mut objects = Vec::new();
    let mut object_cache = HashMap::new();
    let mut current_run: Option<TextRunBuilder> = None;

    for (stream_index, stream) in streams.iter().enumerate() {
        for (token_index, token) in stream.tokens.iter().enumerate() {
            match token.operator {
                Operator::q => {
                    stack.push(state.clone());
                }
                Operator::Q => {
                    if let Some(prev) = stack.pop() {
                        state = prev;
                    }
                }
                Operator::Cm => {
                    if let Some(matrix) = matrix_from_operands(&token.operands) {
                        state.ctm = multiply(state.ctm, matrix);
                        state.last_cm = Some(CmState {
                            stream_index,
                            token_index,
                            span: token.span.clone(),
                            matrix,
                        });
                    }
                }
                Operator::Bt => {
                    state.text_active = true;
                    state.text_matrix = identity();
                    state.text_line_matrix = identity();
                    if let Some(font) = state.font.clone() {
                        state.leading = font.size;
                    }
                    let id = format!("t:{text_counter}");
                    text_counter += 1;
                    current_run = Some(TextRunBuilder {
                        id: id.clone(),
                        stream_index,
                        bt_token_index: token_index,
                        bt_span_start: token.span.start,
                        tm_token_index: None,
                        tm_span: None,
                        base_tm: None,
                        glyph_count: 0,
                        font: state.font.clone(),
                    });
                }
                Operator::Et => {
                    if let Some(run) = current_run.take() {
                        if let Some(base_tm) = run.base_tm {
                            let font =
                                run.font.or_else(|| state.font.clone()).unwrap_or(FontInfo {
                                    res_name: "Unknown".into(),
                                    size: 12.0,
                                });
                            let bbox = text_bbox(base_tm, &font, run.glyph_count);
                            let bt_span = BtSpan {
                                stream_obj: streams[run.stream_index].object_id.0,
                                start: run.bt_span_start as u32,
                                end: token.span.end as u32,
                            };
                            objects.push(PageObject::Text {
                                id: run.id.clone(),
                                bt_span: bt_span.clone(),
                                tm: base_tm,
                                font: font.clone(),
                                bbox,
                            });
                            object_cache.insert(
                                run.id,
                                CachedObject::Text(TextCache {
                                    stream_index: run.stream_index,
                                    bt_span: run.bt_span_start..token.span.end,
                                    tm_span: run.tm_span,
                                    tm_token_index: run.tm_token_index,
                                    bt_token_index: run.bt_token_index,
                                    et_token_index: token_index,
                                    tm: base_tm,
                                    font,
                                }),
                            );
                        }
                    }
                    state.text_active = false;
                    state.text_matrix = identity();
                    state.text_line_matrix = identity();
                }
                Operator::Tf => {
                    if let Some((name, size)) = parse_tf(&token.operands) {
                        state.font = Some(FontInfo {
                            res_name: name,
                            size,
                        });
                        state.leading = size;
                        if let Some(run) = current_run.as_mut() {
                            run.font = state.font.clone();
                        }
                    }
                }
                Operator::Tm => {
                    if let Some(matrix) = matrix_from_operands(&token.operands) {
                        state.text_matrix = matrix;
                        state.text_line_matrix = matrix;
                        if let Some(run) = current_run.as_mut() {
                            if run.base_tm.is_none() {
                                run.tm_token_index = Some(token_index);
                                run.tm_span = Some(token.span.clone());
                            }
                        }
                    }
                }
                Operator::Td => {
                    if let Some((tx, ty)) = parse_td(&token.operands) {
                        let translation = [1.0, 0.0, 0.0, 1.0, tx, ty];
                        state.text_line_matrix = multiply(state.text_line_matrix, translation);
                        state.text_matrix = state.text_line_matrix;
                    }
                }
                Operator::TStar => {
                    let move_y = -state.leading;
                    let translation = [1.0, 0.0, 0.0, 1.0, 0.0, move_y];
                    state.text_line_matrix = multiply(state.text_line_matrix, translation);
                    state.text_matrix = state.text_line_matrix;
                }
                Operator::Tj => {
                    if let Some(run) = current_run.as_mut() {
                        if run.base_tm.is_none() {
                            run.base_tm = Some(state.text_matrix);
                            if run.font.is_none() {
                                run.font = state.font.clone();
                            }
                        }
                        run.glyph_count += count_text_glyphs(&token.operands);
                    }
                }
                Operator::TjArray => {
                    if let Some(run) = current_run.as_mut() {
                        if run.base_tm.is_none() {
                            run.base_tm = Some(state.text_matrix);
                            if run.font.is_none() {
                                run.font = state.font.clone();
                            }
                        }
                        run.glyph_count += count_text_glyphs(&token.operands);
                    }
                }
                Operator::Do => {
                    if let Some(name) = operand_name(token.operands.last()) {
                        if image_resources.contains_key(&name) {
                            if let Some(cm_state) = state.last_cm.clone() {
                                let cm = cm_state.matrix;
                                let bbox = image_bbox(cm);
                                let id = format!("img:{image_counter}");
                                image_counter += 1;
                                objects.push(PageObject::Image {
                                    id: id.clone(),
                                    x_object: name.clone(),
                                    cm,
                                    bbox,
                                });
                                object_cache.insert(
                                    id,
                                    CachedObject::Image(ImageCache {
                                        stream_index,
                                        cm_span: cm_state.span,
                                        cm_token_index: cm_state.token_index,
                                        cm,
                                        name,
                                    }),
                                );
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }

    let page_ir = PageIR {
        index: 0,
        width_pt: width,
        height_pt: height,
        objects,
    };

    let cache = PageCache {
        page_id,
        streams,
        objects: object_cache,
    };

    Ok(PageExtraction { ir: page_ir, cache })
}

pub fn extract_ir(document: &Document) -> Result<DocumentIR> {
    let extraction = extract_page_zero(document)?;
    Ok(DocumentIR {
        pages: vec![extraction.ir.clone()],
    })
}

fn page_dimensions(document: &Document, page: &Dictionary) -> Result<(f64, f64)> {
    let media_box = resolve_array(document, page.get(b"MediaBox")?)?;
    if media_box.len() >= 4 {
        let width = media_box[2] - media_box[0];
        let height = media_box[3] - media_box[1];
        Ok((width, height))
    } else {
        Err(anyhow!("MediaBox missing values"))
    }
}

fn resolve_array(document: &Document, object: &Object) -> Result<Vec<f64>> {
    match object {
        Object::Array(items) => items
            .iter()
            .map(|item| match item {
                Object::Integer(v) => Ok(*v as f64),
                Object::Real(v) => Ok(*v as f64),
                Object::Reference(id) => {
                    let resolved = document.get_object(*id)?;
                    resolve_array(document, resolved)
                }
                _ => Err(anyhow!("unsupported array item")),
            })
            .collect(),
        Object::Reference(id) => {
            let resolved = document.get_object(*id)?;
            resolve_array(document, resolved)
        }
        _ => Err(anyhow!("expected array")),
    }
}

fn collect_image_resources(
    document: &Document,
    page_id: ObjectId,
) -> Result<HashMap<String, ObjectId>> {
    let (maybe_dict, ids) = document.get_page_resources(page_id);
    let mut map = HashMap::new();
    if let Some(dict) = maybe_dict {
        collect_images_from_dict(document, dict, &mut map)?;
    }
    for id in ids {
        if let Ok(dict) = document.get_dictionary(id) {
            collect_images_from_dict(document, dict, &mut map)?;
        }
    }
    Ok(map)
}

fn collect_images_from_dict(
    document: &Document,
    dict: &Dictionary,
    map: &mut HashMap<String, ObjectId>,
) -> Result<()> {
    if let Ok(x_object) = dict.get(b"XObject") {
        match x_object {
            Object::Dictionary(inner) => {
                for (name, value) in inner.iter() {
                    if let Some(id) = resolve_image(document, value.clone())? {
                        map.insert(String::from_utf8_lossy(name).into_owned(), id);
                    }
                }
            }
            Object::Reference(id) => {
                if let Ok(xobj_dict) = document.get_dictionary(*id) {
                    collect_images_from_dict(document, &xobj_dict, map)?;
                }
            }
            _ => {}
        }
    }
    Ok(())
}

fn resolve_image(document: &Document, object: Object) -> Result<Option<ObjectId>> {
    match object {
        Object::Reference(id) => {
            if let Ok(dict) = document.get_dictionary(id) {
                if dict
                    .get(b"Subtype")
                    .and_then(Object::as_name_str)
                    .map(|name| name == "Image")
                    .unwrap_or(false)
                {
                    return Ok(Some(id));
                }
            }
            Ok(None)
        }
        Object::Dictionary(dict) => {
            if dict
                .get(b"Subtype")
                .and_then(Object::as_name_str)
                .map(|name| name == "Image")
                .unwrap_or(false)
            {
                Ok(None)
            } else {
                Ok(None)
            }
        }
        _ => Ok(None),
    }
}

fn parse_tf(operands: &[Operand]) -> Option<(String, f64)> {
    if operands.len() == 2 {
        if let Operand::Name(name) = &operands[0] {
            if let Operand::Number(size) = operands[1] {
                return Some((name.clone(), size));
            }
        }
    }
    None
}

fn parse_td(operands: &[Operand]) -> Option<(f64, f64)> {
    if operands.len() == 2 {
        if let (Operand::Number(a), Operand::Number(b)) = (&operands[0], &operands[1]) {
            return Some((*a, *b));
        }
    }
    None
}

fn matrix_from_operands(operands: &[Operand]) -> Option<[f64; 6]> {
    if operands.len() == 6 {
        let mut values = [0f64; 6];
        for (idx, operand) in operands.iter().enumerate() {
            if let Operand::Number(value) = operand {
                values[idx] = *value;
            } else {
                return None;
            }
        }
        Some(values)
    } else {
        None
    }
}

fn operand_name(operand: Option<&Operand>) -> Option<String> {
    match operand {
        Some(Operand::Name(name)) => Some(name.clone()),
        _ => None,
    }
}

fn count_text_glyphs(operands: &[Operand]) -> usize {
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

fn text_bbox(tm: [f64; 6], font: &FontInfo, glyph_count: usize) -> [f64; 4] {
    let width = font.size * glyph_count as f64 * 0.6;
    let height = font.size;
    [tm[4], tm[5] - height, tm[4] + width, tm[5]]
}

fn image_bbox(cm: [f64; 6]) -> [f64; 4] {
    let width = cm[0];
    let height = cm[3];
    [cm[4], cm[5], cm[4] + width, cm[5] + height]
}

fn identity() -> [f64; 6] {
    [1.0, 0.0, 0.0, 1.0, 0.0, 0.0]
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

#[derive(Clone)]
struct GraphicsState {
    ctm: [f64; 6],
    text_matrix: [f64; 6],
    text_line_matrix: [f64; 6],
    leading: f64,
    font: Option<FontInfo>,
    text_active: bool,
    last_cm: Option<CmState>,
}

impl Default for GraphicsState {
    fn default() -> Self {
        Self {
            ctm: identity(),
            text_matrix: identity(),
            text_line_matrix: identity(),
            leading: 0.0,
            font: None,
            text_active: false,
            last_cm: None,
        }
    }
}

#[derive(Clone)]
struct CmState {
    stream_index: usize,
    token_index: usize,
    span: Range<usize>,
    matrix: [f64; 6],
}

struct TextRunBuilder {
    id: String,
    stream_index: usize,
    bt_token_index: usize,
    bt_span_start: usize,
    tm_token_index: Option<usize>,
    tm_span: Option<Range<usize>>,
    base_tm: Option<[f64; 6]>,
    glyph_count: usize,
    font: Option<FontInfo>,
}
