use serde::{Deserialize, Serialize};

pub type Matrix = [f64; 6];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfObjectRef {
    pub obj: u32,
    pub gen: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextGlyph {
    pub gid: u32,
    pub dx: f64,
    pub dy: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum IrObject {
    #[serde(rename = "text")]
    Text(TextObject),
    #[serde(rename = "image")]
    Image(ImageObject),
    #[serde(rename = "path")]
    Path(PathObject),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextObject {
    pub id: String,
    #[serde(rename = "pdfRef")]
    pub pdf_ref: PdfObjectRef,
    #[serde(rename = "btSpan")]
    pub bt_span: BtSpan,
    #[serde(rename = "Tm")]
    pub tm: Matrix,
    pub font: FontInfo,
    pub unicode: String,
    pub glyphs: Vec<TextGlyph>,
    pub bbox: [f64; 4],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageObject {
    pub id: String,
    #[serde(rename = "pdfRef")]
    pub pdf_ref: PdfObjectRef,
    #[serde(rename = "xObject")]
    pub x_object: String,
    pub cm: Matrix,
    pub bbox: [f64; 4],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathObject {
    pub id: String,
    #[serde(rename = "pdfRef")]
    pub pdf_ref: PdfObjectRef,
    pub cm: Matrix,
    pub bbox: [f64; 4],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BtSpan {
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
pub struct PageIr {
    pub index: usize,
    #[serde(rename = "widthPt")]
    pub width_pt: f64,
    #[serde(rename = "heightPt")]
    pub height_pt: f64,
    pub objects: Vec<IrObject>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentIr {
    pub pages: Vec<PageIr>,
    #[serde(
        rename = "sourcePdfUrl",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub source_pdf_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformPatchOperation {
    pub target: PatchTarget,
    #[serde(rename = "deltaMatrixPt")]
    pub delta_matrix_pt: Matrix,
    pub kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditTextPatchOperation {
    pub target: PatchTarget,
    pub text: String,
    #[serde(rename = "fontPref", default, skip_serializing_if = "Option::is_none")]
    pub font_pref: Option<FontPreference>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetStylePatchOperation {
    pub target: PatchTarget,
    pub style: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op")]
pub enum PatchOperation {
    #[serde(rename = "transform")]
    Transform(TransformPatchOperation),
    #[serde(rename = "editText")]
    EditText(EditTextPatchOperation),
    #[serde(rename = "setStyle")]
    SetStyle(SetStylePatchOperation),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchTarget {
    pub page: usize,
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontPreference {
    #[serde(rename = "preferExisting")]
    pub prefer_existing: bool,
    #[serde(
        rename = "fallbackFamily",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub fallback_family: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchResponse {
    pub ok: bool,
    #[serde(
        rename = "updatedPdf",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub updated_pdf: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remap: Option<serde_json::Value>,
}
