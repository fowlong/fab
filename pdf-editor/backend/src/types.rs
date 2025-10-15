use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfRef {
    pub obj: u32,
    pub gen: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum IrObject {
    #[serde(rename = "text")]
    Text {
        id: String,
        pdf_ref: PdfRef,
        #[serde(rename = "btSpan")]
        bt_span: BtSpan,
        #[serde(rename = "Tm")]
        tm: [f32; 6],
        font: FontInfo,
        unicode: String,
        glyphs: Vec<Glyph>,
        bbox: [f32; 4],
    },
    #[serde(rename = "image")]
    Image {
        id: String,
        pdf_ref: PdfRef,
        #[serde(rename = "xObject")]
        x_object: String,
        #[serde(rename = "cm")]
        cm: [f32; 6],
        bbox: [f32; 4],
    },
    #[serde(rename = "path")]
    Path {
        id: String,
        pdf_ref: PdfRef,
        #[serde(rename = "cm")]
        cm: [f32; 6],
        bbox: [f32; 4],
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BtSpan {
    pub start: usize,
    pub end: usize,
    #[serde(rename = "streamObj")]
    pub stream_obj: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontInfo {
    #[serde(rename = "resName")]
    pub res_name: String,
    pub size: f32,
    #[serde(rename = "type")]
    pub font_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Glyph {
    pub gid: u16,
    pub dx: f32,
    pub dy: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IrPage {
    pub index: usize,
    #[serde(rename = "widthPt")]
    pub width_pt: f32,
    #[serde(rename = "heightPt")]
    pub height_pt: f32,
    pub objects: Vec<IrObject>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IrDocument {
    pub pages: Vec<IrPage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "camelCase")]
pub enum PatchOperation {
    Transform {
        target: PatchTarget,
        #[serde(rename = "kind")]
        kind: String,
        #[serde(rename = "deltaMatrixPt")]
        delta_matrix_pt: [f32; 6],
    },
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
    pub updated_pdf: Option<String>,
}
