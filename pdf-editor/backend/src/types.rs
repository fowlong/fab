use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_with::{serde_as, OneOrMany};
use uuid::Uuid;

pub type DocumentId = Uuid;

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DocumentIr {
    pub pages: Vec<PageIr>,
    #[serde(default)]
    pub meta: HashMap<String, serde_json::Value>,
}

impl DocumentIr {
    pub fn empty() -> Self {
        Self::default()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PageIr {
    pub index: u32,
    pub width_pt: f32,
    pub height_pt: f32,
    #[serde(default)]
    pub objects: Vec<PageObject>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum PageObject {
    Text(TextObject),
    Image(ImageObject),
    Path(PathObject),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfRef {
    pub obj: u32,
    pub gen: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextObject {
    pub id: String,
    pub pdf_ref: Option<PdfRef>,
    pub bt_span: Option<StreamSpan>,
    pub tm: [f32; 6],
    pub font: Option<TextFont>,
    pub unicode: String,
    #[serde(default)]
    pub glyphs: Vec<TextGlyph>,
    pub bbox: [f32; 4],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextFont {
    pub res_name: String,
    pub size: f32,
    #[serde(default)]
    pub r#type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextGlyph {
    pub gid: u32,
    pub dx: f32,
    pub dy: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamSpan {
    pub start: usize,
    pub end: usize,
    pub stream_obj: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageObject {
    pub id: String,
    pub pdf_ref: Option<PdfRef>,
    pub x_object: Option<String>,
    pub cm: [f32; 6],
    pub bbox: [f32; 4],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathObject {
    pub id: String,
    pub pdf_ref: Option<PdfRef>,
    pub cm: [f32; 6],
    pub bbox: [f32; 4],
    #[serde(default)]
    pub style: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "camelCase")]
pub enum PatchOp {
    Transform {
        target: PatchTarget,
        #[serde(rename = "deltaMatrixPt")]
        delta_matrix_pt: [f32; 6],
        kind: String,
    },
    EditText {
        target: PatchTarget,
        text: String,
        #[serde(default)]
        font_pref: Option<FontPreference>,
    },
    SetStyle {
        target: PatchTarget,
        style: HashMap<String, serde_json::Value>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchTarget {
    pub page: u32,
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontPreference {
    #[serde(default)]
    pub prefer_existing: bool,
    #[serde(default)]
    pub fallback_family: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchResponse {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_pdf: Option<String>,
    #[serde(default)]
    pub remap: HashMap<String, RemappedRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemappedRef {
    #[serde(rename = "pdfRef")]
    pub pdf_ref: PdfRef,
}
