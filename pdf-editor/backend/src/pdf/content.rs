//! Minimal placeholder for a PDF content stream parser.

use anyhow::Result;

#[derive(Debug, Clone)]
pub struct ContentStream {
    pub raw: Vec<u8>,
}

impl ContentStream {
    pub fn new(raw: Vec<u8>) -> Self {
        Self { raw }
    }
}

pub fn parse_stream(bytes: &[u8]) -> Result<ContentStream> {
    Ok(ContentStream::new(bytes.to_vec()))
}

pub fn rewrite_stream(stream: &ContentStream) -> Result<Vec<u8>> {
    Ok(stream.raw.clone())
}
