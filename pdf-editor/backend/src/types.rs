use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfRef {
    pub obj: i32,
    pub gen: i32,
}

pub type PdfObjectId = String;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Glyph {
    pub gid: i32,
    pub dx: f32,
    pub dy: f32,
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

impl PageObject {
    pub fn id(&self) -> &str {
        match self {
            PageObject::Text(obj) => &obj.id,
            PageObject::Image(obj) => &obj.id,
            PageObject::Path(obj) => &obj.id,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextObject {
    pub id: PdfObjectId,
    pub pdfRef: PdfRef,
    pub btSpan: Span,
    #[serde(rename = "Tm")]
    pub tm: [f32; 6],
    pub font: FontInfo,
    pub unicode: String,
    pub glyphs: Vec<Glyph>,
    pub bbox: [f32; 4],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageObject {
    pub id: PdfObjectId,
    pub pdfRef: PdfRef,
    pub xObject: String,
    #[serde(rename = "cm")]
    pub cm: [f32; 6],
    pub bbox: [f32; 4],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathObject {
    pub id: PdfObjectId,
    pub pdfRef: PdfRef,
    #[serde(rename = "cm")]
    pub cm: [f32; 6],
    pub bbox: [f32; 4],
    pub gs: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontInfo {
    pub resName: String,
    pub size: f32,
    #[serde(default)]
    pub r#type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub streamObj: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageIR {
    pub index: usize,
    pub widthPt: f32,
    pub heightPt: f32,
    pub objects: Vec<PageObject>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DocumentIR {
    pub pages: Vec<PageIR>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformTarget {
    pub page: usize,
    pub id: PdfObjectId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op")]
pub enum PatchOperation {
    #[serde(rename = "transform")]
    Transform {
        target: TransformTarget,
        #[serde(rename = "deltaMatrixPt")]
        delta_matrix_pt: [f32; 6],
        kind: String,
    },
    #[serde(rename = "editText")]
    EditText {
        target: TransformTarget,
        text: String,
        #[serde(default)]
        fontPref: Option<FontPreference>,
    },
    #[serde(rename = "setStyle")]
    SetStyle {
        target: TransformTarget,
        style: StyleUpdate,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FontPreference {
    #[serde(default)]
    pub preferExisting: bool,
    #[serde(default)]
    pub fallbackFamily: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StyleUpdate {
    #[serde(default)]
    pub colorFill: Option<[f32; 3]>,
    #[serde(default)]
    pub colorStroke: Option<[f32; 3]>,
    #[serde(default)]
    pub opacityFill: Option<f32>,
    #[serde(default)]
    pub opacityStroke: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PatchResponse {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updatedPdf: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remap: Option<HashMap<String, PdfRefWrapper>>,
}

use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfRefWrapper {
    pub pdfRef: PdfRef,
}
