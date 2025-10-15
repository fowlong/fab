use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct PdfRef {
    pub obj: i64,
    pub gen: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TextGlyph {
    pub gid: i64,
    pub dx: f64,
    pub dy: f64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum IrObject {
    #[serde(rename = "text")]
    Text {
        id: String,
        pdf_ref: PdfRef,
        bt_span: BtSpan,
        #[serde(rename = "Tm")]
        tm: [f64; 6],
        font: FontInfo,
        unicode: String,
        glyphs: Vec<TextGlyph>,
        bbox: [f64; 4],
    },
    #[serde(rename = "image")]
    Image {
        id: String,
        pdf_ref: PdfRef,
        x_object: String,
        #[serde(rename = "cm")]
        cm: [f64; 6],
        bbox: [f64; 4],
    },
    #[serde(rename = "path")]
    Path {
        id: String,
        pdf_ref: PdfRef,
        bbox: [f64; 4],
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BtSpan {
    pub start: i64,
    pub end: i64,
    pub stream_obj: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FontInfo {
    pub res_name: String,
    pub size: f64,
    pub r#type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PageIR {
    pub index: usize,
    pub width_pt: f64,
    pub height_pt: f64,
    pub objects: Vec<IrObject>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DocumentIR {
    pub pages: Vec<PageIR>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenResponse {
    pub doc_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PatchResponse {
    pub ok: bool,
    #[serde(rename = "updatedPdf")]
    pub updated_pdf: Option<String>,
    pub remap: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "op")]
pub enum PatchOp {
    #[serde(rename = "transform")]
    Transform {
        target: PatchTarget,
        #[serde(rename = "deltaMatrixPt")]
        delta_matrix_pt: [f64; 6],
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

#[derive(Debug, Serialize, Deserialize)]
pub struct PatchTarget {
    pub page: usize,
    pub id: String,
}
