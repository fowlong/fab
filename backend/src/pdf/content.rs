//! Content stream tokenisation helpers (stub).

use anyhow::{anyhow, Context, Result};

const RECOGNISED_OPERATORS: &[&[u8]] = &[
    b"BT", b"ET", b"Tm", b"Td", b"TD", b"Tj", b"TJ", b"'", b"\"", b"Do", b"cm", b"q", b"Q",
];

/// Byte span for a recognised operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ByteSpan {
    pub start: usize,
    pub end: usize,
}

/// Token representing a PDF operator and the byte range it occupies.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperatorToken {
    pub operator: String,
    pub span: ByteSpan,
}

/// Tokenise a PDF content stream and extract recognised operators.
///
/// The simplified tokenizer scans the stream for ASCII operator names separated
/// by whitespace and records the byte offsets. It intentionally ignores
/// operands and other data, allowing downstream code to reason about where
/// operators live in the original stream.
pub fn tokenize_stream(bytes: &[u8]) -> Result<Vec<OperatorToken>> {
    let mut tokens = Vec::new();
    let mut index = 0;

    while index < bytes.len() {
        // Skip whitespace between tokens
        while index < bytes.len() && bytes[index].is_ascii_whitespace() {
            index += 1;
        }

        if index >= bytes.len() {
            break;
        }

        let start = index;
        while index < bytes.len() && !bytes[index].is_ascii_whitespace() {
            index += 1;
        }

        let end = index;
        if start == end {
            continue;
        }

        if RECOGNISED_OPERATORS.contains(&&bytes[start..end]) {
            let operator = std::str::from_utf8(&bytes[start..end])
                .with_context(|| anyhow!("invalid operator bytes"))?
                .to_string();
            tokens.push(OperatorToken {
                operator,
                span: ByteSpan { start, end },
            });
        }
    }

    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenize_stream_finds_text_operators_with_spans() {
        let stream = b"BT 1 0 0 1 10 20 Tm (Hello) Tj ET";
        let tokens = tokenize_stream(stream).expect("tokenization should succeed");

        assert_eq!(tokens.len(), 4);
        assert_eq!(tokens[0].operator, "BT");
        assert_eq!(tokens[0].span, ByteSpan { start: 0, end: 2 });
        assert_eq!(tokens[1].operator, "Tm");
        assert_eq!(tokens[1].span, ByteSpan { start: 17, end: 19 });
        assert_eq!(tokens[2].operator, "Tj");
        assert_eq!(tokens[2].span, ByteSpan { start: 28, end: 30 });
        assert_eq!(tokens[3].operator, "ET");
        assert_eq!(tokens[3].span, ByteSpan { start: 31, end: 33 });
    }

    #[test]
    fn tokenize_stream_handles_image_and_graphics_ops() {
        let stream = b"q 120 0 0 90 300 500 cm /Im7 Do Q";
        let tokens = tokenize_stream(stream).expect("tokenization should succeed");

        let ops: Vec<_> = tokens.iter().map(|token| token.operator.as_str()).collect();
        assert_eq!(ops, vec!["q", "cm", "Do", "Q"]);

        // Ensure spans are monotonically increasing and map to operator slices.
        for token in tokens {
            assert!(token.span.end > token.span.start);
            assert_eq!(&stream[token.span.start..token.span.end], token.operator.as_bytes());
        }
    }
}
