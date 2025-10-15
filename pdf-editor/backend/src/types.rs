use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};

#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PdfRef {
    #[serde_as(as = "DisplayFromStr")]
    pub obj: u32,
    #[serde_as(as = "DisplayFromStr")]
    pub gen: u16,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BtSpan {
    pub start: usize,
    pub end: usize,
    pub stream_obj: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FontInfo {
    pub res_name: String,
    pub size: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub r#type: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GlyphInfo {
    pub gid: u32,
    #[serde(default)]
    pub dx: f32,
    #[serde(default)]
    pub dy: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum IrObjectData {
    Text {
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(default)]
        bt_span: Option<BtSpan>,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(default)]
        tm: Option<[f32; 6]>,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(default)]
        font: Option<FontInfo>,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(default)]
        unicode: Option<String>,
        #[serde(skip_serializing_if = "Vec::is_empty", default)]
        glyphs: Vec<GlyphInfo>,
    },
    Image {
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(default)]
        x_object: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(default)]
        cm: Option<[f32; 6]>,
    },
    Path {
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(default)]
        cm: Option<[f32; 6]>,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IrObject {
    pub id: String,
    pub pdf_ref: PdfRef,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub bbox: Option<[f32; 4]>,
    #[serde(flatten)]
    pub data: IrObjectData,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PageIr {
    pub index: usize,
    #[serde(rename = "widthPt")]
    pub width_pt: f32,
    #[serde(rename = "heightPt")]
    pub height_pt: f32,
    pub objects: Vec<IrObject>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DocumentIr {
    pub pages: Vec<PageIr>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FontPreference {
    #[serde(default)]
    pub prefer_existing: bool,
    #[serde(default)]
    pub fallback_family: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StyleUpdate {
    #[serde(default)]
    pub fill: Option<[f32; 3]>,
    #[serde(default)]
    pub stroke: Option<[f32; 3]>,
    #[serde(default)]
    pub opacity_fill: Option<f32>,
    #[serde(default)]
    pub opacity_stroke: Option<f32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "camelCase")]
pub enum PatchOperation {
    Transform {
        target: PatchTarget,
        #[serde(rename = "deltaMatrixPt")]
        delta_matrix_pt: [f32; 6],
        kind: PatchKind,
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

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PatchTarget {
    pub page: usize,
    pub id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PatchKind {
    Text,
    Image,
    Path,
}
