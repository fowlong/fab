//! Content stream tokenisation helpers covering a limited PDF operator set.

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
    BT,
    ET,
    Tf,
    Tm,
    Td,
    TStar,
    Tj,
    TJ,
    Do,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub operator: Operator,
    pub operands: Vec<Operand>,
    pub start: usize,
    pub end: usize,
}

pub fn tokenize_stream(bytes: &[u8]) -> Result<Vec<Token>> {
    let mut tokens = Vec::new();
    let mut i = 0;
    let len = bytes.len();
    let mut operands: Vec<(Operand, usize, usize)> = Vec::new();
    while i < len {
        skip_ws(bytes, &mut i);
        if i >= len {
            break;
        }
        if bytes[i] == b'%' {
            skip_comment(bytes, &mut i);
            continue;
        }
        let start = i;
        match bytes[i] {
            b'/' => {
                let (name, end) = parse_name(bytes, i)?;
                operands.push((Operand::Name(name), start, end));
                i = end;
            }
            b'(' => {
                let (string, end) = parse_string(bytes, i)?;
                operands.push((Operand::String(string), start, end));
                i = end;
            }
            b'[' => {
                let (array, end) = parse_array(bytes, i)?;
                operands.push((Operand::Array(array), start, end));
                i = end;
            }
            b'-' | b'+' | b'.' | b'0'..=b'9' => {
                let (number, end) = parse_number(bytes, i)?;
                operands.push((Operand::Number(number), start, end));
                i = end;
            }
            _ => {
                let (word, end) = parse_word(bytes, i);
                if let Some(op) = classify_operator(&word) {
                    let token_start = operands.first().map(|(_, s, _)| *s).unwrap_or(start);
                    let token_end = end;
                    let operands_vec = operands.into_iter().map(|(op, _, _)| op).collect();
                    tokens.push(Token {
                        operator: op,
                        operands: operands_vec,
                        start: token_start,
                        end: token_end,
                    });
                    operands = Vec::new();
                }
                i = end;
            }
        }
    }
    Ok(tokens)
}

fn skip_ws(bytes: &[u8], i: &mut usize) {
    while *i < bytes.len() {
        match bytes[*i] {
            b'\n' | b'\r' | b'\t' | b'\x0c' | b'\x00' | b' ' => *i += 1,
            _ => break,
        }
    }
}

fn skip_comment(bytes: &[u8], i: &mut usize) {
    while *i < bytes.len() && bytes[*i] != b'\n' {
        *i += 1;
    }
}

fn parse_word(bytes: &[u8], mut i: usize) -> (String, usize) {
    let start = i;
    while i < bytes.len() {
        let b = bytes[i];
        if matches!(b, b'\n' | b'\r' | b'\t' | b'\x0c' | b'\x00' | b' ')
            || b == b'%'
            || b == b'['
            || b == b']'
            || b == b'<'
            || b == b'>'
            || b == b'('
            || b == b')'
        {
            break;
        }
        i += 1;
    }
    (String::from_utf8_lossy(&bytes[start..i]).into_owned(), i)
}

fn parse_name(bytes: &[u8], mut i: usize) -> Result<(String, usize)> {
    i += 1; // skip '/'
    let start = i;
    while i < bytes.len() {
        let b = bytes[i];
        if matches!(b, b'\n' | b'\r' | b'\t' | b'\x0c' | b'\x00' | b' ')
            || b == b'/'
            || b == b'['
            || b == b']'
            || b == b'<'
            || b == b'>'
            || b == b'('
            || b == b')'
        {
            break;
        }
        i += 1;
    }
    if start == i {
        return Err(anyhow!("empty name token"));
    }
    Ok((String::from_utf8_lossy(&bytes[start..i]).into_owned(), i))
}

fn parse_string(bytes: &[u8], mut i: usize) -> Result<(Vec<u8>, usize)> {
    i += 1; // skip '('
    let mut depth = 1;
    let mut result = Vec::new();
    while i < bytes.len() {
        let b = bytes[i];
        if b == b'\\' {
            if i + 1 < bytes.len() {
                result.push(bytes[i + 1]);
                i += 2;
                continue;
            } else {
                return Err(anyhow!("unterminated escape sequence"));
            }
        }
        if b == b'(' {
            depth += 1;
        } else if b == b')' {
            depth -= 1;
            if depth == 0 {
                i += 1;
                break;
            }
        }
        result.push(b);
        i += 1;
    }
    if depth != 0 {
        return Err(anyhow!("unterminated string"));
    }
    Ok((result, i))
}

fn parse_array(bytes: &[u8], mut i: usize) -> Result<(Vec<Operand>, usize)> {
    i += 1; // skip '['
    let mut values = Vec::new();
    while i < bytes.len() {
        skip_ws(bytes, &mut i);
        if i >= bytes.len() {
            break;
        }
        if bytes[i] == b']' {
            i += 1;
            break;
        }
        if bytes[i] == b'%' {
            skip_comment(bytes, &mut i);
            continue;
        }
        let start = i;
        let value = match bytes[i] {
            b'/' => {
                let (name, end) = parse_name(bytes, i)?;
                i = end;
                Operand::Name(name)
            }
            b'(' => {
                let (string, end) = parse_string(bytes, i)?;
                i = end;
                Operand::String(string)
            }
            b'[' => {
                let (array, end) = parse_array(bytes, i)?;
                i = end;
                Operand::Array(array)
            }
            b'-' | b'+' | b'.' | b'0'..=b'9' => {
                let (number, end) = parse_number(bytes, i)?;
                i = end;
                Operand::Number(number)
            }
            _ => {
                let (word, end) = parse_word(bytes, i);
                i = end;
                Operand::Name(word)
            }
        };
        if i == start {
            return Err(anyhow!("array parsing stalled"));
        }
        values.push(value);
    }
    Ok((values, i))
}

fn parse_number(bytes: &[u8], mut i: usize) -> Result<(f64, usize)> {
    let start = i;
    if bytes[i] == b'+' || bytes[i] == b'-' {
        i += 1;
    }
    while i < bytes.len() && (bytes[i].is_ascii_digit()) {
        i += 1;
    }
    if i < bytes.len() && bytes[i] == b'.' {
        i += 1;
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            i += 1;
        }
    }
    if start == i {
        return Err(anyhow!("invalid number"));
    }
    let s = std::str::from_utf8(&bytes[start..i])?;
    let value: f64 = s.parse()?;
    Ok((value, i))
}

fn classify_operator(word: &str) -> Option<Operator> {
    match word {
        "q" => Some(Operator::q),
        "Q" => Some(Operator::Q),
        "cm" => Some(Operator::Cm),
        "BT" => Some(Operator::BT),
        "ET" => Some(Operator::ET),
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

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "q 1 0 0 1 0 0 cm BT /F1 12 Tf 10 20 Td (Hello) Tj ET Q";

    #[test]
    fn tokenises_cm_and_bt() {
        let tokens = tokenize_stream(SAMPLE.as_bytes()).unwrap();
        assert!(tokens.iter().any(|t| matches!(t.operator, Operator::Cm)));
        assert!(tokens.iter().any(|t| matches!(t.operator, Operator::BT)));
        assert!(tokens.iter().any(|t| matches!(t.operator, Operator::ET)));
    }

    #[test]
    fn tokenises_tj_and_tj_array() {
        let bytes = b"BT (Hi) Tj [(A) -20 (B)] TJ ET";
        let tokens = tokenize_stream(bytes).unwrap();
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
    }

    #[test]
    fn tokenises_do_with_span() {
        let bytes = b"q 200 0 0 100 10 10 cm /Im1 Do Q";
        let tokens = tokenize_stream(bytes).unwrap();
        let do_token = tokens
            .iter()
            .find(|t| matches!(t.operator, Operator::Do))
            .unwrap();
        assert!(do_token.start < do_token.end);
    }
}
