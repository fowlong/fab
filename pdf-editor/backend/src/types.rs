use serde::{Deserialize, Serialize};

pub type DocumentId = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentIr {
    pub pages: Vec<PageIr>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<DocumentMeta>,
    #[serde(skip)]
    pub last_patch: Option<PatchOperations>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentMeta {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pdf_data: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_count: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageIr {
    pub index: usize,
    pub width_pt: f32,
    pub height_pt: f32,
    pub objects: Vec<PdfObject>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum PdfObject {
    Text(TextObject),
    Image(ImageObject),
    Path(PathObject),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextObject {
    pub id: String,
    pub bbox: [f32; 4],
    #[serde(rename = "pdfRef")]
    pub pdf_ref: PdfRef,
    #[serde(rename = "btSpan")]
    pub bt_span: Span,
    #[serde(rename = "Tm")]
    pub tm: Matrix,
    pub font: FontInfo,
    pub unicode: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageObject {
    pub id: String,
    pub bbox: [f32; 4],
    #[serde(rename = "pdfRef")]
    pub pdf_ref: PdfRef,
    #[serde(rename = "cm")]
    pub cm: Matrix,
    #[serde(rename = "xObject")]
    pub x_object: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PathObject {
    pub id: String,
    pub bbox: [f32; 4],
    #[serde(rename = "pdfRef")]
    pub pdf_ref: PdfRef,
    #[serde(rename = "cm")]
    pub cm: Matrix,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FontInfo {
    pub res_name: String,
    pub size: f32,
    #[serde(rename = "type")]
    pub font_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Span {
    pub start: usize,
    pub end: usize,
    #[serde(rename = "streamObj")]
    pub stream_obj: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PdfRef {
    pub obj: usize,
    #[serde(rename = "gen")]
    pub generation: usize,
}

pub type Matrix = [f32; 6];
pub type PatchOperations = Vec<PatchOp>;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op")]
pub enum PatchOp {
    #[serde(rename = "transform")]
    Transform {
        target: PatchTarget,
        #[serde(rename = "deltaMatrixPt")]
        delta_matrix_pt: Matrix,
        kind: PatchKind,
    },
    #[serde(rename = "editText")]
    EditText {
        target: PatchTarget,
        text: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        font_pref: Option<FontPreference>,
    },
    #[serde(rename = "setStyle")]
    SetStyle {
        target: PatchTarget,
        style: serde_json::Value,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FontPreference {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prefer_existing: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fallback_family: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchTarget {
    pub page: usize,
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PatchKind {
    Text,
    Image,
    Path,
}

impl DocumentIr {
    pub fn sample() -> Self {
        Self {
            pages: vec![PageIr {
                index: 0,
                width_pt: 595.0,
                height_pt: 842.0,
                objects: vec![PdfObject::Text(TextObject {
                    id: "t:1".into(),
                    bbox: [72.0, 720.0, 220.0, 745.0],
                    pdf_ref: PdfRef { obj: 4, generation: 0 },
                    bt_span: Span {
                        start: 0,
                        end: 0,
                        stream_obj: 4,
                    },
                    tm: [1.0, 0.0, 0.0, 1.0, 72.0, 730.0],
                    font: FontInfo {
                        res_name: "F1".into(),
                        size: 12.0,
                        font_type: "Type1".into(),
                    },
                    unicode: "Sample PDF".into(),
                })],
            }],
            meta: Some(DocumentMeta {
                pdf_data: Some("data:application/pdf;base64,JVBERi0xLjQKMSAwIG9iajw8Pj5lbmRvYmoKMiAwIG9iajw8IC9UeXBlIC9QYWdlcyAvS2lkcyBbMyAwIFJdIC9Db3VudCAxID4+ZW5kb2JqCjMgMCBvYmo8PCAvVHlwZSAvUGFnZSAvUGFyZW50IDIgMCBSIC9NZWRpYUJveCBbMCAwIDU5NSA4NDJdIC9Db250ZW50cyA0IDAgUiA+PmVuZG9iago0IDAgb2JqPDwgL0xlbmd0aCAzNSA+PnN0cmVhbQpCVCAvRjEgMTIgVGYgNzIgNzcwIFRkIChTYW1wbGUgUERGKSBUaiBFVAplbmRzdHJlYW0KZW5kb2JqCjUgMCBvYmo8PCAvVHlwZSAvQ2F0YWxvZyAvUGFnZXMgMiAwIFIgPj5lbmRvYmoKNiAwIG9iajw8IC9UeXBlIC9Gb250IC9TdWJ0eXBlIC9UeXBlMSAvQmFzZUZvbnQgL0hlbHZldGljYSA+PmVuZG9iagp4cmVmCjAgNwowMDAwMDAwMDAwIDY1NTM1IGYgCjAwMDAwMDAwMTAgMDAwMDAgbiAKMDAwMDAwMDA1MyAwMDAwMCBuIAowMDAwMDAwMTE0IDAwMDAwIG4gCjAwMDAwMDAxOTggMDAwMDAgbiAKMDAwMDAwMDMwMiAwMDAwMCBuIAowMDAwMDAwMzYyIDAwMDAwIG4gCnRyYWlsZXI8PCAvU2l6ZSA3IC9Sb290IDUgMCBSIC9JbmZvIDEgMCBSID4+CnN0YXJ0eHJlZgo0MjAKJSVFT0Y=".into()),
                page_count: Some(1),
            }),
            last_patch: None,
        }
    }
}
