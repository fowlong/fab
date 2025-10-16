//! Content stream tokenisation helpers.

use anyhow::Result;

const OPERATORS: &[&str] = &[
    "BT", "ET", "Tf", "Td", "TD", "Tm", "Tj", "TJ", "cm", "q", "Q", "Do",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ByteSpan {
    pub start: usize,
    pub end: usize,
}

impl ByteSpan {
    pub const fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub const fn len(self) -> usize {
        self.end - self.start
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContentToken {
    Operator { name: String, span: ByteSpan },
    Operand { raw: Vec<u8>, span: ByteSpan },
}

/// Tokenise a PDF content stream and capture operator byte ranges.
pub fn tokenize_stream(bytes: &[u8]) -> Result<Vec<ContentToken>> {
    let mut tokens = Vec::new();
    let mut index = 0;

    while index < bytes.len() {
        let current = bytes[index];

        if is_whitespace(current) {
            index += 1;
            continue;
        }

        if current == b'%' {
            index = skip_comment(bytes, index + 1);
            continue;
        }

        if current == b'(' {
            let start = index;
            index = consume_string(bytes, index + 1);
            tokens.push(ContentToken::Operand {
                raw: bytes[start..index].to_vec(),
                span: ByteSpan::new(start, index),
            });
            continue;
        }

        if matches!(current, b'[' | b']' | b'{' | b'}') {
            let start = index;
            index += 1;
            tokens.push(ContentToken::Operand {
                raw: bytes[start..index].to_vec(),
                span: ByteSpan::new(start, index),
            });
            continue;
        }

        if matches!(current, b'<' | b'>') {
            let start = index;
            index = consume_angle(bytes, index, current);
            tokens.push(ContentToken::Operand {
                raw: bytes[start..index].to_vec(),
                span: ByteSpan::new(start, index),
            });
            continue;
        }

        let start = index;
        while index < bytes.len() {
            let byte = bytes[index];
            if is_whitespace(byte) || is_delimiter(byte) {
                break;
            }
            index += 1;
        }
        let end = index;
        if end == start {
            index += 1;
            continue;
        }

        let span = ByteSpan::new(start, end);
        let raw = &bytes[start..end];
        if let Ok(name) = std::str::from_utf8(raw) {
            if OPERATORS.contains(&name) {
                tokens.push(ContentToken::Operator {
                    name: name.to_string(),
                    span,
                });
                continue;
            }
        }

        tokens.push(ContentToken::Operand {
            raw: raw.to_vec(),
            span,
        });
    }

    Ok(tokens)
}

fn is_whitespace(byte: u8) -> bool {
    matches!(byte, b' ' | b'\n' | b'\r' | b'\t' | 0x0C | 0x00)
}

fn is_delimiter(byte: u8) -> bool {
    matches!(
        byte,
        b'(' | b')' | b'<' | b'>' | b'[' | b']' | b'{' | b'}' | b'%'
    )
}

fn skip_comment(bytes: &[u8], mut index: usize) -> usize {
    while index < bytes.len() {
        let byte = bytes[index];
        index += 1;
        if byte == b'\n' || byte == b'\r' {
            break;
        }
    }
    index
}

fn consume_string(bytes: &[u8], mut index: usize) -> usize {
    let mut depth = 1;
    while index < bytes.len() && depth > 0 {
        let byte = bytes[index];
        index += 1;
        match byte {
            b'\\' => {
                if index < bytes.len() {
                    index += 1;
                }
            }
            b'(' => depth += 1,
            b')' => depth -= 1,
            _ => {}
        }
    }
    index
}

fn consume_angle(bytes: &[u8], mut index: usize, delimiter: u8) -> usize {
    index += 1;
    if index < bytes.len() && bytes[index] == delimiter {
        index += 1;
        return index;
    }

    while index < bytes.len() {
        let byte = bytes[index];
        index += 1;
        if byte == delimiter {
            break;
        }
    }
    index
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenize_stream_extracts_operator_spans() {
        let stream = b"BT /F1 12 Tf\n0 0 Td (Hello) Tj\nET";
        let tokens = tokenize_stream(stream).expect("tokenization should succeed");

        let operators: Vec<_> = tokens
            .into_iter()
            .filter_map(|token| match token {
                ContentToken::Operator { name, span } => Some((name, span)),
                _ => None,
            })
            .collect();

        fn span_for(stream: &[u8], needle: &str) -> ByteSpan {
            let start = stream
                .windows(needle.len())
                .position(|window| window == needle.as_bytes())
                .expect("needle present");
            ByteSpan::new(start, start + needle.len())
        }

        assert_eq!(
            operators,
            vec![
                ("BT".into(), span_for(stream, "BT")),
                ("Tf".into(), span_for(stream, "Tf")),
                ("Td".into(), span_for(stream, "Td")),
                ("Tj".into(), span_for(stream, "Tj")),
                ("ET".into(), span_for(stream, "ET")),
            ]
        );
    }

    #[test]
    fn tokenize_stream_reports_precise_spans_for_graphics_ops() {
        let stream = b"q 10 0 0 10 0 0 cm\n/Im0 Do\nQ";
        let tokens = tokenize_stream(stream).expect("tokenization should succeed");

        let mut cm_span = None;
        let mut do_span = None;
        let mut q_span = None;
        let mut capital_q_span = None;

        for token in tokens {
            if let ContentToken::Operator { name, span } = token {
                match name.as_str() {
                    "cm" => cm_span = Some(span),
                    "Do" => do_span = Some(span),
                    "q" => q_span = Some(span),
                    "Q" => capital_q_span = Some(span),
                    _ => {}
                }
            }
        }

        let cm_start = stream
            .windows(2)
            .position(|window| window == b"cm")
            .expect("cm present");
        let do_start = stream
            .windows(2)
            .position(|window| window == b"Do")
            .expect("Do present");
        let q_start = stream.iter().position(|&b| b == b'q').expect("q present");
        let capital_q_start = stream.iter().rposition(|&b| b == b'Q').expect("Q present");

        assert_eq!(cm_span, Some(ByteSpan::new(cm_start, cm_start + 2)));
        assert_eq!(do_span, Some(ByteSpan::new(do_start, do_start + 2)));
        assert_eq!(q_span, Some(ByteSpan::new(q_start, q_start + 1)));
        assert_eq!(
            capital_q_span,
            Some(ByteSpan::new(capital_q_start, capital_q_start + 1))
        );
    }
}
