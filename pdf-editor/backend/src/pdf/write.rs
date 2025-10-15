//! Incremental PDF writing utilities.
//!
//! The MVP defers implementing full incremental updates. The helpers are placeholders
//! that will eventually take modified object streams and append them to the original
//! document.

use anyhow::Result;
use lopdf::Document;

pub fn save_incremental(_doc: &mut Document, original_bytes: &[u8]) -> Result<Vec<u8>> {
    // Until incremental updates are implemented, return the original bytes so the
    // frontend can continue rendering previews.
    Ok(original_bytes.to_vec())
}
