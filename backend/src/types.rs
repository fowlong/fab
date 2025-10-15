use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfRef {
    pub obj: u32,
    pub gen: u16,
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
pub struct TextObject {
    pub id: String,
    #[serde(rename = "pdfRef")]
    pub pdf_ref: PdfRef,
    #[serde(rename = "btSpan")]
    pub bt_span: Span,
    #[serde(rename = "Tm")]
    pub tm: [f32; 6],
    pub font: FontInfo,
    pub unicode: String,
    pub glyphs: Vec<Glyph>,
    pub bbox: [f32; 4],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageObject {
    pub id: String,
    #[serde(rename = "pdfRef")]
    pub pdf_ref: PdfRef,
    #[serde(rename = "xObject")]
    pub x_object: String,
    pub cm: [f32; 6],
    pub bbox: [f32; 4],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathObject {
    pub id: String,
    #[serde(rename = "pdfRef")]
    pub pdf_ref: PdfRef,
    pub cm: [f32; 6],
    pub bbox: [f32; 4],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    #[serde(rename = "streamObj")]
    pub stream_obj: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Glyph {
    pub gid: u32,
    pub dx: f32,
    pub dy: f32,
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
pub struct PageIR {
    pub index: usize,
    #[serde(rename = "widthPt")]
    pub width_pt: f32,
    #[serde(rename = "heightPt")]
    pub height_pt: f32,
    pub objects: Vec<PageObject>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentIR {
    pub pages: Vec<PageIR>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op")]
pub enum PatchOperation {
    #[serde(rename = "transform")]
    Transform(TransformOp),
    #[serde(other)]
    Unsupported,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformOp {
    pub target: PatchTarget,
    #[serde(rename = "deltaMatrixPt")]
    pub delta_matrix_pt: [f32; 6],
    pub kind: String,
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
}
