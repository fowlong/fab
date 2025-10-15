use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct DocumentIR {
    pub pages: Vec<PageIR>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PageIR {
    pub index: usize,
    pub widthPt: f64,
    pub heightPt: f64,
    pub objects: Vec<PdfObject>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum PdfObject {
    #[serde(rename = "text")]
    Text(TextObject),
    #[serde(rename = "image")]
    Image(ImageObject),
    #[serde(rename = "path")]
    Path(PathObject),
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PdfRef {
    pub obj: u32,
    pub gen: u16,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct BtSpan {
    pub start: usize,
    pub end: usize,
    pub streamObj: u32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct FontInfo {
    pub resName: String,
    pub size: f64,
    #[serde(rename = "type")]
    pub r#type: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct TextGlyph {
    pub gid: u32,
    pub dx: f64,
    pub dy: f64,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct TextObject {
    pub id: String,
    pub pdfRef: PdfRef,
    pub btSpan: BtSpan,
    pub Tm: [f64; 6],
    pub font: FontInfo,
    pub unicode: String,
    pub glyphs: Vec<TextGlyph>,
    pub bbox: [f64; 4],
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ImageObject {
    pub id: String,
    pub pdfRef: PdfRef,
    pub xObject: String,
    pub cm: [f64; 6],
    pub bbox: [f64; 4],
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PathObject {
    pub id: String,
    pub pdfRef: PdfRef,
    pub bbox: [f64; 4],
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "op")]
pub enum PatchOperation {
    #[serde(rename = "transform")]
    Transform {
        target: PatchTarget,
        deltaMatrixPt: [f64; 6],
        kind: String,
    },
    #[serde(rename = "editText")]
    EditText {
        target: PatchTarget,
        text: String,
    },
    #[serde(rename = "setStyle")]
    SetStyle {
        target: PatchTarget,
        style: serde_json::Value,
    },
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PatchTarget {
    pub page: usize,
    pub id: String,
}
