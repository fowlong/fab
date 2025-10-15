use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_with::{serde_as, skip_serializing_none};
use uuid::Uuid;

pub type Matrix = [f64; 6];

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DocumentIr {
    pub pages: Vec<Page>,
}

impl DocumentIr {
    pub fn new(pages: Vec<Page>) -> Self {
        Self { pages }
    }

    pub fn empty() -> Self {
        Self { pages: Vec::new() }
    }
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Page {
    pub index: usize,
    #[serde(rename = "widthPt")]
    pub width_pt: f64,
    #[serde(rename = "heightPt")]
    pub height_pt: f64,
    pub objects: Vec<PageObject>,
}

#[serde(tag = "kind", rename_all = "camelCase")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PageObject {
    Text(TextObject),
    Image(ImageObject),
    Path(PathObject),
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfRef {
    pub obj: i64,
    pub gen: i64,
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BtSpan {
    pub start: usize,
    pub end: usize,
    #[serde(rename = "streamObj")]
    pub stream_obj: i64,
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlyphRun {
    pub gid: u32,
    pub dx: f64,
    pub dy: f64,
}

#[skip_serializing_none]
#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextObject {
    pub id: String,
    #[serde(rename = "pdfRef")]
    pub pdf_ref: PdfRef,
    #[serde(rename = "btSpan")]
    pub bt_span: BtSpan,
    #[serde(rename = "Tm")]
    pub tm: Matrix,
    pub font: FontDescriptor,
    pub unicode: String,
    pub glyphs: Vec<GlyphRun>,
    pub bbox: [f64; 4],
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontDescriptor {
    #[serde(rename = "resName")]
    pub res_name: String,
    pub size: f64,
    #[serde(rename = "type")]
    pub font_type: String,
}

#[skip_serializing_none]
#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageObject {
    pub id: String,
    #[serde(rename = "pdfRef")]
    pub pdf_ref: PdfRef,
    #[serde(rename = "xObject")]
    pub x_object: String,
    #[serde(rename = "cm")]
    pub cm: Matrix,
    pub bbox: [f64; 4],
}

#[skip_serializing_none]
#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathObject {
    pub id: String,
    #[serde(rename = "pdfRef")]
    pub pdf_ref: PdfRef,
    #[serde(rename = "cm")]
    pub cm: Matrix,
    pub bbox: [f64; 4],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenRequest {
    pub data: String,
    pub filename: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenResponse {
    #[serde(rename = "docId")]
    pub doc_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PatchResponse {
    pub ok: bool,
    #[serde(rename = "updatedPdf")]
    pub updated_pdf: Option<String>,
    pub remap: HashMap<String, RemappedObject>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RemappedObject {
    #[serde(rename = "pdfRef")]
    pub pdf_ref: Option<PdfRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchTarget {
    pub page: usize,
    pub id: String,
}

#[skip_serializing_none]
#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StylePatch {
    #[serde(rename = "colorFill")]
    pub color_fill: Option<[f64; 3]>,
    #[serde(rename = "colorStroke")]
    pub color_stroke: Option<[f64; 3]>,
    #[serde(rename = "opacityFill")]
    pub opacity_fill: Option<f64>,
    #[serde(rename = "opacityStroke")]
    pub opacity_stroke: Option<f64>,
}

#[skip_serializing_none]
#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FontPreference {
    #[serde(rename = "preferExisting")]
    pub prefer_existing: Option<bool>,
    #[serde(rename = "fallbackFamily")]
    pub fallback_family: Option<String>,
}

#[serde(tag = "op", rename_all = "camelCase")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PatchOp {
    Transform {
        target: PatchTarget,
        #[serde(rename = "deltaMatrixPt")]
        delta_matrix_pt: Matrix,
        #[serde(default)]
        kind: Option<String>,
    },
    EditText {
        target: PatchTarget,
        text: String,
        #[serde(rename = "fontPref", default)]
        font_pref: Option<FontPreference>,
    },
    SetStyle {
        target: PatchTarget,
        style: StylePatch,
    },
}
