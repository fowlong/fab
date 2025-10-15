use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DocumentIR {
    pub pages: Vec<PageIR>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PageIR {
    pub index: usize,
    pub width_pt: f32,
    pub height_pt: f32,
    pub objects: Vec<PageObject>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "kind")]
pub enum PageObject {
    #[serde(rename = "text")]
    Text(TextObject),
    #[serde(rename = "image")]
    Image(ImageObject),
    #[serde(rename = "path")]
    Path(PathObject),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TextObject {
    pub id: String,
    pub pdf_ref: PdfRef,
    pub bt_span: SpanRef,
    pub tm: [f32; 6],
    pub font: FontDescriptor,
    pub unicode: String,
    pub glyphs: Vec<TextGlyph>,
    pub bbox: [f32; 4],
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ImageObject {
    pub id: String,
    pub pdf_ref: PdfRef,
    pub x_object: String,
    pub cm: [f32; 6],
    pub bbox: [f32; 4],
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PathObject {
    pub id: String,
    pub pdf_ref: PdfRef,
    pub gs: Option<String>,
    pub cm: [f32; 6],
    pub bbox: [f32; 4],
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PdfRef {
    pub obj: u32,
    pub gen: u16,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SpanRef {
    pub start: usize,
    pub end: usize,
    pub stream_obj: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FontDescriptor {
    pub res_name: String,
    pub size: f32,
    pub r#type: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TextGlyph {
    pub gid: u32,
    pub dx: f32,
    pub dy: f32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "op")]
pub enum PatchOperation {
    #[serde(rename = "transform")]
    Transform {
        target: PatchTarget,
        #[serde(rename = "deltaMatrixPt")]
        delta_matrix_pt: [f32; 6],
        kind: String,
    },
    #[serde(rename = "editText")]
    EditText {
        target: PatchTarget,
        text: String,
        #[serde(rename = "fontPref")]
        font_pref: FontPreference,
    },
    #[serde(rename = "setStyle")]
    SetStyle {
        target: PatchTarget,
        style: StylePatch,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PatchTarget {
    pub page: usize,
    pub id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FontPreference {
    #[serde(rename = "preferExisting")]
    pub prefer_existing: bool,
    #[serde(rename = "fallbackFamily")]
    pub fallback_family: String,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct StylePatch {
    #[serde(rename = "opacityFill")]
    pub opacity_fill: Option<f32>,
    #[serde(rename = "opacityStroke")]
    pub opacity_stroke: Option<f32>,
    #[serde(rename = "fillColor")]
    pub fill_color: Option<String>,
    #[serde(rename = "strokeColor")]
    pub stroke_color: Option<String>,
}
