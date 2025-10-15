//! Placeholder extraction routines that expose a canned IR.

use crate::types::{BtSpan, DocumentIr, FontInfo, GlyphInfo, IrObject, IrObjectData, PageIr, PdfRef};

/// Returns a deterministic demo IR that mirrors the design brief.
#[must_use]
pub fn demo_ir() -> DocumentIr {
    DocumentIr {
        pages: vec![PageIr {
            index: 0,
            width_pt: 595.276,
            height_pt: 841.89,
            objects: vec![
                IrObject {
                    id: "t:42".to_string(),
                    pdf_ref: PdfRef { obj: 187, gen: 0 },
                    bbox: Some([98.4, 688.0, 210.0, 705.0]),
                    data: IrObjectData::Text {
                        bt_span: Some(BtSpan {
                            start: 12034,
                            end: 12345,
                            stream_obj: 155,
                        }),
                        tm: Some([1.0, 0.0, 0.0, 1.0, 100.2, 700.5]),
                        font: Some(FontInfo {
                            res_name: "F2".into(),
                            size: 10.5,
                            r#type: Some("Type0".into()),
                        }),
                        unicode: Some("Invoice #01234".into()),
                        glyphs: vec![GlyphInfo {
                            gid: 123,
                            dx: 500.0,
                            dy: 0.0,
                        }],
                    },
                },
                IrObject {
                    id: "img:9".to_string(),
                    pdf_ref: PdfRef { obj: 200, gen: 0 },
                    bbox: Some([300.0, 500.0, 420.0, 590.0]),
                    data: IrObjectData::Image {
                        x_object: Some("Im7".into()),
                        cm: Some([120.0, 0.0, 0.0, 90.0, 300.0, 500.0]),
                    },
                },
                IrObject {
                    id: "path:3".to_string(),
                    pdf_ref: PdfRef { obj: 250, gen: 0 },
                    bbox: Some([150.0, 400.0, 260.0, 460.0]),
                    data: IrObjectData::Path {
                        cm: Some([1.0, 0.0, 0.0, 1.0, 0.0, 0.0]),
                    },
                },
            ],
        }],
    }
}
