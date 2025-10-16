//! Content stream tokenisation helpers.

use std::ops::Range;

use anyhow::{anyhow, Result};

/// Operand types supported by the lightweight tokenizer.
#[derive(Debug, Clone, PartialEq)]
pub enum Operand {
    Number(f64),
    Name(String),
    String(Vec<u8>),
    Array(Vec<Operand>),
}

/// Supported PDF operators for the MVP editor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operator {
    Save,          // q
    Restore,       // Q
    Concat,        // cm
    BeginText,     // BT
    EndText,       // ET
    SetFont,       // Tf
    SetTextMatrix, // Tm
    TextMove,      // Td
    TextNextLine,  // T*
    ShowText,      // Tj
    ShowTextArray, // TJ
    InvokeXObject, // Do
}

/// Token carrying operands, the operator, and byte span within the content stream.
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub operands: Vec<Operand>,
    pub operator: Operator,
    pub span: Range<usize>,
}

/// Tokenise a PDF content stream covering a subset of operators required for
/// transform editing. The tokenizer keeps other bytes untouched by tracking
/// the exact byte span of each recognised operator.
pub fn tokenize_stream(bytes: &[u8]) -> Result<Vec<Token>> {
    let mut tokens = Vec::new();
    let mut pos = 0usize;
    let mut operands = Vec::new();
    let mut operands_start: Option<usize> = None;

    while pos < bytes.len() {
        skip_whitespace(bytes, &mut pos);
        if pos >= bytes.len() {
            break;
        }
        if bytes[pos] == b'%' {
            skip_comment(bytes, &mut pos);
            continue;
        }

        let start = pos;
        match bytes[pos] {
            b'[' => {
                let (array, end) = parse_array(bytes, pos + 1)?;
                operands_start.get_or_insert(start);
                operands.push(Operand::Array(array));
                pos = end;
            }
            b'(' => {
                let (string, end) = parse_string(bytes, pos + 1)?;
                operands_start.get_or_insert(start);
                operands.push(Operand::String(string));
                pos = end;
            }
            b'/' => {
                let (name, end) = read_name(bytes, pos + 1);
                operands_start.get_or_insert(start);
                operands.push(Operand::Name(name));
                pos = end;
            }
            b'-' | b'+' | b'.' | b'0'..=b'9' => {
                let (value, end) = read_number(bytes, pos)?;
                operands_start.get_or_insert(start);
                operands.push(Operand::Number(value));
                pos = end;
            }
            _ => {
                let (word, end) = read_keyword(bytes, pos);
                if let Some(op) = match_operator(word.as_slice()) {
                    let span_start = operands_start.unwrap_or(start);
                    tokens.push(Token {
                        operands: std::mem::take(&mut operands),
                        operator: op,
                        span: span_start..end,
                    });
                    operands_start = None;
                    pos = end;
                } else if !word.is_empty() {
                    // Treat as bare name operand when not matching an operator.
                    let name = String::from_utf8(word)
                        .map_err(|err| anyhow!("invalid name token: {err}"))?;
                    operands_start.get_or_insert(start);
                    operands.push(Operand::Name(name));
                    pos = end;
                } else {
                    // Unrecognised byte; advance to avoid infinite loop.
                    pos += 1;
                }
            }
        }
    }

    Ok(tokens)
}

fn skip_whitespace(bytes: &[u8], pos: &mut usize) {
    while *pos < bytes.len() {
        match bytes[*pos] {
            b'\x00' | b'\t' | b'\n' | b'\r' | b'\x0C' | b' ' => *pos += 1,
            _ => break,
        }
    }
}

fn skip_comment(bytes: &[u8], pos: &mut usize) {
    while *pos < bytes.len() {
        let b = bytes[*pos];
        *pos += 1;
        if b == b'\n' || b == b'\r' {
            break;
        }
    }
}

fn parse_array(bytes: &[u8], mut pos: usize) -> Result<(Vec<Operand>, usize)> {
    let mut items = Vec::new();
    while pos < bytes.len() {
        skip_whitespace(bytes, &mut pos);
        if pos >= bytes.len() {
            return Err(anyhow!("unterminated array"));
        }
        match bytes[pos] {
            b']' => {
                return Ok((items, pos + 1));
            }
            b'[' => {
                let (nested, end) = parse_array(bytes, pos + 1)?;
                items.push(Operand::Array(nested));
                pos = end;
            }
            b'(' => {
                let (string, end) = parse_string(bytes, pos + 1)?;
                items.push(Operand::String(string));
                pos = end;
            }
            b'/' => {
                let (name, end) = read_name(bytes, pos + 1);
                items.push(Operand::Name(name));
                pos = end;
            }
            b'-' | b'+' | b'.' | b'0'..=b'9' => {
                let (value, end) = read_number(bytes, pos)?;
                items.push(Operand::Number(value));
                pos = end;
            }
            _ => {
                let (word, end) = read_keyword(bytes, pos);
                if word.is_empty() {
                    return Err(anyhow!("invalid token inside array"));
                }
                let name =
                    String::from_utf8(word).map_err(|err| anyhow!("invalid name token: {err}"))?;
                items.push(Operand::Name(name));
                pos = end;
            }
        }
    }
    Err(anyhow!("unterminated array"))
}

fn parse_string(bytes: &[u8], mut pos: usize) -> Result<(Vec<u8>, usize)> {
    let mut depth = 1i32;
    let mut out = Vec::new();
    while pos < bytes.len() {
        let b = bytes[pos];
        pos += 1;
        match b {
            b'\\' => {
                if pos < bytes.len() {
                    out.push(bytes[pos]);
                    pos += 1;
                }
            }
            b'(' => {
                depth += 1;
                out.push(b);
            }
            b')' => {
                depth -= 1;
                if depth == 0 {
                    return Ok((out, pos));
                }
                out.push(b);
            }
            _ => out.push(b),
        }
    }
    Err(anyhow!("unterminated string"))
}

fn read_name(bytes: &[u8], mut pos: usize) -> (String, usize) {
    let start = pos.saturating_sub(1);
    let mut out = Vec::new();
    if pos > 0 {
        out.push(bytes[pos - 1]);
    }
    while pos < bytes.len() {
        let b = bytes[pos];
        if is_delimiter(b) {
            break;
        }
        out.push(b);
        pos += 1;
    }
    let name_bytes = &out[1..];
    let name = String::from_utf8_lossy(name_bytes).to_string();
    (name, pos)
}

fn read_number(bytes: &[u8], mut pos: usize) -> Result<(f64, usize)> {
    let start = pos;
    while pos < bytes.len() {
        let b = bytes[pos];
        if matches!(b, b'+' | b'-' | b'.' | b'0'..=b'9') {
            pos += 1;
        } else {
            break;
        }
    }
    let token = std::str::from_utf8(&bytes[start..pos])?;
    let value: f64 = token.parse()?;
    Ok((value, pos))
}

fn read_keyword(bytes: &[u8], mut pos: usize) -> (Vec<u8>, usize) {
    let start = pos;
    while pos < bytes.len() {
        let b = bytes[pos];
        if is_delimiter(b) {
            break;
        }
        pos += 1;
    }
    (bytes[start..pos].to_vec(), pos)
}

fn is_delimiter(b: u8) -> bool {
    matches!(
        b,
        b'\x00'
            | b'\t'
            | b'\n'
            | b'\r'
            | b'\x0C'
            | b' '
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

fn match_operator(bytes: &[u8]) -> Option<Operator> {
    match bytes {
        b"q" => Some(Operator::Save),
        b"Q" => Some(Operator::Restore),
        b"cm" => Some(Operator::Concat),
        b"BT" => Some(Operator::BeginText),
        b"ET" => Some(Operator::EndText),
        b"Tf" => Some(Operator::SetFont),
        b"Tm" => Some(Operator::SetTextMatrix),
        b"Td" => Some(Operator::TextMove),
        b"T*" => Some(Operator::TextNextLine),
        b"Tj" => Some(Operator::ShowText),
        b"TJ" => Some(Operator::ShowTextArray),
        b"Do" => Some(Operator::InvokeXObject),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn span(token: &Token) -> (usize, usize) {
        (token.span.start, token.span.end)
    }

    #[test]
    fn parse_cm_sequence() {
        let input = b"q 1 0 0 1 50 60 cm /Im1 Do Q";
        let tokens = tokenize_stream(input).expect("tokenise cm");
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0].operator, Operator::Save);
        assert_eq!(tokens[1].operator, Operator::Concat);
        assert_eq!(tokens[1].operands.len(), 6);
        assert_eq!(span(&tokens[1]), (2, 21));
        assert_eq!(tokens[2].operator, Operator::InvokeXObject);
    }

    #[test]
    fn parse_bt_tj_block() {
        let input = b"BT /F1 12 Tf 10 20 Td (Hello) Tj ET";
        let tokens = tokenize_stream(input).expect("tokenise bt/tj");
        assert_eq!(tokens.len(), 5);
        assert_eq!(tokens[0].operator, Operator::BeginText);
        assert_eq!(tokens[1].operator, Operator::SetFont);
        assert_eq!(tokens[2].operator, Operator::TextMove);
        assert_eq!(tokens[3].operator, Operator::ShowText);
        assert_eq!(tokens[4].operator, Operator::EndText);
        assert_eq!(span(&tokens[3]), (23, 32));
    }

    #[test]
    fn parse_tj_array() {
        let input = b"BT [ (A) 50 (B) ] TJ ET";
        let tokens = tokenize_stream(input).expect("tokenise TJ");
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[1].operator, Operator::ShowTextArray);
        match &tokens[1].operands[0] {
            Operand::Array(items) => {
                assert_eq!(items.len(), 3);
            }
            other => panic!("unexpected operand: {other:?}"),
        }
    }

    #[test]
    fn parse_do_operator() {
        let input = b"/Logo Do";
        let tokens = tokenize_stream(input).expect("tokenise Do");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].operator, Operator::InvokeXObject);
        assert_eq!(tokens[0].operands.len(), 1);
    }
}
