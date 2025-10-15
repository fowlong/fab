use anyhow::Context;
use lopdf::{Document as LopdfDocument, Object};

use crate::pdf::loader::Document;
use crate::types::{DocumentIR, PageIR};

pub fn extract_ir(doc_id: &str, doc: &Document) -> anyhow::Result<DocumentIR> {
  let pdf = LopdfDocument::load_mem(&doc.bytes).context("load pdf bytes")?;
  let pages_map = pdf.get_pages();

  let mut pages = Vec::with_capacity(pages_map.len());

  for (index, (_, page_id)) in pages_map.iter().enumerate() {
    let page_dict = pdf
      .get_dictionary(page_id)
      .context("page dictionary")?;

    let media_box = page_dict
      .get(b"MediaBox")
      .or_else(|_| page_dict.get(b"CropBox"))
      .ok()
      .and_then(|obj| match obj {
        Object::Array(arr) if arr.len() == 4 => {
          let llx = arr[0].as_f64().unwrap_or(0.0) as f32;
          let lly = arr[1].as_f64().unwrap_or(0.0) as f32;
          let urx = arr[2].as_f64().unwrap_or(0.0) as f32;
          let ury = arr[3].as_f64().unwrap_or(0.0) as f32;
          Some((urx - llx, ury - lly))
        }
        _ => None,
      })
      .unwrap_or((595.276, 841.89));

    pages.push(PageIR {
      index,
      width_pt: media_box.0,
      height_pt: media_box.1,
      objects: Vec::new(),
    });
  }

  Ok(DocumentIR {
    doc_id: doc_id.to_string(),
    pdf_url: None,
    pages,
  })
}
