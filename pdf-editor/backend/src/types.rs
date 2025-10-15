use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfRef {
    pub obj: i64,
    pub gen: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum IrObject {
    Text(TextRun),
    Image(ImageObject),
    Path(PathObject),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextRun {
    pub id: String,
    #[serde(flatten)]
    pub base: ObjectBase,
    pub bt_span: BtSpan,
    pub tm: [f64; 6],
    pub font: TextFont,
    pub unicode: String,
    pub glyphs: Vec<GlyphAdvance>,
    pub bbox: [f64; 4],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageObject {
    pub id: String,
    #[serde(flatten)]
    pub base: ObjectBase,
    pub x_object: String,
    pub cm: [f64; 6],
    pub bbox: [f64; 4],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathObject {
    pub id: String,
    #[serde(flatten)]
    pub base: ObjectBase,
    pub cm: [f64; 6],
    pub bbox: [f64; 4],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectBase {
    pub pdf_ref: PdfRef,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BtSpan {
    pub start: i64,
    pub end: i64,
    pub stream_obj: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextFont {
    pub res_name: String,
    pub size: f64,
    #[serde(rename = "type")]
    pub font_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlyphAdvance {
    pub gid: i64,
    pub dx: f64,
    pub dy: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageIr {
    pub index: usize,
    pub width_pt: f64,
    pub height_pt: f64,
    pub objects: Vec<IrObject>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentIr {
    pub pages: Vec<PageIr>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "camelCase")]
pub enum PatchOperation {
    Transform {
        target: PatchTarget,
        delta_matrix_pt: [f64; 6],
        kind: String,
    },
    EditText {
        target: PatchTarget,
        text: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        font_pref: Option<FontPreference>,
    },
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
pub struct FontPreference {
    #[serde(default)]
    pub prefer_existing: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fallback_family: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchResponse {
    pub ok: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_pdf: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remap: Option<serde_json::Map<String, serde_json::Value>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}
