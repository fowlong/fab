use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfRef {
  pub obj: i32,
  pub gen: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BtSpan {
  pub start: usize,
  pub end: usize,
  pub stream_obj: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontInfo {
  #[serde(rename = "resName")]
  pub res_name: String,
  pub size: f32,
  #[serde(rename = "type")]
  pub font_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextGlyph {
  pub gid: u32,
  pub dx: f32,
  pub dy: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextObject {
  pub id: String,
  #[serde(rename = "pdfRef")]
  pub pdf_ref: PdfRef,
  #[serde(rename = "btSpan")]
  pub bt_span: BtSpan,
  #[serde(rename = "Tm")]
  pub tm: [f32; 6],
  pub font: FontInfo,
  pub unicode: String,
  pub glyphs: Vec<TextGlyph>,
  pub bbox: [f32; 4],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageObject {
  pub id: String,
  #[serde(rename = "pdfRef")]
  pub pdf_ref: PdfRef,
  #[serde(rename = "xObject")]
  pub x_object: String,
  #[serde(rename = "cm")]
  pub cm: [f32; 6],
  pub bbox: [f32; 4],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathObject {
  pub id: String,
  #[serde(rename = "pdfRef")]
  pub pdf_ref: PdfRef,
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
pub struct Page {
  pub index: usize,
  #[serde(rename = "widthPt")]
  pub width_pt: f32,
  #[serde(rename = "heightPt")]
  pub height_pt: f32,
  pub objects: Vec<PageObject>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentIr {
  pub pages: Vec<Page>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformOp {
  pub op: String,
  pub target: TargetRef,
  #[serde(rename = "deltaMatrixPt")]
  pub delta_matrix_pt: [f32; 6],
  pub kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditTextOp {
  pub op: String,
  pub target: TargetRef,
  pub text: String,
  #[serde(rename = "fontPref")]
  pub font_pref: FontPreference,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontPreference {
  #[serde(rename = "preferExisting")]
  pub prefer_existing: bool,
  #[serde(rename = "fallbackFamily")]
  pub fallback_family: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetStyleOp {
  pub op: String,
  pub target: TargetRef,
  pub style: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetRef {
  pub page: usize,
  pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PatchOp {
  Transform(TransformOp),
  EditText(EditTextOp),
  SetStyle(SetStyleOp),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchResponse {
  pub ok: bool,
  #[serde(rename = "updatedPdf")]
  pub updated_pdf: Option<String>,
  pub remap: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenResponse {
  #[serde(rename = "docId")]
  pub doc_id: String,
  pub ir: DocumentIr,
}
