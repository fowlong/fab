use serde::{Deserialize, Serialize};

pub type Matrix6 = [f64; 6];

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Span {
    pub stream_obj: u32,
    pub start: u32,
    pub end: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FontInfo {
    pub res_name: String,
    pub size: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextObject {
    pub id: String,
    pub bt_span: Span,
    #[serde(rename = "Tm")]
    pub tm: Matrix6,
    pub font: FontInfo,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bbox: Option<[f64; 4]>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageObject {
    pub id: String,
    #[serde(rename = "xObject")]
    pub x_object: String,
    #[serde(rename = "cm")]
    pub cm: Matrix6,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bbox: Option<[f64; 4]>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum PageObject {
    Text(TextObject),
    Image(ImageObject),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageIR {
    pub index: usize,
    pub width_pt: f64,
    pub height_pt: f64,
    pub objects: Vec<PageObject>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentIR {
    pub pages: Vec<PageIR>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PatchTarget {
    pub page: usize,
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PatchKind {
    Text,
    Image,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "camelCase")]
pub enum PatchOperation {
    Transform {
        target: PatchTarget,
        #[serde(rename = "deltaMatrixPt")]
        delta_matrix_pt: Matrix6,
        kind: PatchKind,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PatchResponse {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_pdf: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remap: Option<serde_json::Value>,
}
