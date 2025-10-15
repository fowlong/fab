//! Document loading and caching utilities.

use anyhow::Result;

/// Load a document from raw bytes. This is a placeholder helper that simply
/// validates that the bytes represent a well-formed PDF and returns the original
/// buffer.
pub fn load(bytes: &[u8]) -> Result<Vec<u8>> {
    // lopdf will error if the data is not a PDF file.
    lopdf::Document::load_mem(bytes)?;
    Ok(bytes.to_vec())
}
