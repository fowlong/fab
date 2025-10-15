//! Minimal placeholder content stream utilities.

use anyhow::Result;

/// Represents a slice inside a PDF content stream that can be rewritten.
#[derive(Clone, Debug, Default)]
pub struct ContentSpan {
    pub start: usize,
    pub end: usize,
}

/// Tokenises the provided bytes.
///
/// This is a stub for the MVP implementation and currently returns an empty
/// list. It exists to document the intended API surface for later work.
pub fn tokenise(_bytes: &[u8]) -> Result<Vec<String>> {
    Ok(Vec::new())
}
