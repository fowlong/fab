use anyhow::{Context, Result};
use lopdf::Document;

use crate::types::{DocumentIr, PageIr};

/// Extract a lightweight intermediate representation from the provided PDF bytes.
pub fn extract_ir(bytes: &[u8]) -> Result<DocumentIr> {
    let doc = Document::load_mem(bytes).context("failed to parse PDF")?;
    let mut pages = Vec::new();
    for (index, (_page_number, object_id)) in doc.get_pages().into_iter().enumerate() {
        let page_dict = doc.get_dictionary(object_id).context("missing page dictionary")?;
        let media_box = page_dict
            .get(b"MediaBox")
            .and_then(|obj| doc.deref_object(obj).ok())
            .and_then(|obj| obj.as_array())
            .cloned()
            .unwrap_or_else(|| vec![0.into(), 0.into(), 595.into(), 842.into()]);
        let width = media_box
            .get(2)
            .and_then(|obj| obj.as_f64())
            .unwrap_or(595.0);
        let height = media_box
            .get(3)
            .and_then(|obj| obj.as_f64())
            .unwrap_or(842.0);
        pages.push(PageIr {
            index,
            width_pt: width,
            height_pt: height,
            objects: Vec::new(),
        });
    }
    Ok(DocumentIr { pages })
}
