use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct DocumentIR {
  #[serde(rename = "docId")]
  pub doc_id: String,
  #[serde(rename = "pdfUrl", skip_serializing_if = "Option::is_none")]
  pub pdf_url: Option<String>,
  pub pages: Vec<PageIR>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct PageIR {
  pub index: usize,
  #[serde(rename = "widthPt")]
  pub width_pt: f32,
  #[serde(rename = "heightPt")]
  pub height_pt: f32,
  pub objects: Vec<PdfObjectIR>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "kind")]
pub enum PdfObjectIR {
  #[serde(rename = "text")]
  Text(TextObjectIR),
  #[serde(rename = "image")]
  Image(ImageObjectIR),
  #[serde(rename = "path")]
  Path(PathObjectIR),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TextObjectIR {
  pub id: String,
  #[serde(rename = "pdfRef")]
  pub pdf_ref: PdfRef,
  #[serde(rename = "Tm")]
  pub tm: [f32; 6],
  pub font: FontInfo,
  pub unicode: String,
  pub bbox: [f32; 4],
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ImageObjectIR {
  pub id: String,
  #[serde(rename = "pdfRef")]
  pub pdf_ref: PdfRef,
  #[serde(rename = "xObject")]
  pub x_object: String,
  #[serde(rename = "cm")]
  pub cm: [f32; 6],
  pub bbox: [f32; 4],
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PathObjectIR {
  pub id: String,
  #[serde(rename = "pdfRef")]
  pub pdf_ref: PdfRef,
  #[serde(rename = "cm")]
  pub cm: [f32; 6],
  pub bbox: [f32; 4],
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FontInfo {
  #[serde(rename = "resName")]
  pub res_name: String,
  pub size: f32,
  #[serde(rename = "type")]
  pub font_type: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct PdfRef {
  pub obj: u32,
  pub gen: u16,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "op")]
pub enum PatchOperation {
  #[serde(rename = "transform")]
  Transform(TransformPatch),
  #[serde(rename = "editText")]
  EditText(TextEditPatch),
  #[serde(rename = "setStyle")]
  SetStyle(StylePatch),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PatchTarget {
  pub page: usize,
  pub id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TransformPatch {
  pub target: PatchTarget,
  #[serde(rename = "deltaMatrixPt")]
  pub delta_matrix_pt: [f32; 6],
  pub kind: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TextEditPatch {
  pub target: PatchTarget,
  pub text: String,
  #[serde(rename = "fontPref", skip_serializing_if = "Option::is_none")]
  pub font_pref: Option<FontPreference>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct FontPreference {
  #[serde(rename = "preferExisting")]
  pub prefer_existing: bool,
  #[serde(rename = "fallbackFamily", skip_serializing_if = "Option::is_none")]
  pub fallback_family: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StylePatch {
  pub target: PatchTarget,
  pub style: Style,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Style {
  #[serde(rename = "opacityFill", skip_serializing_if = "Option::is_none")]
  pub opacity_fill: Option<f32>,
  #[serde(rename = "opacityStroke", skip_serializing_if = "Option::is_none")]
  pub opacity_stroke: Option<f32>,
  #[serde(rename = "fillColor", skip_serializing_if = "Option::is_none")]
  pub fill_color: Option<[f32; 3]>,
  #[serde(rename = "strokeColor", skip_serializing_if = "Option::is_none")]
  pub stroke_color: Option<[f32; 3]>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenResponse {
  #[serde(rename = "docId")]
  pub doc_id: String,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct PatchResponse {
  pub ok: bool,
  #[serde(rename = "updatedPdf", skip_serializing_if = "Option::is_none")]
  pub updated_pdf: Option<String>,
  #[serde(default)]
  pub remap: serde_json::Value,
}
