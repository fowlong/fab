use anyhow::Context;

use crate::types::{DocumentIr, PageIr};

use super::loader::prepare_document;

pub fn extract_ir(bytes: &[u8]) -> anyhow::Result<DocumentIr> {
    let doc = prepare_document(bytes).context("failed to prepare pdf")?;
    // Placeholder IR with zero objects until real parser is implemented.
    let pages = vec![PageIr {
        index: 0,
        width_pt: 595.276,
        height_pt: 841.89,
        objects: Vec::new(),
    }];
    Ok(DocumentIr { pages })
}
