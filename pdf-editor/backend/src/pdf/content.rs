//! Minimal content stream tokeniser placeholder.

use anyhow::Result;

#[derive(Debug, Clone)]
pub struct ContentStream;

impl ContentStream {
    pub fn from_bytes(_bytes: &[u8]) -> Result<Self> {
        Ok(ContentStream)
    }
}
