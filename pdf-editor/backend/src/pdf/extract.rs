use anyhow::Result;
use lopdf::Document;

use crate::types::DocumentIr;

/// Derive the intermediate representation (IR) for a PDF document.
///
/// The current implementation returns an empty IR placeholder. The structure and
/// documentation of this module provide the scaffolding for the future
/// extraction pipeline, which will iterate through page content streams and
/// register text runs, images, and vector paths.
pub fn extract_ir(_document: &Document) -> Result<DocumentIr> {
    Ok(DocumentIr::empty())
}
