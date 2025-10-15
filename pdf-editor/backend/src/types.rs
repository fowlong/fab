use serde::{Deserialize, Serialize};

pub type Matrix = [f64; 6];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfRef {
    pub obj: u32,
    #[serde(rename = "gen")]
    pub generation: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextObject {
    pub id: String,
    pub kind: String,
    #[serde(rename = "pdfRef")]
    pub pdf_ref: PdfRef,
    #[serde(rename = "btSpan")]
    pub bt_span: Span,
    #[serde(rename = "Tm")]
    pub tm: Matrix,
    pub font: FontInfo,
    pub unicode: String,
    pub glyphs: Vec<Glyph>,
    pub bbox: [f64; 4],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageObject {
    pub id: String,
    pub kind: String,
    #[serde(rename = "pdfRef")]
    pub pdf_ref: PdfRef,
    #[serde(rename = "xObject")]
    pub x_object: String,
    #[serde(rename = "cm")]
    pub cm: Matrix,
    pub bbox: [f64; 4],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathObject {
    pub id: String,
    pub kind: String,
    #[serde(rename = "pdfRef")]
    pub pdf_ref: PdfRef,
    #[serde(rename = "cm")]
    pub cm: Matrix,
    pub bbox: [f64; 4],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PageObject {
    Text(TextObject),
    Image(ImageObject),
    Path(PathObject),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageIr {
    pub index: u32,
    #[serde(rename = "widthPt")]
    pub width_pt: f64,
    #[serde(rename = "heightPt")]
    pub height_pt: f64,
    pub objects: Vec<PageObject>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentIr {
    pub pages: Vec<PageIr>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    #[serde(rename = "streamObj")]
    pub stream_obj: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontInfo {
    #[serde(rename = "resName")]
    pub res_name: String,
    pub size: f64,
    #[serde(rename = "type")]
    pub font_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Glyph {
    pub gid: u32,
    pub dx: f64,
    pub dy: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformPatch {
    pub op: String,
    pub target: PatchTarget,
    #[serde(rename = "deltaMatrixPt")]
    pub delta_matrix_pt: Matrix,
    pub kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditTextPatch {
    pub op: String,
    pub target: PatchTarget,
    pub text: String,
    #[serde(rename = "fontPref")]
    pub font_pref: Option<FontPreference>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetStylePatch {
    pub op: String,
    pub target: PatchTarget,
    pub style: StylePatch,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PatchOperation {
    Transform(TransformPatch),
    EditText(EditTextPatch),
    SetStyle(SetStylePatch),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchTarget {
    pub page: u32,
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontPreference {
    #[serde(rename = "preferExisting")]
    pub prefer_existing: Option<bool>,
    #[serde(rename = "fallbackFamily")]
    pub fallback_family: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StylePatch {
    #[serde(rename = "opacityFill")]
    pub opacity_fill: Option<f32>,
    #[serde(rename = "opacityStroke")]
    pub opacity_stroke: Option<f32>,
    #[serde(rename = "fillColor")]
    pub fill_color: Option<String>,
    #[serde(rename = "strokeColor")]
    pub stroke_color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectRemap {
    #[serde(rename = "pdfRef")]
    pub pdf_ref: PdfRef,
}
