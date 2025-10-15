//! Content stream tokenisation helpers geared towards the MVP feature-set.

use std::ops::Range;

use anyhow::{anyhow, Result};

/// A recognised PDF operator emitted by the tokenizer.
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
    fn try_from_ident(ident: &str) -> Option<Self> {
        match ident {
            "Q" => Some(Self::Q),
            "q" => Some(Self::q),
            "cm" => Some(Self::Cm),
            "BT" => Some(Self::Bt),
            "ET" => Some(Self::Et),
            "Tf" => Some(Self::Tf),
            "Tm" => Some(Self::Tm),
            "Td" => Some(Self::Td),
            "T*" => Some(Self::TStar),
            "Tj" => Some(Self::Tj),
            "TJ" => Some(Self::TJ),
            "Do" => Some(Self::Do),
            _ => None,
        }
    }
}

/// An operand appearing before an operator.
#[derive(Debug, Clone, PartialEq)]
pub enum Operand {
    Number(f64),
    Name(String),
    String(String),
    Array(Vec<Operand>),
}

/// A token representing an operator and its operands.
#[derive(Debug, Clone, PartialEq)]
pub struct ContentToken {
    pub operator: Operator,
    pub operands: Vec<Operand>,
    pub span: Range<usize>,
}

/// Tokenise a content stream into recognised operator tokens.
pub fn tokenize_stream(bytes: &[u8]) -> Result<Vec<ContentToken>> {
    let mut tokens = Vec::new();
    let mut idx = 0usize;
    let mut pending_operands: Vec<Operand> = Vec::new();
    let mut current_start: Option<usize> = None;

    while idx < bytes.len() {
        skip_whitespace(bytes, &mut idx);
        if idx >= bytes.len() {
            break;
        }

        if bytes[idx] == b'%' {
            skip_comment(bytes, &mut idx);
            continue;
        }

        let token_start = idx;
        let c = bytes[idx];
        if c == b'(' {
            idx += 1;
            let string = parse_literal_string(bytes, &mut idx)?;
            pending_operands.push(Operand::String(string));
            current_start.get_or_insert(token_start);
            continue;
        }
        if c == b'[' {
            idx += 1;
            let array = parse_array(bytes, &mut idx)?;
            pending_operands.push(Operand::Array(array));
            current_start.get_or_insert(token_start);
            continue;
        }
        if c == b'/' {
            idx += 1;
            let name = parse_name(bytes, &mut idx);
            pending_operands.push(Operand::Name(name));
            current_start.get_or_insert(token_start);
            continue;
        }
        if is_number_start(c) {
            let number = parse_number(bytes, &mut idx)?;
            pending_operands.push(Operand::Number(number));
            current_start.get_or_insert(token_start);
            continue;
        }

        let ident = parse_ident(bytes, &mut idx);
        if let Some(op) = Operator::try_from_ident(&ident) {
            let start = current_start.unwrap_or(token_start);
            let span = start..idx;
            tokens.push(ContentToken {
                operator: op,
                operands: std::mem::take(&mut pending_operands),
                span,
            });
            current_start = None;
        } else if !ident.is_empty() {
            // Unknown keyword – clear any pending operands to avoid them
            // leaking into subsequent operators.
            pending_operands.clear();
            current_start = None;
        }
    }

    Ok(tokens)
}

fn parse_array(bytes: &[u8], idx: &mut usize) -> Result<Vec<Operand>> {
    let mut items = Vec::new();
    loop {
        skip_whitespace(bytes, idx);
        if *idx >= bytes.len() {
            return Err(anyhow!("unterminated array"));
        }
        if bytes[*idx] == b']' {
            *idx += 1;
            break;
        }
        if bytes[*idx] == b'%' {
            skip_comment(bytes, idx);
            continue;
        }
        let c = bytes[*idx];
        let operand = if c == b'(' {
            *idx += 1;
            Operand::String(parse_literal_string(bytes, idx)?)
        } else if c == b'[' {
            *idx += 1;
            Operand::Array(parse_array(bytes, idx)?)
        } else if c == b'/' {
            *idx += 1;
            Operand::Name(parse_name(bytes, idx))
        } else if is_number_start(c) {
            Operand::Number(parse_number(bytes, idx)?)
        } else {
            Operand::String(parse_ident(bytes, idx))
        };
        items.push(operand);
    }
    Ok(items)
}

fn parse_literal_string(bytes: &[u8], idx: &mut usize) -> Result<String> {
    let mut depth = 1i32;
    let mut result = String::new();
    while *idx < bytes.len() {
        let ch = bytes[*idx];
        *idx += 1;
        match ch {
            b'\\' => {
                if *idx < bytes.len() {
                    let escaped = bytes[*idx] as char;
                    *idx += 1;
                    result.push(escaped);
                }
            }
            b'(' => {
                depth += 1;
                result.push('(');
            }
            b')' => {
                depth -= 1;
                if depth == 0 {
                    break;
                }
                result.push(')');
            }
            _ => result.push(ch as char),
        }
    }
    if depth != 0 {
        return Err(anyhow!("unterminated literal string"));
    }
    Ok(result)
}

fn parse_name(bytes: &[u8], idx: &mut usize) -> String {
    let mut name = String::new();
    while *idx < bytes.len() {
        let ch = bytes[*idx];
        if is_delimiter(ch) || ch.is_ascii_whitespace() {
            break;
        }
        name.push(ch as char);
        *idx += 1;
    }
    name
}

fn parse_ident(bytes: &[u8], idx: &mut usize) -> String {
    let mut ident = String::new();
    while *idx < bytes.len() {
        let ch = bytes[*idx];
        if ch.is_ascii_whitespace() || is_delimiter(ch) {
            break;
        }
        ident.push(ch as char);
        *idx += 1;
    }
    ident
}

fn parse_number(bytes: &[u8], idx: &mut usize) -> Result<f64> {
    let start = *idx;
    if bytes[*idx] == b'+' || bytes[*idx] == b'-' {
        *idx += 1;
    }
    while *idx < bytes.len() && bytes[*idx].is_ascii_digit() {
        *idx += 1;
    }
    if *idx < bytes.len() && bytes[*idx] == b'.' {
        *idx += 1;
        while *idx < bytes.len() && bytes[*idx].is_ascii_digit() {
            *idx += 1;
        }
    }
    let slice = &bytes[start..*idx];
    let text = std::str::from_utf8(slice)?;
    text.parse::<f64>().map_err(|err| anyhow!(err))
}

fn skip_whitespace(bytes: &[u8], idx: &mut usize) {
    while *idx < bytes.len() {
        let ch = bytes[*idx];
        if ch.is_ascii_whitespace() {
            *idx += 1;
        } else {
            break;
        }
    }
}

fn skip_comment(bytes: &[u8], idx: &mut usize) {
    while *idx < bytes.len() {
        let ch = bytes[*idx];
        *idx += 1;
        if ch == b'\n' || ch == b'\r' {
            break;
        }
    }
}

fn is_number_start(ch: u8) -> bool {
    ch.is_ascii_digit() || ch == b'+' || ch == b'-' || ch == b'.'
}

fn is_delimiter(ch: u8) -> bool {
    matches!(
        ch,
        b'(' | b')' | b'<' | b'>' | b'[' | b']' | b'{' | b'}' | b'/' | b'%'
    )
}

#[cfg(test)]
mod tests {
    use super::{tokenize_stream, Operand, Operator};

    fn assert_number(value: &Operand, expected: f64) {
        match value {
            Operand::Number(actual) => assert!((actual - expected).abs() < 1e-6),
            other => panic!("expected number operand, got {other:?}"),
        }
    }

    #[test]
    fn parses_cm_operator() {
        let input = b"1 0 0 1 10 20 cm";
        let tokens = tokenize_stream(input).unwrap();
        assert_eq!(tokens.len(), 1);
        let token = &tokens[0];
        assert_eq!(token.operator, Operator::Cm);
        assert_eq!(token.operands.len(), 6);
        assert_number(&token.operands[4], 10.0);
        assert_number(&token.operands[5], 20.0);
        assert_eq!(token.span, 0..18);
    }

    #[test]
    fn parses_bt_et_span() {
        let input = b"BT /F1 12 Tf 10 20 Td (Hello) Tj ET";
        let tokens = tokenize_stream(input).unwrap();
        assert_eq!(tokens.len(), 5);
        assert_eq!(tokens[0].operator, Operator::Bt);
        assert_eq!(tokens[4].operator, Operator::Et);
        assert_eq!(tokens[4].span.end, input.len());
    }

    #[test]
    fn parses_tj_and_tj_array() {
        let input = b"(Hi) Tj [(There) 120 (Mate)] TJ";
        let tokens = tokenize_stream(input).unwrap();
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].operator, Operator::Tj);
        assert_eq!(tokens[1].operator, Operator::TJ);
    }

    #[test]
    fn parses_do_operator() {
        let input = b"q 2 0 0 2 50 50 cm /Im0 Do Q";
        let tokens = tokenize_stream(input).unwrap();
        assert_eq!(tokens.len(), 4);
        assert_eq!(tokens[2].operator, Operator::Do);
        match &tokens[2].operands[0] {
            Operand::Name(name) => assert_eq!(name, "Im0"),
            other => panic!("expected name operand, got {other:?}"),
        }
    }
}
