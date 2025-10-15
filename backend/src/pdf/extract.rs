use anyhow::Result;

use crate::types::{
    DocumentIR, FontInfo, Glyph, ImageObject, PageIR, PageObject, PathObject, PdfRef, Span,
    TextObject,
};

pub fn extract_placeholder_ir() -> Result<DocumentIR> {
    let text = PageObject::Text(TextObject {
        id: "t:1".to_string(),
        pdf_ref: PdfRef { obj: 4, gen: 0 },
        bt_span: Span {
            start: 0,
            end: 0,
            stream_obj: 4,
        },
        tm: [1.0, 0.0, 0.0, 1.0, 72.0, 200.0],
        font: FontInfo {
            res_name: "F1".to_string(),
            size: 18.0,
            font_type: "Type1".to_string(),
        },
        unicode: "Hello PDF".to_string(),
        glyphs: vec![Glyph {
            gid: 40,
            dx: 500.0,
            dy: 0.0,
        }],
        bbox: [72.0, 200.0, 172.0, 218.0],
    });

    let image = PageObject::Image(ImageObject {
        id: "img:1".to_string(),
        pdf_ref: PdfRef { obj: 6, gen: 0 },
        x_object: "Im1".to_string(),
        cm: [50.0, 0.0, 0.0, 50.0, 100.0, 100.0],
        bbox: [100.0, 100.0, 150.0, 150.0],
    });

    let path = PageObject::Path(PathObject {
        id: "path:1".to_string(),
        pdf_ref: PdfRef { obj: 7, gen: 0 },
        cm: [1.0, 0.0, 0.0, 1.0, 50.0, 50.0],
        bbox: [50.0, 50.0, 120.0, 120.0],
    });

    Ok(DocumentIR {
        pages: vec![PageIR {
            index: 0,
            width_pt: 300.0,
            height_pt: 300.0,
            objects: vec![text, image, path],
        }],
    })
}
