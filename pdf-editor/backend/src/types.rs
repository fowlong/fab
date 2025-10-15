use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentIR {
    pub doc_id: String,
    pub pages: Vec<PageIR>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_url: Option<String>,
}

impl DocumentIR {
    pub fn empty() -> Self {
        Self {
            doc_id: "placeholder".to_string(),
            pages: vec![PageIR::placeholder()],
            source_url: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageIR {
    pub index: usize,
    pub width_pt: f32,
    pub height_pt: f32,
    pub objects: Vec<IrObject>,
}

impl PageIR {
    pub fn placeholder() -> Self {
        Self {
            index: 0,
            width_pt: 595.0,
            height_pt: 842.0,
            objects: vec![IrObject::Text(TextObject {
                id: "t:placeholder".to_string(),
                unicode: "Select an object to begin".to_string(),
                bbox: [72.0, 720.0, 360.0, 760.0],
                transform: Some([1.0, 0.0, 0.0, 1.0, 0.0, 0.0]),
            })],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum IrObject {
    Text(TextObject),
    Image(ImageObject),
    Path(PathObject),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextObject {
    pub id: String,
    pub unicode: String,
    pub bbox: [f32; 4],
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transform: Option<[f32; 6]>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageObject {
    pub id: String,
    pub bbox: [f32; 4],
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transform: Option<[f32; 6]>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PathObject {
    pub id: String,
    pub bbox: [f32; 4],
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transform: Option<[f32; 6]>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "camelCase")]
pub enum PatchOperation {
    Transform {
        target: PatchTarget,
        delta_matrix_pt: [f32; 6],
        kind: String,
    },
    EditText {
        target: PatchTarget,
        text: String,
    },
    SetStyle {
        target: PatchTarget,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PatchTarget {
    pub page: usize,
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PatchResponse {
    pub ok: bool,
    pub updated_pdf: Option<String>,
    pub message: Option<String>,
}

impl PatchResponse {
    pub fn acknowledged(applied: usize) -> Self {
        Self {
            ok: true,
            updated_pdf: None,
            message: Some(format!("acknowledged {applied} operations")),
        }
    }
}
