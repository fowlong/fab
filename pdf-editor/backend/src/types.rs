use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DocumentIr {
    pub pages: Vec<PageIr>,
    #[serde(skip)]
    pub patches: Vec<PatchRequest>,
}

impl DocumentIr {
    pub fn empty() -> Self {
        Self {
            pages: vec![PageIr {
                index: 0,
                width_pt: 595.276,
                height_pt: 841.89,
                objects: Vec::new(),
            }],
            patches: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PageIr {
    pub index: usize,
    pub width_pt: f32,
    pub height_pt: f32,
    pub objects: Vec<IrObject>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum IrObject {
    #[serde(rename = "text")]
    Text(TextObject),
    #[serde(rename = "image")]
    Image(ImageObject),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TextObject {
    pub id: String,
    pub unicode: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ImageObject {
    pub id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PatchRequest {
    pub ops: Vec<PatchOp>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "op")]
pub enum PatchOp {
    #[serde(rename = "transform")]
    Transform { target: PatchTarget },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PatchTarget {
    pub page: usize,
    pub id: String,
}
