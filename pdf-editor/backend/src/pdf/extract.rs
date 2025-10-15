use anyhow::Result;
use crate::types::{DocumentIr, Page, PageObject, TextObject};

pub fn extract_ir(_data: &[u8]) -> Result<DocumentIr> {
  // Placeholder extraction returns a single empty page so the frontend can render a stub.
  let page = Page {
    index: 0,
    width_pt: 595.276,
    height_pt: 841.89,
    objects: vec![PageObject::Text(TextObject::placeholder())],
  };
  Ok(DocumentIr { pages: vec![page] })
}

pub trait Placeholder {
  fn placeholder() -> Self;
}

impl Placeholder for TextObject {
  fn placeholder() -> Self {
    TextObject {
      id: "t:0".into(),
      pdf_ref: crate::types::PdfRef { obj: 1, gen: 0 },
      bt_span: crate::types::BtSpan {
        start: 0,
        end: 0,
        stream_obj: 1,
      },
      tm: [1.0, 0.0, 0.0, 1.0, 100.0, 700.0],
      font: crate::types::FontInfo {
        res_name: "F1".into(),
        size: 12.0,
        font_type: "Type1".into(),
      },
      unicode: "Placeholder".into(),
      glyphs: vec![],
      bbox: [100.0, 690.0, 200.0, 710.0],
    }
  }
}
