//! Utilities for loading and caching PDFs.

use anyhow::Result;
use lopdf::Document;

/// Placeholder loader that simply parses bytes into a `lopdf::Document`.
/// In the future this module will provide caching and sanitisation.
pub fn parse_document(bytes: &[u8]) -> Result<Document> {
    let mut doc = Document::load_mem(bytes)?;
    doc.version = "1.7".to_string();
    Ok(doc)
}
