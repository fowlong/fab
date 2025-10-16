use std::ops::Range;

use anyhow::{anyhow, bail, Context, Result};

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

impl Operator {
    pub fn from_keyword(keyword: &str) -> Option<Self> {
        match keyword {
            "Q" => Some(Operator::Q),
            "q" => Some(Operator::q),
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

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub stream_obj: u32,
    pub operator: Operator,
    pub operands: Vec<Operand>,
    pub range: Range<u32>,
}

pub fn tokenize_stream(stream_obj: u32, bytes: &[u8]) -> Result<Vec<Token>> {
    let mut tokens = Vec::new();
    let mut index = 0usize;
    let mut operands: Vec<Operand> = Vec::new();
    let mut span_start: Option<usize> = None;

    while index < bytes.len() {
        skip_whitespace(bytes, &mut index);
        if index >= bytes.len() {
            break;
        }

        if bytes[index] == b'%' {
            skip_comment(bytes, &mut index);
            continue;
        }

        let token_start = index;
        match bytes[index] {
            b'[' => {
                let operand = parse_array(bytes, &mut index)?;
                if span_start.is_none() {
                    span_start = Some(token_start);
                }
                operands.push(operand);
            }
            b'(' => {
                let operand = Operand::String(parse_literal_string(bytes, &mut index)?);
                if span_start.is_none() {
                    span_start = Some(token_start);
                }
                operands.push(operand);
            }
            b'<' => {
                if index + 1 < bytes.len() && bytes[index + 1] == b'<' {
                    // dictionary start: treat as unknown and bail to avoid desynchronising parser
                    bail!("dictionary encountered inside content stream");
                }
                let operand = Operand::String(parse_hex_string(bytes, &mut index)?);
                if span_start.is_none() {
                    span_start = Some(token_start);
                }
                operands.push(operand);
            }
            b'/' => {
                let name = parse_name(bytes, &mut index);
                if span_start.is_none() {
                    span_start = Some(token_start);
                }
                operands.push(Operand::Name(name));
            }
            b'+' | b'-' | b'.' | b'0'..=b'9' => {
                let number = parse_number(bytes, &mut index)?;
                if span_start.is_none() {
                    span_start = Some(token_start);
                }
                operands.push(Operand::Number(number));
            }
            _ => {
                let keyword = parse_keyword(bytes, &mut index);
                if let Some(operator) = Operator::from_keyword(&keyword) {
                    let start = span_start.unwrap_or(token_start);
                    let end = index;
                    let token = Token {
                        stream_obj,
                        operator,
                        operands: std::mem::take(&mut operands),
                        range: to_range(start, end)?,
                    };
                    tokens.push(token);
                    span_start = None;
                } else {
                    return Err(anyhow!("unsupported token '{keyword}' in content stream"));
                }
            }
        }
    }

    Ok(tokens)
}

fn skip_whitespace(bytes: &[u8], index: &mut usize) {
    while *index < bytes.len() {
        match bytes[*index] {
            b'\n' | b'\r' | b'\t' | b'\x0C' | b' ' => *index += 1,
            _ => break,
        }
    }
}

fn skip_comment(bytes: &[u8], index: &mut usize) {
    while *index < bytes.len() {
        let b = bytes[*index];
        *index += 1;
        if b == b'\n' || b == b'\r' {
            break;
        }
    }
}

fn parse_keyword(bytes: &[u8], index: &mut usize) -> String {
    let start = *index;
    while *index < bytes.len() {
        let b = bytes[*index];
        if is_delimiter(b) || is_whitespace(b) {
            break;
        }
        *index += 1;
    }
    String::from_utf8_lossy(&bytes[start..*index]).to_string()
}

fn parse_name(bytes: &[u8], index: &mut usize) -> String {
    *index += 1; // skip '/'
    let start = *index;
    while *index < bytes.len() {
        let b = bytes[*index];
        if is_delimiter(b) || is_whitespace(b) {
            break;
        }
        *index += 1;
    }
    String::from_utf8_lossy(&bytes[start..*index]).to_string()
}

fn parse_number(bytes: &[u8], index: &mut usize) -> Result<f64> {
    let start = *index;
    *index += 1;
    while *index < bytes.len() {
        let b = bytes[*index];
        if !(b == b'+' || b == b'-' || b == b'.' || (b'0'..=b'9').contains(&b)) {
            break;
        }
        *index += 1;
    }
    let slice = &bytes[start..*index];
    let number = std::str::from_utf8(slice)
        .context("invalid number token encoding")?
        .parse::<f64>()
        .context("invalid number token")?;
    Ok(number)
}

fn parse_literal_string(bytes: &[u8], index: &mut usize) -> Result<Vec<u8>> {
    *index += 1; // skip '('
    let mut depth = 1usize;
    let mut result = Vec::new();

    while *index < bytes.len() {
        let b = bytes[*index];
        *index += 1;
        match b {
            b'\\' => {
                if *index >= bytes.len() {
                    break;
                }
                let escaped = bytes[*index];
                *index += 1;
                let translated = match escaped {
                    b'n' => b'\n',
                    b'r' => b'\r',
                    b't' => b'\t',
                    b'b' => b'\x08',
                    b'f' => b'\x0C',
                    b'(' => b'(',
                    b')' => b')',
                    b'\\' => b'\\',
                    _ => escaped,
                };
                result.push(translated);
            }
            b'(' => {
                depth += 1;
                result.push(b'(');
            }
            b')' => {
                depth -= 1;
                if depth == 0 {
                    break;
                }
                result.push(b')');
            }
            _ => result.push(b),
        }
    }

    if depth != 0 {
        bail!("unterminated literal string");
    }

    Ok(result)
}

fn parse_hex_string(bytes: &[u8], index: &mut usize) -> Result<Vec<u8>> {
    *index += 1; // skip '<'
    let mut hex = Vec::new();
    while *index < bytes.len() {
        let b = bytes[*index];
        *index += 1;
        if b == b'>' {
            break;
        }
        if is_whitespace(b) {
            continue;
        }
        hex.push(b);
    }
    if hex.len() % 2 != 0 {
        hex.push(b'0');
    }
    let mut result = Vec::new();
    for chunk in hex.chunks(2) {
        let high = char::from(chunk[0]);
        let low = char::from(chunk[1]);
        let byte = u8::from_str_radix(&format!("{high}{low}"), 16)
            .context("invalid hex digit in string")?;
        result.push(byte);
    }
    Ok(result)
}

fn parse_array(bytes: &[u8], index: &mut usize) -> Result<Operand> {
    *index += 1; // skip '['
    let mut values = Vec::new();
    loop {
        skip_whitespace(bytes, index);
        if *index >= bytes.len() {
            bail!("unterminated array");
        }
        if bytes[*index] == b']' {
            *index += 1;
            break;
        }
        let operand = parse_operand(bytes, index)?;
        values.push(operand);
    }
    Ok(Operand::Array(values))
}

fn parse_operand(bytes: &[u8], index: &mut usize) -> Result<Operand> {
    match bytes[*index] {
        b'[' => parse_array(bytes, index),
        b'(' => Ok(Operand::String(parse_literal_string(bytes, index)?)),
        b'<' => Ok(Operand::String(parse_hex_string(bytes, index)?)),
        b'/' => {
            let name = parse_name(bytes, index);
            Ok(Operand::Name(name))
        }
        b'+' | b'-' | b'.' | b'0'..=b'9' => Ok(Operand::Number(parse_number(bytes, index)?)),
        _ => {
            let keyword = parse_keyword(bytes, index);
            if let Some(operator) = Operator::from_keyword(&keyword) {
                // Operators are not valid operands in arrays for our subset.
                Err(anyhow!("unexpected operator {operator:?} inside array"))
            } else {
                Err(anyhow!("unsupported operand token '{keyword}'"))
            }
        }
    }
}

fn is_whitespace(byte: u8) -> bool {
    matches!(byte, b'\n' | b'\r' | b'\t' | b'\x0C' | b' ')
}

fn is_delimiter(byte: u8) -> bool {
    matches!(
        byte,
        b'(' | b')' | b'<' | b'>' | b'[' | b']' | b'{' | b'}' | b'/' | b'%'
    )
}

fn to_range(start: usize, end: usize) -> Result<Range<u32>> {
    let start = u32::try_from(start).context("token start offset exceeds u32")?;
    let end = u32::try_from(end).context("token end offset exceeds u32")?;
    Ok(start..end)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn expect_operator(content: &str) -> Vec<Token> {
        tokenize_stream(5, content.as_bytes()).expect("tokenise")
    }

    #[test]
    fn parses_cm_operator_with_numbers() {
        let tokens = expect_operator("q 1 0 0 1 20 30 cm /Im0 Do Q");
        assert_eq!(tokens.len(), 5);
        assert_eq!(tokens[0].operator, Operator::q);
        assert_eq!(tokens[1].operator, Operator::Cm);
        match &tokens[1].operands[..] {
            [Operand::Number(a), Operand::Number(b), Operand::Number(c), Operand::Number(d), Operand::Number(e), Operand::Number(f)] =>
            {
                assert!((*a - 1.0).abs() < 1e-6);
                assert!((*d - 1.0).abs() < 1e-6);
                assert!((*e - 20.0).abs() < 1e-6);
                assert!((*f - 30.0).abs() < 1e-6);
            }
            _ => panic!("unexpected operands"),
        }
        assert_eq!(tokens[2].operator, Operator::Do);
        assert_eq!(tokens[4].operator, Operator::Q);
        assert!(tokens[1].range.start < tokens[1].range.end);
    }

    #[test]
    fn parses_text_show_sequence() {
        let tokens = expect_operator("BT /F1 12 Tf 72 720 Td (Hello) Tj ET");
        assert_eq!(tokens.len(), 6);
        assert_eq!(tokens[0].operator, Operator::Bt);
        assert_eq!(tokens[1].operator, Operator::Tf);
        assert_eq!(tokens[2].operator, Operator::Td);
        assert_eq!(tokens[3].operator, Operator::Tj);
        assert_eq!(tokens[5].operator, Operator::Et);
        if let Operand::String(bytes) = &tokens[3].operands[0] {
            assert_eq!(bytes, b"Hello");
        } else {
            panic!("expected literal string operand");
        }
    }

    #[test]
    fn parses_tj_array() {
        let tokens = expect_operator("BT (A) Tj [ (B) 30 (C) ] TJ ET");
        assert_eq!(tokens.len(), 5);
        assert_eq!(tokens[3].operator, Operator::TJ);
        if let Operand::Array(entries) = &tokens[3].operands[0] {
            assert_eq!(entries.len(), 3);
        } else {
            panic!("expected array operand");
        }
    }

    #[test]
    fn records_do_operator_range() {
        let tokens = expect_operator("/Im1 Do");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].operator, Operator::Do);
        assert!(tokens[0].range.start < tokens[0].range.end);
        assert_eq!(tokens[0].stream_obj, 5);
    }
}
