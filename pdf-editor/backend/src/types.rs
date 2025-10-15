use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DocumentIr {
    pub pages: Vec<PageIr>,
}

impl DocumentIr {
    pub fn empty() -> Self {
        Self { pages: Vec::new() }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PageIr {
    pub index: usize,
    pub width_pt: f64,
    pub height_pt: f64,
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
    pub obj: i64,
    pub gen: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextSpan {
    pub start: usize,
    pub end: usize,
    pub stream_obj: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Glyph {
    pub gid: u16,
    pub dx: f32,
    pub dy: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextFont {
    pub res_name: String,
    pub size: f32,
    pub r#type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextObject {
    pub id: String,
    pub pdf_ref: PdfRef,
    pub bt_span: TextSpan,
    pub tm: [f64; 6],
    pub font: TextFont,
    pub unicode: String,
    pub glyphs: Vec<Glyph>,
    pub bbox: [f64; 4],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageObject {
    pub id: String,
    pub pdf_ref: PdfRef,
    pub x_object: String,
    pub cm: [f64; 6],
    pub bbox: [f64; 4],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathObject {
    pub id: String,
    pub pdf_ref: PdfRef,
    pub cm: [f64; 6],
    pub bbox: [f64; 4],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "camelCase")]
pub enum PatchOp {
    Transform {
        target: PatchTarget,
        #[serde(rename = "deltaMatrixPt")]
        delta_matrix_pt: [f64; 6],
        kind: PatchTargetKind,
    },
    EditText {
        target: PatchTarget,
        text: String,
        #[serde(default)]
        font_pref: Option<FontPreference>,
    },
    SetStyle {
        target: PatchTarget,
        style: StyleUpdate,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchTarget {
    pub page: usize,
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PatchTargetKind {
    Text,
    Image,
    Path,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FontPreference {
    #[serde(default)]
    pub prefer_existing: bool,
    #[serde(default)]
    pub fallback_family: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StyleUpdate {
    #[serde(default)]
    pub color_fill: Option<[f32; 3]>,
    #[serde(default)]
    pub color_stroke: Option<[f32; 3]>,
    #[serde(default)]
    pub opacity_fill: Option<f32>,
    #[serde(default)]
    pub opacity_stroke: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchResponse {
    pub ok: bool,
    #[serde(default)]
    pub updated_pdf: Option<String>,
    #[serde(default)]
    pub remap: Option<serde_json::Value>,
    #[serde(default)]
    pub error: Option<String>,
}

impl PatchResponse {
    pub fn noop() -> Self {
        Self {
            ok: true,
            updated_pdf: None,
            remap: None,
            error: None,
        }
    }

    pub fn ack(current_pdf: Vec<u8>) -> Self {
        let encoded = base64::encode(current_pdf);
        Self {
            ok: true,
            updated_pdf: Some(format!("data:application/pdf;base64,{}", encoded)),
            remap: None,
            error: None,
        }
    }
}
