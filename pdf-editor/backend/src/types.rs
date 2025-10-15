use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DocumentIr {
    pub pages: Vec<PageIr>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PageIr {
    pub index: usize,
    #[serde(rename = "widthPt")]
    pub width_pt: f32,
    #[serde(rename = "heightPt")]
    pub height_pt: f32,
    #[serde(default)]
    pub objects: Vec<PageObject>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum PageObject {
    Text(TextObject),
    Image(ImageObject),
    Path(PathObject),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfRef {
    pub obj: u32,
    pub gen: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextObject {
    pub id: String,
    #[serde(rename = "pdfRef")]
    pub pdf_ref: PdfRef,
    #[serde(rename = "Tm")]
    pub tm: [f32; 6],
    pub font: FontInfo,
    pub unicode: String,
    pub bbox: [f32; 4],
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
pub struct ImageObject {
    pub id: String,
    #[serde(rename = "pdfRef")]
    pub pdf_ref: PdfRef,
    #[serde(rename = "cm")]
    pub cm: [f32; 6],
    pub bbox: [f32; 4],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathObject {
    pub id: String,
    #[serde(rename = "pdfRef")]
    pub pdf_ref: PdfRef,
    #[serde(rename = "cm")]
    pub cm: [f32; 6],
    pub bbox: [f32; 4],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op")]
pub enum PatchOp {
    #[serde(rename = "transform")]
    Transform {
        target: PatchTarget,
        #[serde(rename = "deltaMatrixPt")]
        delta_matrix_pt: [f32; 6],
        kind: String,
    },
    #[serde(rename = "editText")]
    EditText { target: PatchTarget, text: String },
    #[serde(rename = "setStyle")]
    SetStyle { target: PatchTarget },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchTarget {
    pub page: usize,
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PatchRequest {
    #[serde(default)]
    pub ops: Vec<PatchOp>,
}
