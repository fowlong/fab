//! Content stream tokeniser and helpers.
//!
//! The MVP implementation only exposes the types and function placeholders that
//! the rest of the backend references. Full parsing of PDF operators will be
//! added incrementally as the editor features grow.

use anyhow::Result;

/// Represents a slice of a PDF content stream.
#[derive(Debug, Clone)]
pub struct ContentRange {
    pub start: usize,
    pub end: usize,
}

/// Placeholder for a parsed content stream.
#[derive(Debug, Clone, Default)]
pub struct ParsedContent {
    pub ranges: Vec<ContentRange>,
}

pub fn parse_stream(_bytes: &[u8]) -> Result<ParsedContent> {
    Ok(ParsedContent::default())
}
