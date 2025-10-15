use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PdfRef {
    pub obj: u32,
    pub gen: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextGlyph {
    pub gid: u32,
    pub dx: f64,
    pub dy: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextObject {
    pub id: String,
    pub pdf_ref: PdfRef,
    pub bt_span: Span,
    #[serde(rename = "Tm")]
    pub tm: [f64; 6],
    pub font: FontInfo,
    pub unicode: String,
    pub glyphs: Vec<TextGlyph>,
    pub bbox: [f64; 4],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageObject {
    pub id: String,
    pub pdf_ref: PdfRef,
    #[serde(rename = "xObject")]
    pub x_object: String,
    #[serde(rename = "cm")]
    pub cm: [f64; 6],
    pub bbox: [f64; 4],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PathObject {
    pub id: String,
    pub pdf_ref: PdfRef,
    pub operations: Vec<String>,
    #[serde(rename = "cm")]
    pub cm: [f64; 6],
    pub bbox: [f64; 4],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Span {
    pub start: u64,
    pub end: u64,
    pub stream_obj: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FontInfo {
    pub res_name: String,
    pub size: f64,
    #[serde(rename = "type")]
    pub font_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageIR {
    pub index: usize,
    pub width_pt: f64,
    pub height_pt: f64,
    pub objects: Vec<PageObject>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentIR {
    pub pages: Vec<PageIR>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PatchTarget {
    pub page: usize,
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "camelCase")]
pub enum PatchOperation {
    #[serde(rename_all = "camelCase")]
    Transform {
        target: PatchTarget,
        #[serde(rename = "deltaMatrixPt")]
        delta_matrix_pt: [f64; 6],
        kind: String,
    },
    #[serde(rename_all = "camelCase")]
    EditText {
        target: PatchTarget,
        text: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        font_pref: Option<FontPreference>,
    },
    #[serde(rename_all = "camelCase")]
    SetStyle {
        target: PatchTarget,
        style: StylePayload,
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct StylePayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fill_color: Option<[f64; 3]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stroke_color: Option<[f64; 3]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opacity_fill: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opacity_stroke: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PatchResponse {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_pdf: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remap: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

impl DocumentIR {
    pub fn sample() -> Self {
        Self {
            pages: vec![PageIR {
                index: 0,
                width_pt: 595.276,
                height_pt: 841.89,
                objects: vec![
                    PageObject::Text(TextObject {
                        id: "t:42".into(),
                        pdf_ref: PdfRef { obj: 187, gen: 0 },
                        bt_span: Span {
                            start: 12_034,
                            end: 12_345,
                            stream_obj: 155,
                        },
                        tm: [1.0, 0.0, 0.0, 1.0, 100.2, 700.5],
                        font: FontInfo {
                            res_name: "F2".into(),
                            size: 10.5,
                            font_type: "Type0".into(),
                        },
                        unicode: "Invoice #01234".into(),
                        glyphs: vec![
                            TextGlyph {
                                gid: 123,
                                dx: 500.0,
                                dy: 0.0,
                            },
                            TextGlyph {
                                gid: 87,
                                dx: 480.0,
                                dy: 0.0,
                            },
                        ],
                        bbox: [98.4, 688.0, 210.0, 705.0],
                    }),
                    PageObject::Image(ImageObject {
                        id: "img:9".into(),
                        pdf_ref: PdfRef { obj: 200, gen: 0 },
                        x_object: "Im7".into(),
                        cm: [120.0, 0.0, 0.0, 90.0, 300.0, 500.0],
                        bbox: [300.0, 500.0, 420.0, 590.0],
                    }),
                ],
            }],
        }
    }
}
