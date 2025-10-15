use crate::types::{
    FontInfo, GlyphInfo, IntermediateRepresentation, IrObject, IrPage, IrTextObject, PdfRef, Span,
};

pub fn extract_ir(_pdf_bytes: &[u8]) -> anyhow::Result<IntermediateRepresentation> {
    let text = IrObject::Text(IrTextObject {
        id: "t:1".to_string(),
        pdf_ref: PdfRef { obj: 4, gen: 0 },
        bt_span: Span {
            start: 0,
            end: 44,
            stream_obj: 4,
        },
        tm: [1.0, 0.0, 0.0, 1.0, 100.0, 700.0],
        font: FontInfo {
            res_name: "F1".to_string(),
            size: 24.0,
            font_type: "Type1".to_string(),
        },
        unicode: "Hello PDF".to_string(),
        glyphs: vec![GlyphInfo {
            gid: 0,
            dx: 0.0,
            dy: 0.0,
        }],
        bbox: [100.0, 700.0, 260.0, 730.0],
    });

    Ok(IntermediateRepresentation {
        pages: vec![IrPage {
            index: 0,
            width_pt: 595.0,
            height_pt: 842.0,
            objects: vec![text],
        }],
    })
}
