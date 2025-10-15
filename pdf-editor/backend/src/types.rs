use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PdfRef {
    pub obj: u32,
    pub gen: u16,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Glyph {
    pub gid: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dx: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dy: Option<f32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "kind")]
pub enum PageObject {
    #[serde(rename_all = "camelCase")]
    Text {
        id: String,
        pdf_ref: Option<PdfRef>,
        #[serde(skip_serializing_if = "Option::is_none")]
        bt_span: Option<StreamSpan>,
        tm: [f32; 6],
        font: TextFont,
        unicode: String,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        glyphs: Vec<Glyph>,
        bbox: [f32; 4],
    },
    #[serde(rename_all = "camelCase")]
    Image {
        id: String,
        pdf_ref: Option<PdfRef>,
        x_object: Option<String>,
        cm: [f32; 6],
        bbox: [f32; 4],
    },
    #[serde(rename_all = "camelCase")]
    Path {
        id: String,
        pdf_ref: Option<PdfRef>,
        #[serde(skip_serializing_if = "Option::is_none")]
        cm: Option<[f32; 6]>,
        bbox: [f32; 4],
    },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StreamSpan {
    pub start: usize,
    pub end: usize,
    pub stream_obj: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TextFont {
    pub res_name: String,
    pub size: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PageIr {
    pub index: u32,
    pub width_pt: f32,
    pub height_pt: f32,
    #[serde(default)]
    pub objects: Vec<PageObject>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct DocumentIr {
    pub pages: Vec<PageIr>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "camelCase")]
pub enum PatchOp {
    Transform {
        target: PatchTarget,
        delta_matrix_pt: [f32; 6],
        kind: String,
    },
    EditText {
        target: PatchTarget,
        text: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        font_pref: Option<FontPreference>,
    },
    SetStyle {
        target: PatchTarget,
        style: StylePatch,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PatchTarget {
    pub page: u32,
    pub id: String,
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct FontPreference {
    pub prefer_existing: Option<bool>,
    pub fallback_family: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct StylePatch {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fill_color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stroke_color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opacity_fill: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opacity_stroke: Option<f32>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PatchResponse {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_pdf: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remap: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}
