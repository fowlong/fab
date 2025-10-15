use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "op")]
pub enum PatchOperation {
    #[serde(rename = "transform")]
    Transform {
        target: PatchTarget,
        #[serde(rename = "deltaMatrixPt")]
        delta_matrix_pt: [f32; 6],
        kind: PatchTransformKind,
    },
    #[serde(rename = "editText")]
    EditText {
        target: PatchTarget,
        text: String,
        #[serde(rename = "fontPref")]
        font_pref: FontPreference,
    },
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PatchTransformKind {
    Text,
    Image,
    Path,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchResponse {
    pub ok: bool,
    #[serde(rename = "updatedPdf", skip_serializing_if = "Option::is_none")]
    pub updated_pdf: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remap: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl PatchResponse {
    pub fn ok() -> Self {
        Self {
            ok: true,
            updated_pdf: None,
            remap: None,
            error: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntermediateRepresentation {
    pub pages: Vec<IrPage>,
}

impl IntermediateRepresentation {
    pub fn empty() -> Self {
        Self { pages: Vec::new() }
    }
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
#[serde(tag = "kind")]
pub enum IrObject {
    #[serde(rename = "text")]
    Text(IrTextObject),
    #[serde(rename = "image")]
    Image(IrImageObject),
    #[serde(rename = "path")]
    Path(IrPathObject),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IrTextObject {
    pub id: String,
    #[serde(rename = "pdfRef")]
    pub pdf_ref: PdfRef,
    #[serde(rename = "btSpan")]
    pub bt_span: Span,
    #[serde(rename = "Tm")]
    pub tm: [f32; 6],
    pub font: FontInfo,
    pub unicode: String,
    pub glyphs: Vec<GlyphInfo>,
    pub bbox: [f32; 4],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IrImageObject {
    pub id: String,
    #[serde(rename = "pdfRef")]
    pub pdf_ref: PdfRef,
    #[serde(rename = "xObject")]
    pub x_object: String,
    #[serde(rename = "cm")]
    pub cm: [f32; 6],
    pub bbox: [f32; 4],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IrPathObject {
    pub id: String,
    #[serde(rename = "pdfRef")]
    pub pdf_ref: PdfRef,
    #[serde(rename = "cm")]
    pub cm: [f32; 6],
    pub bbox: [f32; 4],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlyphInfo {
    pub gid: u32,
    pub dx: f32,
    pub dy: f32,
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
pub struct Span {
    pub start: usize,
    pub end: usize,
    #[serde(rename = "streamObj")]
    pub stream_obj: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfRef {
    pub obj: u32,
    pub gen: u16,
}
