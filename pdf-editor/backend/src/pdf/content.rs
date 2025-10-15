//! Tokeniser and helpers for PDF content streams.
//!
//! The MVP implementation exposes placeholder structures so the frontend can be built
//! without committing to a full graphics operator pipeline yet. The module can be
//! expanded to implement the true BT/ET, path, and graphics state parsing required by
//! the editor.

use anyhow::Result;

#[derive(Debug, Clone)]
pub struct ContentStream<'a> {
    pub bytes: &'a [u8],
}

impl<'a> ContentStream<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self { bytes }
    }

    pub fn tokens(&self) -> Result<Vec<String>> {
        let text = std::str::from_utf8(self.bytes).unwrap_or("");
        Ok(text.split_whitespace().map(|s| s.to_string()).collect())
    }
}
