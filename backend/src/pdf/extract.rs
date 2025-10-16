//! Extraction pipeline for page-level IR and patch metadata.

use std::collections::HashMap;
use std::ops::Range;

use anyhow::{anyhow, Result};
use lopdf::{Document, Object, ObjectId};

use crate::types::{DocumentIR, FontInfo, ImageObject, PageIR, PageObject, TextRun};

use super::content::{tokenize_stream, Operand, Operator};
use super::loader;

#[derive(Debug, Clone)]
pub struct ByteRange {
    pub start: u32,
    pub end: u32,
}

#[derive(Debug, Clone)]
pub struct TextMeta {
    pub stream_obj: u32,
    pub bt_span: ByteRange,
    pub tm_span: Option<ByteRange>,
    pub insert_after_bt: u32,
    pub tm: [f64; 6],
    pub font: FontInfo,
}

#[derive(Debug, Clone)]
pub struct ImageMeta {
    pub stream_obj: u32,
    pub cm_span: ByteRange,
    pub cm: [f64; 6],
    pub x_object: String,
    pub bbox: [f64; 4],
}

#[derive(Debug, Clone)]
pub enum ObjectMeta {
    Text(TextMeta),
    Image(ImageMeta),
}

#[derive(Debug, Clone)]
pub struct PageCache {
    pub page_id: ObjectId,
    pub ir: DocumentIR,
    pub width_pt: f64,
    pub height_pt: f64,
    pub objects: HashMap<String, ObjectMeta>,
}

#[derive(Debug, Clone)]
struct CombinedContent {
    data: Vec<u8>,
    segments: Vec<Segment>,
}

#[derive(Debug, Clone)]
struct Segment {
    stream_obj: u32,
    range: Range<usize>,
}

impl CombinedContent {
    fn resolve(&self, span: &Range<usize>) -> Result<(u32, Range<usize>)> {
        for segment in &self.segments {
            if span.start >= segment.range.start && span.end <= segment.range.end {
                let local_start = span.start - segment.range.start;
                let local_end = span.end - segment.range.start;
                return Ok((segment.stream_obj, local_start..local_end));
            }
        }
        Err(anyhow!("Span crosses content segments"))
    }
}

#[derive(Debug, Clone)]
struct GraphicsState {
    ctm: [f64; 6],
}

impl Default for GraphicsState {
    fn default() -> Self {
        Self {
            ctm: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
        }
    }
}

#[derive(Debug, Clone, Default)]
struct TextState {
    active: bool,
    text_matrix: [f64; 6],
    text_line_matrix: [f64; 6],
    font: Option<FontInfo>,
    explicit_tm_span: Option<Range<usize>>,
    bt_span: Option<Range<usize>>,
    bt_end: Option<usize>,
    glyph_count: usize,
    base_tm_recorded: Option<[f64; 6]>,
}

pub fn extract_page_zero(document: &Document) -> Result<PageCache> {
    let (page_id, page_dict) = loader::get_page_zero(document)?;
    let media_box = page_dict
        .get(b"MediaBox")
        .ok_or_else(|| anyhow!("Page missing MediaBox"))?;
    let (width_pt, height_pt) = parse_media_box(media_box)?;
    let streams = loader::resolve_contents(document, page_dict)?;
    let combined = combine_streams(&streams);
    let tokens = tokenize_stream(&combined.data)?;

    let mut gs = GraphicsState::default();
    let mut gs_stack: Vec<GraphicsState> = Vec::new();
    let mut text_state = TextState::default();
    let mut last_cm_span: Option<(Range<usize>, [f64; 6])> = None;
    let mut objects: Vec<PageObject> = Vec::new();
    let mut meta_map: HashMap<String, ObjectMeta> = HashMap::new();
    let mut text_index = 0usize;
    let mut image_index = 0usize;

    for token in &tokens {
        match token.operator {
            Operator::q => {
                gs_stack.push(gs.clone());
            }
            Operator::Q => {
                gs = gs_stack.pop().unwrap_or_default();
                last_cm_span = None;
            }
            Operator::Cm => {
                let matrix = parse_matrix(&token.operands)?;
                gs.ctm = multiply(&gs.ctm, &matrix);
                last_cm_span = Some((token.span.clone(), matrix));
            }
            Operator::Bt => {
                text_state = TextState {
                    active: true,
                    text_matrix: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
                    text_line_matrix: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
                    font: text_state.font.clone(),
                    explicit_tm_span: None,
                    bt_span: Some(token.span.clone()),
                    bt_end: Some(token.span.end),
                    glyph_count: 0,
                    base_tm_recorded: None,
                };
            }
            Operator::Et => {
                if text_state.active && text_state.glyph_count > 0 {
                    let tm = text_state
                        .base_tm_recorded
                        .unwrap_or(text_state.text_matrix);
                    let id = format!("t:{text_index}");
                    text_index += 1;
                    let bt_span = text_state
                        .bt_span
                        .clone()
                        .ok_or_else(|| anyhow!("Missing BT span"))?;
                    let et_span = token.span.clone();
                    let combined_span = bt_span.start..et_span.end;
                    let (stream_obj, local_span) = combined.resolve(&combined_span)?;
                    let tm_span = match text_state.explicit_tm_span.clone() {
                        Some(span) => {
                            let (tm_stream, tm_local) = combined.resolve(&span)?;
                            if tm_stream != stream_obj {
                                None
                            } else {
                                Some(ByteRange {
                                    start: tm_local.start as u32,
                                    end: tm_local.end as u32,
                                })
                            }
                        }
                        None => None,
                    };
                    let font = text_state
                        .font
                        .clone()
                        .unwrap_or(FontInfo { res_name: "F1".into(), size: 12.0 });
                    let width = font.size * text_state.glyph_count as f64;
                    let height = font.size;
                    let bbox = [tm[4], tm[5] - height, tm[4] + width, tm[5]];
                    let text_object = TextRun {
                        id: id.clone(),
                        tm,
                        font: font.clone(),
                        bt_span: crate::types::BtSpan {
                            stream_obj,
                            start: local_span.start as u32,
                            end: local_span.end as u32,
                        },
                        bbox,
                    };
                    objects.push(PageObject::Text(text_object));
                    let insert_after_bt = text_state
                        .bt_end
                        .ok_or_else(|| anyhow!("Missing BT end span"))?;
                    let (_, local_insert) = combined.resolve(&(bt_span.start..insert_after_bt))?;
                    meta_map.insert(
                        id.clone(),
                        ObjectMeta::Text(TextMeta {
                            stream_obj,
                            bt_span: ByteRange {
                                start: local_span.start as u32,
                                end: local_span.end as u32,
                            },
                            tm_span,
                            insert_after_bt: local_insert.end as u32,
                            tm,
                            font,
                        }),
                    );
                }
                text_state = TextState::default();
            }
            Operator::Tf => {
                if token.operands.len() == 2 {
                    if let Operand::Name(name) = &token.operands[0] {
                        if let Operand::Number(size) = token.operands[1] {
                            text_state.font = Some(FontInfo {
                                res_name: name.trim_start_matches('/').to_string(),
                                size,
                            });
                        }
                    }
                }
            }
            Operator::Tm => {
                let matrix = parse_matrix(&token.operands)?;
                text_state.text_matrix = matrix;
                text_state.text_line_matrix = matrix;
                text_state.explicit_tm_span = Some(token.span.clone());
                if text_state.base_tm_recorded.is_none() {
                    text_state.base_tm_recorded = Some(matrix);
                }
            }
            Operator::Td => {
                let translation = parse_translation(&token.operands)?;
                text_state.text_line_matrix = multiply(&text_state.text_line_matrix, &translation);
                text_state.text_matrix = text_state.text_line_matrix;
            }
            Operator::TStar => {
                let leading = text_state
                    .font
                    .as_ref()
                    .map(|font| -font.size)
                    .unwrap_or(-12.0);
                let translation = [1.0, 0.0, 0.0, 1.0, 0.0, leading];
                text_state.text_line_matrix = multiply(&text_state.text_line_matrix, &translation);
                text_state.text_matrix = text_state.text_line_matrix;
            }
            Operator::Tj => {
                if text_state.base_tm_recorded.is_none() {
                    text_state.base_tm_recorded = Some(text_state.text_matrix);
                }
                text_state.glyph_count += glyph_count_from_operands(&token.operands);
            }
            Operator::TjArray => {
                if text_state.base_tm_recorded.is_none() {
                    text_state.base_tm_recorded = Some(text_state.text_matrix);
                }
                text_state.glyph_count += glyph_count_from_operands(&token.operands);
            }
            Operator::Do => {
                let name = token
                    .operands
                    .first()
                    .and_then(|operand| match operand {
                        Operand::Name(name) => Some(name.clone()),
                        _ => None,
                    })
                    .ok_or_else(|| anyhow!("Do operator missing name operand"))?;
                let cm = gs.ctm;
                let id = format!("img:{image_index}");
                image_index += 1;
                let bbox = [cm[4], cm[5] - cm[3], cm[4] + cm[0], cm[5]];
                let (stream_obj, span, matrix) = if let Some((span, matrix)) = last_cm_span.clone() {
                    let (stream_obj, local_span) = combined.resolve(&span)?;
                    (stream_obj, local_span, matrix)
                } else {
                    let (stream_obj, local_span) = combined.resolve(&token.span)?;
                    (stream_obj, local_span, cm)
                };
                let image = ImageObject {
                    id: id.clone(),
                    x_object: name.trim_start_matches('/').to_string(),
                    cm,
                    bbox,
                };
                objects.push(PageObject::Image(image));
                meta_map.insert(
                    id,
                    ObjectMeta::Image(ImageMeta {
                        stream_obj,
                        cm_span: ByteRange {
                            start: span.start as u32,
                            end: span.end as u32,
                        },
                        cm: matrix,
                        x_object: name.trim_start_matches('/').to_string(),
                        bbox,
                    }),
                );
            }
            _ => {}
        }
    }

    let page_ir = PageIR {
        index: 0,
        width_pt,
        height_pt,
        objects,
    };
    let ir = DocumentIR { pages: vec![page_ir] };

    Ok(PageCache {
        page_id,
        ir,
        width_pt,
        height_pt,
        objects: meta_map,
    })
}

fn combine_streams(streams: &[(ObjectId, Vec<u8>)]) -> CombinedContent {
    let mut data = Vec::new();
    let mut segments = Vec::new();
    for (index, (object_id, bytes)) in streams.iter().enumerate() {
        let start = data.len();
        data.extend_from_slice(bytes);
        let end = data.len();
        segments.push(Segment {
            stream_obj: object_id.0,
            range: start..end,
        });
        if index + 1 < streams.len() {
            data.push(b'\n');
        }
    }
    CombinedContent { data, segments }
}

fn parse_media_box(object: &Object) -> Result<(f64, f64)> {
    let array = object
        .as_array()
        .map_err(|_| anyhow!("MediaBox is not an array"))?;
    if array.len() < 4 {
        return Err(anyhow!("MediaBox missing coordinates"));
    }
    let llx = array[0].as_f64()?;
    let lly = array[1].as_f64()?;
    let urx = array[2].as_f64()?;
    let ury = array[3].as_f64()?;
    Ok((urx - llx, ury - lly))
}

fn parse_matrix(operands: &[Operand]) -> Result<[f64; 6]> {
    if operands.len() != 6 {
        return Err(anyhow!("Matrix operator requires six operands"));
    }
    let mut values = [0f64; 6];
    for (idx, operand) in operands.iter().enumerate() {
        values[idx] = match operand {
            Operand::Number(value) => *value,
            Operand::Name(name) => name.parse()?,
            _ => return Err(anyhow!("Unexpected operand type")),
        };
    }
    Ok(values)
}

fn parse_translation(operands: &[Operand]) -> Result<[f64; 6]> {
    if operands.len() != 2 {
        return Err(anyhow!("Translation requires two operands"));
    }
    let tx = match operands[0] {
        Operand::Number(value) => value,
        _ => return Err(anyhow!("Invalid translation operand")),
    };
    let ty = match operands[1] {
        Operand::Number(value) => value,
        _ => return Err(anyhow!("Invalid translation operand")),
    };
    Ok([1.0, 0.0, 0.0, 1.0, tx, ty])
}

fn multiply(a: &[f64; 6], b: &[f64; 6]) -> [f64; 6] {
    [
        a[0] * b[0] + a[2] * b[1],
        a[1] * b[0] + a[3] * b[1],
        a[0] * b[2] + a[2] * b[3],
        a[1] * b[2] + a[3] * b[3],
        a[0] * b[4] + a[2] * b[5] + a[4],
        a[1] * b[4] + a[3] * b[5] + a[5],
    ]
}

fn glyph_count_from_operands(operands: &[Operand]) -> usize {
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
