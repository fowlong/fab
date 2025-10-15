use anyhow::Context;
use lopdf::Document;

use crate::types::{DocumentIR, PageIR};

pub fn extract_document(bytes: &[u8]) -> anyhow::Result<DocumentIR> {
    let doc = Document::load_mem(bytes).context("failed to parse PDF")?;
    let mut pages = Vec::new();

    for (index, _page_id) in doc.get_pages().keys().enumerate() {
        let (width, height) = (595.0, 842.0);

        pages.push(PageIR {
            index,
            width_pt: width,
            height_pt: height,
            objects: Vec::new(),
        });
    }

    Ok(DocumentIR { pages })
}
