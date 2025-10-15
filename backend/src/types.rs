use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
use uuid::Uuid;

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentHandle {
    #[serde_as(as = "DisplayFromStr")]
    pub doc_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DocumentIr {
    pub pages: Vec<PageIr>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PageIr {
    pub index: usize,
    pub width_pt: f32,
    pub height_pt: f32,
    pub objects: Vec<IrObject>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum IrObject {
    #[serde(rename = "text")]
    Text(TextObject),
    #[serde(rename = "image")]
    Image(ImageObject),
    #[serde(rename = "path")]
    Path(PathObject),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfRef {
    pub obj: u32,
    pub gen: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Matrix(pub [f32; 6]);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundingBox(pub [f32; 4]);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextObject {
    pub id: String,
    #[serde(rename = "pdfRef")]
    pub pdf_ref: PdfRef,
    #[serde(rename = "btSpan")]
    pub bt_span: StreamSpan,
    #[serde(rename = "Tm")]
    pub text_matrix: Matrix,
    pub font: TextFont,
    pub unicode: String,
    pub glyphs: Vec<Glyph>,
    #[serde(rename = "bbox")]
    pub bbox: BoundingBox,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextFont {
    #[serde(rename = "resName")]
    pub resource_name: String,
    pub size: f32,
    #[serde(rename = "type")]
    pub font_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Glyph {
    pub gid: u32,
    pub dx: f32,
    pub dy: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamSpan {
    #[serde(rename = "start")]
    pub start_offset: usize,
    #[serde(rename = "end")]
    pub end_offset: usize,
    #[serde(rename = "streamObj")]
    pub stream_object: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageObject {
    pub id: String,
    #[serde(rename = "pdfRef")]
    pub pdf_ref: PdfRef,
    #[serde(rename = "xObject")]
    pub x_object: String,
    #[serde(rename = "cm")]
    pub transform: Matrix,
    #[serde(rename = "bbox")]
    pub bbox: BoundingBox,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathObject {
    pub id: String,
    #[serde(rename = "pdfRef")]
    pub pdf_ref: PdfRef,
    #[serde(rename = "cm")]
    pub transform: Matrix,
    #[serde(rename = "bbox")]
    pub bbox: BoundingBox,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformTarget {
    pub page: usize,
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformOp {
    #[serde(rename = "deltaMatrixPt")]
    pub delta_matrix_pt: Matrix,
    pub kind: TransformKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TransformKind {
    Text,
    Image,
    Path,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op")]
#[serde(rename_all = "camelCase")]
pub enum PatchOperation {
    Transform {
        target: TransformTarget,
        #[serde(flatten)]
        transform: TransformOp,
    },
    EditText {
        target: TransformTarget,
        text: String,
        #[serde(rename = "fontPref")]
        font_pref: Option<FontPreference>,
    },
    SetStyle {
        target: TransformTarget,
        style: StylePatch,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontPreference {
    #[serde(rename = "preferExisting")]
    pub prefer_existing: bool,
    #[serde(rename = "fallbackFamily")]
    pub fallback_family: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct StylePatch {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fill_color: Option<[f32; 3]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stroke_color: Option<[f32; 3]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opacity_fill: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opacity_stroke: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PatchResponse {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_pdf: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remap: Option<serde_json::Value>,
}
