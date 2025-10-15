use serde::{Deserialize, Serialize};

type PdfMatrix = [f32; 6];

pub type DocumentId = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentIR {
    pub doc_id: DocumentId,
    pub pages: Vec<PageIR>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_pdf: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageIR {
    pub index: usize,
    pub width_pt: f32,
    pub height_pt: f32,
    pub objects: Vec<DocumentObjectIR>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum DocumentObjectIR {
    #[serde(rename = "text")]
    Text(TextObjectIR),
    #[serde(rename = "image")]
    Image(ImageObjectIR),
    #[serde(rename = "path")]
    Path(PathObjectIR),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseObjectIR {
    pub id: String,
    pub page_index: usize,
    pub bbox: [f32; 4],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextObjectIR {
    #[serde(flatten)]
    pub base: BaseObjectIR,
    pub tm: PdfMatrix,
    pub unicode: String,
    pub font: TextFont,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextFont {
    pub res_name: String,
    pub size: f32,
    #[serde(rename = "type")]
    pub type_: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageObjectIR {
    #[serde(flatten)]
    pub base: BaseObjectIR,
    pub cm: PdfMatrix,
    pub x_object: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathObjectIR {
    #[serde(flatten)]
    pub base: BaseObjectIR,
    pub cm: PdfMatrix,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op")]
pub enum PatchOperation {
    #[serde(rename = "transform")]
    Transform(TransformPatch),
    #[serde(rename = "editText")]
    EditText(EditTextPatch),
    #[serde(rename = "setStyle")]
    SetStyle(SetStylePatch),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchTarget {
    pub page: usize,
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformPatch {
    pub kind: String,
    pub target: PatchTarget,
    pub delta_matrix_pt: PdfMatrix,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditTextPatch {
    pub target: PatchTarget,
    pub text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub font_pref: Option<FontPreference>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontPreference {
    pub prefer_existing: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fallback_family: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetStylePatch {
    pub target: PatchTarget,
    pub style: StylePatch,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StylePatch {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub opacity_fill: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub opacity_stroke: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fill_color: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stroke_color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchResponse {
    pub ok: bool,
    pub updated_pdf: Option<String>,
}
