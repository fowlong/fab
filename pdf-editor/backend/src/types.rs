use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PdfRef {
    pub obj: u32,
    pub gen: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpanInfo {
    pub start: usize,
    pub end: usize,
    #[serde(rename = "streamObj")]
    pub stream_obj: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FontInfo {
    #[serde(rename = "resName")]
    pub res_name: String,
    pub size: f64,
    #[serde(rename = "type")]
    pub r#type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Glyph {
    pub gid: u32,
    pub dx: f64,
    pub dy: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageIr {
    pub index: usize,
    pub width_pt: f64,
    pub height_pt: f64,
    pub objects: Vec<IrObject>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentIr {
    pub pages: Vec<PageIr>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum IrObject {
    #[serde(rename_all = "camelCase")]
    #[serde(rename = "text")]
    Text {
        id: String,
        #[serde(rename = "pdfRef")]
        pdf_ref: PdfRef,
        #[serde(rename = "btSpan")]
        bt_span: SpanInfo,
        #[serde(rename = "Tm")]
        tm: [f64; 6],
        font: FontInfo,
        unicode: String,
        glyphs: Vec<Glyph>,
        bbox: [f64; 4],
    },
    #[serde(rename_all = "camelCase")]
    #[serde(rename = "image")]
    Image {
        id: String,
        #[serde(rename = "pdfRef")]
        pdf_ref: PdfRef,
        #[serde(rename = "xObject")]
        x_object: String,
        cm: [f64; 6],
        bbox: [f64; 4],
    },
    #[serde(rename_all = "camelCase")]
    #[serde(rename = "path")]
    Path {
        id: String,
        #[serde(rename = "pdfRef")]
        pdf_ref: PdfRef,
        cm: [f64; 6],
        bbox: [f64; 4],
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PatchTarget {
    pub page: usize,
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FontPreference {
    pub prefer_existing: bool,
    pub fallback_family: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct StylePatch {
    pub opacity_fill: Option<f64>,
    pub opacity_stroke: Option<f64>,
    pub fill_color_rgb: Option<[f64; 3]>,
    pub stroke_color_rgb: Option<[f64; 3]>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "camelCase")]
pub enum PatchOperation {
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
        font_pref: FontPreference,
    },
    #[serde(rename = "setStyle")]
    SetStyle {
        target: PatchTarget,
        style: StylePatch,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PatchResponse {
    pub ok: bool,
    pub updated_pdf: Option<String>,
    pub remap: Option<serde_json::Value>,
    pub error: Option<String>,
}
