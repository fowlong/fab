use serde::{Deserialize, Serialize};

pub type PdfMatrix = [f64; 6];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfRef {
    pub obj: u32,
    pub gen: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Glyph {
    pub gid: u32,
    pub dx: f64,
    pub dy: f64,
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

impl PageObject {
    pub fn id(&self) -> &str {
        match self {
            PageObject::Text(t) => &t.id,
            PageObject::Image(i) => &i.id,
            PageObject::Path(p) => &p.id,
        }
    }

    pub fn bbox(&self) -> &[f64; 4] {
        match self {
            PageObject::Text(t) => &t.bbox,
            PageObject::Image(i) => &i.bbox,
            PageObject::Path(p) => &p.bbox,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextObject {
    pub id: String,
    #[serde(rename = "pdfRef")]
    pub pdf_ref: PdfRef,
    #[serde(rename = "btSpan")]
    pub bt_span: StreamSpan,
    #[serde(rename = "Tm")]
    pub tm: PdfMatrix,
    pub font: TextFont,
    pub unicode: String,
    pub glyphs: Vec<Glyph>,
    pub bbox: [f64; 4],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextFont {
    #[serde(rename = "resName")]
    pub res_name: String,
    pub size: f64,
    #[serde(rename = "type")]
    pub font_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamSpan {
    pub start: usize,
    pub end: usize,
    #[serde(rename = "streamObj")]
    pub stream_obj: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageObject {
    pub id: String,
    #[serde(rename = "pdfRef")]
    pub pdf_ref: PdfRef,
    #[serde(rename = "xObject")]
    pub x_object: String,
    pub cm: PdfMatrix,
    pub bbox: [f64; 4],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathObject {
    pub id: String,
    #[serde(rename = "pdfRef")]
    pub pdf_ref: PdfRef,
    pub cm: PdfMatrix,
    pub bbox: [f64; 4],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageIr {
    pub index: usize,
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
#[serde(tag = "op")]
pub enum PatchOperation {
    #[serde(rename = "transform")]
    Transform(TransformPatch),
    #[serde(rename = "editText")]
    EditText(EditTextPatch),
    #[serde(rename = "setStyle")]
    SetStyle(SetStylePatch),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformPatch {
    pub target: PatchTarget,
    #[serde(rename = "deltaMatrixPt")]
    pub delta_matrix_pt: PdfMatrix,
    pub kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditTextPatch {
    pub target: PatchTarget,
    pub text: String,
    #[serde(default, rename = "fontPref")]
    pub font_pref: Option<FontPreference>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetStylePatch {
    pub target: PatchTarget,
    pub style: StylePatch,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StylePatch {
    #[serde(rename = "fillColor")]
    pub fill_color: Option<String>,
    #[serde(rename = "strokeColor")]
    pub stroke_color: Option<String>,
    #[serde(rename = "opacityFill")]
    pub opacity_fill: Option<f64>,
    #[serde(rename = "opacityStroke")]
    pub opacity_stroke: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontPreference {
    #[serde(rename = "preferExisting")]
    pub prefer_existing: Option<bool>,
    #[serde(rename = "fallbackFamily")]
    pub fallback_family: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchTarget {
    pub page: usize,
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchResponse {
    pub ok: bool,
    #[serde(rename = "updatedPdf")]
    pub updated_pdf: Option<String>,
    pub remap: Option<serde_json::Value>,
    pub error: Option<String>,
}
