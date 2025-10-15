use anyhow::Context;
use lopdf::Document;

use crate::types::{DocumentIR, PageIR};

pub fn extract_ir(bytes: &[u8]) -> anyhow::Result<DocumentIR> {
    let doc = Document::load_mem(bytes).context("failed to parse PDF")?;
    let page_count = doc.get_pages().len().max(1);

    let mut pages = Vec::new();
    for (index, _) in doc.get_pages() {
        pages.push(PageIR {
            index: pages.len(),
            widthPt: 595.0,
            heightPt: 842.0,
            objects: Vec::new(),
        });
        if pages.len() >= page_count {
            break;
        }
    }

    if pages.is_empty() {
        pages.push(PageIR {
            index: 0,
            widthPt: 595.0,
            heightPt: 842.0,
            objects: Vec::new(),
        });
    }

    Ok(DocumentIR { pages })
}
