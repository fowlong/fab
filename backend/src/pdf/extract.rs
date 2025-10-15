use std::{
    collections::{HashMap, HashSet},
    ops::Range,
};

use anyhow::{anyhow, Result};
use lopdf::{Dictionary, Document, Object, ObjectId, Stream};

use crate::{
    pdf::content::{tokenize_stream, Operand, Token},
    types::{DocumentIR, FontInfo, ImageObject, PageIR, PageObject, Span, TextObject},
};

pub type Matrix = [f64; 6];

#[derive(Clone)]
pub struct TokenLocator {
    pub index: usize,
    pub range: Range<usize>,
}

#[derive(Clone)]
pub struct TextCache {
    pub id: String,
    pub tm: Matrix,
    pub bt_range: Range<usize>,
    pub tm_token: Option<TokenLocator>,
    pub insertion_point: usize,
}

#[derive(Clone)]
pub struct ImageCache {
    pub id: String,
    pub cm: Matrix,
    pub cm_token: Option<TokenLocator>,
    pub x_object: String,
}

#[derive(Clone)]
pub struct PageCache {
    pub page_id: ObjectId,
    pub stream_obj: ObjectId,
    pub stream_bytes: Vec<u8>,
    pub tokens: Vec<Token>,
    pub text_lookup: HashMap<String, TextCache>,
    pub image_lookup: HashMap<String, ImageCache>,
}

#[derive(Clone)]
pub struct PageExtraction {
    pub ir: DocumentIR,
    pub cache: PageCache,
}

pub fn extract_page0(
    document: &Document,
    overrides: Option<&HashMap<ObjectId, Vec<u8>>>,
) -> Result<PageExtraction> {
    let (&page_index, &page_id) = document
        .get_pages()
        .iter()
        .next()
        .ok_or_else(|| anyhow!("document has no pages"))?;
    let page_dict = document.get_dictionary(page_id)?.clone();
    let (width_pt, height_pt) = page_dimensions(document, &page_dict)?;

    let mut streams = document.get_page_contents(page_id);
    if streams.is_empty() {
        return Err(anyhow!("page content stream missing"));
    }
    let stream_obj = streams.remove(0);
    let stream = document.get_object(stream_obj)?.as_stream()?;
    let stream_bytes = stream_bytes(stream_obj, stream, overrides)?;
    let tokens = tokenize_stream(&stream_bytes)?;

    let resources = resolve_resources(document, &page_dict)?;
    let image_names = gather_image_names(document, &resources)?;

    let mut objects = Vec::new();
    let mut text_lookup = HashMap::new();
    let mut image_lookup = HashMap::new();

    let mut text_state = TextState::default();
    let mut text_counter = 0usize;
    let mut image_counter = 0usize;

    let mut graphics_state = GraphicsState::default();
    let mut stack: Vec<GraphicsState> = Vec::new();

    for (index, token) in tokens.iter().enumerate() {
        match token.operator.as_str() {
            "q" => {
                stack.push(graphics_state.clone());
            }
            "Q" => {
                graphics_state = stack.pop().unwrap_or_default();
            }
            "cm" => {
                if let Some(matrix) = matrix_from_operands(token) {
                    graphics_state.ctm = multiply(graphics_state.ctm, matrix);
                    graphics_state.last_cm = Some(TokenMatrix {
                        locator: TokenLocator {
                            index,
                            range: token.byte_range.clone(),
                        },
                        matrix,
                    });
                }
            }
            "BT" => {
                text_state = TextState::start(token.byte_range.clone());
            }
            "ET" => {
                if let Some((text_object, cache_entry)) =
                    text_state.finish(token.byte_range.clone(), stream_obj, text_counter)
                {
                    objects.push(PageObject::Text(text_object));
                    text_lookup.insert(cache_entry.id.clone(), cache_entry);
                    text_counter += 1;
                }
            }
            "Tf" => {
                text_state.set_font(token);
            }
            "Tm" => {
                text_state.set_tm(token, index);
            }
            "Td" => {
                text_state.translate(token);
            }
            "T*" => {
                text_state.line_feed();
            }
            "Tj" => {
                text_state.touch(token, text_counter);
            }
            "TJ" => {
                text_state.touch(token, text_counter);
            }
            "Do" => {
                if let Some(name) = token.operands.first().and_then(as_name) {
                    if image_names.contains(name) {
                        let (image, cache_entry) =
                            capture_image(name, &graphics_state, image_counter);
                        objects.push(PageObject::Image(image));
                        image_lookup.insert(cache_entry.id.clone(), cache_entry);
                        image_counter += 1;
                    }
                }
            }
            _ => {}
        }
    }

    let page_ir = PageIR {
        index: (page_index - 1) as usize,
        width_pt,
        height_pt,
        objects,
    };

    let cache = PageCache {
        page_id,
        stream_obj,
        stream_bytes: stream_bytes.clone(),
        tokens,
        text_lookup,
        image_lookup,
    };

    Ok(PageExtraction {
        ir: DocumentIR {
            pages: vec![page_ir],
        },
        cache,
    })
}

#[derive(Clone, Default)]
struct GraphicsState {
    ctm: Matrix,
    last_cm: Option<TokenMatrix>,
}

#[derive(Clone)]
struct TokenMatrix {
    locator: TokenLocator,
    matrix: Matrix,
}

#[derive(Clone)]
struct TextState {
    bt_range: Option<Range<usize>>,
    tm_matrix: Option<Matrix>,
    tm_token: Option<TokenLocator>,
    base_matrix: Option<Matrix>,
    translation: Matrix,
    font_name: Option<String>,
    font_size: f64,
    glyph_count: usize,
    active: bool,
}

impl Default for TextState {
    fn default() -> Self {
        Self {
            bt_range: None,
            tm_matrix: None,
            tm_token: None,
            base_matrix: None,
            translation: IDENTITY,
            font_name: None,
            font_size: 12.0,
            glyph_count: 0,
            active: false,
        }
    }
}

impl TextState {
    fn start(range: Range<usize>) -> Self {
        let mut state = Self::default();
        state.bt_range = Some(range);
        state.active = true;
        state
    }

    fn set_font(&mut self, token: &Token) {
        if let (Some(name), Some(size)) = (token.operands.get(0), token.operands.get(1)) {
            if let Some(name) = as_name(name) {
                self.font_name = Some(name.trim_start_matches('/').to_string());
            }
            if let Operand::Number(value) = size {
                self.font_size = *value;
            }
        }
    }

    fn set_tm(&mut self, token: &Token, index: usize) {
        if let Some(matrix) = matrix_from_operands(token) {
            self.tm_matrix = Some(matrix);
            self.base_matrix = Some(matrix);
            self.translation = matrix;
            self.tm_token = Some(TokenLocator {
                index,
                range: token.byte_range.clone(),
            });
        }
    }

    fn translate(&mut self, token: &Token) {
        if let (Some(Operand::Number(tx)), Some(Operand::Number(ty))) =
            (token.operands.get(0), token.operands.get(1))
        {
            let translation = [1.0, 0.0, 0.0, 1.0, *tx, *ty];
            self.translation = multiply(self.translation, translation);
            if self.base_matrix.is_none() {
                self.base_matrix = Some(self.translation);
            }
        }
    }

    fn line_feed(&mut self) {
        let offset = -self.font_size * 1.2;
        let translation = [1.0, 0.0, 0.0, 1.0, 0.0, offset];
        self.translation = multiply(self.translation, translation);
        if self.base_matrix.is_none() {
            self.base_matrix = Some(self.translation);
        }
    }

    fn touch(&mut self, token: &Token, counter: usize) {
        if !self.active {
            return;
        }
        self.glyph_count += glyph_count(token);
        if self.base_matrix.is_none() {
            self.base_matrix = self.tm_matrix.or(Some(self.translation));
        }
        if self.font_name.is_none() {
            self.font_name = Some(format!("F{}", counter));
        }
    }

    fn finish(
        &mut self,
        end_range: Range<usize>,
        stream_obj: ObjectId,
        counter: usize,
    ) -> Option<(TextObject, TextCache)> {
        if !self.active || self.glyph_count == 0 {
            return None;
        }
        let bt_span = self.bt_range.clone()?;
        let matrix = self.base_matrix.unwrap_or(IDENTITY);
        let font_name = self.font_name.clone().unwrap_or_else(|| "F0".to_string());
        let bbox = approximate_bbox(&matrix, self.font_size, self.glyph_count);
        let id = format!("t:{}", counter);
        let text_object = TextObject {
            id: id.clone(),
            tm: matrix,
            font: FontInfo {
                res_name: font_name,
                size: self.font_size,
            },
            bt_span: Span {
                start: bt_span.start as u32,
                end: end_range.end as u32,
                stream_obj: stream_obj.0,
            },
            bbox,
        };
        let cache_entry = TextCache {
            id,
            tm: matrix,
            bt_range: bt_span.clone(),
            tm_token: self.tm_token.clone(),
            insertion_point: bt_span.end,
        };
        self.active = false;
        Some((text_object, cache_entry))
    }
}

fn capture_image(
    name: &str,
    graphics: &GraphicsState,
    counter: usize,
) -> (ImageObject, ImageCache) {
    let matrix = graphics
        .last_cm
        .as_ref()
        .map(|m| m.matrix)
        .unwrap_or(IDENTITY);
    let locator = graphics.last_cm.as_ref().map(|m| m.locator.clone());
    let bbox = image_bbox(&matrix);
    let id = format!("img:{}", counter);
    let image = ImageObject {
        id: id.clone(),
        x_object: name.trim_start_matches('/').to_string(),
        cm: matrix,
        bbox,
    };
    let cache_entry = ImageCache {
        id,
        cm: matrix,
        cm_token: locator,
        x_object: name.to_string(),
    };
    (image, cache_entry)
}

fn glyph_count(token: &Token) -> usize {
    match token.operator.as_str() {
        "Tj" => token
            .operands
            .iter()
            .filter_map(|op| match op {
                Operand::String(bytes) => Some(bytes.len()),
                _ => None,
            })
            .sum(),
        "TJ" => token
            .operands
            .iter()
            .filter_map(|op| match op {
                Operand::Array(items) => Some(
                    items
                        .iter()
                        .filter_map(|inner| match inner {
                            Operand::String(bytes) => Some(bytes.len()),
                            _ => None,
                        })
                        .sum(),
                ),
                _ => None,
            })
            .sum(),
        _ => 0,
    }
}

fn as_name(op: &Operand) -> Option<&str> {
    if let Operand::Name(name) = op {
        Some(name)
    } else {
        None
    }
}

fn matrix_from_operands(token: &Token) -> Option<Matrix> {
    if token.operands.len() >= 6 {
        let mut values = [0.0; 6];
        for i in 0..6 {
            if let Operand::Number(value) = token.operands[i] {
                values[i] = value;
            } else {
                return None;
            }
        }
        Some(values)
    } else {
        None
    }
}

fn multiply(a: Matrix, b: Matrix) -> Matrix {
    [
        a[0] * b[0] + a[2] * b[1],
        a[1] * b[0] + a[3] * b[1],
        a[0] * b[2] + a[2] * b[3],
        a[1] * b[2] + a[3] * b[3],
        a[0] * b[4] + a[2] * b[5] + a[4],
        a[1] * b[4] + a[3] * b[5] + a[5],
    ]
}

fn approximate_bbox(matrix: &Matrix, font_size: f64, glyphs: usize) -> [f64; 4] {
    let width = font_size * glyphs as f64 * 0.6;
    let height = font_size * 1.2;
    [matrix[4], matrix[5], matrix[4] + width, matrix[5] + height]
}

fn image_bbox(matrix: &Matrix) -> [f64; 4] {
    let width = matrix[0].abs();
    let height = matrix[3].abs();
    [matrix[4], matrix[5], matrix[4] + width, matrix[5] + height]
}

const IDENTITY: Matrix = [1.0, 0.0, 0.0, 1.0, 0.0, 0.0];

fn stream_bytes(
    object_id: ObjectId,
    stream: &Stream,
    overrides: Option<&HashMap<ObjectId, Vec<u8>>>,
) -> Result<Vec<u8>> {
    if let Some(bytes) = overrides.and_then(|map| map.get(&object_id)) {
        return Ok(bytes.clone());
    }
    Ok(stream
        .decompressed_content()
        .unwrap_or_else(|_| stream.content.clone()))
}

fn page_dimensions(document: &Document, page: &Dictionary) -> Result<(f64, f64)> {
    let media_box = match page.get(b"MediaBox") {
        Ok(Object::Array(items)) => items.clone(),
        Ok(Object::Reference(id)) => document.get_object(id)?.as_array()?.clone(),
        _ => return Err(anyhow!("page MediaBox missing")),
    };
    if media_box.len() < 4 {
        return Err(anyhow!("invalid MediaBox"));
    }
    let x0 = object_number(&media_box[0])?;
    let y0 = object_number(&media_box[1])?;
    let x1 = object_number(&media_box[2])?;
    let y1 = object_number(&media_box[3])?;
    Ok((x1 - x0, y1 - y0))
}

fn object_number(object: &Object) -> Result<f64> {
    match object {
        Object::Integer(value) => Ok(*value as f64),
        Object::Real(value) => Ok(*value),
        _ => Err(anyhow!("expected numeric object")),
    }
}

fn resolve_resources(document: &Document, page: &Dictionary) -> Result<Dictionary> {
    match page.get(b"Resources") {
        Ok(Object::Dictionary(dict)) => Ok(dict.clone()),
        Ok(Object::Reference(id)) => Ok(document.get_dictionary(id)?.clone()),
        _ => Ok(Dictionary::new()),
    }
}

fn gather_image_names(document: &Document, resources: &Dictionary) -> Result<HashSet<String>> {
    let mut names = HashSet::new();
    if let Ok(Object::Dictionary(xobjects)) = resources.get(b"XObject") {
        for (name, value) in xobjects.iter() {
            if let Ok(id) = value.as_reference() {
                if let Ok(object) = document.get_object(id) {
                    if let Object::Stream(stream) = object {
                        if stream
                            .dict
                            .get(b"Subtype")
                            .map(|obj| matches!(obj, Object::Name(sub) if sub == b"Image"))
                            .unwrap_or(false)
                        {
                            let name_str = format!("/{}", String::from_utf8_lossy(name));
                            names.insert(name_str);
                        }
                    }
                }
            }
        }
    }
    Ok(names)
}
