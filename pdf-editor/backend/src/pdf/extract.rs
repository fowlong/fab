use anyhow::Result;
use lopdf::Document;

use crate::types::{DocumentIR, PageIR};

pub fn extract_ir(bytes: &[u8]) -> Result<DocumentIR> {
    let doc = Document::load_mem(bytes)?;
    let mut pages = Vec::new();
    for (index, page_id) in doc.get_pages().keys().enumerate() {
        let page = doc.get_page_content(*page_id).unwrap_or_default();
        let (_width, _height) = doc.get_page_size(*page_id).unwrap_or((595.0, 842.0));
        pages.push(PageIR {
            index,
            width_pt: _width,
            height_pt: _height,
            objects: Vec::new(),
        });
        let _ = page; // suppress unused warning for placeholder.
    }
    Ok(DocumentIR { pages })
}
