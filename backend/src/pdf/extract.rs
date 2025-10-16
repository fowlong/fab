//! IR extraction for text runs and image XObjects on page 0.

use std::collections::HashMap;
use std::ops::Range;

use anyhow::{anyhow, Context, Result};
use lopdf::{Dictionary, Document, Object, ObjectId};

use crate::pdf::content::{tokenize_stream, Operand, Operator, Token};
use crate::types::{DocumentIR, FontInfo, ImageObject, PageIR, PageObject, TextObject};

#[derive(Debug, Clone)]
pub struct ContentStream {
    pub object_id: ObjectId,
    pub bytes: Vec<u8>,
    pub tokens: Vec<Token>,
}

#[derive(Debug, Clone)]
pub struct TextRunMeta {
    pub stream_index: usize,
    pub stream_obj: ObjectId,
    pub span: Range<usize>,
    pub tm_span: Option<Range<usize>>,
    pub insert_offset: usize,
    pub tm: [f64; 6],
    pub font: FontInfo,
}

#[derive(Debug, Clone)]
pub struct ImageMeta {
    pub stream_index: usize,
    pub cm_span: Range<usize>,
    pub combined_cm: [f64; 6],
    pub raw_cm: [f64; 6],
    pub x_object: String,
}

#[derive(Debug, Clone)]
pub struct PageExtraction {
    pub page_id: ObjectId,
    pub ir: PageIR,
    pub streams: Vec<ContentStream>,
    pub resources: Dictionary,
    pub text_map: HashMap<String, TextRunMeta>,
    pub image_map: HashMap<String, ImageMeta>,
}

#[derive(Debug, Clone)]
pub struct ExtractionResult {
    pub document_ir: DocumentIR,
    pub page: PageExtraction,
}

pub fn extract_page_zero(doc: &Document) -> Result<ExtractionResult> {
    let pages = doc.get_pages();
    let (&page_number, &page_id) = pages
        .iter()
        .find(|(page_index, _)| **page_index == 1)
        .ok_or_else(|| anyhow!("document missing page 0"))?;
    debug_assert_eq!(page_number, 1, "lopdf pages are 1-indexed");

    let page_dict = doc
        .get_dictionary(page_id)
        .with_context(|| anyhow!("page dictionary {page_id:?} missing"))?;
    let (width, height) = page_dimensions(doc, page_dict)?;

    let mut streams = resolve_contents(doc, page_dict)?;
    let resources = resolve_resources(doc, page_dict).unwrap_or_default();

    let (page_objects, text_map, image_map) =
        analyse_streams(width, height, &mut streams, doc, &resources)?;

    let page_ir = PageIR {
        index: 0,
        width_pt: width,
        height_pt: height,
        objects: page_objects.clone(),
    };

    Ok(ExtractionResult {
        document_ir: DocumentIR {
            pages: vec![page_ir.clone()],
        },
        page: PageExtraction {
            page_id,
            ir: page_ir,
            streams,
            resources,
            text_map,
            image_map,
        },
    })
}

pub fn rebuild_page(page: &mut PageExtraction, doc: &Document) -> Result<()> {
    let (objects, text_map, image_map) = analyse_streams(
        page.ir.width_pt,
        page.ir.height_pt,
        &mut page.streams,
        doc,
        &page.resources,
    )?;
    page.ir.objects = objects;
    page.text_map = text_map;
    page.image_map = image_map;
    Ok(())
}

fn analyse_streams(
    _width: f64,
    _height: f64,
    streams: &mut [ContentStream],
    doc: &Document,
    resources: &Dictionary,
) -> Result<(
    Vec<PageObject>,
    HashMap<String, TextRunMeta>,
    HashMap<String, ImageMeta>,
)> {
    let xobject_types = load_xobject_types(doc, resources);

    let mut text_counter = 0usize;
    let mut image_counter = 0usize;
    let mut text_map = HashMap::new();
    let mut image_map = HashMap::new();
    let mut page_objects = Vec::new();

    let mut graphics_stack = vec![GraphicsState::default()];
    let mut graphics_state = GraphicsState::default();
    let mut text_state: Option<TextState> = None;
    let mut last_cm: Option<CmSnapshot> = None;

    for (stream_index, stream) in streams.iter_mut().enumerate() {
        stream.tokens = tokenize_stream(&stream.bytes)
            .with_context(|| anyhow!("failed to tokenise stream {:?}", stream.object_id))?;

        for token in &stream.tokens {
            match token.operator {
                Operator::Save => graphics_stack.push(graphics_state.clone()),
                Operator::Restore => {
                    graphics_state = graphics_stack.pop().unwrap_or_default();
                    last_cm = None;
                }
                Operator::Concat => {
                    if let Some(matrix) = matrix_from_operands(&token.operands) {
                        graphics_state.ctm = multiply(graphics_state.ctm, matrix);
                        last_cm = Some(CmSnapshot {
                            raw: matrix,
                            combined: graphics_state.ctm,
                            span: token.span.clone(),
                            stream_index,
                            stream_obj: stream.object_id,
                        });
                    }
                }
                Operator::BeginText => {
                    text_state = Some(TextState::new(token.span.end));
                }
                Operator::EndText => {
                    if let Some(state) = text_state.take() {
                        if let Some(run) = state.run {
                            let id = format!("t:{}", text_counter);
                            text_counter += 1;
                            let bbox =
                                approximate_text_bbox(&run.tm, run.glyph_count, run.font.size);
                            let text_object = TextObject {
                                id: id.clone(),
                                bt_span: crate::types::BtSpan {
                                    stream_obj: run.stream_obj.0,
                                    start: run.span.start as u32,
                                    end: run.span.end as u32,
                                },
                                tm: run.tm,
                                font: run.font.clone(),
                                bbox: Some(bbox),
                            };
                            text_map.insert(
                                id.clone(),
                                TextRunMeta {
                                    stream_index: run.stream_index,
                                    stream_obj: run.stream_obj,
                                    span: run.span,
                                    tm_span: run.tm_span,
                                    insert_offset: state.bt_insert_offset,
                                    tm: run.tm,
                                    font: run.font.clone(),
                                },
                            );
                            page_objects.push(PageObject::Text(text_object));
                        }
                    }
                }
                Operator::SetFont => {
                    if let Some(state) = text_state.as_mut() {
                        if let Some((name, size)) = parse_font_operand(&token.operands) {
                            state.font = Some(FontInfo {
                                res_name: name,
                                size,
                            });
                            if state.leading == 0.0 {
                                state.leading = size;
                            }
                        }
                    }
                }
                Operator::SetTextMatrix => {
                    if let Some(state) = text_state.as_mut() {
                        if let Some(matrix) = matrix_from_operands(&token.operands) {
                            state.text_matrix = matrix;
                            state.line_matrix = matrix;
                            state.tm_token_span = Some(token.span.clone());
                        }
                    }
                }
                Operator::TextMove => {
                    if let Some(state) = text_state.as_mut() {
                        if let Some((tx, ty)) = parse_translation(&token.operands) {
                            let translate = [1.0, 0.0, 0.0, 1.0, tx, ty];
                            state.line_matrix = multiply(state.line_matrix, translate);
                            state.text_matrix = state.line_matrix;
                            state.tm_token_span = None;
                        }
                    }
                }
                Operator::TextNextLine => {
                    if let Some(state) = text_state.as_mut() {
                        let translate = [1.0, 0.0, 0.0, 1.0, 0.0, -state.leading];
                        state.line_matrix = multiply(state.line_matrix, translate);
                        state.text_matrix = state.line_matrix;
                        state.tm_token_span = None;
                    }
                }
                Operator::ShowText | Operator::ShowTextArray => {
                    if let Some(state) = text_state.as_mut() {
                        if let Some(font) = state.font.clone() {
                            let glyph_count = glyph_count(&token.operands);
                            if glyph_count == 0 {
                                continue;
                            }
                            let run_entry = state.run.get_or_insert_with(|| CurrentRun {
                                span: token.span.clone(),
                                tm: state.text_matrix,
                                tm_span: state.tm_token_span.clone(),
                                stream_index,
                                stream_obj: stream.object_id,
                                font: font.clone(),
                                glyph_count: 0,
                            });
                            run_entry.span.end = token.span.end;
                            run_entry.glyph_count += glyph_count;
                        }
                    }
                }
                Operator::InvokeXObject => {
                    if let Some(name) = token.operands.first().and_then(as_name) {
                        if let Some(kind) = xobject_types.get(name.as_str()) {
                            if kind == "Image" {
                                if let Some(cm) = last_cm.clone() {
                                    let id = format!("img:{}", image_counter);
                                    image_counter += 1;
                                    let bbox = approximate_image_bbox(&cm.combined);
                                    let image_object = ImageObject {
                                        id: id.clone(),
                                        x_object: name.clone(),
                                        cm: cm.combined,
                                        bbox: Some(bbox),
                                    };
                                    image_map.insert(
                                        id.clone(),
                                        ImageMeta {
                                            stream_index: cm.stream_index,
                                            cm_span: cm.span,
                                            combined_cm: cm.combined,
                                            raw_cm: cm.raw,
                                            x_object: name.clone(),
                                        },
                                    );
                                    page_objects.push(PageObject::Image(image_object));
                                    last_cm = None;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok((page_objects, text_map, image_map))
}

fn page_dimensions(doc: &Document, page: &Dictionary) -> Result<(f64, f64)> {
    let array = page
        .get(b"MediaBox")
        .or_else(|_| page.get(b"CropBox"))?
        .as_array()?;
    if array.len() != 4 {
        return Err(anyhow!("invalid MediaBox"));
    }
    let llx = resolve_number(doc, &array[0])?;
    let lly = resolve_number(doc, &array[1])?;
    let urx = resolve_number(doc, &array[2])?;
    let ury = resolve_number(doc, &array[3])?;
    Ok((urx - llx, ury - lly))
}

fn resolve_number(doc: &Document, object: &Object) -> Result<f64> {
    match object {
        Object::Integer(i) => Ok(*i as f64),
        Object::Real(r) => Ok(*r as f64),
        Object::Reference(id) => resolve_number(doc, doc.get_object(*id)?),
        other => Err(anyhow!("unexpected number object: {other:?}")),
    }
}

fn resolve_contents(doc: &Document, page: &Dictionary) -> Result<Vec<ContentStream>> {
    let contents = page.get(b"Contents")?;
    let mut streams = Vec::new();
    match contents {
        Object::Reference(id) => {
            streams.push(load_stream(doc, *id)?);
        }
        Object::Array(items) => {
            for item in items {
                let id = item.as_reference()?;
                streams.push(load_stream(doc, id)?);
            }
        }
        Object::Stream(_) => {
            return Err(anyhow!("expected indirect stream reference"));
        }
        Object::Null => {}
        other => return Err(anyhow!("unexpected Contents: {other:?}")),
    }
    Ok(streams)
}

fn load_stream(doc: &Document, id: ObjectId) -> Result<ContentStream> {
    let stream = doc
        .get_object(id)
        .and_then(Object::as_stream)
        .with_context(|| anyhow!("missing stream {id:?}"))?;
    let bytes = stream
        .decompressed_content()
        .unwrap_or_else(|_| stream.content.clone());
    Ok(ContentStream {
        object_id: id,
        bytes,
        tokens: Vec::new(),
    })
}

fn resolve_resources(doc: &Document, page: &Dictionary) -> Result<Dictionary> {
    match page.get(b"Resources")? {
        Object::Dictionary(dict) => Ok(dict.clone()),
        Object::Reference(id) => Ok(doc.get_dictionary(id)?.clone()),
        other => Err(anyhow!("unexpected resources type: {other:?}")),
    }
}

fn load_xobject_types(doc: &Document, resources: &Dictionary) -> HashMap<String, String> {
    let mut mapping = HashMap::new();
    if let Ok(xobj) = resources.get(b"XObject") {
        match xobj {
            Object::Dictionary(dict) => {
                for (key, value) in dict.iter() {
                    if let Ok(name) = std::str::from_utf8(key) {
                        if let Ok(id) = value.as_reference() {
                            if let Ok(object) = doc.get_object(id) {
                                if let Ok(dict) = object.as_dict() {
                                    if let Ok(subtype) =
                                        dict.get(b"Subtype").and_then(Object::as_name_str)
                                    {
                                        mapping.insert(name.to_string(), subtype.to_string());
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Object::Reference(id) => {
                if let Ok(dict) = doc.get_dictionary(*id) {
                    for (key, value) in dict.iter() {
                        if let Ok(name) = std::str::from_utf8(key) {
                            if let Ok(sub_id) = value.as_reference() {
                                if let Ok(object) = doc.get_object(sub_id) {
                                    if let Ok(dict) = object.as_dict() {
                                        if let Ok(subtype) =
                                            dict.get(b"Subtype").and_then(Object::as_name_str)
                                        {
                                            mapping.insert(name.to_string(), subtype.to_string());
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
    mapping
}

fn parse_font_operand(operands: &[Operand]) -> Option<(String, f64)> {
    if operands.len() != 2 {
        return None;
    }
    let name = match &operands[0] {
        Operand::Name(name) => name.clone(),
        _ => return None,
    };
    let size = match operands[1] {
        Operand::Number(value) => value,
        _ => return None,
    };
    Some((name, size))
}

fn parse_translation(operands: &[Operand]) -> Option<(f64, f64)> {
    if operands.len() != 2 {
        return None;
    }
    match (&operands[0], &operands[1]) {
        (Operand::Number(tx), Operand::Number(ty)) => Some((*tx, *ty)),
        _ => None,
    }
}

fn matrix_from_operands(operands: &[Operand]) -> Option<[f64; 6]> {
    if operands.len() != 6 {
        return None;
    }
    let mut values = [0.0; 6];
    for (i, operand) in operands.iter().enumerate() {
        if let Operand::Number(value) = operand {
            values[i] = *value;
        } else {
            return None;
        }
    }
    Some(values)
}

fn as_name(operand: &Operand) -> Option<String> {
    match operand {
        Operand::Name(name) => Some(name.clone()),
        Operand::String(bytes) => std::str::from_utf8(bytes).map(|s| s.to_string()).ok(),
        _ => None,
    }
}

fn glyph_count(operands: &[Operand]) -> usize {
    operands
        .iter()
        .map(|operand| match operand {
            Operand::String(bytes) => bytes.len(),
            Operand::Array(items) => items.iter().map(glyph_count).sum(),
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

fn approximate_text_bbox(tm: &[f64; 6], glyphs: usize, font_size: f64) -> [f64; 4] {
    let width = font_size * glyphs as f64 * 0.6;
    let height = font_size;
    let corners = [
        transform(tm, 0.0, 0.0),
        transform(tm, width, 0.0),
        transform(tm, width, height),
        transform(tm, 0.0, height),
    ];
    let (min_x, max_x) = corners
        .iter()
        .fold((f64::MAX, f64::MIN), |(min, max), &(x, _)| {
            (min.min(x), max.max(x))
        });
    let (min_y, max_y) = corners
        .iter()
        .fold((f64::MAX, f64::MIN), |(min, max), &(_, y)| {
            (min.min(y), max.max(y))
        });
    [min_x, min_y, max_x, max_y]
}

fn approximate_image_bbox(cm: &[f64; 6]) -> [f64; 4] {
    let width = cm[0];
    let height = cm[3];
    let x = cm[4];
    let y = cm[5];
    [x, y - height, x + width, y]
}

fn transform(matrix: &[f64; 6], x: f64, y: f64) -> (f64, f64) {
    (
        matrix[0] * x + matrix[2] * y + matrix[4],
        matrix[1] * x + matrix[3] * y + matrix[5],
    )
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

#[derive(Debug, Clone)]
struct CmSnapshot {
    raw: [f64; 6],
    combined: [f64; 6],
    span: Range<usize>,
    stream_index: usize,
    stream_obj: ObjectId,
}

#[derive(Debug, Clone)]
struct TextState {
    text_matrix: [f64; 6],
    line_matrix: [f64; 6],
    leading: f64,
    font: Option<FontInfo>,
    tm_token_span: Option<Range<usize>>,
    bt_insert_offset: usize,
    run: Option<CurrentRun>,
}

impl TextState {
    fn new(insert_offset: usize) -> Self {
        Self {
            text_matrix: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
            line_matrix: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
            leading: 0.0,
            font: None,
            tm_token_span: None,
            bt_insert_offset: insert_offset,
            run: None,
        }
    }
}

#[derive(Debug, Clone)]
struct CurrentRun {
    span: Range<usize>,
    tm: [f64; 6],
    tm_span: Option<Range<usize>>,
    stream_index: usize,
    stream_obj: ObjectId,
    font: FontInfo,
    glyph_count: usize,
}
