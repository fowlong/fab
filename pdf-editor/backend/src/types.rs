use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfRef {
    pub obj: u32,
    pub gen: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextObject {
    pub id: String,
    pub kind: String,
    pub pdf_ref: PdfRef,
    pub bt_span: Span,
    pub tm: [f32; 6],
    pub font: FontInfo,
    pub unicode: String,
    pub glyphs: Vec<GlyphInfo>,
    pub bbox: [f32; 4],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageObject {
    pub id: String,
    pub kind: String,
    pub pdf_ref: PdfRef,
    pub x_object: String,
    pub cm: [f32; 6],
    pub bbox: [f32; 4],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathObject {
    pub id: String,
    pub kind: String,
    pub pdf_ref: PdfRef,
    pub cm: [f32; 6],
    pub bbox: [f32; 4],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum PageObject {
    #[serde(rename = "text")]
    Text(TextObject),
    #[serde(rename = "image")]
    Image(ImageObject),
    #[serde(rename = "path")]
    Path(PathObject),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageIR {
    pub index: usize,
    pub width_pt: f32,
    pub height_pt: f32,
    pub objects: Vec<PageObject>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentIR {
    pub pages: Vec<PageIR>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub stream_obj: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontInfo {
    pub res_name: String,
    pub size: f32,
    #[serde(rename = "type")]
    pub font_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlyphInfo {
    pub gid: u32,
    pub dx: f32,
    pub dy: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op")]
pub enum PatchOp {
    #[serde(rename = "transform")]
    Transform {
        target: PatchTarget,
        delta_matrix_pt: [f32; 6],
        kind: String,
    },
    #[serde(rename = "editText")]
    EditText {
        target: PatchTarget,
        text: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        font_pref: Option<FontPreference>,
    },
    #[serde(rename = "setStyle")]
    SetStyle {
        target: PatchTarget,
        style: StylePatch,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchTarget {
    pub page: usize,
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontPreference {
    #[serde(default)]
    pub prefer_existing: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fallback_family: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StylePatch {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fill_color: Option<[f32; 3]>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stroke_color: Option<[f32; 3]>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub opacity_fill: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub opacity_stroke: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchResponse {
    pub ok: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_pdf: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remap: Option<HashMap<String, RemapEntry>>,
}

impl PatchResponse {
    pub fn error(_err: anyhow::Error) -> Self {
        Self {
            ok: false,
            updated_pdf: None,
            remap: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemapEntry {
    pub pdf_ref: PdfRef,
}

use std::collections::HashMap;
