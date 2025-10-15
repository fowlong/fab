//! Tokenisation utilities for PDF content streams.
//!
//! The implementation intentionally focuses on the operators required for the
//! stage-two MVP. It recognises graphics state operations (`q`, `Q`, `cm`),
//! text state operators (`BT`, `ET`, `Tf`, `Tm`, `Td`, `T*`, `Tj`, `TJ`) and
//! image painting (`Do`). The parser keeps track of the byte spans of each
//! operator block so that patching can perform surgical edits without having to
//! reserialise the entire stream. The comments use Australian spelling to keep
//! the project consistent.

use std::{borrow::Cow, ops::Range};

use anyhow::{anyhow, Result};

/// A parsed operand that precedes an operator in a content stream.
#[derive(Clone, Debug, PartialEq)]
pub enum Operand {
    Number(f64),
    Name(String),
    String(Vec<u8>),
    Array(Vec<Operand>),
}

/// Supported operators extracted by the MVP parser.
#[derive(Clone, Debug, PartialEq)]
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
}

/// A token consisting of the operator, its operands and the byte span occupied
/// by the full instruction.
#[derive(Clone, Debug, PartialEq)]
pub struct ContentToken {
    pub operator: Operator,
    pub operands: Vec<Operand>,
    pub byte_range: Range<usize>,
}

/// Tokenise a PDF content stream into a sequence of [`ContentToken`] values.
///
/// The parser is intentionally lenient: unrecognised operators simply clear the
/// operand buffer so that subsequent instructions continue to parse correctly.
/// This keeps the stage-two pipeline resilient when PDFs contain drawing
/// commands outside the MVP scope.
pub fn tokenize_stream(bytes: &[u8]) -> Result<Vec<ContentToken>> {
    let mut cursor = 0usize;
    let mut tokens = Vec::new();
    let mut operands: Vec<Operand> = Vec::new();
    let mut operands_start: Option<usize> = None;

    while cursor < bytes.len() {
        skip_whitespace(bytes, &mut cursor);
        if cursor >= bytes.len() {
            break;
        }

        if bytes[cursor] == b'%' {
            skip_comment(bytes, &mut cursor);
            continue;
        }

        let start = cursor;
        match bytes[cursor] {
            b'(' => {
                let (value, end) = parse_literal_string(bytes, cursor)?;
                cursor = end;
                if operands_start.is_none() {
                    operands_start = Some(start);
                }
                operands.push(Operand::String(value));
            }
            b'<' => {
                if cursor + 1 < bytes.len() && bytes[cursor + 1] == b'<' {
                    // Dictionary or inline image start – unsupported for now.
                    // Skip token to avoid mis-parsing subsequent instructions.
                    let (_, end) = parse_word(bytes, cursor)?;
                    cursor = end;
                    operands.clear();
                    operands_start = None;
                } else {
                    let (value, end) = parse_hex_string(bytes, cursor)?;
                    cursor = end;
                    if operands_start.is_none() {
                        operands_start = Some(start);
                    }
                    operands.push(Operand::String(value));
                }
            }
            b'[' => {
                let (value, end) = parse_array(bytes, cursor)?;
                cursor = end;
                if operands_start.is_none() {
                    operands_start = Some(start);
                }
                operands.push(Operand::Array(value));
            }
            b'/' => {
                let (name, end) = parse_name(bytes, cursor)?;
                cursor = end;
                if operands_start.is_none() {
                    operands_start = Some(start);
                }
                operands.push(Operand::Name(name.into_owned()));
            }
            _ => {
                let (word, end) = parse_word(bytes, cursor)?;
                cursor = end;
                if let Some(op) = to_operator(&word) {
                    let op_start = operands_start.unwrap_or(start);
                    let range = op_start..cursor;
                    tokens.push(ContentToken {
                        operator: op,
                        operands: std::mem::take(&mut operands),
                        byte_range: range,
                    });
                    operands_start = None;
                } else if let Ok(number) = word.parse::<f64>() {
                    if operands_start.is_none() {
                        operands_start = Some(start);
                    }
                    operands.push(Operand::Number(number));
                } else {
                    // Unknown token; clear operand buffer to remain synchronised.
                    operands.clear();
                    operands_start = None;
                }
            }
        }
    }

    Ok(tokens)
}

fn to_operator(word: &str) -> Option<Operator> {
    match word {
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
        "TJ" => Some(Operator::TjArray),
        "Do" => Some(Operator::Do),
        _ => None,
    }
}

fn skip_whitespace(bytes: &[u8], cursor: &mut usize) {
    while *cursor < bytes.len() {
        let b = bytes[*cursor];
        if matches!(b, b' ' | b'\t' | b'\r' | b'\n' | b'\x0c' | b'\x00') {
            *cursor += 1;
        } else {
            break;
        }
    }
}

fn skip_comment(bytes: &[u8], cursor: &mut usize) {
    while *cursor < bytes.len() {
        let b = bytes[*cursor];
        *cursor += 1;
        if matches!(b, b'\r' | b'\n') {
            break;
        }
    }
}

fn parse_word(bytes: &[u8], start: usize) -> Result<(String, usize)> {
    let mut end = start;
    while end < bytes.len() {
        let b = bytes[end];
        if is_delimiter(b) {
            break;
        }
        end += 1;
    }
    if end == start {
        return Err(anyhow!("expected word token"));
    }
    let slice = &bytes[start..end];
    let word = std::str::from_utf8(slice)?.to_string();
    Ok((word, end))
}

fn parse_name<'a>(bytes: &'a [u8], start: usize) -> Result<(Cow<'a, str>, usize)> {
    if bytes.get(start) != Some(&b'/') {
        return Err(anyhow!("expected name"));
    }
    let mut end = start + 1;
    while end < bytes.len() {
        let b = bytes[end];
        if is_delimiter(b) {
            break;
        }
        end += 1;
    }
    if end == start + 1 {
        return Err(anyhow!("empty name"));
    }
    let slice = &bytes[start + 1..end];
    Ok((Cow::Owned(std::str::from_utf8(slice)?.to_string()), end))
}

fn parse_literal_string(bytes: &[u8], start: usize) -> Result<(Vec<u8>, usize)> {
    if bytes.get(start) != Some(&b'(') {
        return Err(anyhow!("expected literal string"));
    }
    let mut result = Vec::new();
    let mut cursor = start + 1;
    let mut depth = 1;

    while cursor < bytes.len() {
        let b = bytes[cursor];
        cursor += 1;
        match b {
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
            b'\\' => {
                if cursor >= bytes.len() {
                    return Err(anyhow!("unterminated escape in string"));
                }
                let escaped = bytes[cursor];
                cursor += 1;
                match escaped {
                    b'n' => result.push(b'\n'),
                    b'r' => result.push(b'\r'),
                    b't' => result.push(b'\t'),
                    b'b' => result.push(b'\x08'),
                    b'f' => result.push(b'\x0c'),
                    b'(' | b')' | b'\\' => result.push(escaped),
                    b'0'..=b'7' => {
                        let mut value = (escaped - b'0') as u16;
                        for _ in 0..2 {
                            if cursor >= bytes.len() {
                                break;
                            }
                            let next = bytes[cursor];
                            if !(b'0'..=b'7').contains(&next) {
                                break;
                            }
                            cursor += 1;
                            value = (value << 3) + (next - b'0') as u16;
                        }
                        result.push((value & 0xFF) as u8);
                    }
                    b'\r' => {
                        if cursor < bytes.len() && bytes[cursor] == b'\n' {
                            cursor += 1;
                        }
                    }
                    b'\n' => {}
                    other => result.push(other),
                }
            }
            other => result.push(other),
        }
    }

    if depth != 0 {
        return Err(anyhow!("unterminated literal string"));
    }

    Ok((result, cursor))
}

fn parse_hex_string(bytes: &[u8], start: usize) -> Result<(Vec<u8>, usize)> {
    if bytes.get(start) != Some(&b'<') {
        return Err(anyhow!("expected hex string"));
    }
    let mut cursor = start + 1;
    let mut hex_bytes = Vec::new();
    while cursor < bytes.len() {
        let b = bytes[cursor];
        cursor += 1;
        if b == b'>' {
            break;
        }
        if matches!(b, b' ' | b'\t' | b'\r' | b'\n' | b'\x0c' | b'\x00') {
            continue;
        }
        hex_bytes.push(b);
    }

    if hex_bytes.is_empty() {
        return Ok((Vec::new(), cursor));
    }

    if hex_bytes.len() % 2 == 1 {
        hex_bytes.push(b'0');
    }

    let mut result = Vec::with_capacity(hex_bytes.len() / 2);
    for pair in hex_bytes.chunks(2) {
        let high = hex_value(pair[0])?;
        let low = hex_value(pair[1])?;
        result.push((high << 4) | low);
    }
    Ok((result, cursor))
}

fn parse_array(bytes: &[u8], start: usize) -> Result<(Vec<Operand>, usize)> {
    if bytes.get(start) != Some(&b'[') {
        return Err(anyhow!("expected array"));
    }
    let mut cursor = start + 1;
    let mut items = Vec::new();
    loop {
        skip_whitespace(bytes, &mut cursor);
        if cursor >= bytes.len() {
            return Err(anyhow!("unterminated array"));
        }
        if bytes[cursor] == b']' {
            cursor += 1;
            break;
        }
        if bytes[cursor] == b'%' {
            skip_comment(bytes, &mut cursor);
            continue;
        }
        match bytes[cursor] {
            b'(' => {
                let (value, end) = parse_literal_string(bytes, cursor)?;
                cursor = end;
                items.push(Operand::String(value));
            }
            b'<' => {
                if cursor + 1 < bytes.len() && bytes[cursor + 1] == b'<' {
                    let (_, end) = parse_word(bytes, cursor)?;
                    cursor = end;
                } else {
                    let (value, end) = parse_hex_string(bytes, cursor)?;
                    cursor = end;
                    items.push(Operand::String(value));
                }
            }
            b'[' => {
                let (value, end) = parse_array(bytes, cursor)?;
                cursor = end;
                items.push(Operand::Array(value));
            }
            b'/' => {
                let (name, end) = parse_name(bytes, cursor)?;
                cursor = end;
                items.push(Operand::Name(name.into_owned()));
            }
            _ => {
                let (word, end) = parse_word(bytes, cursor)?;
                cursor = end;
                if let Ok(number) = word.parse::<f64>() {
                    items.push(Operand::Number(number));
                } else {
                    items.push(Operand::Name(word));
                }
            }
        }
    }

    Ok((items, cursor))
}

fn is_delimiter(byte: u8) -> bool {
    matches!(
        byte,
        b' ' | b'\t'
            | b'\r'
            | b'\n'
            | b'\x0c'
            | b'\x00'
            | b'('
            | b')'
            | b'<'
            | b'>'
            | b'['
            | b']'
            | b'{'
            | b'}'
            | b'/'
            | b'%'
    )
}

fn hex_value(byte: u8) -> Result<u8> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        b'A'..=b'F' => Ok(byte - b'A' + 10),
        _ => Err(anyhow!("invalid hex digit")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn number(n: f64) -> Operand {
        Operand::Number(n)
    }

    fn name(n: &str) -> Operand {
        Operand::Name(n.into())
    }

    fn lit(s: &str) -> Operand {
        Operand::String(s.as_bytes().to_vec())
    }

    #[test]
    fn parses_cm_operation() {
        let input = b"1 0 0 1 10 20 cm\n";
        let tokens = tokenize_stream(input).unwrap();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].operator, Operator::Cm);
        assert_eq!(
            tokens[0].operands,
            vec![
                number(1.0),
                number(0.0),
                number(0.0),
                number(1.0),
                number(10.0),
                number(20.0)
            ]
        );
        assert_eq!(tokens[0].byte_range, 0..input.len());
    }

    #[test]
    fn parses_bt_et_block_with_tj() {
        let input = b"BT /F1 12 Tf 100 200 Td (Hello) Tj ET";
        let tokens = tokenize_stream(input).unwrap();
        assert_eq!(tokens.len(), 6);
        assert_eq!(tokens[0].operator, Operator::Bt);
        assert_eq!(tokens[1].operator, Operator::Tf);
        assert_eq!(tokens[2].operator, Operator::Td);
        assert_eq!(tokens[3].operator, Operator::Tj);
        assert_eq!(tokens[3].operands, vec![lit("Hello")]);
        assert_eq!(tokens[4].operator, Operator::Et);
    }

    #[test]
    fn parses_tj_array() {
        let input = b"BT [ (Hi) 120 (There) ] TJ ET";
        let tokens = tokenize_stream(input).unwrap();
        assert_eq!(tokens.len(), 4);
        assert_eq!(tokens[1].operator, Operator::TjArray);
        match &tokens[1].operands[0] {
            Operand::Array(items) => {
                assert_eq!(items.len(), 3);
            }
            other => panic!("unexpected operand: {other:?}"),
        }
    }

    #[test]
    fn parses_do_operation() {
        let input = b"q 200 0 0 100 50 80 cm /Im0 Do Q";
        let tokens = tokenize_stream(input).unwrap();
        assert_eq!(tokens.len(), 5);
        assert_eq!(tokens[2].operator, Operator::Do);
        assert_eq!(tokens[2].operands, vec![name("Im0")]);
    }
}
