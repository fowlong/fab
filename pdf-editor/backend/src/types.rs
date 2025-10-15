use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PdfRef {
    pub obj: i64,
    pub gen: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TextGlyph {
    pub gid: i64,
    pub dx: f64,
    pub dy: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TextRun {
    pub id: String,
    pub pdf_ref: PdfRef,
    pub bt_span: BtSpan,
    #[serde(rename = "Tm")]
    pub tm: [f64; 6],
    pub font: TextFont,
    pub unicode: String,
    pub glyphs: Vec<TextGlyph>,
    pub bbox: [f64; 4],
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BtSpan {
    pub start: i64,
    pub end: i64,
    pub stream_obj: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TextFont {
    pub res_name: String,
    pub size: f64,
    #[serde(default)]
    pub r#type: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ImageObject {
    pub id: String,
    pub pdf_ref: PdfRef,
    pub x_object: String,
    #[serde(rename = "cm")]
    pub cm: [f64; 6],
    pub bbox: [f64; 4],
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PathObject {
    pub id: String,
    pub pdf_ref: PdfRef,
    #[serde(rename = "cm")]
    pub cm: [f64; 6],
    pub bbox: [f64; 4],
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum PageObject {
    Text(TextRun),
    Image(ImageObject),
    Path(PathObject),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PageIR {
    pub index: usize,
    pub width_pt: f64,
    pub height_pt: f64,
    pub objects: Vec<PageObject>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DocumentIR {
    pub pages: Vec<PageIR>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "camelCase")]
pub enum PatchOp {
    Transform {
        target: PatchTarget,
        #[serde(rename = "deltaMatrixPt")]
        delta_matrix_pt: [f64; 6],
        kind: String,
    },
    EditText {
        target: PatchTarget,
        text: String,
        font_pref: FontPreference,
    },
    SetStyle {
        target: PatchTarget,
        style: StylePatch,
    },
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
    pub opacity_fill: Option<f64>,
    pub opacity_stroke: Option<f64>,
    pub fill_color: Option<String>,
    pub stroke_color: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PatchResponse {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_pdf: Option<String>,
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub remap: HashMap<String, PatchRemap>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PatchRemap {
    #[serde(rename = "pdfRef")]
    pub pdf_ref: PdfRef,
}
