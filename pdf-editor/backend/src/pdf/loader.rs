use anyhow::Result;

use crate::types::DocumentIr;

pub fn open_document(_bytes: &[u8]) -> Result<DocumentIr> {
    // Placeholder implementation until full PDF extraction is wired in.
    Ok(DocumentIr::sample())
}
