//! Extraction pipeline for the MVP transformation workflow.

use std::{
    collections::{HashMap, HashSet},
    ops::Range,
};

use anyhow::{anyhow, Context, Result};
use lopdf::{Dictionary, Document, Object, ObjectId, Stream};

use crate::{
    pdf::content::{tokenize_stream, ContentToken, Operand, Operator},
    types::{DocumentIR, FontSummary, ImageObject, PageIR, PageObject, StreamSpan, TextObject},
};

/// Result returned by the extraction pipeline.
pub struct ExtractResult {
    pub ir: DocumentIR,
    pub cache: ExtractCache,
}

/// Cached lookup information used by the patcher.
#[derive(Default)]
pub struct ExtractCache {
    pub page0: Option<PageCache>,
}

#[derive(Clone)]
pub struct PageCache {
    pub page_ref: ObjectId,
    pub width: f64,
    pub height: f64,
    pub streams: HashMap<u32, StreamCache>,
    pub order: Vec<u32>,
    pub objects: HashMap<String, CachedObject>,
}

#[derive(Clone)]
pub struct StreamCache {
    pub object_id: ObjectId,
    pub bytes: Vec<u8>,
    pub tokens: Vec<ContentToken>,
}

#[derive(Clone)]
pub enum CachedObject {
    Text(TextLocator),
    Image(ImageLocator),
}

#[derive(Clone)]
pub struct TextLocator {
    pub stream_obj: u32,
    pub bt_span: Range<usize>,
    pub tm_span: Option<Range<usize>>,
    pub insert_after_bt: usize,
}

#[derive(Clone)]
pub struct ImageLocator {
    pub stream_obj: u32,
    pub cm_span: Range<usize>,
}

/// Extract the MVP IR for the first page of the document.
pub fn extract_ir(document: &Document) -> Result<ExtractResult> {
    let pages = document.get_pages();
    let (&page_index, &page_ref) = pages
        .iter()
        .next()
        .ok_or_else(|| anyhow!("document has no pages"))?;
    let page_dict = document
        .get_object(page_ref)
        .with_context(|| format!("page {page_index} dictionary missing"))?
        .as_dict()
        .context("page object is not a dictionary")?
        .clone();

    let (width, height) = extract_dimensions(document, &page_dict)?;
    let resources = load_resources(document, &page_dict)?;
    let image_names = collect_image_names(document, &resources)?;

    let (stream_refs, stream_order) = resolve_content_streams(document, &page_dict)?;
    let mut streams = HashMap::new();
    let mut objects = Vec::new();
    let mut cached_objects = HashMap::new();

    let mut text_counter = 0usize;
    let mut image_counter = 0usize;

    for stream_id in &stream_order {
        let stream_cache = stream_refs
            .get(stream_id)
            .ok_or_else(|| anyhow!("missing stream {stream_id}"))?;
        let tokens = stream_cache.tokens.clone();
        let bytes = stream_cache.bytes.clone();

        let mut graphics_state = GraphicsState::default();
        let mut stack: Vec<GraphicsState> = Vec::new();
        let mut text_state: Option<TextState> = None;

        for token in &tokens {
            match token.operator {
                Operator::q => {
                    stack.push(graphics_state.clone());
                }
                Operator::Q => {
                    graphics_state = stack.pop().unwrap_or_default();
                }
                Operator::Cm => {
                    let matrix = parse_six_numbers(&token.operands)?;
                    graphics_state.ctm = multiply(graphics_state.ctm, matrix);
                    graphics_state.last_cm_span = Some(token.span.clone());
                }
                Operator::Bt => {
                    text_state = Some(TextState::new(token.span.start, token.span.clone()));
                }
                Operator::Tf => {
                    if let Some(state) = text_state.as_mut() {
                        let font_name = match token.operands.get(0) {
                            Some(Operand::Name(name)) => name.clone(),
                            _ => String::from("F1"),
                        };
                        let font_size = match token.operands.get(1) {
                            Some(Operand::Number(size)) => *size,
                            _ => state.font_size,
                        };
                        state.font_res = font_name;
                        state.font_size = font_size;
                    }
                }
                Operator::Tm => {
                    let matrix = parse_six_numbers(&token.operands)?;
                    if let Some(state) = text_state.as_mut() {
                        state.text_matrix = matrix;
                        state.text_line_matrix = matrix;
                        state.pending_tm = Some((matrix, Some(token.span.clone())));
                    }
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
                    if let Some(state) = text_state.as_mut() {
                        let translate = [1.0, 0.0, 0.0, 1.0, tx, ty];
                        state.text_line_matrix = multiply(state.text_line_matrix, translate);
                        state.text_matrix = state.text_line_matrix;
                        state.pending_tm = Some((state.text_matrix, None));
                    }
                }
                Operator::TStar => {
                    if let Some(state) = text_state.as_mut() {
                        let leading = if state.font_size.abs() < f64::EPSILON {
                            12.0
                        } else {
                            state.font_size
                        };
                        let translate = [1.0, 0.0, 0.0, 1.0, 0.0, -leading];
                        state.text_line_matrix = multiply(state.text_line_matrix, translate);
                        state.text_matrix = state.text_line_matrix;
                        state.pending_tm = Some((state.text_matrix, None));
                    }
                }
                Operator::Tj | Operator::TJ => {
                    if let Some(state) = text_state.as_mut() {
                        if !state.run_started {
                            let (matrix, span) = state
                                .pending_tm
                                .clone()
                                .unwrap_or((state.text_matrix, None));
                            state.base_matrix = matrix;
                            state.tm_span = span;
                            state.run_started = true;
                        }
                        state.char_count += count_text_chars(&token.operands);
                    }
                }
                Operator::Et => {
                    if let Some(state) = text_state.take() {
                        if state.run_started {
                            let id = format!("t:{text_counter}");
                            text_counter += 1;

                            let stream_obj = *stream_id;
                            let span = StreamSpan {
                                stream_obj,
                                start: u32::try_from(state.bt_start).unwrap_or(u32::MAX),
                                end: u32::try_from(token.span.end).unwrap_or(u32::MAX),
                            };
                            let tm = state.base_matrix;
                            let font_size = if state.font_size.abs() < f64::EPSILON {
                                12.0
                            } else {
                                state.font_size
                            };
                            let char_count = state.char_count.max(1) as f64;
                            let approx_width = font_size * 0.6 * char_count;
                            let approx_height = font_size;
                            let bbox = [tm[4], tm[5], approx_width, approx_height];
                            let font = FontSummary {
                                res_name: state.font_res.clone(),
                                size: font_size,
                            };
                            objects.push(PageObject::Text(TextObject {
                                id: id.clone(),
                                bt_span: span,
                                tm,
                                font,
                                bbox_pt: bbox,
                            }));
                            cached_objects.insert(
                                id,
                                CachedObject::Text(TextLocator {
                                    stream_obj,
                                    bt_span: state.bt_start..token.span.end,
                                    tm_span: state.tm_span,
                                    insert_after_bt: state.bt_token_span.end,
                                }),
                            );
                        }
                    }
                }
                Operator::Do => {
                    if let Some(name) = token.operands.get(0).and_then(as_name) {
                        if !image_names.is_empty() && !image_names.contains(name) {
                            continue;
                        }
                        if let Some(cm_span) = graphics_state.last_cm_span.clone() {
                            let id = format!("img:{image_counter}");
                            image_counter += 1;
                            let cm = graphics_state.ctm;
                            let width = (cm[0].powi(2) + cm[1].powi(2)).sqrt();
                            let height = (cm[2].powi(2) + cm[3].powi(2)).sqrt();
                            let bbox = [cm[4], cm[5], width, height];
                            objects.push(PageObject::Image(ImageObject {
                                id: id.clone(),
                                x_object: name.clone(),
                                cm,
                                bbox_pt: bbox,
                            }));
                            cached_objects.insert(
                                id,
                                CachedObject::Image(ImageLocator {
                                    stream_obj: *stream_id,
                                    cm_span,
                                }),
                            );
                        }
                    }
                }
                _ => {}
            }
        }

        streams.insert(*stream_id, stream_cache.clone());
    }

    let page_ir = PageIR {
        index: page_index as usize,
        width_pt: width,
        height_pt: height,
        objects,
    };

    let cache = ExtractCache {
        page0: Some(PageCache {
            page_ref,
            width,
            height,
            streams,
            order: stream_order,
            objects: cached_objects,
        }),
    };

    Ok(ExtractResult {
        ir: DocumentIR {
            pages: vec![page_ir],
        },
        cache,
    })
}

fn extract_dimensions(document: &Document, page_dict: &Dictionary) -> Result<(f64, f64)> {
    let media_box = page_dict
        .get(b"MediaBox")
        .or_else(|| page_dict.get(b"CropBox"))
        .ok_or_else(|| anyhow!("page lacks MediaBox"))?;
    let array = resolve_array(document, media_box)?;
    if array.len() != 4 {
        return Err(anyhow!("unexpected MediaBox length"));
    }
    let x0 = as_number(&array[0])?;
    let y0 = as_number(&array[1])?;
    let x1 = as_number(&array[2])?;
    let y1 = as_number(&array[3])?;
    Ok((x1 - x0, y1 - y0))
}

fn load_resources(document: &Document, page_dict: &Dictionary) -> Result<Dictionary> {
    match page_dict.get(b"Resources") {
        Some(Object::Dictionary(dict)) => Ok(dict.clone()),
        Some(Object::Reference(id)) => document
            .get_object(*id)
            .context("failed to resolve resource dictionary")?
            .as_dict()
            .context("referenced resource is not a dictionary")
            .map(|d| d.clone()),
        None => Ok(Dictionary::new()),
        _ => Err(anyhow!("unexpected resources entry")),
    }
}

fn collect_image_names(document: &Document, resources: &Dictionary) -> Result<HashSet<String>> {
    let mut names = HashSet::new();
    if let Some(Object::Dictionary(xobjects)) = resources.get(b"XObject") {
        for (name, entry) in xobjects.iter() {
            if let Ok(dict) = resolve_stream_dict(document, entry) {
                if let Some(Object::Name(subtype)) = dict.get(b"Subtype") {
                    if subtype == b"Image" {
                        names.insert(String::from_utf8_lossy(name).to_string());
                    }
                }
            }
        }
    }
    Ok(names)
}

fn resolve_content_streams(
    document: &Document,
    page_dict: &Dictionary,
) -> Result<(HashMap<u32, StreamCache>, Vec<u32>)> {
    let contents = page_dict
        .get(b"Contents")
        .ok_or_else(|| anyhow!("page lacks /Contents"))?;
    let mut map = HashMap::new();
    let mut order = Vec::new();
    match contents {
        Object::Reference(id) => {
            let cache = load_stream(document, *id)?;
            order.push(id.obj);
            map.insert(id.obj, cache);
        }
        Object::Array(arr) => {
            for item in arr {
                if let Object::Reference(id) = item {
                    let cache = load_stream(document, *id)?;
                    order.push(id.obj);
                    map.insert(id.obj, cache);
                }
            }
        }
        Object::Stream(stream) => {
            let fake_id = 0u32;
            let cache = stream_cache_from_stream(
                stream.clone(),
                ObjectId {
                    obj: fake_id,
                    gen: 0,
                },
            )?;
            order.push(fake_id);
            map.insert(fake_id, cache);
        }
        _ => return Err(anyhow!("unsupported Contents variant")),
    }
    Ok((map, order))
}

fn load_stream(document: &Document, id: ObjectId) -> Result<StreamCache> {
    let object = document
        .get_object(id)
        .with_context(|| format!("missing stream object {id:?}"))?;
    let stream = object
        .as_stream()
        .with_context(|| format!("object {id:?} is not a stream"))?;
    stream_cache_from_stream(stream.clone(), id)
}

fn stream_cache_from_stream(mut stream: Stream, id: ObjectId) -> Result<StreamCache> {
    stream.decompress()?;
    let bytes = stream.content.clone();
    let tokens = tokenize_stream(&bytes)?;
    Ok(StreamCache {
        object_id: id,
        bytes,
        tokens,
    })
}

fn resolve_array(document: &Document, object: &Object) -> Result<Vec<Object>> {
    match object {
        Object::Array(items) => Ok(items.clone()),
        Object::Reference(id) => {
            let resolved = document.get_object(*id)?;
            resolve_array(document, resolved)
        }
        _ => Err(anyhow!("expected array")),
    }
}

fn resolve_stream_dict(document: &Document, object: &Object) -> Result<Dictionary> {
    match object {
        Object::Stream(stream) => Ok(stream.dict.clone()),
        Object::Reference(id) => {
            let resolved = document.get_object(*id)?;
            resolve_stream_dict(document, resolved)
        }
        _ => Err(anyhow!("expected stream")),
    }
}

fn parse_six_numbers(operands: &[Operand]) -> Result<[f64; 6]> {
    if operands.len() < 6 {
        return Err(anyhow!("expected six operands"));
    }
    let mut result = [0.0; 6];
    for (i, slot) in result.iter_mut().enumerate() {
        if let Operand::Number(value) = operands.get(i).ok_or_else(|| anyhow!("missing operand"))? {
            *slot = *value;
        } else {
            return Err(anyhow!("operand {i} is not a number"));
        }
    }
    Ok(result)
}

fn as_name(operand: &Operand) -> Option<&String> {
    if let Operand::Name(name) = operand {
        Some(name)
    } else {
        None
    }
}

fn as_number(object: &Object) -> Result<f64> {
    match object {
        Object::Integer(i) => Ok(*i as f64),
        Object::Real(r) => Ok(*r),
        Object::Reference(_) => Err(anyhow!("indirect numeric objects are not supported")),
        _ => Err(anyhow!("expected numeric object")),
    }
}

fn count_text_chars(operands: &[Operand]) -> usize {
    operands
        .iter()
        .map(|operand| match operand {
            Operand::String(s) => s.chars().count(),
            Operand::Array(items) => items
                .iter()
                .map(|item| match item {
                    Operand::String(text) => text.chars().count(),
                    _ => 0,
                })
                .sum(),
            _ => 0,
        })
        .sum()
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
    last_cm_span: Option<Range<usize>>,
}

impl Default for GraphicsState {
    fn default() -> Self {
        Self {
            ctm: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
            last_cm_span: None,
        }
    }
}

struct TextState {
    bt_start: usize,
    bt_token_span: Range<usize>,
    tm_span: Option<Range<usize>>,
    base_matrix: [f64; 6],
    text_matrix: [f64; 6],
    text_line_matrix: [f64; 6],
    pending_tm: Option<([f64; 6], Option<Range<usize>>)>,
    font_res: String,
    font_size: f64,
    char_count: usize,
    run_started: bool,
}

impl TextState {
    fn new(start: usize, bt_span: Range<usize>) -> Self {
        Self {
            bt_start: start,
            bt_token_span: bt_span,
            tm_span: None,
            base_matrix: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
            text_matrix: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
            text_line_matrix: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
            pending_tm: Some(([1.0, 0.0, 0.0, 1.0, 0.0, 0.0], None)),
            font_res: String::from("F1"),
            font_size: 12.0,
            char_count: 0,
            run_started: false,
        }
    }
}
