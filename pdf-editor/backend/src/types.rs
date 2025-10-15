use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PdfState {
    pub pages: Vec<IrPage>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct IrPage {
    pub index: usize,
    pub width_pt: f64,
    pub height_pt: f64,
    pub objects: Vec<IrObject>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum IrObject {
    #[serde(rename = "text")]
    Text(IrTextObject),
    #[serde(rename = "image")]
    Image(IrImageObject),
    #[serde(rename = "path")]
    Path(IrPathObject),
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct IrTextObject {
    pub id: String,
    pub bbox: [f64; 4],
    pub unicode: String,
    pub font: FontInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct IrImageObject {
    pub id: String,
    pub bbox: [f64; 4],
    pub x_object: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct IrPathObject {
    pub id: String,
    pub bbox: [f64; 4],
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct FontInfo {
    pub res_name: String,
    pub size: f64,
    pub type_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfOpenResponse {
    #[serde(rename = "docId")]
    pub doc_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op")]
pub enum PatchOperation {
    #[serde(rename = "transform")]
    Transform {
        target: PatchTarget,
        #[serde(rename = "deltaMatrixPt")]
        delta_matrix_pt: [f64; 6],
        kind: String,
    },
    #[serde(rename = "editText")]
    EditText { target: PatchTarget, text: String },
    #[serde(rename = "setStyle")]
    SetStyle {
        target: PatchTarget,
        style: serde_json::Value,
    },
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
