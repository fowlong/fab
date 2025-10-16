//! Extract a lightweight IR for page 0 of a PDF document.

use std::collections::HashMap;

use anyhow::{bail, Context, Result};
use lopdf::{Dictionary, Document, Object, ObjectId};

use crate::pdf::content::{tokenize_stream, ContentToken, Operand, Operator};
use crate::types::{DocumentIR, FontInfo, ImageObject, PageIR, PageObject, Span, TextObject};

#[derive(Debug, Clone)]
pub struct ExtractedDocument {
    pub ir: DocumentIR,
    pub page0: Option<ExtractedPage>,
}

#[derive(Debug, Clone)]
pub struct ExtractedPage {
    pub page_ir: PageIR,
    pub meta: PageMeta,
}

#[derive(Debug, Clone)]
pub struct PageMeta {
    pub page_id: ObjectId,
    pub streams: HashMap<u32, StreamCache>,
    pub text_map: HashMap<String, TextMeta>,
    pub image_map: HashMap<String, ImageMeta>,
}

#[derive(Debug, Clone)]
pub struct StreamCache {
    pub id: ObjectId,
    pub data: Vec<u8>,
    pub tokens: Vec<ContentToken>,
}

#[derive(Debug, Clone)]
pub struct TextMeta {
    pub id: String,
    pub stream_obj: ObjectId,
    pub bt_span: Span,
    pub bt_token_span: (usize, usize),
    pub tm_span: Option<(usize, usize)>,
    pub translation_spans: Vec<(usize, usize)>,
    pub base_tm: [f64; 6],
    pub font: FontInfo,
    pub approx_chars: usize,
    pub has_tm: bool,
}

#[derive(Debug, Clone)]
pub struct ImageMeta {
    pub id: String,
    pub stream_obj: ObjectId,
    pub cm_span: Option<(usize, usize)>,
    pub cm_matrix: [f64; 6],
    pub bbox: [f64; 4],
    pub name: String,
}

pub fn extract_document(doc: &Document) -> Result<ExtractedDocument> {
    let mut ir = DocumentIR::default();
    let mut page0_meta = None;

    if let Some((&page_number, &page_id)) = doc.get_pages().iter().next() {
        let (page_ir, meta) = extract_page(doc, page_number - 1, page_id)?;
        ir.pages.push(page_ir.clone());
        page0_meta = Some(ExtractedPage { page_ir, meta });
    }

    Ok(ExtractedDocument {
        ir,
        page0: page0_meta,
    })
}

fn extract_page(doc: &Document, page_index: u32, page_id: ObjectId) -> Result<(PageIR, PageMeta)> {
    let (width, height) = page_size(doc, page_id)?;
    let mut objects = Vec::new();
    let mut text_map = HashMap::new();
    let mut image_map = HashMap::new();
    let mut streams = HashMap::new();

    let xobjects = collect_xobjects(doc, page_id)?;

    let mut text_counter = 0usize;
    let mut image_counter = 0usize;

    let mut graphics_state = GraphicsState::default();
    let mut text_state = TextState::default();

    for content_id in doc.get_page_contents(page_id) {
        let stream_obj = doc
            .get_object(content_id)
            .context("content stream missing")?;
        let stream = match stream_obj {
            Object::Stream(ref s) => s.clone(),
            _ => continue,
        };
        let data = stream
            .decompressed_content()
            .unwrap_or_else(|_| stream.content.clone());
        let tokens = tokenize_stream(&data)?;
        let cache = StreamCache {
            id: content_id,
            data: data.clone(),
            tokens: tokens.clone(),
        };
        streams.insert(content_id.0, cache);

        for token in tokens {
            match token.operator {
                Operator::q => {
                    graphics_state.push();
                }
                Operator::Q => {
                    graphics_state.pop();
                }
                Operator::Cm => {
                    let matrix = read_matrix(&token.operands)?;
                    graphics_state.apply_cm(matrix, token.span.clone());
                }
                Operator::BT => {
                    text_state.begin(token.span.clone());
                }
                Operator::ET => {
                    if let Some((text_obj, meta)) = text_state.end(content_id)? {
                        text_map.insert(meta.id.clone(), meta.clone());
                        objects.push(PageObject::Text(text_obj));
                    }
                }
                Operator::Tf => {
                    text_state.set_font(parse_tf(&token.operands)?);
                }
                Operator::Tm => {
                    let matrix = read_matrix(&token.operands)?;
                    text_state.set_tm(matrix, token.span.clone());
                }
                Operator::Td => {
                    let (tx, ty) = parse_td(&token.operands)?;
                    text_state.translate(tx, ty, token.span.clone());
                }
                Operator::TStar => {
                    text_state.translate(0.0, -text_state.leading, token.span.clone());
                }
                Operator::Tj | Operator::TJ => {
                    text_state.consume_text(&token, content_id, &mut text_counter)?;
                }
                Operator::Do => {
                    if let Some(image_name) = parse_do_name(&token.operands) {
                        if let Some(&(_object_id, ref kind)) = xobjects.get(image_name.as_bytes()) {
                            if kind.as_slice() == b"Image" {
                                let id = format!("img:{}", image_counter);
                                image_counter += 1;
                                let cm_matrix = graphics_state
                                    .last_cm_matrix()
                                    .unwrap_or_else(|| graphics_state.current_cm());
                                let bbox = image_bbox(cm_matrix);
                                let meta = ImageMeta {
                                    id: id.clone(),
                                    stream_obj: content_id,
                                    cm_span: graphics_state.last_cm_span(),
                                    cm_matrix,
                                    bbox,
                                    name: image_name.clone(),
                                };
                                let ir_obj = ImageObject {
                                    id,
                                    x_object: image_name.clone(),
                                    cm: meta.cm_matrix,
                                    bbox: meta.bbox,
                                };
                                image_map.insert(meta.id.clone(), meta.clone());
                                objects.push(PageObject::Image(ir_obj));
                                graphics_state.clear_last_cm();
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }

    let page_ir = PageIR {
        index: page_index as usize,
        width_pt: width,
        height_pt: height,
        objects,
    };

    Ok((
        page_ir,
        PageMeta {
            page_id,
            streams,
            text_map,
            image_map,
        },
    ))
}

fn page_size(doc: &Document, page_id: ObjectId) -> Result<(f64, f64)> {
    let page = doc.get_dictionary(page_id)?;
    let mediabox = resolve_array(doc, page.get(b"MediaBox")?)?;
    if mediabox.len() == 4 {
        let width = as_number(doc, &mediabox[2])? - as_number(doc, &mediabox[0])?;
        let height = as_number(doc, &mediabox[3])? - as_number(doc, &mediabox[1])?;
        Ok((width, height))
    } else {
        bail!("unexpected MediaBox contents")
    }
}

fn collect_xobjects(
    doc: &Document,
    page_id: ObjectId,
) -> Result<HashMap<Vec<u8>, (ObjectId, Vec<u8>)>> {
    let mut map = HashMap::new();
    let (resource_dict, resource_ids) = doc.get_page_resources(page_id);
    if let Some(dict) = resource_dict {
        collect_xobjects_from_dict(doc, dict, &mut map)?;
    }
    for id in resource_ids {
        if let Ok(dict) = doc.get_dictionary(id) {
            collect_xobjects_from_dict(doc, &dict, &mut map)?;
        }
    }
    Ok(map)
}

fn collect_xobjects_from_dict(
    doc: &Document,
    dict: &Dictionary,
    map: &mut HashMap<Vec<u8>, (ObjectId, Vec<u8>)>,
) -> Result<()> {
    if let Ok(xobj) = dict.get(b"XObject") {
        match xobj {
            Object::Reference(id) => {
                if let Ok(real_dict) = doc.get_dictionary(*id) {
                    collect_xobjects_from_dict(doc, &real_dict, map)?;
                }
            }
            Object::Dictionary(inner) => {
                for (name, value) in inner.iter() {
                    if let Ok(obj_id) = value.as_reference() {
                        if let Ok(obj_dict) = doc.get_dictionary(obj_id) {
                            let subtype = obj_dict
                                .get(b"Subtype")
                                .and_then(Object::as_name_str)
                                .ok()
                                .map(|s| s.as_bytes().to_vec())
                                .unwrap_or_else(|| b"Unknown".to_vec());
                            map.insert(name.clone(), (obj_id, subtype));
                        }
                    }
                }
            }
            _ => {}
        }
    }
    Ok(())
}

fn as_number(doc: &Document, object: &Object) -> Result<f64> {
    match object {
        Object::Integer(i) => Ok(*i as f64),
        Object::Real(f) => Ok(*f),
        Object::Reference(id) => {
            let resolved = doc.get_object(*id)?;
            as_number(doc, resolved)
        }
        _ => bail!("expected numeric value"),
    }
}

fn resolve_array(doc: &Document, object: &Object) -> Result<Vec<Object>> {
    match object {
        Object::Array(items) => Ok(items.clone()),
        Object::Reference(id) => {
            let target = doc.get_object(*id)?;
            resolve_array(doc, target)
        }
        _ => bail!("expected array"),
    }
}

#[derive(Debug, Clone)]
struct GraphicsState {
    stack: Vec<GsSnapshot>,
    current: GsSnapshot,
}

#[derive(Debug, Clone)]
struct GsSnapshot {
    ctm: [f64; 6],
    last_cm: Option<CmSnapshot>,
}

#[derive(Debug, Clone)]
struct CmSnapshot {
    span: (usize, usize),
    matrix: [f64; 6],
}

impl Default for GraphicsState {
    fn default() -> Self {
        Self {
            stack: Vec::new(),
            current: GsSnapshot {
                ctm: IDENTITY,
                last_cm: None,
            },
        }
    }
}

impl GraphicsState {
    fn push(&mut self) {
        self.stack.push(self.current.clone());
    }

    fn pop(&mut self) {
        if let Some(snapshot) = self.stack.pop() {
            self.current = snapshot;
        } else {
            self.current = GsSnapshot {
                ctm: IDENTITY,
                last_cm: None,
            };
        }
    }

    fn apply_cm(&mut self, matrix: [f64; 6], span: std::ops::Range<usize>) {
        self.current.ctm = multiply(self.current.ctm, matrix);
        self.current.last_cm = Some(CmSnapshot {
            span: (span.start, span.end),
            matrix,
        });
    }

    fn current_cm(&self) -> [f64; 6] {
        self.current.ctm
    }

    fn last_cm_span(&self) -> Option<(usize, usize)> {
        self.current.last_cm.as_ref().map(|cm| cm.span)
    }

    fn last_cm_matrix(&self) -> Option<[f64; 6]> {
        self.current.last_cm.as_ref().map(|cm| cm.matrix)
    }

    fn clear_last_cm(&mut self) {
        self.current.last_cm = None;
    }
}

#[derive(Debug, Clone)]
struct TextState {
    active: bool,
    text_matrix: [f64; 6],
    line_matrix: [f64; 6],
    leading: f64,
    font: Option<FontInfo>,
    pending: Option<PendingRun>,
    last_tm: Option<(std::ops::Range<usize>, [f64; 6])>,
    translation_spans: Vec<std::ops::Range<usize>>,
    bt_span: Option<std::ops::Range<usize>>,
}

#[derive(Debug, Clone)]
struct PendingRun {
    id: String,
    stream_obj: ObjectId,
    start: usize,
    end: usize,
    base_tm: [f64; 6],
    tm_span: Option<(usize, usize)>,
    translation_spans: Vec<(usize, usize)>,
    font: FontInfo,
    approx_chars: usize,
    has_tm: bool,
}

impl Default for TextState {
    fn default() -> Self {
        Self {
            active: false,
            text_matrix: IDENTITY,
            line_matrix: IDENTITY,
            leading: 0.0,
            font: None,
            pending: None,
            last_tm: None,
            translation_spans: Vec::new(),
            bt_span: None,
        }
    }
}

impl TextState {
    fn begin(&mut self, span: std::ops::Range<usize>) {
        self.active = true;
        self.text_matrix = IDENTITY;
        self.line_matrix = IDENTITY;
        self.pending = None;
        self.last_tm = None;
        self.translation_spans.clear();
        self.bt_span = Some(span);
    }

    fn end(&mut self, stream_obj: ObjectId) -> Result<Option<(TextObject, TextMeta)>> {
        if let Some(pending) = self.pending.take() {
            let id = pending.id.clone();
            let span = Span {
                start: pending.start as u64,
                end: pending.end as u64,
                stream_obj: stream_obj.0,
            };
            let bbox = text_bbox(&pending.base_tm, pending.font.size, pending.approx_chars);
            let text_obj = TextObject {
                id: id.clone(),
                bt_span: span.clone(),
                tm: pending.base_tm,
                font: pending.font.clone(),
                bbox,
            };
            let meta = TextMeta {
                id,
                stream_obj,
                bt_span: span,
                bt_token_span: self
                    .bt_span
                    .as_ref()
                    .map(|s| (s.start, s.end))
                    .unwrap_or((0, 0)),
                tm_span: pending.tm_span,
                translation_spans: pending.translation_spans,
                base_tm: pending.base_tm,
                font: pending.font,
                approx_chars: pending.approx_chars,
                has_tm: pending.has_tm,
            };
            self.active = false;
            Ok(Some((text_obj, meta)))
        } else {
            self.active = false;
            Ok(None)
        }
    }

    fn set_font(&mut self, (name, size): (String, f64)) {
        self.font = Some(FontInfo {
            res_name: name,
            size,
        });
        self.leading = size;
    }

    fn set_tm(&mut self, matrix: [f64; 6], span: std::ops::Range<usize>) {
        self.text_matrix = matrix;
        self.line_matrix = matrix;
        self.last_tm = Some((span, matrix));
        self.translation_spans.clear();
    }

    fn translate(&mut self, tx: f64, ty: f64, span: std::ops::Range<usize>) {
        let translation = [1.0, 0.0, 0.0, 1.0, tx, ty];
        self.line_matrix = multiply(self.line_matrix, translation);
        self.text_matrix = self.line_matrix;
        if let Some(ref mut pending) = self.pending {
            pending.translation_spans.push((span.start, span.end));
        } else {
            self.translation_spans.push(span);
        }
        self.last_tm = None;
    }

    fn consume_text(
        &mut self,
        token: &ContentToken,
        stream_obj: ObjectId,
        counter: &mut usize,
    ) -> Result<()> {
        if !self.active {
            return Ok(());
        }
        let chars = count_text(&token.operands);
        if chars == 0 {
            return Ok(());
        }
        let font = self.font.clone().unwrap_or(FontInfo {
            res_name: "F1".into(),
            size: 12.0,
        });
        if let Some(ref mut pending) = self.pending {
            pending.end = token.span.end;
            pending.approx_chars += chars;
            return Ok(());
        }
        let id = format!("t:{}", *counter);
        *counter += 1;
        let (tm_span, base_tm, has_tm) = if let Some((span, matrix)) = self.last_tm.take() {
            ((Some((span.start, span.end))), matrix, true)
        } else {
            (None, self.text_matrix, false)
        };
        let translation_spans = self
            .translation_spans
            .iter()
            .map(|s| (s.start, s.end))
            .collect();
        self.translation_spans.clear();
        self.pending = Some(PendingRun {
            id,
            stream_obj,
            start: token.span.start,
            end: token.span.end,
            base_tm,
            tm_span,
            translation_spans,
            font,
            approx_chars: chars,
            has_tm,
        });
        Ok(())
    }
}

const IDENTITY: [f64; 6] = [1.0, 0.0, 0.0, 1.0, 0.0, 0.0];

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

fn read_matrix(operands: &[Operand]) -> Result<[f64; 6]> {
    if operands.len() < 6 {
        bail!("matrix expects 6 operands");
    }
    let mut values = [0.0; 6];
    for (idx, operand) in operands.iter().take(6).enumerate() {
        values[idx] = match operand {
            Operand::Number(n) => *n,
            Operand::Name(name) => name.parse::<f64>().unwrap_or(0.0),
            _ => bail!("unsupported operand type in matrix"),
        };
    }
    Ok(values)
}

fn parse_td(operands: &[Operand]) -> Result<(f64, f64)> {
    if operands.len() < 2 {
        bail!("Td expects two operands");
    }
    let tx = as_f64(&operands[0])?;
    let ty = as_f64(&operands[1])?;
    Ok((tx, ty))
}

fn parse_tf(operands: &[Operand]) -> Result<(String, f64)> {
    if operands.len() < 2 {
        bail!("Tf expects name and size");
    }
    let name = match &operands[0] {
        Operand::Name(n) => n.trim_start_matches('/').to_string(),
        _ => "F1".to_string(),
    };
    let size = as_f64(&operands[1])?;
    Ok((name, size))
}

fn parse_do_name(operands: &[Operand]) -> Option<String> {
    operands.first().and_then(|operand| match operand {
        Operand::Name(name) => Some(name.trim_start_matches('/').to_string()),
        _ => None,
    })
}

fn as_f64(operand: &Operand) -> Result<f64> {
    match operand {
        Operand::Number(n) => Ok(*n),
        Operand::Name(n) => Ok(n.parse::<f64>().unwrap_or(0.0)),
        Operand::String(bytes) => std::str::from_utf8(bytes)
            .ok()
            .and_then(|s| s.parse::<f64>().ok())
            .context("invalid numeric string"),
        _ => bail!("unsupported operand for numeric conversion"),
    }
}

fn count_text(operands: &[Operand]) -> usize {
    operands
        .iter()
        .map(|operand| match operand {
            Operand::String(bytes) => bytes.len(),
            Operand::Array(items) => items.iter().map(count_operand_chars).sum(),
            _ => 0,
        })
        .sum()
}

fn count_operand_chars(operand: &Operand) -> usize {
    match operand {
        Operand::String(bytes) => bytes.len(),
        Operand::Array(items) => items.iter().map(count_operand_chars).sum(),
        _ => 0,
    }
}

fn text_bbox(base_tm: [f64; 6], font_size: f64, char_count: usize) -> [f64; 4] {
    let width = font_size * char_count as f64 * 0.5;
    let height = font_size * 1.2;
    let x0 = base_tm[4];
    let y0 = base_tm[5];
    [x0, y0 - height, x0 + width, y0 + height * 0.2]
}

fn image_bbox(matrix: [f64; 6]) -> [f64; 4] {
    let width = (matrix[0].powi(2) + matrix[1].powi(2)).sqrt();
    let height = (matrix[2].powi(2) + matrix[3].powi(2)).sqrt();
    let x0 = matrix[4];
    let y0 = matrix[5];
    [x0, y0, x0 + width, y0 + height]
}
