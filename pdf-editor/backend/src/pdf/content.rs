//! Utilities for working with PDF content streams.
//!
//! The MVP implementation exposes placeholders so that future work can
//! focus on tokenisation and rewriting logic without breaking the API
//! surface consumed by the HTTP handlers.

use anyhow::Result;
use lopdf::Document;

/// Represents a parsed content stream. In the MVP we treat it as an opaque
/// byte vector while the extraction pipeline is still under construction.
#[derive(Debug, Clone)]
pub struct ContentStream {
    pub object_id: u32,
    pub data: Vec<u8>,
}

pub fn load_stream(_doc: &Document, object_id: u32) -> Result<ContentStream> {
    Ok(ContentStream {
        object_id,
        data: Vec::new(),
    })
}

pub fn rewrite_stream(stream: &ContentStream, _replacement: &[u8]) -> Result<ContentStream> {
    Ok(ContentStream {
        object_id: stream.object_id,
        data: stream.data.clone(),
    })
}
