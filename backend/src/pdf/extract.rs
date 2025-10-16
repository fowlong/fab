//! Extraction pipeline building the runtime IR and caches.

use std::{collections::HashMap, ops::Range};

use anyhow::{anyhow, Context, Result};
use lopdf::{Document, Object, ObjectId};

use crate::{
    pdf::content::{self, Matrix, Operand, Token, IDENTITY},
    types::{DocumentIR, FontInfo, ImageObject, PageIR, PageObject, StreamSpan, TextObject},
};

/// Cached metadata for the currently opened document.
#[derive(Debug, Clone)]
pub struct DocumentCache {
    pub ir: DocumentIR,
    pub page0: PageCache,
}

/// Cached metadata for page 0 (the only page handled in the MVP).
#[derive(Debug, Clone)]
pub struct PageCache {
    pub page_id: ObjectId,
    pub width_pt: f64,
    pub height_pt: f64,
    pub contents_order: Vec<ObjectId>,
    pub contents_is_array: bool,
    pub streams: HashMap<u32, StreamCache>,
    pub text_objects: HashMap<String, TextCache>,
    pub image_objects: HashMap<String, ImageCache>,
}

/// Raw stream data associated with a page.
#[derive(Debug, Clone)]
pub struct StreamCache {
    pub object_id: ObjectId,
    pub object_number: u32,
    pub data: Vec<u8>,
}

/// Cached text metadata used during patching.
#[derive(Debug, Clone)]
pub struct TextCache {
    pub stream_id: ObjectId,
    pub bt_span: Range<usize>,
    pub tm_range: Option<Range<usize>>,
    pub base_tm: Matrix,
    pub explicit_tm: bool,
    pub tm_insert_pos: usize,
    pub font: FontInfo,
}

/// Cached image metadata used during patching.
#[derive(Debug, Clone)]
pub struct ImageCache {
    pub stream_id: ObjectId,
    pub cm_span: Range<usize>,
    pub cm_matrix: Matrix,
    pub x_object: String,
}

/// Build the document cache from the parsed PDF document.
pub fn extract_document(doc: &Document) -> Result<DocumentCache> {
    let (page_index, page_id) = doc
        .get_pages()
        .into_iter()
        .next()
        .ok_or_else(|| anyhow!("document has no pages"))?;
    let (width_pt, height_pt) = resolve_media_box(doc, page_id)?;

    let page_dict = doc
        .get_dictionary(page_id)
        .with_context(|| "page dictionary missing")?;
    let contents_is_array = matches!(page_dict.get(b"Contents"), Ok(Object::Array(_)));
    let contents_order = doc.get_page_contents(page_id);

    let mut streams = HashMap::new();
    let mut text_objects = HashMap::new();
    let mut image_objects = HashMap::new();
    let mut page_objects = Vec::new();
    let mut text_index = 0usize;
    let mut image_index = 0usize;

    for stream_id in &contents_order {
        let stream_object = doc
            .get_object(*stream_id)
            .with_context(|| format!("stream object {:?} missing", stream_id))?
            .as_stream()
            .with_context(|| format!("object {:?} is not a stream", stream_id))?;
        let data = stream_object
            .decompressed_content()
            .with_context(|| format!("failed to decode stream {:?}", stream_id))?;
        let tokens = content::tokenize_stream(&data)?;

        let cache_entry = StreamCache {
            object_id: *stream_id,
            object_number: stream_id.0,
            data: data.clone(),
        };
        streams.insert(stream_id.0, cache_entry);

        let extraction = extract_stream_objects(&tokens, *stream_id, text_index, image_index);
        text_index = extraction.next_text_index;
        image_index = extraction.next_image_index;
        page_objects.extend(extraction.page_objects);
        text_objects.extend(extraction.text_map);
        image_objects.extend(extraction.image_map);
    }

    let page_ir = PageIR {
        index: (page_index - 1) as usize,
        width_pt,
        height_pt,
        objects: page_objects,
    };

    let cache = PageCache {
        page_id,
        width_pt,
        height_pt,
        contents_order,
        contents_is_array,
        streams,
        text_objects,
        image_objects,
    };

    Ok(DocumentCache {
        ir: DocumentIR {
            pages: vec![page_ir],
        },
        page0: cache,
    })
}

struct StreamExtraction {
    page_objects: Vec<PageObject>,
    text_map: HashMap<String, TextCache>,
    image_map: HashMap<String, ImageCache>,
    next_text_index: usize,
    next_image_index: usize,
}

fn extract_stream_objects(
    tokens: &[Token],
    stream_id: ObjectId,
    text_index_start: usize,
    image_index_start: usize,
) -> StreamExtraction {
    let mut page_objects = Vec::new();
    let mut text_map = HashMap::new();
    let mut image_map = HashMap::new();

    let mut state = GraphicsState::default();
    let mut stack: Vec<GraphicsState> = Vec::new();
    let mut current_text: Option<TextBuilder> = None;
    let mut text_index = text_index_start;
    let mut image_index = image_index_start;

    for token in tokens {
        match token.operator.as_str() {
            "q" => {
                stack.push(state.clone());
            }
            "Q" => {
                if let Some(previous) = stack.pop() {
                    state = previous;
                } else {
                    state = GraphicsState::default();
                }
            }
            "cm" => {
                if let Some(matrix) = matrix_from_operands(&token.operands) {
                    state.ctm = content::multiply(&state.ctm, &matrix);
                    state.last_cm = Some((matrix, token.span.clone()));
                }
            }
            "BT" => {
                state.text_matrix = IDENTITY;
                state.text_line_matrix = IDENTITY;
                current_text = Some(TextBuilder::new(
                    stream_id,
                    text_index,
                    token.span.clone(),
                    state.font_name.clone(),
                    state.font_size,
                ));
                text_index += 1;
            }
            "ET" => {
                if let Some(builder) = current_text.take() {
                    if let Some(text_object) = builder.finish(token.span.end) {
                        page_objects.push(PageObject::Text(text_object.ir.clone()));
                        text_map.insert(text_object.ir.id.clone(), text_object.cache);
                    }
                }
                state.text_matrix = IDENTITY;
                state.text_line_matrix = IDENTITY;
            }
            "Tf" => {
                if let (Some(name), Some(size)) = (
                    token.operands.get(0).and_then(Operand::as_name),
                    token.operands.get(1).and_then(Operand::as_number),
                ) {
                    state.font_name = Some(name.to_string());
                    state.font_size = size;
                    state.text_leading = size;
                    if let Some(builder) = current_text.as_mut() {
                        builder.update_font(name.to_string(), size);
                    }
                }
            }
            "Tm" => {
                if let Some(matrix) = matrix_from_operands(&token.operands) {
                    state.text_matrix = matrix;
                    state.text_line_matrix = matrix;
                    if let Some(builder) = current_text.as_mut() {
                        builder.record_tm(matrix, token.span.clone());
                    }
                }
            }
            "Td" => {
                if let (Some(tx), Some(ty)) = (
                    token.operands.get(0).and_then(Operand::as_number),
                    token.operands.get(1).and_then(Operand::as_number),
                ) {
                    let translation = [1.0, 0.0, 0.0, 1.0, tx, ty];
                    state.text_line_matrix =
                        content::multiply(&translation, &state.text_line_matrix);
                    state.text_matrix = state.text_line_matrix;
                    if let Some(builder) = current_text.as_mut() {
                        builder.record_position_op(token.span.end);
                    }
                }
            }
            "TD" => {
                if let (Some(tx), Some(ty)) = (
                    token.operands.get(0).and_then(Operand::as_number),
                    token.operands.get(1).and_then(Operand::as_number),
                ) {
                    let translation = [1.0, 0.0, 0.0, 1.0, tx, ty];
                    state.text_line_matrix =
                        content::multiply(&translation, &state.text_line_matrix);
                    state.text_matrix = state.text_line_matrix;
                    state.text_leading = -ty;
                    if let Some(builder) = current_text.as_mut() {
                        builder.record_position_op(token.span.end);
                    }
                }
            }
            "T*" => {
                let ty = -state.text_leading;
                let translation = [1.0, 0.0, 0.0, 1.0, 0.0, ty];
                state.text_line_matrix = content::multiply(&translation, &state.text_line_matrix);
                state.text_matrix = state.text_line_matrix;
                if let Some(builder) = current_text.as_mut() {
                    builder.record_position_op(token.span.end);
                }
            }
            "Tj" | "TJ" => {
                if let Some(builder) = current_text.as_mut() {
                    let units = count_text_units(&token.operands);
                    builder.record_text(&state, units, &token.span);
                }
            }
            "Do" => {
                if let Some(name) = token.operands.get(0).and_then(Operand::as_name) {
                    if let Some((matrix, span)) = &state.last_cm {
                        let object_id = format!("img:{}", image_index);
                        image_index += 1;
                        let width = matrix[0].abs();
                        let height = matrix[3].abs();
                        let bbox = [matrix[4], matrix[5] - height, matrix[4] + width, matrix[5]];
                        page_objects.push(PageObject::Image(ImageObject {
                            id: object_id.clone(),
                            x_object: name.to_string(),
                            cm: state.ctm,
                            bbox,
                        }));
                        image_map.insert(
                            object_id,
                            ImageCache {
                                stream_id,
                                cm_span: span.clone(),
                                cm_matrix: *matrix,
                                x_object: name.to_string(),
                            },
                        );
                    }
                }
            }
            _ => {}
        }

        if let Some(builder) = current_text.as_mut() {
            builder.update_last_span(token.span.end);
        }
    }

    StreamExtraction {
        page_objects,
        text_map,
        image_map,
        next_text_index: text_index,
        next_image_index: image_index,
    }
}

#[derive(Clone, Debug)]
struct GraphicsState {
    ctm: Matrix,
    text_matrix: Matrix,
    text_line_matrix: Matrix,
    text_leading: f64,
    font_name: Option<String>,
    font_size: f64,
    last_cm: Option<(Matrix, Range<usize>)>,
}

impl Default for GraphicsState {
    fn default() -> Self {
        Self {
            ctm: IDENTITY,
            text_matrix: IDENTITY,
            text_line_matrix: IDENTITY,
            text_leading: 0.0,
            font_name: None,
            font_size: 12.0,
            last_cm: None,
        }
    }
}

struct BuiltText {
    ir: TextObject,
    cache: TextCache,
}

struct TextBuilder {
    stream_id: ObjectId,
    id: String,
    bt_span: Range<usize>,
    tm_range: Option<Range<usize>>,
    tm_insert_pos: usize,
    base_tm: Matrix,
    explicit_tm: bool,
    font: FontInfo,
    seen_text: bool,
    last_span_end: usize,
    glyph_count: usize,
}

impl TextBuilder {
    fn new(
        stream_id: ObjectId,
        index: usize,
        bt_span: Range<usize>,
        font_name: Option<String>,
        font_size: f64,
    ) -> Self {
        Self {
            stream_id,
            id: format!("t:{}", index),
            bt_span: bt_span.clone(),
            tm_range: None,
            tm_insert_pos: bt_span.end,
            base_tm: IDENTITY,
            explicit_tm: false,
            font: FontInfo {
                res_name: font_name.unwrap_or_else(|| "F0".into()),
                size: font_size,
            },
            seen_text: false,
            last_span_end: bt_span.end,
            glyph_count: 0,
        }
    }

    fn update_font(&mut self, name: String, size: f64) {
        self.font = FontInfo {
            res_name: name,
            size,
        };
    }

    fn record_tm(&mut self, matrix: Matrix, span: Range<usize>) {
        if !self.seen_text {
            self.tm_range = Some(span);
            self.base_tm = matrix;
            self.explicit_tm = true;
        }
    }

    fn record_position_op(&mut self, span_end: usize) {
        if !self.seen_text {
            self.tm_insert_pos = span_end;
        }
    }

    fn record_text(&mut self, state: &GraphicsState, units: usize, span: &Range<usize>) {
        if !self.seen_text {
            if self.tm_range.is_none() {
                self.tm_insert_pos = self.tm_insert_pos.max(span.start);
            }
            self.base_tm = state.text_matrix;
            self.seen_text = true;
            if self.font.res_name.is_empty() {
                self.font.res_name = state.font_name.clone().unwrap_or_else(|| "F0".into());
            }
            self.font.size = state.font_size;
        }
        self.glyph_count += units;
        self.last_span_end = span.end;
    }

    fn update_last_span(&mut self, span_end: usize) {
        self.last_span_end = span_end;
    }

    fn finish(mut self, et_end: usize) -> Option<BuiltText> {
        if !self.seen_text {
            return None;
        }
        self.last_span_end = et_end;
        let span = self.bt_span.start..self.last_span_end;
        let tm = self.base_tm;
        let width = self.font.size * self.glyph_count as f64 * 0.6;
        let height = self.font.size;
        let x = tm[4];
        let y = tm[5];
        let bbox = [x, y - height, x + width, y];

        Some(BuiltText {
            ir: TextObject {
                id: self.id.clone(),
                bt_span: stream_span(&span, self.stream_id),
                tm,
                font: self.font.clone(),
                bbox,
            },
            cache: TextCache {
                stream_id: self.stream_id,
                bt_span: span,
                tm_range: self.tm_range.clone(),
                base_tm: tm,
                explicit_tm: self.explicit_tm,
                tm_insert_pos: self.tm_insert_pos,
                font: self.font.clone(),
            },
        })
    }
}

fn resolve_media_box(doc: &Document, mut page_id: ObjectId) -> Result<(f64, f64)> {
    loop {
        let dict = doc
            .get_dictionary(page_id)
            .with_context(|| "page dictionary missing during MediaBox lookup")?;
        if let Ok(Object::Array(values)) = dict.get(b"MediaBox") {
            if values.len() == 4 {
                let llx = value_as_f64(&values[0])?;
                let lly = value_as_f64(&values[1])?;
                let urx = value_as_f64(&values[2])?;
                let ury = value_as_f64(&values[3])?;
                return Ok((urx - llx, ury - lly));
            }
        }
        if let Ok(parent) = dict.get(b"Parent").and_then(Object::as_reference) {
            page_id = parent;
        } else {
            break;
        }
    }
    Ok((595.0, 842.0))
}

fn value_as_f64(object: &Object) -> Result<f64> {
    match object {
        Object::Integer(value) => Ok(*value as f64),
        Object::Real(value) => Ok(*value as f64),
        _ => Err(anyhow!("expected numeric value")),
    }
}

fn matrix_from_operands(operands: &[Operand]) -> Option<Matrix> {
    if operands.len() >= 6 {
        let mut values = [0.0; 6];
        for (idx, operand) in operands.iter().take(6).enumerate() {
            values[idx] = operand.as_number()?;
        }
        Some(values)
    } else {
        None
    }
}

fn count_text_units(operands: &[Operand]) -> usize {
    operands
        .iter()
        .map(|operand| match operand {
            Operand::String(bytes) => bytes.len(),
            Operand::Array(items) => items
                .iter()
                .map(|value| match value {
                    Operand::String(bytes) => bytes.len(),
                    _ => 0,
                })
                .sum(),
            _ => 0,
        })
        .sum()
}

fn stream_span(span: &Range<usize>, stream_id: ObjectId) -> StreamSpan {
    StreamSpan {
        stream_obj: stream_id.0,
        start: span.start as u32,
        end: span.end as u32,
    }
}
