use std::{ops::Range, str};

use anyhow::{anyhow, Result};

#[derive(Debug, Clone, PartialEq)]
pub enum Operand {
    Number(f64),
    Name(String),
    Array(Vec<Operand>),
    String(Vec<u8>),
    Other(String),
}

#[derive(Debug, Clone)]
pub struct Token {
    pub operands: Vec<Operand>,
    pub operator: String,
    pub byte_range: Range<usize>,
}

pub fn tokenize_stream(bytes: &[u8]) -> Result<Vec<Token>> {
    let mut index = 0usize;
    let mut operands = Vec::new();
    let mut token_start: Option<usize> = None;
    let mut tokens = Vec::new();

    while index < bytes.len() {
        skip_whitespace(bytes, &mut index);
        if index >= bytes.len() {
            break;
        }

        if bytes[index] == b'%' {
            skip_comment(bytes, &mut index);
            continue;
        }

        if bytes[index] == b'[' {
            let start = index;
            let (value, next) = parse_array(bytes, index + 1)?;
            operands.push(Operand::Array(value));
            token_start.get_or_insert(start);
            index = next;
            continue;
        }

        if bytes[index] == b'(' {
            let start = index;
            let (value, next) = parse_string(bytes, index + 1)?;
            operands.push(Operand::String(value));
            token_start.get_or_insert(start);
            index = next;
            continue;
        }

        let ch = bytes[index];
        if is_name_start(ch) {
            let start = index;
            let (name, next) = parse_name(bytes, index + 1)?;
            operands.push(Operand::Name(name));
            token_start.get_or_insert(start);
            index = next;
            continue;
        }

        if is_number_start(ch) {
            let start = index;
            let (number, next) = parse_number(bytes, index)?;
            operands.push(Operand::Number(number));
            token_start.get_or_insert(start);
            index = next;
            continue;
        }

        // Treat everything else as an operator token.
        let op_start = index;
        let operator = parse_operator(bytes, &mut index)?;
        let start = token_start.unwrap_or(op_start);
        let end = index;
        tokens.push(Token {
            operands: std::mem::take(&mut operands),
            operator,
            byte_range: start..end,
        });
        token_start = None;
    }

    Ok(tokens)
}

fn skip_whitespace(bytes: &[u8], index: &mut usize) {
    while *index < bytes.len() {
        match bytes[*index] {
            b'\x00' | b'\x09' | b'\x0a' | b'\x0c' | b'\x0d' | b'\x20' => *index += 1,
            _ => break,
        }
    }
}

fn skip_comment(bytes: &[u8], index: &mut usize) {
    while *index < bytes.len() && bytes[*index] != b'\n' && bytes[*index] != b'\r' {
        *index += 1;
    }
}

fn parse_array(bytes: &[u8], mut index: usize) -> Result<(Vec<Operand>, usize)> {
    let mut values = Vec::new();
    loop {
        skip_whitespace(bytes, &mut index);
        if index >= bytes.len() {
            return Err(anyhow!("unterminated array"));
        }
        match bytes[index] {
            b']' => {
                index += 1;
                break;
            }
            b'%' => {
                skip_comment(bytes, &mut index);
            }
            b'[' => {
                let (inner, next) = parse_array(bytes, index + 1)?;
                values.push(Operand::Array(inner));
                index = next;
            }
            b'(' => {
                let (value, next) = parse_string(bytes, index + 1)?;
                values.push(Operand::String(value));
                index = next;
            }
            _ if is_name_start(bytes[index]) => {
                let (name, next) = parse_name(bytes, index + 1)?;
                values.push(Operand::Name(name));
                index = next;
            }
            _ if is_number_start(bytes[index]) => {
                let (number, next) = parse_number(bytes, index)?;
                values.push(Operand::Number(number));
                index = next;
            }
            _ => {
                let op = parse_operator(bytes, &mut index)?;
                values.push(Operand::Other(op));
            }
        }
    }
    Ok((values, index))
}

fn parse_string(bytes: &[u8], mut index: usize) -> Result<(Vec<u8>, usize)> {
    let mut result = Vec::new();
    let mut depth = 1isize;
    while index < bytes.len() {
        let byte = bytes[index];
        if byte == b'\\' {
            if index + 1 < bytes.len() {
                result.push(bytes[index + 1]);
                index += 2;
                continue;
            }
        }
        if byte == b'(' {
            depth += 1;
        } else if byte == b')' {
            depth -= 1;
            if depth == 0 {
                index += 1;
                break;
            }
        }
        result.push(byte);
        index += 1;
    }
    if depth != 0 {
        return Err(anyhow!("unterminated string"));
    }
    Ok((result, index))
}

fn parse_name(bytes: &[u8], mut index: usize) -> Result<String> {
    let start = index - 1; // include leading '/'
    while index < bytes.len() && !is_delimiter(bytes[index]) && !is_whitespace(bytes[index]) {
        index += 1;
    }
    let slice = &bytes[start..index];
    Ok(str::from_utf8(slice)?.to_string())
}

fn parse_number(bytes: &[u8], mut index: usize) -> Result<(f64, usize)> {
    let start = index;
    if bytes[index] == b'+' || bytes[index] == b'-' {
        index += 1;
    }
    while index < bytes.len() && (bytes[index].is_ascii_digit() || bytes[index] == b'.') {
        index += 1;
    }
    let slice = &bytes[start..index];
    let text = str::from_utf8(slice)?;
    let value: f64 = text.parse()?;
    Ok((value, index))
}

fn parse_operator(bytes: &[u8], index: &mut usize) -> Result<String> {
    let start = *index;
    while *index < bytes.len() {
        let byte = bytes[*index];
        if is_whitespace(byte) || is_delimiter(byte) {
            break;
        }
        *index += 1;
    }
    if start == *index {
        return Err(anyhow!("expected operator"));
    }
    Ok(str::from_utf8(&bytes[start..*index])?.to_string())
}

fn is_name_start(byte: u8) -> bool {
    byte == b'/'
}

fn is_number_start(byte: u8) -> bool {
    byte == b'+' || byte == b'-' || byte == b'.' || byte.is_ascii_digit()
}

fn is_whitespace(byte: u8) -> bool {
    matches!(
        byte,
        b'\x00' | b'\x09' | b'\x0a' | b'\x0c' | b'\x0d' | b'\x20'
    )
}

fn is_delimiter(byte: u8) -> bool {
    matches!(
        byte,
        b'(' | b')' | b'<' | b'>' | b'[' | b']' | b'{' | b'}' | b'/' | b'%'
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_basic_text_sequence() {
        let bytes = b"BT /F1 12 Tf 72 720 Td (Hello) Tj ET";
        let tokens = tokenize_stream(bytes).unwrap();
        let ops: Vec<&str> = tokens.iter().map(|t| t.operator.as_str()).collect();
        assert_eq!(ops, vec!["BT", "Tf", "Td", "Tj", "ET"]);
        assert_eq!(tokens[0].byte_range.clone(), 0..2);
        assert_eq!(tokens[3].byte_range.clone(), 22..30);
    }

    #[test]
    fn parses_cm_operator() {
        let bytes = b"q 1 0 0 1 0 0 cm Q";
        let tokens = tokenize_stream(bytes).unwrap();
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[1].operator, "cm");
        assert_eq!(tokens[1].operands.len(), 6);
        assert_eq!(tokens[1].byte_range.clone(), 2..17);
    }

    #[test]
    fn parses_tj_and_tj_array() {
        let bytes = b"BT (Hi) Tj [(Hel) 50 (lo)] TJ ET";
        let tokens = tokenize_stream(bytes).unwrap();
        let ops: Vec<&str> = tokens.iter().map(|t| t.operator.as_str()).collect();
        assert_eq!(ops, vec!["BT", "Tj", "TJ", "ET"]);
        assert_eq!(tokens[2].byte_range.clone(), 10..31);
    }

    #[test]
    fn parses_do_operator() {
        let bytes = b"q 0.5 0 0 0.5 10 20 cm /Im1 Do Q";
        let tokens = tokenize_stream(bytes).unwrap();
        assert_eq!(
            tokens
                .iter()
                .map(|t| t.operator.as_str())
                .collect::<Vec<_>>(),
            vec!["q", "cm", "Do", "Q"]
        );
        assert_eq!(tokens[2].byte_range.clone(), 28..36);
    }
}
