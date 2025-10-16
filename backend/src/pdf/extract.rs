//! Intermediate representation extraction for page 0.
//!
//! The implementation focuses on text runs and image XObjects so that the
//! Fabric overlay can drive matrix updates. It keeps enough bookkeeping to let
//! the patcher rewrite content streams incrementally without reparsing the
//! entire document.

use std::{collections::HashMap, ops::Range};

use anyhow::{anyhow, Context, Result};
use lopdf::{Dictionary, Document, Object, ObjectId};

use crate::{
    pdf::content::{tokenize_stream, ContentToken, Operand, Operator},
    types::{DocumentIR, FontInfo, ImageObject, PageIR, PageObject, StreamSpan, TextObject},
    util::matrix::Matrix2D,
};

/// Extraction output for the MVP: the IR plus cache metadata for patching.
#[derive(Debug, Clone)]
pub struct ExtractionResult {
    pub ir: DocumentIR,
    pub page_cache: PageCache,
}

/// Cached data for page 0 used by the patcher.
#[derive(Debug, Clone)]
pub struct PageCache {
    pub page_id: ObjectId,
    pub width_pt: f64,
    pub height_pt: f64,
    pub streams: Vec<StreamCache>,
    pub objects: HashMap<String, CachedObject>,
}

#[derive(Debug, Clone)]
pub struct StreamCache {
    pub object_id: ObjectId,
    pub bytes: Vec<u8>,
    pub tokens: Vec<ContentToken>,
}

#[derive(Debug, Clone)]
pub enum CachedObject {
    Text(TextCache),
    Image(ImageCache),
}

#[derive(Debug, Clone)]
pub struct TokenLocator {
    pub stream_obj: ObjectId,
    pub token_index: usize,
    pub byte_range: Range<usize>,
}

#[derive(Debug, Clone)]
pub struct TextCache {
    pub stream_obj: ObjectId,
    pub bt: TokenLocator,
    pub et: TokenLocator,
    pub tm: Option<TokenLocator>,
    pub base_tm: [f64; 6],
}

#[derive(Debug, Clone)]
pub struct ImageCache {
    pub stream_obj: ObjectId,
    pub cm: Option<TokenLocator>,
    pub do_token: TokenLocator,
    pub current_cm: [f64; 6],
    pub xobject: String,
}

/// Extract the IR and cache metadata for the first page of the document.
pub fn extract_page_zero(doc: &Document) -> Result<ExtractionResult> {
    let (page_number, page_id) = doc
        .get_pages()
        .into_iter()
        .next()
        .ok_or_else(|| anyhow!("document has no pages"))?;
    if page_number != 1 {
        // The iterator is one-indexed; we only support the first logical page.
        tracing::debug!(page_number, "non-standard page index encountered");
    }

    let page_dict = doc
        .get_object(page_id)
        .context("failed to load page dictionary")?
        .as_dict()
        .context("page is not a dictionary")?
        .clone();

    let (width_pt, height_pt) = resolve_page_size(doc, page_id, &page_dict)?;
    let resources = resolve_resources(doc, page_id, &page_dict)?;
    let image_map = build_image_map(doc, &resources)?;
    let streams = collect_streams(doc, &page_dict)?;

    let mut page_objects: Vec<PageObject> = Vec::new();
    let mut cache_map: HashMap<String, CachedObject> = HashMap::new();
    let mut stream_caches: Vec<StreamCache> = Vec::new();

    let mut graphics_stack: Vec<(Matrix2D, Option<TokenLocator>)> = Vec::new();
    let mut current_ctm = Matrix2D::identity();
    let mut last_cm_locator: Option<TokenLocator> = None;

    let mut text_state = TextState::default();
    let mut text_counter = 0usize;
    let mut image_counter = 0usize;

    for (stream_id, bytes) in streams {
        let tokens = tokenize_stream(&bytes).context("tokenising content stream")?;
        stream_caches.push(StreamCache {
            object_id: stream_id,
            bytes: bytes.clone(),
            tokens: tokens.clone(),
        });

        for (index, token) in tokens.iter().enumerate() {
            let locator = TokenLocator {
                stream_obj: stream_id,
                token_index: index,
                byte_range: token.byte_range.clone(),
            };
            match token.operator {
                Operator::q => {
                    graphics_stack.push((current_ctm, last_cm_locator.clone()));
                    last_cm_locator = None;
                }
                Operator::Q => {
                    if let Some((ctm, cm_locator)) = graphics_stack.pop() {
                        current_ctm = ctm;
                        last_cm_locator = cm_locator;
                    } else {
                        current_ctm = Matrix2D::identity();
                        last_cm_locator = None;
                    }
                }
                Operator::Cm => {
                    let numbers = as_numbers(&token.operands, 6)?;
                    let matrix = Matrix2D::from_array(numbers.try_into().unwrap());
                    current_ctm = current_ctm.multiply(matrix);
                    last_cm_locator = Some(locator);
                }
                Operator::Bt => {
                    text_state.begin(locator.clone());
                }
                Operator::Et => {
                    if let Some(result) = text_state.end(locator.clone()) {
                        let TextRunFinished {
                            id,
                            stream_obj,
                            bt_locator,
                            et_locator,
                            tm,
                            tm_locator,
                            font,
                            glyphs,
                        } = result;
                        let span = StreamSpan {
                            stream_obj: stream_obj.0,
                            start: bt_locator.byte_range.start as u32,
                            end: et_locator.byte_range.end as u32,
                        };
                        let bbox = approximate_text_bbox(&tm, &font, glyphs);
                        page_objects.push(PageObject::Text(TextObject {
                            id: id.clone(),
                            bt_span: span,
                            tm,
                            font: font.clone(),
                            bbox,
                        }));
                        cache_map.insert(
                            id,
                            CachedObject::Text(TextCache {
                                stream_obj,
                                bt: bt_locator,
                                et: et_locator,
                                tm: tm_locator,
                                base_tm: tm,
                            }),
                        );
                    }
                    text_state.reset_after_et();
                }
                Operator::Tf => {
                    if token.operands.len() == 2 {
                        if let (Operand::Name(name), Operand::Number(size)) =
                            (&token.operands[0], &token.operands[1])
                        {
                            text_state.set_font(FontInfo {
                                res_name: name.clone(),
                                size: *size,
                            });
                        }
                    }
                }
                Operator::Tm => {
                    let numbers = as_numbers(&token.operands, 6)?;
                    let matrix = Matrix2D::from_array(numbers.try_into().unwrap());
                    text_state.set_tm(matrix, locator.clone());
                }
                Operator::Td => {
                    let numbers = as_numbers(&token.operands, 2)?;
                    let translation = Matrix2D::translation(numbers[0], numbers[1]);
                    text_state.translate(translation);
                }
                Operator::TStar => {
                    text_state.line_feed();
                }
                Operator::Tj => {
                    let glyphs = count_glyphs(&token.operands);
                    if glyphs > 0 {
                        let run_id = format!("t:{text_counter}");
                        if text_state.ensure_run(stream_id, &run_id) {
                            text_counter += 1;
                        }
                        text_state.add_glyphs(glyphs);
                    }
                }
                Operator::TjArray => {
                    let glyphs = count_glyphs(&token.operands);
                    if glyphs > 0 {
                        let run_id = format!("t:{text_counter}");
                        if text_state.ensure_run(stream_id, &run_id) {
                            text_counter += 1;
                        }
                        text_state.add_glyphs(glyphs);
                    }
                }
                Operator::Do => {
                    if let Some(Operand::Name(name)) = token.operands.first() {
                        if image_map.contains_key(name) {
                            let id = format!("img:{image_counter}");
                            image_counter += 1;
                            let cm_matrix = current_ctm.to_array();
                            let bbox = approximate_image_bbox(&cm_matrix);
                            page_objects.push(PageObject::Image(ImageObject {
                                id: id.clone(),
                                x_object: name.clone(),
                                cm: cm_matrix,
                                bbox,
                            }));
                            cache_map.insert(
                                id,
                                CachedObject::Image(ImageCache {
                                    stream_obj: stream_id,
                                    cm: last_cm_locator.clone(),
                                    do_token: locator.clone(),
                                    current_cm: cm_matrix,
                                    xobject: name.clone(),
                                }),
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
        objects: page_objects,
    };

    Ok(ExtractionResult {
        ir: DocumentIR {
            pages: vec![page_ir],
        },
        page_cache: PageCache {
            page_id,
            width_pt,
            height_pt,
            streams: stream_caches,
            objects: cache_map,
        },
    })
}

#[derive(Debug, Clone)]
struct TextRunFinished {
    id: String,
    stream_obj: ObjectId,
    bt_locator: TokenLocator,
    et_locator: TokenLocator,
    tm: [f64; 6],
    tm_locator: Option<TokenLocator>,
    font: FontInfo,
    glyphs: usize,
}

#[derive(Debug, Default, Clone)]
struct TextState {
    active: bool,
    font: Option<FontInfo>,
    text_matrix: Matrix2D,
    line_matrix: Matrix2D,
    last_tm_locator: Option<TokenLocator>,
    run: Option<CurrentRun>,
    last_bt: Option<TokenLocator>,
}

#[derive(Debug, Clone)]
struct CurrentRun {
    id: String,
    stream_obj: ObjectId,
    tm_at_start: Matrix2D,
    tm_locator: Option<TokenLocator>,
    glyphs: usize,
}

impl TextState {
    fn begin(&mut self, bt: TokenLocator) {
        self.active = true;
        self.text_matrix = Matrix2D::identity();
        self.line_matrix = Matrix2D::identity();
        self.last_tm_locator = None;
        self.run = None;
        self.last_bt = Some(bt);
    }

    fn end(&mut self, et: TokenLocator) -> Option<TextRunFinished> {
        if !self.active {
            return None;
        }
        let run = self.run.take()?;
        let font = self.font.clone().unwrap_or_else(|| FontInfo {
            res_name: String::new(),
            size: 0.0,
        });
        let bt_locator = self.last_bt.clone()?;
        Some(TextRunFinished {
            id: run.id,
            stream_obj: run.stream_obj,
            bt_locator,
            et_locator: et,
            tm: run.tm_at_start.to_array(),
            tm_locator: run.tm_locator,
            font,
            glyphs: run.glyphs,
        })
    }

    fn reset_after_et(&mut self) {
        self.active = false;
        self.run = None;
        self.last_tm_locator = None;
        self.last_bt = None;
    }

    fn set_font(&mut self, font: FontInfo) {
        self.font = Some(font);
    }

    fn set_tm(&mut self, matrix: Matrix2D, locator: TokenLocator) {
        self.text_matrix = matrix;
        self.line_matrix = matrix;
        self.last_tm_locator = Some(locator);
    }

    fn translate(&mut self, translation: Matrix2D) {
        self.line_matrix = self.line_matrix.multiply(translation);
        self.text_matrix = self.line_matrix;
    }

    fn line_feed(&mut self) {
        let leading = self.font.as_ref().map(|f| f.size).unwrap_or(0.0);
        let translation = Matrix2D::translation(0.0, -leading);
        self.translate(translation);
    }

    fn ensure_run(&mut self, stream_obj: ObjectId, id: &str) -> bool {
        if self.run.is_none() {
            let tm_locator = self.last_tm_locator.clone();
            self.run = Some(CurrentRun {
                id: id.to_string(),
                stream_obj,
                tm_at_start: self.text_matrix,
                tm_locator,
                glyphs: 0,
            });
            true
        } else {
            false
        }
    }

    fn add_glyphs(&mut self, count: usize) {
        if let Some(run) = &mut self.run {
            run.glyphs += count;
        }
    }
}

fn as_numbers(operands: &[Operand], expected: usize) -> Result<Vec<f64>> {
    if operands.len() < expected {
        return Err(anyhow!("insufficient operands"));
    }
    let mut values = Vec::with_capacity(expected);
    for operand in operands.iter().take(expected) {
        match operand {
            Operand::Number(n) => values.push(*n),
            other => {
                return Err(anyhow!("expected number operand, found {other:?}"));
            }
        }
    }
    Ok(values)
}

fn count_glyphs(operands: &[Operand]) -> usize {
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

fn approximate_text_bbox(tm: &[f64; 6], font: &FontInfo, glyphs: usize) -> Option<[f64; 4]> {
    if glyphs == 0 {
        return None;
    }
    let width = font.size * glyphs as f64 * 0.5;
    let height = font.size;
    let min_x = tm[4];
    let min_y = tm[5] - height;
    let max_x = min_x + width;
    let max_y = tm[5];
    Some([min_x, min_y, max_x, max_y])
}

fn approximate_image_bbox(cm: &[f64; 6]) -> Option<[f64; 4]> {
    let min_x = cm[4];
    let min_y = cm[5];
    let max_x = min_x + cm[0];
    let max_y = min_y + cm[3];
    Some([min_x, min_y, max_x, max_y])
}

fn resolve_page_size(doc: &Document, page_id: ObjectId, dict: &Dictionary) -> Result<(f64, f64)> {
    if let Some(Object::Array(values)) = dict.get(b"MediaBox") {
        return media_box_to_size(values);
    }
    if let Some(parent) = dict.get(b"Parent") {
        let parent_id = parent.as_reference()?;
        let parent_dict = doc
            .get_object(parent_id)?
            .as_dict()
            .context("parent is not a dictionary")?;
        return resolve_page_size(doc, parent_id, parent_dict);
    }
    Err(anyhow!("unable to resolve MediaBox for page {page_id:?}"))
}

fn media_box_to_size(array: &[Object]) -> Result<(f64, f64)> {
    if array.len() != 4 {
        return Err(anyhow!("MediaBox must contain four numbers"));
    }
    let x0 = array[0].as_f64()?;
    let y0 = array[1].as_f64()?;
    let x1 = array[2].as_f64()?;
    let y1 = array[3].as_f64()?;
    Ok((x1 - x0, y1 - y0))
}

fn resolve_resources(doc: &Document, page_id: ObjectId, dict: &Dictionary) -> Result<Dictionary> {
    if let Some(resources) = dict.get(b"Resources") {
        return to_dictionary(doc, resources);
    }
    if let Some(parent) = dict.get(b"Parent") {
        let parent_id = parent.as_reference()?;
        let parent_dict = doc
            .get_object(parent_id)?
            .as_dict()
            .context("parent is not a dictionary")?;
        return resolve_resources(doc, parent_id, parent_dict);
    }
    Ok(Dictionary::new())
}

fn to_dictionary(doc: &Document, object: &Object) -> Result<Dictionary> {
    match object {
        Object::Dictionary(dict) => Ok(dict.clone()),
        Object::Reference(id) => doc
            .get_object(*id)?
            .as_dict()
            .context("referenced object is not a dictionary")
            .map(|d| d.clone()),
        _ => Err(anyhow!("object is not a dictionary")),
    }
}

fn build_image_map(doc: &Document, resources: &Dictionary) -> Result<HashMap<String, ObjectId>> {
    let mut map = HashMap::new();
    if let Some(xobjects) = resources.get(b"XObject") {
        let dict = to_dictionary(doc, xobjects)?;
        for (name_bytes, object) in dict.iter() {
            let name = String::from_utf8_lossy(name_bytes).into_owned();
            let object_id = match object {
                Object::Reference(id) => *id,
                Object::Stream(_) => continue,
                _ => continue,
            };
            if let Ok(Object::Dictionary(stream_dict)) = doc.get_object(object_id) {
                if stream_dict
                    .get(b"Subtype")
                    .map(|obj| matches!(obj, Object::Name(sub) if sub.as_slice() == b"Image"))
                    .unwrap_or(false)
                {
                    map.insert(name, object_id);
                }
            }
        }
    }
    Ok(map)
}

fn collect_streams(doc: &Document, page_dict: &Dictionary) -> Result<Vec<(ObjectId, Vec<u8>)>> {
    let contents = page_dict
        .get(b"Contents")
        .ok_or_else(|| anyhow!("page has no contents"))?;
    match contents {
        Object::Reference(id) => {
            let stream = doc.get_stream(*id)?.decompressed_content()?;
            Ok(vec![(*id, stream)])
        }
        Object::Array(items) => {
            let mut streams = Vec::new();
            for item in items {
                if let Object::Reference(id) = item {
                    let stream = doc.get_stream(*id)?.decompressed_content()?;
                    streams.push((*id, stream));
                }
            }
            Ok(streams)
        }
        Object::Stream(stream) => Ok(vec![((0, 0), stream.decompressed_content()?)]),
        _ => Err(anyhow!("unexpected /Contents type")),
    }
}

trait ObjectExt {
    fn as_f64(&self) -> Result<f64>;
}

impl ObjectExt for Object {
    fn as_f64(&self) -> Result<f64> {
        match self {
            Object::Real(value) => Ok(*value as f64),
            Object::Integer(value) => Ok(*value as f64),
            _ => Err(anyhow!("expected numeric object")),
        }
    }
}
