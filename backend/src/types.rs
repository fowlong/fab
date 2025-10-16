use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BtSpan {
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
    #[serde(rename = "btSpan")]
    pub bt_span: BtSpan,
    #[serde(rename = "Tm")]
    pub tm: [f64; 6],
    pub font: FontInfo,
    pub bbox: [f64; 4],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageObject {
    pub id: String,
    #[serde(rename = "xObject")]
    pub x_object: String,
    #[serde(rename = "cm")]
    pub cm: [f64; 6],
    pub bbox: [f64; 4],
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
pub enum TransformKind {
    Text,
    Image,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "camelCase")]
pub enum PatchOperation {
    Transform {
        target: PatchTarget,
        #[serde(rename = "deltaMatrixPt")]
        delta_matrix_pt: [f64; 6],
        kind: TransformKind,
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
