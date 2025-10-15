use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub type PdfMatrix = [f64; 6];

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PdfRef {
    pub obj: u32,
    pub gen: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BtSpan {
    pub start: u64,
    pub end: u64,
    pub stream_obj: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FontInfo {
    pub res_name: String,
    pub size: f64,
    #[serde(default)]
    pub r#type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextGlyph {
    pub gid: u32,
    pub dx: f64,
    pub dy: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum IrObject {
    Text {
        id: String,
        pdf_ref: PdfRef,
        bt_span: BtSpan,
        #[serde(rename = "Tm")]
        tm: PdfMatrix,
        font: FontInfo,
        unicode: String,
        glyphs: Vec<TextGlyph>,
        bbox: [f64; 4],
    },
    Image {
        id: String,
        pdf_ref: PdfRef,
        #[serde(default)]
        x_object: Option<String>,
        cm: PdfMatrix,
        bbox: [f64; 4],
    },
    Path {
        id: String,
        pdf_ref: PdfRef,
        cm: PdfMatrix,
        bbox: [f64; 4],
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IrPage {
    pub index: u32,
    pub width_pt: f64,
    pub height_pt: f64,
    pub objects: Vec<IrObject>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IrDocument {
    pub pages: Vec<IrPage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "camelCase")]
pub enum PatchOp {
    Transform {
        target: PatchTarget,
        delta_matrix_pt: PdfMatrix,
        kind: PatchTargetKind,
    },
    EditText {
        target: PatchTarget,
        text: String,
        #[serde(default)]
        font_pref: Option<FontPreference>,
    },
    SetStyle {
        target: PatchTarget,
        style: StylePatch,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PatchTarget {
    pub page: u32,
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FontPreference {
    #[serde(default)]
    pub prefer_existing: bool,
    #[serde(default)]
    pub fallback_family: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StylePatch {
    #[serde(default)]
    pub opacity_fill: Option<f32>,
    #[serde(default)]
    pub opacity_stroke: Option<f32>,
    #[serde(default)]
    pub fill_color: Option<[f32; 3]>,
    #[serde(default)]
    pub stroke_color: Option<[f32; 3]>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RefUpdate {
    pub pdf_ref: PdfRef,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PatchResponse {
    pub ok: bool,
    #[serde(default)]
    pub updated_pdf: Option<String>,
    #[serde(default)]
    pub remap: HashMap<String, RefUpdate>,
    #[serde(default)]
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PatchTargetKind {
    Text,
    Image,
    Path,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenResponse {
    pub doc_id: Uuid,
}
