//! Content stream tokenisation helpers specialised for the MVP editing flow.

use std::{ops::Range, str};

use anyhow::{anyhow, Result};

/// Represents an operand supplied to a PDF operator.
#[derive(Debug, Clone, PartialEq)]
pub enum Operand {
    Number(f64),
    Name(String),
    String(Vec<u8>),
    Array(Vec<Operand>),
}

/// Supported operators recognised by the tokenizer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Operator {
    Q,
    q,
    Cm,
    BT,
    ET,
    Tf,
    Tm,
    Td,
    TStar,
    Tj,
    TJ,
    Do,
    Other(String),
}

impl Operator {
    fn from_bytes(bytes: &[u8]) -> Option<Self> {
        match bytes {
            b"Q" => Some(Self::Q),
            b"q" => Some(Self::q),
            b"cm" => Some(Self::Cm),
            b"BT" => Some(Self::BT),
            b"ET" => Some(Self::ET),
            b"Tf" => Some(Self::Tf),
            b"Tm" => Some(Self::Tm),
            b"Td" => Some(Self::Td),
            b"T*" => Some(Self::TStar),
            b"Tj" => Some(Self::Tj),
            b"TJ" => Some(Self::TJ),
            b"Do" => Some(Self::Do),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Operator::Q => "Q",
            Operator::q => "q",
            Operator::Cm => "cm",
            Operator::BT => "BT",
            Operator::ET => "ET",
            Operator::Tf => "Tf",
            Operator::Tm => "Tm",
            Operator::Td => "Td",
            Operator::TStar => "T*",
            Operator::Tj => "Tj",
            Operator::TJ => "TJ",
            Operator::Do => "Do",
            Operator::Other(name) => name,
        }
    }
}

/// A token within the content stream.
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub operands: Vec<Operand>,
    pub operator: Operator,
    pub range: Range<usize>,
}

/// Tokenise a PDF content stream into recognised operators with operand ranges.
pub fn tokenize_stream(bytes: &[u8]) -> Result<Vec<Token>> {
    let mut tokens = Vec::new();
    let mut operands: Vec<Operand> = Vec::new();
    let mut span_start: Option<usize> = None;
    let mut index = 0;

    while index < bytes.len() {
        index = skip_whitespace(bytes, index);
        if index >= bytes.len() {
            break;
        }
        match bytes[index] {
            b'%' => {
                index = skip_comment(bytes, index);
            }
            b'(' => {
                let (value, start, end) = parse_string(bytes, index)?;
                span_start.get_or_insert(start);
                operands.push(Operand::String(value));
                index = end;
            }
            b'[' => {
                let (value, start, end) = parse_array(bytes, index)?;
                span_start.get_or_insert(start);
                operands.push(Operand::Array(value));
                index = end;
            }
            b'/' => {
                let (value, start, end) = parse_name(bytes, index)?;
                span_start.get_or_insert(start);
                operands.push(Operand::Name(value));
                index = end;
            }
            _ => {
                let (word, start, end) = parse_word(bytes, index)?;
                index = end;
                if word.is_empty() {
                    continue;
                }
                if let Some(op) = Operator::from_bytes(word.as_bytes()) {
                    let op_start = span_start.unwrap_or(start);
                    let token = Token {
                        operands: std::mem::take(&mut operands),
                        operator: op,
                        range: op_start..end,
                    };
                    tokens.push(token);
                    span_start = None;
                } else if let Ok(number) = parse_number(&word) {
                    span_start.get_or_insert(start);
                    operands.push(Operand::Number(number));
                } else {
                    // Treat as an operator we do not explicitly recognise to avoid
                    // accidentally carrying the operands over to the next instruction.
                    let op_start = span_start.unwrap_or(start);
                    let token = Token {
                        operands: std::mem::take(&mut operands),
                        operator: Operator::Other(word),
                        range: op_start..end,
                    };
                    tokens.push(token);
                    span_start = None;
                }
            }
        }
    }

    Ok(tokens)
}

fn skip_whitespace(bytes: &[u8], mut index: usize) -> usize {
    while index < bytes.len() {
        match bytes[index] {
            b'\x00' | b'\x09' | b'\x0c' | b'\x0a' | b'\x0d' | b' ' => index += 1,
            _ => break,
        }
    }
    index
}

fn skip_comment(bytes: &[u8], mut index: usize) -> usize {
    while index < bytes.len() && bytes[index] != b'\n' && bytes[index] != b'\r' {
        index += 1;
    }
    index
}

fn parse_word(bytes: &[u8], start: usize) -> Result<(String, usize, usize)> {
    let mut index = start;
    while index < bytes.len() {
        let ch = bytes[index];
        if is_delimiter(ch) || is_whitespace(ch) {
            break;
        }
        index += 1;
    }
    let slice = &bytes[start..index];
    let word = str::from_utf8(slice)?.to_string();
    Ok((word, start, index))
}

fn is_delimiter(ch: u8) -> bool {
    matches!(
        ch,
        b'(' | b')' | b'<' | b'>' | b'[' | b']' | b'{' | b'}' | b'/' | b'%'
    )
}

fn is_whitespace(ch: u8) -> bool {
    matches!(ch, b'\x00' | b'\t' | b'\n' | b'\x0c' | b'\r' | b' ')
}

fn parse_number(word: &str) -> Result<f64> {
    if word == "-" {
        return Err(anyhow!("single minus"));
    }
    Ok(word.parse::<f64>()?)
}

fn parse_name(bytes: &[u8], start: usize) -> Result<(String, usize, usize)> {
    let mut index = start + 1; // Skip '/'
    while index < bytes.len() {
        let ch = bytes[index];
        if is_whitespace(ch) || is_delimiter(ch) {
            break;
        }
        index += 1;
    }
    let slice = &bytes[start + 1..index];
    Ok((String::from_utf8(slice.to_vec())?, start, index))
}

fn parse_string(bytes: &[u8], start: usize) -> Result<(Vec<u8>, usize, usize)> {
    let mut index = start + 1;
    let mut nesting = 1i32;
    let mut result = Vec::new();
    while index < bytes.len() {
        let ch = bytes[index];
        if ch == b'(' {
            nesting += 1;
            result.push(ch);
            index += 1;
            continue;
        }
        if ch == b')' {
            nesting -= 1;
            if nesting == 0 {
                index += 1;
                break;
            }
            result.push(ch);
            index += 1;
            continue;
        }
        if ch == b'\\' {
            if index + 1 < bytes.len() {
                let next = bytes[index + 1];
                result.push(match next {
                    b'n' => b'\n',
                    b'r' => b'\r',
                    b't' => b'\t',
                    b'b' => 0x08,
                    b'f' => 0x0c,
                    _ => next,
                });
                index += 2;
                continue;
            }
        }
        result.push(ch);
        index += 1;
    }
    if nesting != 0 {
        return Err(anyhow!("unterminated string"));
    }
    Ok((result, start, index))
}

fn parse_array(bytes: &[u8], start: usize) -> Result<(Vec<Operand>, usize, usize)> {
    let mut index = start + 1;
    let mut values = Vec::new();
    loop {
        index = skip_whitespace(bytes, index);
        if index >= bytes.len() {
            return Err(anyhow!("unterminated array"));
        }
        if bytes[index] == b']' {
            index += 1;
            break;
        }
        let (value, new_index) = parse_operand(bytes, index)?;
        values.push(value);
        index = new_index;
    }
    Ok((values, start, index))
}

fn parse_operand(bytes: &[u8], start: usize) -> Result<(Operand, usize)> {
    match bytes[start] {
        b'(' => {
            let (value, _, end) = parse_string(bytes, start)?;
            Ok((Operand::String(value), end))
        }
        b'[' => {
            let (value, _, end) = parse_array(bytes, start)?;
            Ok((Operand::Array(value), end))
        }
        b'/' => {
            let (value, _, end) = parse_name(bytes, start)?;
            Ok((Operand::Name(value), end))
        }
        _ => {
            let (word, _, end) = parse_word(bytes, start)?;
            if let Ok(number) = parse_number(&word) {
                Ok((Operand::Number(number), end))
            } else {
                Ok((Operand::Name(word), end))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_cm_operator() {
        let data = b"q 1 0 0 1 0 0 cm /Im0 Do Q";
        let tokens = tokenize_stream(data).unwrap();
        let cm = tokens
            .iter()
            .find(|t| matches!(t.operator, Operator::Cm))
            .unwrap();
        assert_eq!(cm.operands.len(), 6);
        assert_eq!(cm.range, 2..17);
    }

    #[test]
    fn parses_bt_et_block() {
        let data = b"BT /F1 12 Tf 10 20 Td (Hello) Tj ET";
        let tokens = tokenize_stream(data).unwrap();
        assert!(tokens.iter().any(|t| matches!(t.operator, Operator::BT)));
        assert!(tokens.iter().any(|t| matches!(t.operator, Operator::ET)));
        let tj = tokens
            .iter()
            .find(|t| matches!(t.operator, Operator::Tj))
            .unwrap();
        assert_eq!(tj.operands.len(), 1);
    }

    #[test]
    fn parses_tj_and_tj_array() {
        let data = b"BT (Hello) Tj [(World) -120 (Again)] TJ ET";
        let tokens = tokenize_stream(data).unwrap();
        let tj = tokens
            .iter()
            .find(|t| matches!(t.operator, Operator::Tj))
            .unwrap();
        assert_eq!(tj.operands.len(), 1);
        let tj_array = tokens
            .iter()
            .find(|t| matches!(t.operator, Operator::TJ))
            .unwrap();
        assert_eq!(tj_array.operands.len(), 1);
        if let Operand::Array(items) = &tj_array.operands[0] {
            assert_eq!(items.len(), 3);
        } else {
            panic!("expected array operand");
        }
    }

    #[test]
    fn parses_do_operator() {
        let data = b"q 200 0 0 150 50 60 cm /Im0 Do Q";
        let tokens = tokenize_stream(data).unwrap();
        let do_token = tokens
            .iter()
            .find(|t| matches!(t.operator, Operator::Do))
            .unwrap();
        assert_eq!(do_token.operands.len(), 1);
    }
}
