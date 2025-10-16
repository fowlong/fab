//! PDF content stream tokenisation helpers.

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
    pub span: Range<usize>,
}

struct ParsedOperand {
    value: Operand,
    span: Range<usize>,
}

pub fn tokenize_stream(bytes: &[u8]) -> Result<Vec<Token>> {
    let mut tokens = Vec::new();
    let mut operands: Vec<ParsedOperand> = Vec::new();
    let mut i = 0usize;

    while i < bytes.len() {
        skip_ws(bytes, &mut i);
        if i >= bytes.len() {
            break;
        }
        if bytes[i] == b'%' {
            skip_comment(bytes, &mut i);
            continue;
        }

        let parsed = parse_item(bytes, &mut i)?;
        match parsed {
            ParsedItem::Operand(value, span) => operands.push(ParsedOperand { value, span }),
            ParsedItem::Operator(op, span) => {
                let start = operands
                    .first()
                    .map(|operand| operand.span.start)
                    .unwrap_or(span.start);
                let end = span.end;
                let token = Token {
                    operator: op,
                    operands: operands.into_iter().map(|p| p.value).collect(),
                    span: start..end,
                };
                tokens.push(token);
                operands = Vec::new();
            }
        }
    }

    Ok(tokens)
}

enum ParsedItem {
    Operand(Operand, Range<usize>),
    Operator(Operator, Range<usize>),
}

fn parse_item(bytes: &[u8], i: &mut usize) -> Result<ParsedItem> {
    if *i >= bytes.len() {
        return Err(anyhow!("unexpected end of content"));
    }
    match bytes[*i] {
        b'/' => parse_name(bytes, i)
            .map(|(value, span)| ParsedItem::Operand(Operand::Name(value), span)),
        b'[' => parse_array(bytes, i)
            .map(|(value, span)| ParsedItem::Operand(Operand::Array(value), span)),
        b'(' => parse_string(bytes, i)
            .map(|(value, span)| ParsedItem::Operand(Operand::String(value), span)),
        b'-' | b'+' | b'.' | b'0'..=b'9' => parse_number(bytes, i)
            .map(|(value, span)| ParsedItem::Operand(Operand::Number(value), span)),
        _ => parse_keyword(bytes, i),
    }
}

fn parse_keyword(bytes: &[u8], i: &mut usize) -> Result<ParsedItem> {
    let start = *i;
    while *i < bytes.len() && !is_delimiter(bytes[*i]) {
        *i += 1;
    }
    if start == *i {
        return Err(anyhow!("empty keyword"));
    }
    let span = start..*i;
    let word = &bytes[start..*i];
    let op = match word {
        b"q" => Some(Operator::q),
        b"Q" => Some(Operator::Q),
        b"cm" => Some(Operator::Cm),
        b"BT" => Some(Operator::Bt),
        b"ET" => Some(Operator::Et),
        b"Tf" => Some(Operator::Tf),
        b"Tm" => Some(Operator::Tm),
        b"Td" => Some(Operator::Td),
        b"T*" => Some(Operator::TStar),
        b"Tj" => Some(Operator::Tj),
        b"TJ" => Some(Operator::TJ),
        b"Do" => Some(Operator::Do),
        _ => None,
    };

    if let Some(operator) = op {
        Ok(ParsedItem::Operator(operator, span))
    } else {
        let name = std::str::from_utf8(word)?.to_string();
        Ok(ParsedItem::Operand(Operand::Name(name), span))
    }
}

fn parse_number(bytes: &[u8], i: &mut usize) -> Result<(f64, Range<usize>)> {
    let start = *i;
    if bytes[*i] == b'+' || bytes[*i] == b'-' {
        *i += 1;
    }
    while *i < bytes.len() && (bytes[*i].is_ascii_digit() || bytes[*i] == b'.') {
        *i += 1;
    }
    let slice = &bytes[start..*i];
    let text = std::str::from_utf8(slice)?;
    let value = text.parse::<f64>()?;
    Ok((value, start..*i))
}

fn parse_name(bytes: &[u8], i: &mut usize) -> Result<(String, Range<usize>)> {
    let start = *i;
    *i += 1; // skip '/'
    while *i < bytes.len() && !is_delimiter(bytes[*i]) {
        *i += 1;
    }
    let name = std::str::from_utf8(&bytes[start + 1..*i])?.to_string();
    Ok((name, start..*i))
}

fn parse_string(bytes: &[u8], i: &mut usize) -> Result<(Vec<u8>, Range<usize>)> {
    let start = *i;
    *i += 1; // skip '('
    let mut depth = 1;
    let mut out = Vec::new();
    while *i < bytes.len() {
        let b = bytes[*i];
        *i += 1;
        match b {
            b'\\' => {
                if *i < bytes.len() {
                    out.push(bytes[*i]);
                    *i += 1;
                }
            }
            b'(' => {
                depth += 1;
                out.push(b);
            }
            b')' => {
                depth -= 1;
                if depth == 0 {
                    let span = start..*i;
                    return Ok((out, span));
                }
                out.push(b);
            }
            _ => out.push(b),
        }
    }
    Err(anyhow!("unterminated string literal"))
}

fn parse_array(bytes: &[u8], i: &mut usize) -> Result<(Vec<Operand>, Range<usize>)> {
    let start = *i;
    *i += 1; // skip '['
    let mut items = Vec::new();
    loop {
        skip_ws(bytes, i);
        if *i >= bytes.len() {
            return Err(anyhow!("unterminated array"));
        }
        if bytes[*i] == b']' {
            *i += 1;
            break;
        }
        let parsed = parse_item(bytes, i)?;
        match parsed {
            ParsedItem::Operand(value, _) => items.push(value),
            ParsedItem::Operator(_, _) => return Err(anyhow!("unexpected operator inside array")),
        }
    }
    Ok((items, start..*i))
}

fn skip_ws(bytes: &[u8], i: &mut usize) {
    while *i < bytes.len() {
        let b = bytes[*i];
        if matches!(b, b' ' | b'\n' | b'\r' | b'\t' | b'\x0c' | b'\x00') {
            *i += 1;
        } else {
            break;
        }
    }
}

fn skip_comment(bytes: &[u8], i: &mut usize) {
    while *i < bytes.len() && bytes[*i] != b'\n' {
        *i += 1;
    }
}

fn is_delimiter(b: u8) -> bool {
    matches!(
        b,
        b' ' | b'\n'
            | b'\r'
            | b'\t'
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_cm_token() {
        let input = b"1 0 0 1 10 20 cm";
        let tokens = tokenize_stream(input).unwrap();
        assert_eq!(tokens.len(), 1);
        let token = &tokens[0];
        assert_eq!(token.operator, Operator::Cm);
        assert_eq!(token.operands.len(), 6);
        assert!(matches!(
            token.operands[0],
            Operand::Number(value) if (value - 1.0).abs() < 1e-6
        ));
    }

    #[test]
    fn parses_bt_et_block_with_text() {
        let input = b"BT /F1 12 Tf 100 0 Td (Hello) Tj ET";
        let tokens = tokenize_stream(input).unwrap();
        assert_eq!(tokens.len(), 5);
        assert_eq!(tokens[0].operator, Operator::Bt);
        assert_eq!(tokens[1].operator, Operator::Tf);
        assert_eq!(tokens[2].operator, Operator::Td);
        assert_eq!(tokens[3].operator, Operator::Tj);
        assert_eq!(tokens[4].operator, Operator::Et);
        if let Operand::String(data) = &tokens[3].operands[0] {
            assert_eq!(data, b"Hello");
        } else {
            panic!("expected string operand");
        }
    }

    #[test]
    fn parses_tj_array() {
        let input = b"[ (A) 120 (B) ] TJ";
        let tokens = tokenize_stream(input).unwrap();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].operator, Operator::TJ);
        assert_eq!(tokens[0].operands.len(), 1);
        match &tokens[0].operands[0] {
            Operand::Array(items) => {
                assert_eq!(items.len(), 3);
            }
            _ => panic!("expected array operand"),
        }
    }

    #[test]
    fn parses_do_operator() {
        let input = b"q 10 0 0 10 0 0 cm /Im1 Do Q";
        let tokens = tokenize_stream(input).unwrap();
        assert_eq!(tokens.len(), 4);
        assert_eq!(tokens[2].operator, Operator::Do);
        if let Operand::Name(name) = &tokens[2].operands[0] {
            assert_eq!(name, "Im1");
        } else {
            panic!("expected name operand");
        }
    }
}
