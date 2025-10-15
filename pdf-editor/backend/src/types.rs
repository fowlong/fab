use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfRef {
    pub obj: u32,
    pub gen: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase", tag = "kind")]
pub enum PageObject {
    Text(TextObject),
    Image(ImageObject),
    Path(PathObject),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextObject {
    pub id: String,
    pub pdf_ref: PdfRef,
    pub bt_span: BtSpan,
    pub tm: [f64; 6],
    pub font: FontInfo,
    pub unicode: String,
    pub glyphs: Vec<TextGlyph>,
    pub bbox: [f64; 4],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageObject {
    pub id: String,
    pub pdf_ref: PdfRef,
    pub x_object: String,
    pub cm: [f64; 6],
    pub bbox: [f64; 4],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathObject {
    pub id: String,
    pub pdf_ref: PdfRef,
    pub cm: [f64; 6],
    pub bbox: [f64; 4],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BtSpan {
    pub start: usize,
    pub end: usize,
    pub stream_obj: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontInfo {
    pub res_name: String,
    pub size: f64,
    #[serde(rename = "type")]
    pub type_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextGlyph {
    pub gid: u32,
    pub dx: f64,
    pub dy: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageIR {
    pub index: usize,
    pub width_pt: f64,
    pub height_pt: f64,
    pub objects: Vec<PageObject>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentIR {
    pub pages: Vec<PageIR>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchTarget {
    pub page: usize,
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "camelCase")]
pub enum PatchOp {
    Transform {
        target: PatchTarget,
        #[serde(rename = "deltaMatrixPt")]
        delta_matrix_pt: [f64; 6],
        kind: String,
    },
    EditText {
        target: PatchTarget,
        text: String,
        #[serde(default)]
        font_pref: Option<FontPref>,
    },
    SetStyle {
        target: PatchTarget,
        style: StylePatch,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FontPref {
    #[serde(default)]
    pub prefer_existing: bool,
    #[serde(default)]
    pub fallback_family: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StylePatch {
    #[serde(default)]
    pub fill_color: Option<String>,
    #[serde(default)]
    pub stroke_color: Option<String>,
    #[serde(default)]
    pub opacity_fill: Option<f32>,
    #[serde(default)]
    pub opacity_stroke: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchResponse {
    pub ok: bool,
    #[serde(default)]
    pub updated_pdf: Option<String>,
    #[serde(default)]
    pub remap: Option<serde_json::Value>,
    #[serde(default)]
    pub error: Option<String>,
}
