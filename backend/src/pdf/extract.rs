//! Extraction pipeline for producing a lightweight IR for page 0.

use std::{
    collections::{HashMap, HashSet},
    ops::Range,
};

use anyhow::{anyhow, Context, Result};
use lopdf::{Dictionary, Document, Object, ObjectId};

use crate::types::{BtSpan, FontInfo, ImageObject, PageIR, PageObject, TextObject};

use super::content::{self, identity, multiply, translation, Matrix, Operand, Operator, Token};

#[derive(Debug, Clone)]
pub struct StreamContent {
    pub object_id: u32,
    pub bytes: Vec<u8>,
    pub tokens: Vec<Token>,
}

#[derive(Debug, Clone)]
pub struct ImageLocation {
    pub stream_obj: u32,
    pub cm_span: Option<Range<usize>>,
}

#[derive(Debug, Clone)]
pub struct PageExtraction {
    pub page_id: ObjectId,
    pub ir: PageIR,
    pub streams: Vec<StreamContent>,
    pub image_lookup: HashMap<String, ImageLocation>,
}

pub fn extract_page0(doc: &Document) -> Result<PageExtraction> {
    let (page_index, page_id) = doc
        .get_pages()
        .into_iter()
        .next()
        .ok_or_else(|| anyhow!("document has no pages"))?;
    let page_dict = doc.get_dictionary(page_id)?;
    let (width_pt, height_pt) = media_box_dims(&page_dict)?;

    let contents = page_dict
        .get(b"Contents")
        .context("page is missing /Contents")?;
    let mut streams = Vec::new();
    collect_streams(doc, contents, &mut streams)?;

    let image_names = collect_image_names(doc, page_id)?;

    let mut state = GraphicsState::default();
    let mut text_objects = Vec::new();
    let mut image_objects = Vec::new();
    let mut image_lookup = HashMap::new();
    let mut text_counter = 0usize;
    let mut image_counter = 0usize;

    for stream in &streams {
        let stream_obj = stream.object_id;
        for token in &stream.tokens {
            match token.operator {
                Operator::q => state.push(),
                Operator::Q => state.pop(),
                Operator::Cm => {
                    if let Some(matrix) = operands_to_matrix(&token.operands) {
                        state.apply_cm(matrix);
                        state.set_last_cm(CmInfo {
                            matrix,
                            span: token.span.clone(),
                            stream_obj,
                        });
                    }
                }
                Operator::BT => {
                    state.begin_text(token.span.start, stream_obj);
                }
                Operator::ET => {
                    if let Some(obj) = state.end_text(token.span.end, stream_obj) {
                        let id = format!("t:{text_counter}");
                        text_counter += 1;
                        text_objects.push(PageObject::Text(TextObject { id, ..obj }));
                    }
                }
                Operator::Tf => state.set_font(&token.operands),
                Operator::Tm => {
                    if let Some(matrix) = operands_to_matrix(&token.operands) {
                        state.set_text_matrix(matrix);
                    }
                }
                Operator::Td => state.translate_text(&token.operands),
                Operator::TStar => state.newline_text(),
                Operator::Tj | Operator::TJ => state.register_text_show(&token.operands),
                Operator::Do => {
                    if let Some(name) = token.operands.get(0).and_then(|op| match op {
                        Operand::Name(n) => Some(n.clone()),
                        _ => None,
                    }) {
                        if image_names.contains(&name) {
                            let cm_matrix = state.current_ctm();
                            let bbox = image_bbox(cm_matrix);
                            let id = format!("img:{image_counter}");
                            image_counter += 1;
                            image_lookup.insert(
                                id.clone(),
                                ImageLocation {
                                    stream_obj,
                                    cm_span: state.last_cm().map(|cm| cm.span.clone()),
                                },
                            );
                            image_objects.push(PageObject::Image(ImageObject {
                                id,
                                x_object: name,
                                cm: cm_matrix,
                                bbox,
                            }));
                        }
                    }
                }
                _ => {}
            }
        }
    }

    let mut objects = text_objects;
    objects.extend(image_objects);

    let page_ir = PageIR {
        index: page_index as usize,
        width_pt,
        height_pt,
        objects,
    };

    Ok(PageExtraction {
        page_id,
        ir: page_ir,
        streams,
        image_lookup,
    })
}

fn media_box_dims(dict: &Dictionary) -> Result<(f64, f64)> {
    let array = dict
        .get(b"MediaBox")
        .or_else(|_| dict.get(b"CropBox"))
        .context("page missing MediaBox")?
        .as_array()?;
    if array.len() < 4 {
        return Err(anyhow!("invalid MediaBox"));
    }
    let llx = array[0].as_f64()?;
    let lly = array[1].as_f64()?;
    let urx = array[2].as_f64()?;
    let ury = array[3].as_f64()?;
    Ok((urx - llx, ury - lly))
}

fn collect_streams(doc: &Document, object: &Object, out: &mut Vec<StreamContent>) -> Result<()> {
    match object {
        Object::Reference(id) => {
            let object = doc.get_object(*id)?;
            let stream = object.as_stream()?.clone();
            let bytes = stream
                .decompressed_content()
                .unwrap_or_else(|_| stream.content.clone());
            let tokens = content::tokenize_stream(&bytes)?;
            out.push(StreamContent {
                object_id: id.0,
                bytes,
                tokens,
            });
        }
        Object::Array(items) => {
            for item in items {
                collect_streams(doc, item, out)?;
            }
        }
        Object::Stream(stream) => {
            let bytes = stream
                .decompressed_content()
                .unwrap_or_else(|_| stream.content.clone());
            let tokens = content::tokenize_stream(&bytes)?;
            out.push(StreamContent {
                object_id: 0,
                bytes,
                tokens,
            });
        }
        _ => {}
    }
    Ok(())
}

fn collect_image_names(doc: &Document, page_id: ObjectId) -> Result<HashSet<String>> {
    let mut names = HashSet::new();
    let (direct_dict, inherited_ids) = doc.get_page_resources(page_id);
    if let Some(dict) = direct_dict {
        collect_image_names_from_dict(doc, dict.clone(), &mut names)?;
    }
    for id in inherited_ids {
        if let Ok(dict) = doc.get_dictionary(id) {
            collect_image_names_from_dict(doc, dict, &mut names)?;
        }
    }
    Ok(names)
}

fn collect_image_names_from_dict(
    doc: &Document,
    dict: Dictionary,
    names: &mut HashSet<String>,
) -> Result<()> {
    if let Ok(xobjects) = dict.get(b"XObject") {
        match xobjects {
            Object::Dictionary(map) => {
                for (name, value) in map.iter() {
                    if is_image_xobject(doc, value)? {
                        names.insert(String::from_utf8_lossy(name).into_owned());
                    }
                }
            }
            Object::Reference(id) => {
                if let Ok(res_dict) = doc.get_dictionary(id) {
                    for (name, value) in res_dict.iter() {
                        if is_image_xobject(doc, value)? {
                            names.insert(String::from_utf8_lossy(name).into_owned());
                        }
                    }
                }
            }
            _ => {}
        }
    }
    Ok(())
}

fn is_image_xobject(doc: &Document, object: &Object) -> Result<bool> {
    let dict = match object {
        Object::Reference(id) => doc.get_dictionary(*id)?,
        Object::Dictionary(dict) => dict.clone(),
        _ => return Ok(false),
    };
    Ok(matches!(
        dict.get(b"Subtype"),
        Ok(Object::Name(ref name)) if name == b"Image"
    ))
}

pub fn operands_to_matrix(operands: &[Operand]) -> Option<Matrix> {
    if operands.len() < 6 {
        return None;
    }
    let mut values = [0.0; 6];
    for (idx, value) in operands.iter().take(6).enumerate() {
        match value {
            Operand::Number(n) => values[idx] = *n,
            _ => return None,
        }
    }
    Some(values)
}

fn image_bbox(matrix: Matrix) -> [f64; 4] {
    let origin = (matrix[4], matrix[5]);
    let x_axis = (matrix[0], matrix[1]);
    let y_axis = (matrix[2], matrix[3]);
    let p1 = (origin.0 + x_axis.0, origin.1 + x_axis.1);
    let p2 = (
        origin.0 + x_axis.0 + y_axis.0,
        origin.1 + x_axis.1 + y_axis.1,
    );
    let p3 = (origin.0 + y_axis.0, origin.1 + y_axis.1);
    let xs = [origin.0, p1.0, p2.0, p3.0];
    let ys = [origin.1, p1.1, p2.1, p3.1];
    [
        xs.iter().cloned().fold(f64::INFINITY, f64::min),
        ys.iter().cloned().fold(f64::INFINITY, f64::min),
        xs.iter().cloned().fold(f64::NEG_INFINITY, f64::max),
        ys.iter().cloned().fold(f64::NEG_INFINITY, f64::max),
    ]
}

#[derive(Debug, Clone)]
struct CmInfo {
    matrix: Matrix,
    span: Range<usize>,
    stream_obj: u32,
}

#[derive(Debug, Clone)]
struct CurrentRun {
    bt_start: usize,
    stream_obj: u32,
    tm: Matrix,
    font_name: Option<String>,
    font_size: f64,
    glyph_estimate: usize,
}

#[derive(Debug, Clone)]
struct GraphicsState {
    stack: Vec<GraphicsSnapshot>,
    snapshot: GraphicsSnapshot,
}

#[derive(Debug, Clone)]
struct GraphicsSnapshot {
    ctm: Matrix,
    text_matrix: Matrix,
    text_line_matrix: Matrix,
    font_name: Option<String>,
    font_size: f64,
    leading: f64,
    in_text: bool,
    bt_span_start: Option<usize>,
    bt_stream: Option<u32>,
    current_run: Option<CurrentRun>,
    last_cm: Option<CmInfo>,
}

impl Default for GraphicsState {
    fn default() -> Self {
        Self {
            stack: Vec::new(),
            snapshot: GraphicsSnapshot {
                ctm: identity(),
                text_matrix: identity(),
                text_line_matrix: identity(),
                font_name: None,
                font_size: 12.0,
                leading: 12.0,
                in_text: false,
                bt_span_start: None,
                bt_stream: None,
                current_run: None,
                last_cm: None,
            },
        }
    }
}

impl GraphicsState {
    fn push(&mut self) {
        self.stack.push(self.snapshot.clone());
    }

    fn pop(&mut self) {
        if let Some(snapshot) = self.stack.pop() {
            self.snapshot = snapshot;
        }
    }

    fn apply_cm(&mut self, matrix: Matrix) {
        self.snapshot.ctm = multiply(self.snapshot.ctm, matrix);
    }

    fn set_last_cm(&mut self, cm: CmInfo) {
        self.snapshot.last_cm = Some(cm);
    }

    fn begin_text(&mut self, start: usize, stream_obj: u32) {
        self.snapshot.in_text = true;
        self.snapshot.text_matrix = identity();
        self.snapshot.text_line_matrix = identity();
        self.snapshot.bt_span_start = Some(start);
        self.snapshot.bt_stream = Some(stream_obj);
        self.snapshot.current_run = None;
    }

    fn end_text(&mut self, end: usize, stream_obj: u32) -> Option<TextObject> {
        let snapshot = &mut self.snapshot;
        snapshot.in_text = false;
        snapshot.bt_stream = None;
        snapshot.text_matrix = identity();
        snapshot.text_line_matrix = identity();
        let run = snapshot.current_run.take()?;
        let font_name = run.font_name.unwrap_or_else(|| "F?".into());
        let font_size = if run.font_size.abs() < f64::EPSILON {
            snapshot.font_size
        } else {
            run.font_size
        };
        let tm = run.tm;
        let device_tm = multiply(self.snapshot.ctm, tm);
        let bbox = text_bbox(device_tm, font_size, run.glyph_estimate);
        Some(TextObject {
            id: String::new(),
            bt_span: BtSpan {
                start: run.bt_start as u32,
                end: end as u32,
                stream_obj,
            },
            tm,
            font: FontInfo {
                res_name: font_name,
                size: font_size,
            },
            bbox,
        })
    }

    fn set_font(&mut self, operands: &[Operand]) {
        if operands.len() < 2 {
            return;
        }
        let name = operands[0]
            .as_name()
            .map(|s| s.to_string())
            .or_else(|| match &operands[0] {
                Operand::Name(n) => Some(n.clone()),
                _ => None,
            });
        let size = operands[1].as_number().unwrap_or(self.snapshot.font_size);
        self.snapshot.font_name = name;
        self.snapshot.font_size = size;
        self.snapshot.leading = size;
    }

    fn set_text_matrix(&mut self, matrix: Matrix) {
        self.snapshot.text_matrix = matrix;
        self.snapshot.text_line_matrix = matrix;
    }

    fn translate_text(&mut self, operands: &[Operand]) {
        if operands.len() < 2 {
            return;
        }
        let tx = operands[0].as_number().unwrap_or(0.0);
        let ty = operands[1].as_number().unwrap_or(0.0);
        let translation = translation(tx, ty);
        self.snapshot.text_line_matrix = multiply(self.snapshot.text_line_matrix, translation);
        self.snapshot.text_matrix = self.snapshot.text_line_matrix;
    }

    fn newline_text(&mut self) {
        let translation = translation(0.0, -self.snapshot.leading);
        self.snapshot.text_line_matrix = multiply(self.snapshot.text_line_matrix, translation);
        self.snapshot.text_matrix = self.snapshot.text_line_matrix;
    }

    fn register_text_show(&mut self, operands: &[Operand]) {
        if !self.snapshot.in_text {
            return;
        }
        let glyphs = estimate_glyphs(operands);
        if glyphs == 0 {
            return;
        }
        if self.snapshot.current_run.is_none() {
            if let (Some(start), Some(stream)) =
                (self.snapshot.bt_span_start, self.snapshot.bt_stream)
            {
                self.snapshot.current_run = Some(CurrentRun {
                    bt_start: start,
                    stream_obj: stream,
                    tm: self.snapshot.text_matrix,
                    font_name: self.snapshot.font_name.clone(),
                    font_size: self.snapshot.font_size,
                    glyph_estimate: glyphs,
                });
            }
        } else if let Some(run) = self.snapshot.current_run.as_mut() {
            run.glyph_estimate += glyphs;
        }
    }

    fn current_ctm(&self) -> Matrix {
        self.snapshot.ctm
    }

    fn last_cm(&self) -> Option<&CmInfo> {
        self.snapshot.last_cm.as_ref()
    }
}

fn text_bbox(matrix: Matrix, font_size: f64, glyphs: usize) -> [f64; 4] {
    let width_scale = font_size * 0.6 * glyphs as f64;
    let height_scale = font_size;
    let origin = (matrix[4], matrix[5]);
    let x_axis = (matrix[0] * width_scale, matrix[1] * width_scale);
    let y_axis = (matrix[2] * height_scale, matrix[3] * height_scale);
    let p1 = (origin.0 + x_axis.0, origin.1 + x_axis.1);
    let p2 = (
        origin.0 + x_axis.0 + y_axis.0,
        origin.1 + x_axis.1 + y_axis.1,
    );
    let p3 = (origin.0 + y_axis.0, origin.1 + y_axis.1);
    let xs = [origin.0, p1.0, p2.0, p3.0];
    let ys = [origin.1, p1.1, p2.1, p3.1];
    [
        xs.iter().cloned().fold(f64::INFINITY, f64::min),
        ys.iter().cloned().fold(f64::INFINITY, f64::min),
        xs.iter().cloned().fold(f64::NEG_INFINITY, f64::max),
        ys.iter().cloned().fold(f64::NEG_INFINITY, f64::max),
    ]
}

fn estimate_glyphs(operands: &[Operand]) -> usize {
    operands
        .iter()
        .map(|operand| match operand {
            Operand::String(bytes) => bytes.len(),
            Operand::Array(items) => estimate_glyphs(items),
            _ => 0,
        })
        .sum()
}

trait OperandExt {
    fn as_number(&self) -> Option<f64>;
    fn as_name(&self) -> Option<&str>;
}

impl OperandExt for Operand {
    fn as_number(&self) -> Option<f64> {
        match self {
            Operand::Number(n) => Some(*n),
            _ => None,
        }
    }

    fn as_name(&self) -> Option<&str> {
        match self {
            Operand::Name(name) => Some(name.as_str()),
            _ => None,
        }
    }
}
