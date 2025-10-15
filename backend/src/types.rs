use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentIr {
    pub pages: Vec<PageIr>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageIr {
    pub index: usize,
    #[serde(rename = "widthPt")]
    pub width_pt: f32,
    #[serde(rename = "heightPt")]
    pub height_pt: f32,
    pub objects: Vec<IrObject>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum IrObject {
    #[serde(rename = "text")]
    Text(TextObject),
    #[serde(rename = "image")]
    Image(ImageObject),
    #[serde(rename = "path")]
    Path(PathObject),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseObject {
    pub id: String,
    pub bbox: Option<[f32; 4]>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextObject {
    #[serde(flatten)]
    pub base: BaseObject,
    #[serde(rename = "Tm")]
    pub tm: [f32; 6],
    pub font: PdfFont,
    pub unicode: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageObject {
    #[serde(flatten)]
    pub base: BaseObject,
    #[serde(rename = "cm")]
    pub cm: [f32; 6],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathObject {
    #[serde(flatten)]
    pub base: BaseObject,
    #[serde(rename = "cm")]
    pub cm: [f32; 6],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfFont {
    #[serde(rename = "resName")]
    pub res_name: String,
    pub size: f32,
    #[serde(rename = "type")]
    pub r#type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "camelCase")]
pub enum PatchOperation {
    Transform(TransformOp),
    EditText(EditTextOp),
    SetStyle(SetStyleOp),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformOp {
    pub kind: String,
    pub target: PatchTarget,
    #[serde(rename = "deltaMatrixPt")]
    pub delta_matrix_pt: [f32; 6],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditTextOp {
    pub target: PatchTarget,
    pub text: String,
    #[serde(default)]
    pub font_pref: Option<FontPreference>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontPreference {
    #[serde(default)]
    pub prefer_existing: Option<bool>,
    #[serde(default)]
    pub fallback_family: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetStyleOp {
    pub target: PatchTarget,
    pub style: serde_json::Value,
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
    pub updated_pdf: String,
    #[serde(default)]
    pub remap: Option<serde_json::Value>,
}
