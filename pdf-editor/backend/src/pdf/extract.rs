use anyhow::Result;

use crate::types::{DocumentIr, PageIr};

pub fn extract_ir(_bytes: &[u8]) -> Result<DocumentIr> {
    // TODO: parse PDF objects into intermediate representation
    Ok(DocumentIr { pages: Vec::new() })
}

pub fn placeholder_page() -> PageIr {
    PageIr {
        index: 0,
        width_pt: 595.0,
        height_pt: 842.0,
        objects: Vec::new(),
    }
}
