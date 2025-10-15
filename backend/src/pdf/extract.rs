use std::collections::HashMap;

use anyhow::{anyhow, bail, Context, Result};
use lopdf::{Dictionary, Document, Object, ObjectId};

use crate::pdf::content::{tokenize_stream, ContentToken, Operand, Operator};
use crate::types::{BtSpan, DocumentIR, FontInfo, ImageObject, PageIR, PageObject, TextObject};

#[derive(Debug, Clone)]
pub struct ExtractedDocument {
    pub ir: DocumentIR,
    pub pages: HashMap<usize, PageExtraction>,
}

#[derive(Debug, Clone)]
pub struct PageExtraction {
    pub page_id: ObjectId,
    pub width_pt: f64,
    pub height_pt: f64,
    pub contents: PageContents,
    pub objects: HashMap<String, CachedObject>,
}

#[derive(Debug, Clone)]
pub struct PageContents {
    pub is_array: bool,
    pub streams: Vec<ContentStream>,
}

#[derive(Debug, Clone)]
pub struct ContentStream {
    pub id: ObjectId,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone)]
pub enum CachedObject {
    Text(CachedText),
    Image(CachedImage),
}

#[derive(Debug, Clone)]
pub struct CachedText {
    pub stream_index: usize,
    pub bt_span: (usize, usize),
    pub tm_range: Option<(usize, usize)>,
    pub bt_token_end: usize,
    pub base_tm: [f64; 6],
    pub font: FontInfo,
}

#[derive(Debug, Clone)]
pub struct CachedImage {
    pub stream_index: usize,
    pub cm_range: (usize, usize),
    pub explicit_cm: [f64; 6],
    pub effective_cm: [f64; 6],
    pub x_object: String,
}

pub fn extract_document(bytes: &[u8]) -> Result<ExtractedDocument> {
    let doc = Document::load_mem(bytes)?;
    extract_from_document(&doc)
}

pub fn extract_from_document(doc: &Document) -> Result<ExtractedDocument> {
    let pages = doc.get_pages();
    if pages.is_empty() {
        bail!("document has no pages");
    }
    let (&page_number, &page_id) = pages.iter().next().context("missing first page")?;
    let page_index = page_number.saturating_sub(1) as usize;

    let page_dict = doc
        .get_object(page_id)
        .context("page object missing")?
        .as_dict()
        .context("page object is not a dictionary")?
        .clone();

    let (width_pt, height_pt) = media_box_size(&page_dict)?;
    let contents_obj = page_dict.get(b"Contents").cloned().unwrap_or(Object::Null);
    let contents = resolve_contents(doc, &contents_obj)?;

    let mut page_objects = Vec::new();
    let mut cache_map = HashMap::new();

    let mut text_counter = 0u32;
    let mut image_counter = 0u32;

    let mut state_stack = vec![GraphicsState::default()];
    let mut current_bt: Option<TextContext> = None;
    let mut last_cm: Option<CmToken> = None;

    for (stream_index, stream) in contents.streams.iter().enumerate() {
        let tokens = tokenize_stream(&stream.data)?;
        for token in tokens.iter() {
            match token.operator {
                Operator::q => {
                    let clone = state_stack
                        .last()
                        .cloned()
                        .unwrap_or_else(GraphicsState::default);
                    state_stack.push(clone);
                }
                Operator::Q => {
                    state_stack.pop();
                    if state_stack.is_empty() {
                        state_stack.push(GraphicsState::default());
                    }
                    last_cm = None;
                }
                _ => {
                    let state = state_stack
                        .last_mut()
                        .expect("graphics state stack should not be empty");
                    match token.operator {
                        Operator::Cm => {
                            let matrix = operands_to_matrix(&token.operands)?;
                            state.ctm = multiply(state.ctm, matrix);
                            last_cm = Some(CmToken {
                                stream_index,
                                span: token.span.start..token.span.end,
                                matrix,
                            });
                        }
                        Operator::Bt => {
                            state.text_open = true;
                            state.text_matrix = identity();
                            state.line_matrix = identity();
                            current_bt = Some(TextContext {
                                stream_index,
                                bt_span_start: token.span.start,
                                bt_token_end: token.span.end,
                                span_end: token.span.end,
                                tm_range: None,
                                explicit_tm: None,
                                base_tm: None,
                                glyph_estimate: 0,
                                font: state.font.clone(),
                            });
                        }
                        Operator::Et => {
                            if let Some(ctx) = current_bt.take() {
                                if ctx.glyph_estimate > 0 {
                                    if let Some(base_tm) = ctx.base_tm {
                                        let font = ctx.font.unwrap_or(FontInfo {
                                            res_name: "F1".into(),
                                            size: 12.0,
                                        });
                                        let width_guess =
                                            (ctx.glyph_estimate as f64) * font.size * 0.6;
                                        let height_guess = font.size;
                                        let bbox =
                                            rect_from_matrix(base_tm, width_guess, height_guess);
                                        let id = format!("t:{text_counter}");
                                        text_counter += 1;
                                        let bt_span = BtSpan {
                                            stream_obj: contents.streams[ctx.stream_index].id.0,
                                            start: ctx.bt_span_start as u32,
                                            end: ctx.span_end as u32,
                                        };
                                        let text_object = TextObject {
                                            id: id.clone(),
                                            bt_span,
                                            tm: base_tm,
                                            font: font.clone(),
                                            bbox: Some(bbox),
                                        };
                                        page_objects.push(PageObject::Text(text_object));
                                        cache_map.insert(
                                            id,
                                            CachedObject::Text(CachedText {
                                                stream_index: ctx.stream_index,
                                                bt_span: (ctx.bt_span_start, ctx.span_end),
                                                tm_range: ctx.tm_range.map(|r| (r.start, r.end)),
                                                bt_token_end: ctx.bt_token_end,
                                                base_tm,
                                                font,
                                            }),
                                        );
                                    }
                                }
                            }
                            state.text_open = false;
                        }
                        Operator::Tf => {
                            let res_name = match token.operands.get(0) {
                                Some(Operand::Name(name)) => name.clone(),
                                _ => continue,
                            };
                            let size = match token.operands.get(1) {
                                Some(Operand::Number(v)) => *v,
                                _ => 12.0,
                            };
                            state.font = Some(FontInfo { res_name, size });
                            state.leading = -size;
                            if let Some(ctx) = &mut current_bt {
                                ctx.font = state.font.clone();
                            }
                        }
                        Operator::Tm => {
                            let matrix = operands_to_matrix(&token.operands)?;
                            state.text_matrix = matrix;
                            state.line_matrix = matrix;
                            if let Some(ctx) = &mut current_bt {
                                ctx.explicit_tm = Some(matrix);
                                ctx.tm_range = Some(token.span.clone());
                                ctx.base_tm = Some(matrix);
                            }
                        }
                        Operator::Td => {
                            let tx = match token.operands.get(0) {
                                Some(Operand::Number(v)) => *v,
                                _ => 0.0,
                            };
                            let ty = match token.operands.get(1) {
                                Some(Operand::Number(v)) => *v,
                                _ => 0.0,
                            };
                            let translate = [1.0, 0.0, 0.0, 1.0, tx, ty];
                            state.line_matrix = multiply(state.line_matrix, translate);
                            state.text_matrix = state.line_matrix;
                            state.leading = -ty;
                        }
                        Operator::TStar => {
                            let translate = [1.0, 0.0, 0.0, 1.0, 0.0, state.leading];
                            state.line_matrix = multiply(state.line_matrix, translate);
                            state.text_matrix = state.line_matrix;
                        }
                        Operator::Tj | Operator::TjArray => {
                            if let Some(ctx) = &mut current_bt {
                                ctx.span_end = token.span.end;
                                if ctx.base_tm.is_none() {
                                    if let Some(explicit) = ctx.explicit_tm {
                                        ctx.base_tm = Some(explicit);
                                    } else {
                                        ctx.base_tm = Some(state.text_matrix);
                                    }
                                }
                                let glyphs = estimate_glyphs(token);
                                ctx.glyph_estimate = ctx.glyph_estimate.saturating_add(glyphs);
                            }
                        }
                        Operator::Do => {
                            let name = match token.operands.get(0) {
                                Some(Operand::Name(name)) => name.clone(),
                                _ => continue,
                            };
                            if let Some(cm_token) = last_cm.take() {
                                if cm_token.stream_index == stream_index {
                                    let id = format!("img:{image_counter}");
                                    image_counter += 1;
                                    let bbox = image_bbox(cm_token.matrix);
                                    let effective = state.ctm;
                                    page_objects.push(PageObject::Image(ImageObject {
                                        id: id.clone(),
                                        x_object: name.clone(),
                                        cm: effective,
                                        bbox: Some(bbox),
                                    }));
                                    cache_map.insert(
                                        id,
                                        CachedObject::Image(CachedImage {
                                            stream_index,
                                            cm_range: (cm_token.span.start, cm_token.span.end),
                                            explicit_cm: cm_token.matrix,
                                            effective_cm: effective,
                                            x_object: name,
                                        }),
                                    );
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    let page_ir = PageIR {
        index: page_index,
        width_pt,
        height_pt,
        objects: page_objects,
    };

    let mut pages_map = HashMap::new();
    pages_map.insert(
        0,
        PageExtraction {
            page_id,
            width_pt,
            height_pt,
            contents,
            objects: cache_map,
        },
    );

    Ok(ExtractedDocument {
        ir: DocumentIR {
            pages: vec![page_ir],
        },
        pages: pages_map,
    })
}

fn media_box_size(dict: &Dictionary) -> Result<(f64, f64)> {
    let media_box = dict
        .get(b"MediaBox")
        .cloned()
        .or_else(|| dict.get(b"CropBox").cloned())
        .context("page missing MediaBox")?;
    let array = match media_box {
        Object::Array(items) => items,
        _ => bail!("MediaBox is not an array"),
    };
    if array.len() != 4 {
        bail!("MediaBox should contain four numbers");
    }
    let numbers: Vec<f64> = array
        .into_iter()
        .map(|obj| match obj {
            Object::Integer(v) => Ok(v as f64),
            Object::Real(v) => Ok(v),
            other => Err(anyhow::anyhow!("unexpected MediaBox value {other:?}")),
        })
        .collect::<Result<_>>()?;
    let width = numbers[2] - numbers[0];
    let height = numbers[3] - numbers[1];
    Ok((width, height))
}

fn resolve_contents(doc: &Document, object: &Object) -> Result<PageContents> {
    match object {
        Object::Null => Ok(PageContents {
            is_array: false,
            streams: Vec::new(),
        }),
        Object::Reference(id) => Ok(PageContents {
            is_array: false,
            streams: vec![ContentStream {
                id: *id,
                data: load_stream(doc, *id)?,
            }],
        }),
        Object::Array(items) => {
            let mut streams = Vec::new();
            for item in items {
                match item {
                    Object::Reference(id) => streams.push(ContentStream {
                        id: *id,
                        data: load_stream(doc, *id)?,
                    }),
                    _ => {}
                }
            }
            Ok(PageContents {
                is_array: true,
                streams,
            })
        }
        _ => bail!("unsupported Contents type"),
    }
}

fn load_stream(doc: &Document, id: ObjectId) -> Result<Vec<u8>> {
    let stream = doc
        .get_object(id)
        .context("missing stream")?
        .as_stream()
        .context("content reference is not a stream")?;
    let mut data = stream.decompressed_content()?;
    if !data.ends_with(b"\n") {
        data.push(b'\n');
    }
    Ok(data)
}

#[derive(Clone)]
struct GraphicsState {
    ctm: [f64; 6],
    text_matrix: [f64; 6],
    line_matrix: [f64; 6],
    text_open: bool,
    font: Option<FontInfo>,
    leading: f64,
}

impl Default for GraphicsState {
    fn default() -> Self {
        Self {
            ctm: identity(),
            text_matrix: identity(),
            line_matrix: identity(),
            text_open: false,
            font: None,
            leading: -12.0,
        }
    }
}

#[derive(Clone)]
struct TextContext {
    stream_index: usize,
    bt_span_start: usize,
    bt_token_end: usize,
    span_end: usize,
    tm_range: Option<std::ops::Range<usize>>,
    explicit_tm: Option<[f64; 6]>,
    base_tm: Option<[f64; 6]>,
    glyph_estimate: usize,
    font: Option<FontInfo>,
}

#[derive(Clone)]
struct CmToken {
    stream_index: usize,
    span: std::ops::Range<usize>,
    matrix: [f64; 6],
}

fn operands_to_matrix(operands: &[Operand]) -> Result<[f64; 6]> {
    if operands.len() < 6 {
        bail!("matrix operator missing operands");
    }
    let mut values = [0.0f64; 6];
    for (slot, operand) in operands.iter().take(6).enumerate() {
        values[slot] = match operand {
            Operand::Number(v) => *v,
            _ => bail!("matrix operand must be numeric"),
        };
    }
    Ok(values)
}

fn estimate_glyphs(token: &ContentToken) -> usize {
    match token.operator {
        Operator::Tj => token.operands.get(0).map(operand_length).unwrap_or(0),
        Operator::TjArray => token.operands.iter().map(operand_length).sum(),
        _ => 0,
    }
}

fn operand_length(op: &Operand) -> usize {
    match op {
        Operand::String(bytes) => bytes.len(),
        Operand::Array(items) => items.iter().map(operand_length).sum(),
        _ => 0,
    }
}

fn rect_from_matrix(matrix: [f64; 6], width: f64, height: f64) -> [f64; 4] {
    let corners = [
        apply_matrix(matrix, 0.0, 0.0),
        apply_matrix(matrix, width, 0.0),
        apply_matrix(matrix, width, height),
        apply_matrix(matrix, 0.0, height),
    ];
    let (mut min_x, mut min_y) = corners[0];
    let (mut max_x, mut max_y) = corners[0];
    for &(x, y) in &corners[1..] {
        if x < min_x {
            min_x = x;
        }
        if y < min_y {
            min_y = y;
        }
        if x > max_x {
            max_x = x;
        }
        if y > max_y {
            max_y = y;
        }
    }
    [min_x, min_y, max_x, max_y]
}

fn image_bbox(matrix: [f64; 6]) -> [f64; 4] {
    let width = matrix[0].abs();
    let height = matrix[3].abs();
    let x0 = matrix[4];
    let y0 = matrix[5] - height;
    [x0, y0, x0 + width, y0 + height]
}

fn apply_matrix(matrix: [f64; 6], x: f64, y: f64) -> (f64, f64) {
    let [a, b, c, d, e, f] = matrix;
    let nx = a * x + c * y + e;
    let ny = b * x + d * y + f;
    (nx, ny)
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

fn identity() -> [f64; 6] {
    [1.0, 0.0, 0.0, 1.0, 0.0, 0.0]
}
