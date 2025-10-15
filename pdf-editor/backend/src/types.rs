use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct DocId(#[serde(with = "uuid::serde::compact")] Uuid);

impl DocId {
    pub fn new(inner: Uuid) -> Self {
        Self(inner)
    }

    pub fn parse(value: String) -> anyhow::Result<Self> {
        let uuid = Uuid::parse_str(&value)?;
        Ok(Self(uuid))
    }

    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl std::fmt::Display for DocId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct IrDocument {
    pub pages: Vec<IrPage>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct IrPage {
    pub index: usize,
    pub width_pt: f32,
    pub height_pt: f32,
    #[serde(default)]
    pub objects: Vec<IrObject>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum IrObject {
    #[serde(rename = "text")]
    Text(TextObject),
    #[serde(rename = "image")]
    Image(ImageObject),
    #[serde(rename = "path")]
    Path(PathObject),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TextObject {
    pub id: String,
    pub unicode: String,
    #[serde(default)]
    pub bbox: [f32; 4],
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ImageObject {
    pub id: String,
    #[serde(default)]
    pub bbox: [f32; 4],
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PathObject {
    pub id: String,
    #[serde(default)]
    pub bbox: [f32; 4],
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OpenPdfRequest {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(rename = "pdfBase64")]
    pub pdf_base64: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OpenPdfResponse {
    #[serde(rename = "docId")]
    pub doc_id: DocId,
}

pub type PatchRequest = Vec<PatchOperation>;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "camelCase")]
pub enum PatchOperation {
    Transform(TransformOp),
    EditText(EditTextOp),
    SetStyle(SetStyleOp),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TransformOp {
    pub target: PatchTarget,
    #[serde(rename = "deltaMatrixPt")]
    pub delta_matrix_pt: [f32; 6],
    pub kind: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EditTextOp {
    pub target: PatchTarget,
    pub text: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SetStyleOp {
    pub target: PatchTarget,
    #[serde(default)]
    pub style: serde_json::Value,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PatchTarget {
    pub page: usize,
    pub id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PatchResponse {
    pub ok: bool,
    #[serde(rename = "updatedPdf")]
    pub updated_pdf: Option<String>,
    #[serde(default)]
    pub remap: serde_json::Value,
}
