//! Content stream tokenisation helpers (stub).

use anyhow::Result;

/// Tokenise a PDF content stream.
///
/// The MVP ships with a simplified tokenizer that recognises only a subset
/// of PDF operators. The real implementation will expand this list and build
/// a structured intermediate representation for editing.
pub fn tokenize_stream(_bytes: &[u8]) -> Result<Vec<String>> {
    Ok(Vec::new())
}
