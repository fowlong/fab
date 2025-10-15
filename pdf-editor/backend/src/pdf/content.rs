//! Lightweight tokenizer placeholder for PDF content streams.
//!
//! The MVP implementation does not yet provide tokenisation logic but
//! establishes the module structure required for future work.

use anyhow::Result;
use lopdf::Document;

#[allow(unused_variables)]
pub fn parse_content_stream(_doc: &Document, _object_id: lopdf::ObjectId) -> Result<()> {
    Ok(())
}
