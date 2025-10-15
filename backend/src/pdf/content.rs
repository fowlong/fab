//! Content stream tokenisation helpers.
//!
//! The tokenizer is intentionally lightweight. It understands a curated list of
//! operators that drive the MVP transform workflow and records the byte spans
//! required for patching while leaving the raw stream bytes untouched.

use std::ops::Range;

use anyhow::{anyhow, Result};

#[derive(Debug, Clone, PartialEq)]
pub enum Operand {
    Number(f64),
    Name(String),
    String(Vec<u8>),
    Array(Vec<Operand>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operator {
    Q,
    q,
    Cm,
    Bt,
    Et,
    Tf,
    Tm,
    Td,
    TStar,
    Tj,
    TJ,
    Do,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub operator: Operator,
    pub operands: Vec<Operand>,
    pub range: Range<usize>,
}

impl Operator {
    fn parse(keyword: &str) -> Option<Self> {
        match keyword {
            "q" => Some(Operator::q),
            "Q" => Some(Operator::Q),
            "cm" => Some(Operator::Cm),
            "BT" => Some(Operator::Bt),
            "ET" => Some(Operator::Et),
            "Tf" => Some(Operator::Tf),
            "Tm" => Some(Operator::Tm),
            "Td" => Some(Operator::Td),
            "T*" => Some(Operator::TStar),
            "Tj" => Some(Operator::Tj),
            "TJ" => Some(Operator::TJ),
            "Do" => Some(Operator::Do),
            _ => None,
        }
    }
}

/// Tokenise a PDF content stream and emit recognised operators with their
/// operands and source byte spans.
pub fn tokenize_stream(bytes: &[u8]) -> Result<Vec<Token>> {
    let mut out = Vec::new();
    let mut stack: Vec<(Operand, Range<usize>)> = Vec::new();
    let mut cursor = 0usize;

    while cursor < bytes.len() {
        skip_whitespace_and_comments(bytes, &mut cursor);
        if cursor >= bytes.len() {
            break;
        }
        let start = cursor;
        match bytes[cursor] {
            b'(' => {
                let (value, end) = parse_string(bytes, cursor)?;
                stack.push((Operand::String(value), start..end));
                cursor = end;
            }
            b'[' => {
                let (value, end) = parse_array(bytes, cursor)?;
                stack.push((Operand::Array(value), start..end));
                cursor = end;
            }
            b'/' => {
                let (name, end) = parse_name(bytes, cursor)?;
                stack.push((Operand::Name(name), start..end));
                cursor = end;
            }
            b'+' | b'-' | b'.' | b'0'..=b'9' => {
                let (number, end) = parse_number(bytes, cursor)?;
                stack.push((Operand::Number(number), start..end));
                cursor = end;
            }
            _ => {
                let (word, end) = parse_word(bytes, cursor);
                cursor = end;
                if let Some(op) = Operator::parse(&word) {
                    let range_start = stack.first().map(|(_, range)| range.start).unwrap_or(start);
                    let token_range = range_start..end;
                    let operands = stack.iter().map(|(op, _)| op.clone()).collect();
                    out.push(Token {
                        operator: op,
                        operands,
                        range: token_range,
                    });
                    stack.clear();
                } else {
                    stack.clear();
                }
            }
        }
    }

    Ok(out)
}

fn skip_whitespace_and_comments(bytes: &[u8], cursor: &mut usize) {
    while *cursor < bytes.len() {
        match bytes[*cursor] {
            b'\n' | b'\r' | b'\t' | b'\x0C' | b'\x00' | b' ' => {
                *cursor += 1;
            }
            b'%' => {
                *cursor += 1;
                while *cursor < bytes.len() && bytes[*cursor] != b'\n' && bytes[*cursor] != b'\r' {
                    *cursor += 1;
                }
            }
            _ => break,
        }
    }
}

fn parse_word(bytes: &[u8], start: usize) -> (String, usize) {
    let mut end = start;
    while end < bytes.len() {
        let b = bytes[end];
        if b"\n\r\t\x0C %[]()<>".contains(&b) {
            break;
        }
        end += 1;
    }
    let word = std::str::from_utf8(&bytes[start..end])
        .unwrap_or("")
        .to_string();
    (word, end)
}

fn parse_name(bytes: &[u8], start: usize) -> Result<(String, usize)> {
    let mut end = start + 1; // skip leading '/'
    while end < bytes.len() {
        let b = bytes[end];
        if b"\n\r\t\x0C []()%<>".contains(&b) {
            break;
        }
        end += 1;
    }
    let slice = &bytes[start + 1..end];
    let name = std::str::from_utf8(slice)?.to_string();
    Ok((name, end))
}

fn parse_number(bytes: &[u8], start: usize) -> Result<(f64, usize)> {
    let mut end = start;
    while end < bytes.len() {
        let b = bytes[end];
        if !matches!(b, b'+' | b'-' | b'.' | b'0'..=b'9') {
            break;
        }
        end += 1;
    }
    let slice = &bytes[start..end];
    let text = std::str::from_utf8(slice)?;
    let number: f64 = text.parse()?;
    Ok((number, end))
}

fn parse_string(bytes: &[u8], start: usize) -> Result<(Vec<u8>, usize)> {
    let mut depth = 1usize;
    let mut pos = start + 1;
    let mut out = Vec::new();
    while pos < bytes.len() {
        let b = bytes[pos];
        match b {
            b'\\' => {
                if pos + 1 >= bytes.len() {
                    return Err(anyhow!("unterminated escape in string"));
                }
                let next = bytes[pos + 1];
                match next {
                    b'n' => out.push(b'\n'),
                    b'r' => out.push(b'\r'),
                    b't' => out.push(b'\t'),
                    b'b' => out.push(b'\x08'),
                    b'f' => out.push(b'\x0C'),
                    _ => out.push(next),
                }
                pos += 2;
            }
            b'(' => {
                depth += 1;
                out.push(b);
                pos += 1;
            }
            b')' => {
                depth -= 1;
                if depth == 0 {
                    return Ok((out, pos + 1));
                }
                out.push(b);
                pos += 1;
            }
            _ => {
                out.push(b);
                pos += 1;
            }
        }
    }
    Err(anyhow!("unterminated literal string"))
}

fn parse_array(bytes: &[u8], start: usize) -> Result<(Vec<Operand>, usize)> {
    let mut cursor = start + 1;
    let mut values = Vec::new();
    loop {
        skip_whitespace_and_comments(bytes, &mut cursor);
        if cursor >= bytes.len() {
            return Err(anyhow!("unterminated array"));
        }
        if bytes[cursor] == b']' {
            return Ok((values, cursor + 1));
        }
        let token_start = cursor;
        match bytes[cursor] {
            b'(' => {
                let (value, end) = parse_string(bytes, cursor)?;
                values.push(Operand::String(value));
                cursor = end;
            }
            b'[' => {
                let (inner, end) = parse_array(bytes, cursor)?;
                values.push(Operand::Array(inner));
                cursor = end;
            }
            b'/' => {
                let (name, end) = parse_name(bytes, cursor)?;
                values.push(Operand::Name(name));
                cursor = end;
            }
            b'+' | b'-' | b'.' | b'0'..=b'9' => {
                let (number, end) = parse_number(bytes, cursor)?;
                values.push(Operand::Number(number));
                cursor = end;
            }
            _ => {
                let (word, end) = parse_word(bytes, cursor);
                if word.is_empty() {
                    return Err(anyhow!("unexpected token in array at byte {token_start}"));
                }
                values.push(Operand::Name(word));
                cursor = end;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tokenise(input: &str) -> Vec<Token> {
        tokenize_stream(input.as_bytes()).expect("tokenize")
    }

    #[test]
    fn parse_cm_operator() {
        let tokens = tokenise("1 0 0 1 10 20 cm");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].operator, Operator::Cm);
        assert_eq!(tokens[0].operands.len(), 6);
        assert_eq!(tokens[0].range, 0..18);
    }

    #[test]
    fn parse_bt_et_with_text() {
        let tokens = tokenise("BT /F1 12 Tf 10 20 Td (Hello) Tj ET");
        assert_eq!(tokens.len(), 5);
        assert_eq!(tokens[0].operator, Operator::Bt);
        assert_eq!(tokens[1].operator, Operator::Tf);
        assert_eq!(tokens[2].operator, Operator::Td);
        assert_eq!(tokens[3].operator, Operator::Tj);
        assert_eq!(tokens[4].operator, Operator::Et);
        if let Operand::String(bytes) = &tokens[3].operands[0] {
            assert_eq!(bytes, b"Hello");
        } else {
            panic!("expected string operand");
        }
    }

    #[test]
    fn parse_tj_and_tj_array() {
        let tokens = tokenise("BT (Hi) Tj [(Ho) 120 (There)] TJ ET");
        assert_eq!(tokens.len(), 5);
        assert_eq!(tokens[2].operator, Operator::Tj);
        assert_eq!(tokens[3].operator, Operator::TJ);
        match &tokens[3].operands[0] {
            Operand::Array(items) => {
                assert_eq!(items.len(), 3);
            }
            other => panic!("unexpected operand {other:?}"),
        }
    }

    #[test]
    fn parse_do_operator() {
        let tokens = tokenise("q 2 0 0 2 0 0 cm /Im1 Do Q");
        assert_eq!(tokens.len(), 4);
        assert_eq!(tokens[0].operator, Operator::q);
        assert_eq!(tokens[2].operator, Operator::Do);
        assert_eq!(tokens[2].operands.len(), 1);
    }
}
