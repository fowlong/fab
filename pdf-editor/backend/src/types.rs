use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PdfRef {
    pub obj: u32,
    pub gen: u16,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum IrObject {
    Text(TextObject),
    Image(ImageObject),
    Path(PathObject),
}

impl IrObject {
    pub fn id(&self) -> &str {
        match self {
            IrObject::Text(obj) => &obj.id,
            IrObject::Image(obj) => &obj.id,
            IrObject::Path(obj) => &obj.id,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextObject {
    pub id: String,
    pub pdf_ref: PdfRef,
    pub bt_span: Span,
    pub tm: [f32; 6],
    pub font: TextFont,
    pub unicode: String,
    pub bbox: [f32; 4],
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageObject {
    pub id: String,
    pub pdf_ref: PdfRef,
    pub x_object: String,
    pub cm: [f32; 6],
    pub bbox: [f32; 4],
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PathObject {
    pub id: String,
    pub pdf_ref: PdfRef,
    pub cm: [f32; 6],
    pub bbox: [f32; 4],
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextFont {
    pub res_name: String,
    pub size: f32,
    #[serde(rename = "type")]
    pub type_: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Span {
    pub start: u64,
    pub end: u64,
    pub stream_obj: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PageIR {
    pub index: usize,
    pub width_pt: f32,
    pub height_pt: f32,
    pub objects: Vec<IrObject>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct DocumentIR {
    pub pages: Vec<PageIR>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "camelCase")]
pub enum PatchOp {
    Transform(TransformPatch),
    EditText(EditTextPatch),
    SetStyle(SetStylePatch),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformPatch {
    pub target: PatchTarget,
    pub delta_matrix_pt: [f32; 6],
    pub kind: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditTextPatch {
    pub target: PatchTarget,
    pub text: String,
    pub font_pref: FontPreference,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetStylePatch {
    pub target: PatchTarget,
    pub style: StylePatch,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PatchTarget {
    pub page: usize,
    pub id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FontPreference {
    pub prefer_existing: bool,
    pub fallback_family: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct StylePatch {
    pub opacity_fill: Option<f32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PatchResponse {
    pub ok: bool,
    pub updated_pdf: Option<String>,
    pub remap: Option<serde_json::Value>,
}
