use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentMeta {
    pub file_name: String,
    pub page_count: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub original_pdf: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentIR {
    pub document_meta: DocumentMeta,
    pub pages: Vec<PageIR>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageIR {
    pub index: usize,
    pub width_pt: f32,
    pub height_pt: f32,
    pub objects: Vec<PageObject>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum PageObject {
    #[serde(rename = "text")]
    Text(TextObject),
    #[serde(rename = "image")]
    Image(ImageObject),
    #[serde(rename = "path")]
    Path(PathObject),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfRef {
    pub obj: u32,
    pub gen: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextObject {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pdf_ref: Option<PdfRef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unicode: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tm: Option<[f32; 6]>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub font: Option<TextFont>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub glyphs: Vec<TextGlyph>,
    pub bbox: [f32; 4],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextFont {
    pub res_name: String,
    pub size: f32,
    #[serde(rename = "type")]
    pub kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextGlyph {
    pub gid: u32,
    pub dx: f32,
    pub dy: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageObject {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pdf_ref: Option<PdfRef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub x_object: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cm: Option<[f32; 6]>,
    pub bbox: [f32; 4],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PathObject {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pdf_ref: Option<PdfRef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cm: Option<[f32; 6]>,
    pub bbox: [f32; 4],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PatchTarget {
    pub page: usize,
    pub id: String,
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
#[serde(rename_all = "camelCase")]
pub struct TransformPatch {
    pub target: PatchTarget,
    #[serde(rename = "kind")]
    pub object_kind: String,
    pub delta_matrix_pt: [f32; 6],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditTextPatch {
    pub target: PatchTarget,
    pub text: String,
    #[serde(default)]
    pub font_pref: FontPreference,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct FontPreference {
    #[serde(default)]
    pub prefer_existing: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fallback_family: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetStylePatch {
    pub target: PatchTarget,
    pub style: StyleDefinition,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct StyleDefinition {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub colour_fill: Option<[f32; 3]>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub opacity_fill: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PatchResponse {
    pub ok: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_pdf: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remap: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenResponse {
    pub doc_id: String,
}
