//! Content stream tokenisation helpers specialised for the MVP editing flow.

use std::ops::Range;

use anyhow::{anyhow, bail, Result};

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

#[derive(Debug, Clone)]
pub struct ContentToken {
    pub operands: Vec<Operand>,
    pub operator: Operator,
    pub span: Range<usize>,
}

pub fn tokenize_stream(bytes: &[u8]) -> Result<Vec<ContentToken>> {
    let mut tokens = Vec::new();
    let mut i = 0usize;
    let mut operands: Vec<(Operand, Range<usize>)> = Vec::new();

    while i < bytes.len() {
        skip_whitespace(bytes, &mut i);
        if i >= bytes.len() {
            break;
        }

        if bytes[i] == b'%' {
            skip_comment(bytes, &mut i);
            continue;
        }

        match bytes[i] {
            b'[' => {
                let (arr, span) = parse_array(bytes, i + 1)?;
                operands.push((Operand::Array(arr), i..span.end));
                i = span.end;
            }
            b'(' => {
                let (string, span) = parse_string(bytes, i + 1)?;
                operands.push((Operand::String(string), i..span));
                i = span;
            }
            b'/' => {
                let (name, span) = parse_name(bytes, i + 1)?;
                operands.push((Operand::Name(name), i..span));
                i = span;
            }
            b'T' if matches!(bytes.get(i + 1), Some(b'*')) => {
                let op_span = i..i + 2;
                push_token(&mut tokens, &mut operands, Operator::TStar, op_span);
                i += 2;
            }
            b'+' | b'-' | b'.' | b'0'..=b'9' => {
                let (num, span) = parse_number(bytes, i)?;
                operands.push((Operand::Number(num), span.clone()));
                i = span.end;
            }
            _ => {
                let (word, span) = parse_word(bytes, i)?;
                let op = match word.as_str() {
                    "q" => Operator::q,
                    "Q" => Operator::Q,
                    "cm" => Operator::Cm,
                    "BT" => Operator::BT,
                    "ET" => Operator::ET,
                    "Tf" => Operator::Tf,
                    "Tm" => Operator::Tm,
                    "Td" => Operator::Td,
                    "Tj" => Operator::Tj,
                    "TJ" => Operator::TJ,
                    "Do" => Operator::Do,
                    other => Operator::Other(other.to_string()),
                };
                if !matches!(op, Operator::Other(_)) {
                    push_token(&mut tokens, &mut operands, op, span.clone());
                }
                i = span.end;
            }
        }
    }

    Ok(tokens)
}

fn push_token(
    tokens: &mut Vec<ContentToken>,
    operands: &mut Vec<(Operand, Range<usize>)>,
    operator: Operator,
    op_span: Range<usize>,
) {
    let span_start = operands
        .first()
        .map(|(_, span)| span.start)
        .unwrap_or(op_span.start);
    let span = span_start..op_span.end;
    let ops = operands.drain(..).map(|(op, _)| op).collect();
    tokens.push(ContentToken {
        operands: ops,
        operator,
        span,
    });
}

fn skip_whitespace(bytes: &[u8], i: &mut usize) {
    while *i < bytes.len() && matches!(bytes[*i], b'\x00' | b'\t' | b'\n' | b'\r' | b'\x0C' | b' ')
    {
        *i += 1;
    }
}

fn skip_comment(bytes: &[u8], i: &mut usize) {
    while *i < bytes.len() && bytes[*i] != b'\n' && bytes[*i] != b'\r' {
        *i += 1;
    }
}

fn parse_number(bytes: &[u8], start: usize) -> Result<(f64, Range<usize>)> {
    let mut i = start;
    if matches!(bytes[i], b'+' | b'-') {
        i += 1;
    }
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        i += 1;
    }
    if i < bytes.len() && bytes[i] == b'.' {
        i += 1;
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            i += 1;
        }
    }
    let number = std::str::from_utf8(&bytes[start..i])?
        .parse::<f64>()
        .map_err(|err| anyhow!(err))?;
    Ok((number, start..i))
}

fn parse_name(bytes: &[u8], start: usize) -> Result<(String, usize)> {
    let mut i = start;
    while i < bytes.len() && !is_delimiter(bytes[i]) && !bytes[i].is_ascii_whitespace() {
        i += 1;
    }
    let name = std::str::from_utf8(&bytes[start..i])?.to_string();
    Ok((name, i))
}

fn parse_word(bytes: &[u8], start: usize) -> Result<(String, Range<usize>)> {
    let mut i = start;
    while i < bytes.len() && !is_delimiter(bytes[i]) && !bytes[i].is_ascii_whitespace() {
        i += 1;
    }
    let word = std::str::from_utf8(&bytes[start..i])?.to_string();
    Ok((word, start..i))
}

fn parse_string(bytes: &[u8], mut i: usize) -> Result<(Vec<u8>, usize)> {
    let mut depth = 1i32;
    let mut out = Vec::new();
    while i < bytes.len() {
        let b = bytes[i];
        if b == b'\\' {
            if i + 1 >= bytes.len() {
                bail!("unterminated escape in string literal");
            }
            out.push(bytes[i]);
            i += 1;
            out.push(bytes[i]);
            i += 1;
            continue;
        }
        if b == b'(' {
            depth += 1;
            out.push(b);
            i += 1;
            continue;
        }
        if b == b')' {
            depth -= 1;
            if depth == 0 {
                i += 1;
                break;
            }
            out.push(b);
            i += 1;
            continue;
        }
        out.push(b);
        i += 1;
    }
    if depth != 0 {
        bail!("unterminated string literal");
    }
    Ok((out, i))
}

fn parse_array(bytes: &[u8], mut i: usize) -> Result<(Vec<Operand>, Range<usize>)> {
    let start = i - 1;
    let mut values = Vec::new();
    skip_whitespace(bytes, &mut i);
    while i < bytes.len() {
        skip_whitespace(bytes, &mut i);
        if i >= bytes.len() {
            bail!("unterminated array");
        }
        match bytes[i] {
            b']' => {
                i += 1;
                break;
            }
            b'%' => {
                skip_comment(bytes, &mut i);
            }
            b'[' => {
                let (nested, span) = parse_array(bytes, i + 1)?;
                values.push(Operand::Array(nested));
                i = span.end;
            }
            b'(' => {
                let (string, span) = parse_string(bytes, i + 1)?;
                values.push(Operand::String(string));
                i = span;
            }
            b'/' => {
                let (name, span_end) = parse_name(bytes, i + 1)?;
                values.push(Operand::Name(name));
                i = span_end;
            }
            b'+' | b'-' | b'.' | b'0'..=b'9' => {
                let (num, span) = parse_number(bytes, i)?;
                values.push(Operand::Number(num));
                i = span.end;
            }
            _ => {
                let (word, span) = parse_word(bytes, i)?;
                values.push(Operand::Name(word));
                i = span.end;
            }
        }
    }
    Ok((values, start..i))
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
    fn parse_cm_sequence() {
        let input = b"1 0 0 1 10 20 cm\n";
        let tokens = tokenize_stream(input).unwrap();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].operator, Operator::Cm);
        assert_eq!(tokens[0].operands.len(), 6);
        assert_eq!(tokens[0].span, 0..14);
    }

    #[test]
    fn parse_bt_et_block() {
        let input = b"BT /F1 12 Tf 1 0 0 1 50 60 Tm (Hello) Tj ET";
        let tokens = tokenize_stream(input).unwrap();
        assert_eq!(tokens.len(), 5);
        assert!(matches!(tokens[0].operator, Operator::BT));
        assert!(matches!(tokens[1].operator, Operator::Tf));
        assert!(matches!(tokens[2].operator, Operator::Tm));
        assert!(matches!(tokens[3].operator, Operator::Tj));
        assert!(matches!(tokens[4].operator, Operator::ET));
    }

    #[test]
    fn parse_tj_and_tj_array() {
        let input = b"BT (Hello) Tj [(World) 120] TJ ET";
        let tokens = tokenize_stream(input).unwrap();
        assert_eq!(tokens.len(), 5);
        assert!(matches!(tokens[2].operator, Operator::Tj));
        assert!(matches!(tokens[3].operator, Operator::TJ));
    }

    #[test]
    fn parse_do_operator() {
        let input = b"q 200 0 0 100 10 20 cm /Im1 Do Q";
        let tokens = tokenize_stream(input).unwrap();
        assert_eq!(tokens.len(), 4);
        assert!(matches!(tokens[0].operator, Operator::q));
        assert!(matches!(tokens[1].operator, Operator::Cm));
        assert!(matches!(tokens[2].operator, Operator::Do));
        assert!(matches!(tokens[3].operator, Operator::Q));
    }
}
