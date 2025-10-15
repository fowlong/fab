//! Minimal PDF content stream helpers for tokenisation and byte span tracking.

use std::ops::Range;

use anyhow::{anyhow, Result};

#[derive(Debug, Clone, PartialEq)]
pub enum Operand {
    Number(f64),
    Name(String),
    String(Vec<u8>),
    Array(Vec<Operand>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
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
    TjArray,
    Do,
    Other(String),
}

#[derive(Debug, Clone)]
pub struct Token {
    pub operands: Vec<Operand>,
    pub operator: Operator,
    pub span: Range<usize>,
}

pub fn tokenize_stream(bytes: &[u8]) -> Result<Vec<Token>> {
    let mut tokens = Vec::new();
    let mut pending: Vec<(Operand, Range<usize>)> = Vec::new();
    let mut pos = 0usize;

    while let Some(next) = next_non_whitespace(bytes, pos) {
        pos = next;
        if bytes[pos] == b'%' {
            pos = skip_comment(bytes, pos);
            continue;
        }

        match bytes[pos] {
            b'[' => {
                let (array, span_end) = parse_array(bytes, pos + 1)?;
                pending.push((Operand::Array(array), pos..span_end));
                pos = span_end;
            }
            b'(' => {
                let (string, span_end) = parse_literal_string(bytes, pos + 1)?;
                pending.push((Operand::String(string), pos..span_end));
                pos = span_end;
            }
            b'<' => {
                if matches!(bytes.get(pos + 1), Some(b'<')) {
                    let (name, span_end) = parse_literal(bytes, pos)?;
                    let operand = Operand::Name(name);
                    pending.push((operand, pos..span_end));
                    pos = span_end;
                } else {
                    let (string, span_end) = parse_hex_string(bytes, pos + 1)?;
                    pending.push((Operand::String(string), pos..span_end));
                    pos = span_end;
                }
            }
            b'/' => {
                let (name, span_end) = parse_name(bytes, pos + 1);
                pending.push((Operand::Name(name), pos..span_end));
                pos = span_end;
            }
            b'+' | b'-' | b'.' => {
                let (num, span_end) = parse_number(bytes, pos)?;
                pending.push((Operand::Number(num), pos..span_end));
                pos = span_end;
            }
            b'0'..=b'9' => {
                let (num, span_end) = parse_number(bytes, pos)?;
                pending.push((Operand::Number(num), pos..span_end));
                pos = span_end;
            }
            _ => {
                let (literal, span_end) = parse_literal(bytes, pos)?;
                let operator = match literal.as_str() {
                    "q" => Operator::q,
                    "Q" => Operator::Q,
                    "cm" => Operator::Cm,
                    "BT" => Operator::Bt,
                    "ET" => Operator::Et,
                    "Tf" => Operator::Tf,
                    "Tm" => Operator::Tm,
                    "Td" => Operator::Td,
                    "T*" => Operator::TStar,
                    "Tj" => Operator::Tj,
                    "TJ" => Operator::TjArray,
                    "Do" => Operator::Do,
                    other => Operator::Other(other.to_string()),
                };
                let start = pending
                    .first()
                    .map(|(_, span)| span.start)
                    .unwrap_or(pos);
                tokens.push(Token {
                    operands: pending.iter().map(|(op, _)| op.clone()).collect(),
                    operator,
                    span: start..span_end,
                });
                pending.clear();
                pos = span_end;
            }
        }
    }

    Ok(tokens)
}

fn next_non_whitespace(bytes: &[u8], mut pos: usize) -> Option<usize> {
    while pos < bytes.len() {
        match bytes[pos] {
            b' ' | b'\n' | b'\r' | b'\t' | b'\x0c' | b'\x0b' => pos += 1,
            _ => return Some(pos),
        }
    }
    None
}

fn skip_comment(bytes: &[u8], mut pos: usize) -> usize {
    while pos < bytes.len() && bytes[pos] != b'\n' {
        pos += 1;
    }
    pos
}

fn parse_array(bytes: &[u8], mut pos: usize) -> Result<(Vec<Operand>, usize)> {
    let mut result = Vec::new();
    loop {
        if pos >= bytes.len() {
            return Err(anyhow!("Unterminated array"));
        }
        match bytes[pos] {
            b' ' | b'\n' | b'\r' | b'\t' | b'\x0c' | b'\x0b' => pos += 1,
            b'%' => pos = skip_comment(bytes, pos),
            b']' => return Ok((result, pos + 1)),
            b'[' => {
                let (array, span_end) = parse_array(bytes, pos + 1)?;
                result.push(Operand::Array(array));
                pos = span_end;
            }
            b'(' => {
                let (string, span_end) = parse_literal_string(bytes, pos + 1)?;
                result.push(Operand::String(string));
                pos = span_end;
            }
            b'<' => {
                if matches!(bytes.get(pos + 1), Some(b'<')) {
                    let (name, span_end) = parse_literal(bytes, pos)?;
                    result.push(Operand::Name(name));
                    pos = span_end;
                } else {
                    let (string, span_end) = parse_hex_string(bytes, pos + 1)?;
                    result.push(Operand::String(string));
                    pos = span_end;
                }
            }
            b'/' => {
                let (name, span_end) = parse_name(bytes, pos + 1);
                result.push(Operand::Name(name));
                pos = span_end;
            }
            b'+' | b'-' | b'.' => {
                let (num, span_end) = parse_number(bytes, pos)?;
                result.push(Operand::Number(num));
                pos = span_end;
            }
            b'0'..=b'9' => {
                let (num, span_end) = parse_number(bytes, pos)?;
                result.push(Operand::Number(num));
                pos = span_end;
            }
            _ => {
                let (literal, span_end) = parse_literal(bytes, pos)?;
                result.push(Operand::Name(literal));
                pos = span_end;
            }
        }
    }
}

fn parse_literal(bytes: &[u8], mut pos: usize) -> Result<(String, usize)> {
    let start = pos;
    while pos < bytes.len() {
        let byte = bytes[pos];
        if byte.is_ascii_whitespace() || matches!(byte, b'[' | b']' | b'<' | b'>' | b'/' | b'(' | b')') {
            break;
        }
        pos += 1;
    }
    let literal = std::str::from_utf8(&bytes[start..pos])?.to_string();
    Ok((literal, pos))
}

fn parse_name(bytes: &[u8], mut pos: usize) -> (String, usize) {
    let start = pos - 1;
    while pos < bytes.len() {
        let byte = bytes[pos];
        if byte.is_ascii_whitespace() || matches!(byte, b'[' | b']' | b'<' | b'>' | b'/' | b'(' | b')') {
            break;
        }
        pos += 1;
    }
    let name = std::str::from_utf8(&bytes[start..pos])
        .unwrap_or_default()
        .to_string();
    (name, pos)
}

fn parse_number(bytes: &[u8], mut pos: usize) -> Result<(f64, usize)> {
    let start = pos;
    while pos < bytes.len() {
        let byte = bytes[pos];
        if byte.is_ascii_whitespace() || matches!(byte, b'[' | b']' | b'<' | b'>' | b'/' | b'(' | b')') {
            break;
        }
        pos += 1;
    }
    let slice = std::str::from_utf8(&bytes[start..pos])?;
    let value = slice.parse::<f64>()?;
    Ok((value, pos))
}

fn parse_literal_string(bytes: &[u8], mut pos: usize) -> Result<(Vec<u8>, usize)> {
    let mut depth = 1i32;
    let mut result = Vec::new();
    while pos < bytes.len() {
        let byte = bytes[pos];
        match byte {
            b'(' => {
                depth += 1;
                result.push(byte);
                pos += 1;
            }
            b')' => {
                depth -= 1;
                if depth == 0 {
                    return Ok((result, pos + 1));
                }
                result.push(byte);
                pos += 1;
            }
            b'\\' => {
                if let Some(&next) = bytes.get(pos + 1) {
                    result.push(match next {
                        b'n' => b'\n',
                        b'r' => b'\r',
                        b't' => b'\t',
                        b'b' => 0x08,
                        b'f' => 0x0C,
                        b'(' => b'(',
                        b')' => b')',
                        b'\\' => b'\\',
                        digit if digit >= b'0' && digit <= b'7' => {
                            let octal = read_octal_escape(bytes, pos + 1);
                            octal
                        }
                        other => other,
                    });
                    pos += 2;
                } else {
                    pos += 1;
                }
            }
            _ => {
                result.push(byte);
                pos += 1;
            }
        }
    }
    Err(anyhow!("Unterminated literal string"))
}

fn read_octal_escape(bytes: &[u8], start: usize) -> u8 {
    let mut value = 0u8;
    let mut consumed = 0;
    for idx in start..(start + 3).min(bytes.len()) {
        let byte = bytes[idx];
        if byte < b'0' || byte > b'7' {
            break;
        }
        value = (value << 3) + (byte - b'0');
        consumed += 1;
        if consumed == 3 {
            break;
        }
    }
    value
}

fn parse_hex_string(bytes: &[u8], mut pos: usize) -> Result<(Vec<u8>, usize)> {
    let mut result = Vec::new();
    let mut buffer = Vec::new();
    while pos < bytes.len() {
        match bytes[pos] {
            b'>' => {
                if !buffer.is_empty() {
                    buffer.push(b'0');
                }
                break;
            }
            byte if byte.is_ascii_whitespace() => pos += 1,
            byte => {
                buffer.push(byte);
                pos += 1;
                if buffer.len() == 2 {
                    let high = hex_value(buffer[0])?;
                    let low = hex_value(buffer[1])?;
                    result.push((high << 4) + low);
                    buffer.clear();
                }
            }
        }
    }
    if !buffer.is_empty() {
        let high = hex_value(buffer[0])?;
        result.push(high << 4);
    }
    Ok((result, pos + 1))
}

fn hex_value(byte: u8) -> Result<u8> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(10 + byte - b'a'),
        b'A'..=b'F' => Ok(10 + byte - b'A'),
        _ => Err(anyhow!("Invalid hex digit")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn token_names(tokens: &[Token]) -> Vec<&'static str> {
        tokens
            .iter()
            .map(|token| match token.operator {
                Operator::q => "q",
                Operator::Q => "Q",
                Operator::Cm => "cm",
                Operator::Bt => "BT",
                Operator::Et => "ET",
                Operator::Tf => "Tf",
                Operator::Tm => "Tm",
                Operator::Td => "Td",
                Operator::TStar => "T*",
                Operator::Tj => "Tj",
                Operator::TjArray => "TJ",
                Operator::Do => "Do",
                Operator::Other(_) => "other",
            })
            .collect()
    }

    #[test]
    fn parses_cm_and_q() {
        let input = b"q 1 0 0 1 0 0 cm Q";
        let tokens = tokenize_stream(input).unwrap();
        assert_eq!(token_names(&tokens), vec!["q", "cm", "Q"]);
        assert_eq!(tokens[1].operands.len(), 6);
    }

    #[test]
    fn parses_bt_tj_et() {
        let input = b"BT /F1 12 Tf (Hello) Tj ET";
        let tokens = tokenize_stream(input).unwrap();
        assert_eq!(token_names(&tokens), vec!["BT", "Tf", "Tj", "ET"]);
        assert_eq!(tokens[2].operands.len(), 1);
        match &tokens[2].operands[0] {
            Operand::String(bytes) => assert_eq!(bytes, b"Hello"),
            other => panic!("Unexpected operand: {other:?}"),
        }
    }

    #[test]
    fn parses_tj_array() {
        let input = b"[(Hel) 120 (lo)] TJ";
        let tokens = tokenize_stream(input).unwrap();
        assert_eq!(token_names(&tokens), vec!["TJ"]);
        assert_eq!(tokens[0].operands.len(), 1);
        match &tokens[0].operands[0] {
            Operand::Array(items) => {
                assert_eq!(items.len(), 3);
            }
            _ => panic!("expected array"),
        }
    }

    #[test]
    fn parses_do_operator() {
        let input = b"/Im1 Do";
        let tokens = tokenize_stream(input).unwrap();
        assert_eq!(token_names(&tokens), vec!["Do"]);
        match &tokens[0].operands[0] {
            Operand::Name(name) => assert_eq!(name, "/Im1"),
            _ => panic!("expected name"),
        }
    }
}
