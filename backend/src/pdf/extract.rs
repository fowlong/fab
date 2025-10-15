use anyhow::Result;

use crate::types::{DocumentIr, PageIr};

/// Build the initial intermediate representation from a PDF file.
///
/// The current implementation is a placeholder that returns a single
/// empty page. This keeps the HTTP contract stable while we wire the
/// remainder of the system. Future work will parse the PDF content
/// streams and populate the `objects` vector accordingly.
pub async fn build_initial_ir(_pdf_bytes: &[u8]) -> Result<DocumentIr> {
    let page = PageIr {
        index: 0,
        width_pt: 595.276,
        height_pt: 841.89,
        objects: Vec::new(),
    };
    Ok(DocumentIr { pages: vec![page] })
}
