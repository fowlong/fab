//! Content extraction for page zero.

use std::{
    collections::{BTreeMap, HashMap, HashSet},
    ops::Range,
    str::FromStr,
};

use anyhow::{anyhow, bail, Context, Result};
use lopdf::{Dictionary, Document, Object, ObjectId};

use crate::types::{BtSpan, DocumentIR, FontInfo, ImageObject, PageIR, PageObject, TextObject};

use super::content::{self, ContentOp, Matrix, Operand, Operator};

/// Cached information about page zero used when applying patches.
#[derive(Clone, Debug)]
pub struct PageCache {
    pub page_id: ObjectId,
    pub streams: HashMap<u32, StreamCache>,
    pub objects: HashMap<String, ObjectCache>,
    pub width_pt: f64,
    pub height_pt: f64,
    pub order: Vec<u32>,
}

#[derive(Clone, Debug)]
pub struct StreamCache {
    pub object_id: ObjectId,
    pub dictionary: Dictionary,
    pub content: Vec<u8>,
    pub operations: Vec<ContentOp>,
}

#[derive(Clone, Debug)]
pub enum ObjectCache {
    Text(TextCache),
    Image(ImageCache),
}

#[derive(Clone, Debug)]
pub struct TextCache {
    pub stream_obj: u32,
    pub span: Range<usize>,
    pub tm: Matrix,
    pub tm_range: Option<Range<usize>>,
    pub bt_range: Range<usize>,
    pub insert_after_bt: usize,
    pub font: FontInfo,
}

#[derive(Clone, Debug)]
pub struct ImageCache {
    pub stream_obj: u32,
    pub cm_range: Range<usize>,
    pub cm: Matrix,
    pub x_object: String,
}

#[derive(Clone, Debug, Default)]
pub struct ExtractionCaches {
    pub page0: Option<PageCache>,
}

/// Extract page zero from the parsed document.
pub fn extract_page_zero(doc: &Document) -> Result<(DocumentIR, ExtractionCaches)> {
    let mut pages: BTreeMap<u32, ObjectId> = doc.get_pages();
    let (&page_number, &page_id) = pages
        .iter()
        .min_by_key(|(index, _)| *index)
        .ok_or_else(|| anyhow!("document contains no pages"))?;
    let page_dict = doc
        .get_object(page_id)?
        .as_dict()
        .map_err(|_| anyhow!("page object is not a dictionary"))?;

    let (width_pt, height_pt) = resolve_media_box(doc, page_dict)?;
    let image_names = resolve_image_xobjects(doc, page_dict)?;
    let mut streams = HashMap::new();
    let stream_entries = resolve_streams(doc, page_dict)?;
    let mut stream_order = Vec::new();

    let mut page_ir = PageIR {
        index: (page_number - 1) as usize,
        width_pt,
        height_pt,
        objects: Vec::new(),
    };

    let mut objects = HashMap::new();
    let mut text_counter = 0usize;
    let mut image_counter = 0usize;

    #[derive(Clone)]
    struct GraphicsState {
        ctm: Matrix,
        last_cm_range: Option<Range<usize>>,
        last_cm_value: Option<Matrix>,
    }

    impl Default for GraphicsState {
        fn default() -> Self {
            Self {
                ctm: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
                last_cm_range: None,
                last_cm_value: None,
            }
        }
    }

    let mut state = GraphicsState::default();
    let mut state_stack: Vec<GraphicsState> = Vec::new();
    let mut current_font: Option<FontInfo> = None;
    let mut inside_text = false;
    let mut text_matrix: Option<Matrix> = None;
    let mut run_start: Option<usize> = None;
    let mut run_stream: Option<u32> = None;
    let mut run_tm: Option<Matrix> = None;
    let mut run_bt_range: Option<Range<usize>> = None;
    let mut run_tm_range: Option<Range<usize>> = None;
    let mut run_insert_after_bt: Option<usize> = None;
    let mut run_font: Option<FontInfo> = None;
    let mut run_char_estimate: usize = 0;

    for stream_data in &stream_entries {
        streams.insert(
            stream_data.object_id.0,
            StreamCache {
                object_id: stream_data.object_id,
                dictionary: stream_data.dictionary.clone(),
                content: stream_data.content.clone(),
                operations: stream_data.operations.clone(),
            },
        );
        stream_order.push(stream_data.object_id.0);

        for op in &stream_data.operations {
            match op.operator {
                Operator::q => {
                    state_stack.push(state.clone());
                }
                Operator::Q => {
                    state = state_stack.pop().unwrap_or_default();
                }
                Operator::Cm => {
                    let matrix = content::parse_matrix(&op.operands)?;
                    state.ctm = content::multiply(&state.ctm, &matrix);
                    state.last_cm_range = Some(op.range.clone());
                    state.last_cm_value = Some(matrix);
                }
                Operator::Bt => {
                    inside_text = true;
                    text_matrix = None;
                    run_start = None;
                    run_tm = None;
                    run_stream = None;
                    run_tm_range = None;
                    run_insert_after_bt = Some(op.range.end);
                    run_bt_range = Some(op.range.clone());
                    run_char_estimate = 0;
                }
                Operator::Et => {
                    if let (Some(start), Some(end_stream), Some(tm), Some(font)) = (
                        run_start.take(),
                        run_stream.take(),
                        run_tm.take(),
                        run_font.clone(),
                    ) {
                        let span_end = op.range.start;
                        let span = start..span_end;
                        let id = format!("t:{text_counter}");
                        text_counter += 1;
                        let bbox = text_bbox(&tm, &font, run_char_estimate.max(1));
                        let bt_span = BtSpan {
                            stream_obj: end_stream,
                            start: span.start as u32,
                            end: span.end as u32,
                        };
                        page_ir.objects.push(PageObject::Text(TextObject {
                            id: id.clone(),
                            bt_span,
                            tm,
                            font: font.clone(),
                            bbox,
                        }));
                        objects.insert(
                            id,
                            ObjectCache::Text(TextCache {
                                stream_obj: end_stream,
                                span,
                                tm,
                                tm_range: run_tm_range.clone(),
                                bt_range: run_bt_range.clone().unwrap_or(op.range.clone()),
                                insert_after_bt: run_insert_after_bt.unwrap_or(op.range.start),
                                font,
                            }),
                        );
                    }
                    inside_text = false;
                    run_font = None;
                    run_char_estimate = 0;
                }
                Operator::Tf => {
                    if op.operands.len() >= 2 {
                        if let Operand::Name(name) = &op.operands[0] {
                            let size = operand_to_f64(&op.operands[1])?;
                            current_font = Some(FontInfo {
                                res_name: name.clone(),
                                size,
                            });
                        }
                    }
                }
                Operator::Tm => {
                    if !inside_text {
                        continue;
                    }
                    let matrix = content::parse_matrix(&op.operands)?;
                    text_matrix = Some(matrix);
                    run_tm_range = Some(op.range.clone());
                }
                Operator::Td => {
                    if !inside_text {
                        continue;
                    }
                    let matrix = content::parse_matrix(&op.operands)?;
                    let translation = [1.0, 0.0, 0.0, 1.0, matrix[4], matrix[5]];
                    text_matrix = Some(match text_matrix {
                        Some(current) => content::multiply(&current, &translation),
                        None => translation,
                    });
                }
                Operator::TStar => {
                    if !inside_text {
                        continue;
                    }
                    // Move to start of next line. Approximate with unit downwards movement.
                    let translation = [1.0, 0.0, 0.0, 1.0, 0.0, -1.0];
                    text_matrix = Some(match text_matrix {
                        Some(current) => content::multiply(&current, &translation),
                        None => translation,
                    });
                }
                Operator::Tj | Operator::TjArray => {
                    if !inside_text {
                        continue;
                    }
                    if run_start.is_none() {
                        run_start = Some(op.range.start);
                        run_stream = Some(stream_data.object_id.0);
                        let base_tm = text_matrix.unwrap_or([1.0, 0.0, 0.0, 1.0, 0.0, 0.0]);
                        run_tm = Some(base_tm);
                        run_font = current_font.clone();
                    }
                    run_char_estimate += estimate_glyphs(&op.operands);
                }
                Operator::Do => {
                    if op.operands.is_empty() {
                        continue;
                    }
                    if let Operand::Name(name) = &op.operands[0] {
                        if image_names.contains(name) {
                            let id = format!("img:{image_counter}");
                            image_counter += 1;
                            let operator_cm = state
                                .last_cm_value
                                .unwrap_or([1.0, 0.0, 0.0, 1.0, 0.0, 0.0]);
                            let effective_cm = state.ctm;
                            let bbox = image_bbox(&effective_cm);
                            let cm_range = state.last_cm_range.clone().unwrap_or(op.range.clone());
                            page_ir.objects.push(PageObject::Image(ImageObject {
                                id: id.clone(),
                                x_object: name.clone(),
                                cm: operator_cm,
                                bbox,
                            }));
                            objects.insert(
                                id,
                                ObjectCache::Image(ImageCache {
                                    stream_obj: stream_data.object_id.0,
                                    cm_range,
                                    cm: operator_cm,
                                    x_object: name.clone(),
                                }),
                            );
                            state.last_cm_range = None;
                            state.last_cm_value = None;
                        }
                    }
                }
                _ => {}
            }
        }
    }

    let mut doc_ir = DocumentIR::default();
    doc_ir.pages.push(page_ir);
    Ok((
        doc_ir,
        ExtractionCaches {
            page0: Some(PageCache {
                page_id,
                streams,
                objects,
                width_pt,
                height_pt,
                order: stream_order,
            }),
        },
    ))
}

fn operand_to_f64(operand: &Operand) -> Result<f64> {
    match operand {
        Operand::Number(value) => Ok(*value),
        Operand::Raw(text) => f64::from_str(text).context("invalid numeric literal"),
        _ => bail!("expected numeric operand"),
    }
}

fn resolve_media_box(doc: &Document, page: &Dictionary) -> Result<(f64, f64)> {
    let media_box = page
        .get(b"MediaBox")
        .or_else(|_| page.get(b"CropBox"))
        .context("page missing MediaBox")?;
    let array = resolve_array(doc, media_box)?;
    if array.len() != 4 {
        bail!("media box must have four numbers");
    }
    let llx = object_to_f64(&array[0])?;
    let lly = object_to_f64(&array[1])?;
    let urx = object_to_f64(&array[2])?;
    let ury = object_to_f64(&array[3])?;
    Ok((urx - llx, ury - lly))
}

fn resolve_image_xobjects(doc: &Document, page: &Dictionary) -> Result<HashSet<String>> {
    let mut set = HashSet::new();
    if let Ok(resources) = page.get(b"Resources") {
        let dict = resolve_dict(doc, resources)?;
        if let Ok(xobjects_obj) = dict.get(b"XObject") {
            let xobjects_dict = resolve_dict(doc, xobjects_obj)?;
            for (name, object) in xobjects_dict.iter() {
                if let Object::Reference(id) = object {
                    if let Ok(Object::Stream(stream)) = doc.get_object(*id) {
                        if let Ok(Object::Name(subtype)) = stream.dict.get(b"Subtype") {
                            if subtype == b"Image" {
                                set.insert(String::from_utf8_lossy(name).into_owned());
                            }
                        }
                    }
                }
            }
        }
    }
    Ok(set)
}

struct StreamEntry {
    object_id: ObjectId,
    dictionary: Dictionary,
    content: Vec<u8>,
    operations: Vec<ContentOp>,
}

fn resolve_streams(doc: &Document, page: &Dictionary) -> Result<Vec<StreamEntry>> {
    let contents = page
        .get(b"Contents")
        .context("page missing Contents entry")?;
    let mut streams = Vec::new();
    match contents {
        Object::Reference(id) => {
            let entry = load_stream(doc, *id)?;
            streams.push(entry);
        }
        Object::Array(array) => {
            for object in array {
                match object {
                    Object::Reference(id) => {
                        streams.push(load_stream(doc, *id)?);
                    }
                    _ => bail!("unsupported inline content stream"),
                }
            }
        }
        _ => bail!("unexpected /Contents type"),
    }
    Ok(streams)
}

fn load_stream(doc: &Document, id: ObjectId) -> Result<StreamEntry> {
    let stream = doc.get_object(id)?.as_stream()?.clone();
    let content = stream.decompressed_content()?;
    let operations = content::tokenize_stream(&content)?;
    Ok(StreamEntry {
        object_id: id,
        dictionary: stream.dict.clone(),
        content,
        operations,
    })
}

fn resolve_dict<'a>(doc: &'a Document, object: &'a Object) -> Result<&'a Dictionary> {
    match object {
        Object::Reference(id) => doc
            .get_object(*id)?
            .as_dict()
            .map_err(|_| anyhow!("expected dictionary")),
        Object::Dictionary(dict) => Ok(dict),
        _ => bail!("expected dictionary"),
    }
}

fn resolve_array<'a>(doc: &'a Document, object: &'a Object) -> Result<Vec<Object>> {
    match object {
        Object::Reference(id) => {
            let obj = doc.get_object(*id)?;
            resolve_array(doc, obj)
        }
        Object::Array(items) => Ok(items.clone()),
        _ => bail!("expected array"),
    }
}

fn object_to_f64(object: &Object) -> Result<f64> {
    match object {
        Object::Integer(value) => Ok(*value as f64),
        Object::Real(value) => Ok(*value as f64),
        Object::Reference(_) => bail!("unexpected reference in MediaBox"),
        _ => bail!("unexpected number type"),
    }
}

fn text_bbox(tm: &Matrix, font: &FontInfo, glyphs: usize) -> [f64; 4] {
    let glyphs = glyphs as f64;
    let width = font.size * 0.6 * glyphs;
    let height = font.size;
    let x0 = tm[4];
    let y1 = tm[5];
    let y0 = y1 - height;
    let x1 = x0 + width;
    [x0, y0, x1, y1]
}

fn image_bbox(cm: &Matrix) -> [f64; 4] {
    let width = cm[0].abs();
    let height = cm[3].abs();
    let x0 = cm[4];
    let y1 = cm[5];
    let y0 = y1 - height;
    let x1 = x0 + width;
    [x0, y0, x1, y1]
}

fn estimate_glyphs(operands: &[Operand]) -> usize {
    operands
        .iter()
        .map(|operand| match operand {
            Operand::Str(s) => s.chars().count(),
            Operand::Array(items) => items
                .iter()
                .map(|item| match item {
                    Operand::Str(s) => s.chars().count(),
                    _ => 0,
                })
                .sum(),
            _ => 0,
        })
        .sum()
}
