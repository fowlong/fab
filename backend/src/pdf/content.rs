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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenize_stream_returns_empty_collection() {
        let tokens = tokenize_stream(b"BT (Hello) Tj ET").expect("tokenization should succeed");
        assert!(tokens.is_empty());
    }
}
