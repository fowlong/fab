//! Minimal placeholder module for future content stream parsing.
//! The MVP scaffolding keeps interfaces ready for a richer implementation.

use anyhow::Result;

#[derive(Debug, Clone, Default)]
pub struct ContentStream {
    pub raw: Vec<u8>,
}

impl ContentStream {
    pub fn new(raw: Vec<u8>) -> Self {
        Self { raw }
    }

    pub fn rewrite(&mut self, new_raw: Vec<u8>) {
        self.raw = new_raw;
    }
}

pub fn parse_stream(data: &[u8]) -> Result<ContentStream> {
    Ok(ContentStream::new(data.to_vec()))
}
