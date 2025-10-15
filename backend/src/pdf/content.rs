//! Content stream tokenisation helpers.

use std::{
    ops::Range,
    str::{self, FromStr},
};

use anyhow::{bail, Result};

/// Affine matrix represented as six coefficients.
pub type Matrix = [f64; 6];

/// Operand recognised by the tokenizer.
#[derive(Debug, Clone, PartialEq)]
pub enum Operand {
    Number(f64),
    Name(String),
    String(Vec<u8>),
    Array(Vec<Operand>),
}

impl Operand {
    pub fn as_number(&self) -> Option<f64> {
        match self {
            Operand::Number(value) => Some(*value),
            _ => None,
        }
    }

    pub fn as_name(&self) -> Option<&str> {
        match self {
            Operand::Name(name) => Some(name.as_str()),
            _ => None,
        }
    }
}

/// Token extracted from the content stream.
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub operator: String,
    pub operands: Vec<Operand>,
    pub span: Range<usize>,
}

/// Tokenise a PDF content stream and return the recognised tokens.
pub fn tokenize_stream(bytes: &[u8]) -> Result<Vec<Token>> {
    let mut tokens = Vec::new();
    let mut cursor = 0usize;
    let len = bytes.len();
    let mut operands = Vec::new();
    let mut operand_span_start: Option<usize> = None;

    while cursor < len {
        skip_whitespace(bytes, &mut cursor);
        if cursor >= len {
            break;
        }

        if bytes[cursor] == b'%' {
            skip_comment(bytes, &mut cursor);
            continue;
        }

        let token_start = cursor;
        match bytes[cursor] {
            b'[' => {
                let (array, next) = parse_array(bytes, cursor + 1)?;
                if operand_span_start.is_none() {
                    operand_span_start = Some(token_start);
                }
                operands.push(Operand::Array(array));
                cursor = next;
                continue;
            }
            b']' => {
                cursor += 1;
                continue;
            }
            b'(' => {
                let (string, next) = parse_literal_string(bytes, cursor + 1)?;
                if operand_span_start.is_none() {
                    operand_span_start = Some(token_start);
                }
                operands.push(Operand::String(string));
                cursor = next;
                continue;
            }
            b'<' => {
                if cursor + 1 < len && bytes[cursor + 1] == b'<' {
                    // Dictionary open – treat it as an operator to maintain stream alignment.
                    let text = read_simple_token(bytes, &mut cursor)?;
                    flush_operator(
                        text,
                        token_start,
                        cursor,
                        &mut operands,
                        &mut operand_span_start,
                        &mut tokens,
                    );
                    continue;
                }
                let (string, next) = parse_hex_string(bytes, cursor + 1)?;
                if operand_span_start.is_none() {
                    operand_span_start = Some(token_start);
                }
                operands.push(Operand::String(string));
                cursor = next;
                continue;
            }
            _ => {}
        }

        let text = read_simple_token(bytes, &mut cursor)?;
        if text.is_empty() {
            continue;
        }

        if is_operator(&text) {
            flush_operator(
                text,
                operand_span_start.unwrap_or(token_start),
                cursor,
                &mut operands,
                &mut operand_span_start,
                &mut tokens,
            );
        } else {
            if operand_span_start.is_none() {
                operand_span_start = Some(token_start);
            }
            operands.push(parse_operand_value(&text));
        }
    }

    Ok(tokens)
}

fn flush_operator(
    operator: String,
    start: usize,
    end: usize,
    operands: &mut Vec<Operand>,
    span_start: &mut Option<usize>,
    tokens: &mut Vec<Token>,
) {
    tokens.push(Token {
        operator,
        operands: std::mem::take(operands),
        span: start..end,
    });
    *span_start = None;
}

fn parse_operand_value(text: &str) -> Operand {
    if let Ok(value) = f64::from_str(text) {
        Operand::Number(value)
    } else if let Some(stripped) = text.strip_prefix('/') {
        Operand::Name(stripped.to_string())
    } else {
        Operand::Name(text.to_string())
    }
}

fn is_operator(token: &str) -> bool {
    matches!(
        token,
        "q" | "Q" | "cm" | "BT" | "ET" | "Tf" | "Tm" | "Td" | "T*" | "Tj" | "TJ" | "Do"
    )
}

fn parse_array(bytes: &[u8], mut cursor: usize) -> Result<(Vec<Operand>, usize)> {
    let mut values = Vec::new();
    loop {
        skip_whitespace(bytes, &mut cursor);
        if cursor >= bytes.len() {
            bail!("unterminated array");
        }
        if bytes[cursor] == b']' {
            cursor += 1;
            break;
        }

        let value = parse_operand(bytes, &mut cursor)?;
        values.push(value);
    }
    Ok((values, cursor))
}

fn parse_operand(bytes: &[u8], cursor: &mut usize) -> Result<Operand> {
    skip_whitespace(bytes, cursor);
    if *cursor >= bytes.len() {
        bail!("unexpected end of content stream");
    }
    let value = match bytes[*cursor] {
        b'[' => {
            let (array, next) = parse_array(bytes, *cursor + 1)?;
            *cursor = next;
            Operand::Array(array)
        }
        b'(' => {
            let (string, next) = parse_literal_string(bytes, *cursor + 1)?;
            *cursor = next;
            Operand::String(string)
        }
        b'<' => {
            if *cursor + 1 < bytes.len() && bytes[*cursor + 1] == b'<' {
                let token = read_simple_token(bytes, cursor)?;
                Operand::Name(token)
            } else {
                let (string, next) = parse_hex_string(bytes, *cursor + 1)?;
                *cursor = next;
                Operand::String(string)
            }
        }
        b'/' => {
            let text = read_simple_token(bytes, cursor)?;
            Operand::Name(text.trim_start_matches('/').to_string())
        }
        _ => {
            let text = read_simple_token(bytes, cursor)?;
            if let Ok(value) = f64::from_str(&text) {
                Operand::Number(value)
            } else {
                Operand::Name(text)
            }
        }
    };
    Ok(value)
}

fn parse_literal_string(bytes: &[u8], mut cursor: usize) -> Result<(Vec<u8>, usize)> {
    let mut nesting = 1i32;
    let mut buffer = Vec::new();
    while cursor < bytes.len() {
        let byte = bytes[cursor];
        match byte {
            b'(' => {
                nesting += 1;
                buffer.push(byte);
                cursor += 1;
            }
            b')' => {
                nesting -= 1;
                if nesting == 0 {
                    cursor += 1;
                    break;
                }
                buffer.push(byte);
                cursor += 1;
            }
            b'\\' => {
                cursor += 1;
                if cursor >= bytes.len() {
                    bail!("unterminated escape sequence");
                }
                let escaped = bytes[cursor];
                match escaped {
                    b'n' => buffer.push(b'\n'),
                    b'r' => buffer.push(b'\r'),
                    b't' => buffer.push(b'\t'),
                    b'b' => buffer.push(0x08),
                    b'f' => buffer.push(0x0C),
                    b'(' | b')' | b'\\' => buffer.push(escaped),
                    b'0'..=b'7' => {
                        let mut octal = vec![escaped];
                        for _ in 0..2 {
                            if cursor + 1 < bytes.len() && bytes[cursor + 1].is_ascii_digit() {
                                cursor += 1;
                                octal.push(bytes[cursor]);
                            } else {
                                break;
                            }
                        }
                        let value = u8::from_str_radix(str::from_utf8(&octal)?, 8)?;
                        buffer.push(value);
                    }
                    _ => buffer.push(escaped),
                }
                cursor += 1;
            }
            _ => {
                buffer.push(byte);
                cursor += 1;
            }
        }
    }
    if nesting != 0 {
        bail!("unterminated literal string");
    }
    Ok((buffer, cursor))
}

fn parse_hex_string(bytes: &[u8], mut cursor: usize) -> Result<(Vec<u8>, usize)> {
    let mut buffer = Vec::new();
    let mut current = Vec::new();
    while cursor < bytes.len() {
        let byte = bytes[cursor];
        cursor += 1;
        if byte == b'>' {
            break;
        }
        if byte.is_ascii_whitespace() {
            continue;
        }
        current.push(byte);
        if current.len() == 2 {
            let value = u8::from_str_radix(str::from_utf8(&current)?, 16)?;
            buffer.push(value);
            current.clear();
        }
    }
    if !current.is_empty() {
        // Odd number of digits -> pad with 0.
        current.push(b'0');
        let value = u8::from_str_radix(str::from_utf8(&current)?, 16)?;
        buffer.push(value);
    }
    Ok((buffer, cursor))
}

fn read_simple_token(bytes: &[u8], cursor: &mut usize) -> Result<String> {
    let start = *cursor;
    while *cursor < bytes.len() {
        let byte = bytes[*cursor];
        if is_delimiter(byte) {
            break;
        }
        *cursor += 1;
    }
    let slice = &bytes[start..*cursor];
    Ok(str::from_utf8(slice)?.to_string())
}

fn skip_whitespace(bytes: &[u8], cursor: &mut usize) {
    while *cursor < bytes.len() {
        let byte = bytes[*cursor];
        if byte.is_ascii_whitespace() {
            *cursor += 1;
        } else {
            break;
        }
    }
}

fn skip_comment(bytes: &[u8], cursor: &mut usize) {
    while *cursor < bytes.len() {
        let byte = bytes[*cursor];
        *cursor += 1;
        if byte == b'\n' || byte == b'\r' {
            break;
        }
    }
}

fn is_delimiter(byte: u8) -> bool {
    matches!(
        byte,
        b'(' | b')'
            | b'<'
            | b'>'
            | b'['
            | b']'
            | b'{'
            | b'}'
            | b'/'
            | b'%'
            | b' '
            | b'\t'
            | b'\r'
            | b'\n'
    )
}

/// Multiply two affine matrices in PDF order.
pub fn multiply(a: &Matrix, b: &Matrix) -> Matrix {
    [
        a[0] * b[0] + a[2] * b[1],
        a[1] * b[0] + a[3] * b[1],
        a[0] * b[2] + a[2] * b[3],
        a[1] * b[2] + a[3] * b[3],
        a[0] * b[4] + a[2] * b[5] + a[4],
        a[1] * b[4] + a[3] * b[5] + a[5],
    ]
}

/// Identity affine matrix.
pub const IDENTITY: Matrix = [1.0, 0.0, 0.0, 1.0, 0.0, 0.0];

#[cfg(test)]
mod tests {
    use super::*;

    fn token_ops(bytes: &str) -> Vec<(String, Vec<Operand>, Range<usize>)> {
        tokenize_stream(bytes.as_bytes())
            .unwrap()
            .into_iter()
            .map(|token| (token.operator, token.operands, token.span))
            .collect()
    }

    #[test]
    fn parses_cm_operator() {
        let tokens = token_ops("1 0 0 1 10 20 cm");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].0, "cm");
        assert_eq!(tokens[0].1.len(), 6);
        assert_eq!(tokens[0].2, 0..14);
    }

    #[test]
    fn parses_text_block() {
        let tokens = token_ops("BT /F1 12 Tf 10 20 Td (Hi) Tj ET");
        assert_eq!(tokens.len(), 5);
        assert_eq!(tokens[0].0, "BT");
        assert_eq!(tokens[1].0, "Tf");
        assert_eq!(tokens[2].0, "Td");
        assert_eq!(tokens[3].0, "Tj");
        assert_eq!(tokens[4].0, "ET");
        assert_eq!(tokens[3].1.len(), 1);
        match &tokens[3].1[0] {
            Operand::String(buf) => assert_eq!(buf, &b"Hi".to_vec()),
            _ => panic!("expected string"),
        }
    }

    #[test]
    fn parses_tj_array() {
        let tokens = token_ops("BT [ (A) 120 (B) ] TJ ET");
        assert_eq!(tokens[2].0, "TJ");
        match &tokens[2].1[0] {
            Operand::Array(items) => {
                assert_eq!(items.len(), 3);
                match &items[0] {
                    Operand::String(bytes) => assert_eq!(bytes, &b"A".to_vec()),
                    _ => panic!("expected string"),
                }
                assert_eq!(items[1].as_number(), Some(120.0));
            }
            _ => panic!("expected array"),
        }
    }

    #[test]
    fn parses_do_operator() {
        let tokens = token_ops("q 200 0 0 200 0 0 cm /Im1 Do Q");
        assert_eq!(tokens.len(), 5);
        assert_eq!(tokens[3].0, "Do");
        assert_eq!(tokens[3].1[0].as_name(), Some("Im1"));
    }
}
