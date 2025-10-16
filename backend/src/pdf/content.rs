//! Content stream tokenisation helpers.

use anyhow::{anyhow, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ByteRange {
    pub start: usize,
    pub end: usize,
}

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
    Unknown(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ContentToken {
    pub operator: Operator,
    pub operands: Vec<Operand>,
    pub span: ByteRange,
}

pub fn tokenize_stream(bytes: &[u8]) -> Result<Vec<ContentToken>> {
    let mut cursor = 0usize;
    let mut tokens = Vec::new();
    let mut operands: Vec<(Operand, usize)> = Vec::new();

    while cursor < bytes.len() {
        skip_whitespace(bytes, &mut cursor);
        if cursor >= bytes.len() {
            break;
        }
        match bytes[cursor] {
            b'%' => {
                skip_comment(bytes, &mut cursor);
            }
            b'/' => {
                let start = cursor;
                let (name, next) = parse_name(bytes, cursor + 1)?;
                operands.push((Operand::Name(name), start));
                cursor = next;
            }
            b'(' => {
                let start = cursor;
                let (data, next) = parse_string(bytes, cursor + 1)?;
                operands.push((Operand::String(data), start));
                cursor = next;
            }
            b'[' => {
                let start = cursor;
                let (array, next) = parse_array(bytes, cursor + 1)?;
                operands.push((Operand::Array(array), start));
                cursor = next;
            }
            ch if is_number_start(ch) => {
                let start = cursor;
                let (value, next) = parse_number(bytes, cursor)?;
                operands.push((Operand::Number(value), start));
                cursor = next;
            }
            ch if is_operator_start(ch) => {
                let op_start = cursor;
                let (word, next) = parse_operator(bytes, cursor)?;
                cursor = next;
                let op = match word.as_str() {
                    "Q" => Operator::Q,
                    "q" => Operator::q,
                    "cm" => Operator::Cm,
                    "BT" => Operator::BT,
                    "ET" => Operator::ET,
                    "Tf" => Operator::Tf,
                    "Tm" => Operator::Tm,
                    "Td" => Operator::Td,
                    "T*" => Operator::TStar,
                    "Tj" => Operator::Tj,
                    "TJ" => Operator::TJ,
                    "Do" => Operator::Do,
                    _ => Operator::Unknown(word.clone()),
                };
                let span_start = operands
                    .first()
                    .map(|(_, start)| *start)
                    .unwrap_or(op_start);
                let span = ByteRange {
                    start: span_start,
                    end: cursor,
                };
                let args = operands.drain(..).map(|(op, _)| op).collect();
                tokens.push(ContentToken {
                    operator: op,
                    operands: args,
                    span,
                });
            }
            _ => {
                // Unrecognised token delimiter; skip to avoid infinite loop.
                cursor += 1;
            }
        }
    }

    Ok(tokens)
}

fn skip_whitespace(bytes: &[u8], cursor: &mut usize) {
    while *cursor < bytes.len() {
        match bytes[*cursor] {
            b' ' | b'\n' | b'\r' | b'\t' | 0x0C | 0x00 => *cursor += 1,
            b'%' => {
                skip_comment(bytes, cursor);
            }
            _ => break,
        }
    }
}

fn skip_comment(bytes: &[u8], cursor: &mut usize) {
    while *cursor < bytes.len() {
        let ch = bytes[*cursor];
        *cursor += 1;
        if ch == b'\n' || ch == b'\r' {
            break;
        }
    }
}

fn parse_name(bytes: &[u8], mut cursor: usize) -> Result<(String, usize)> {
    let start = cursor;
    while cursor < bytes.len() {
        let ch = bytes[cursor];
        if ch.is_ascii_whitespace() || is_delimiter(ch) {
            break;
        }
        cursor += 1;
    }
    let name = std::str::from_utf8(&bytes[start..cursor])?.to_string();
    Ok((name, cursor))
}

fn parse_number(bytes: &[u8], mut cursor: usize) -> Result<(f64, usize)> {
    let start = cursor;
    if cursor < bytes.len() && (bytes[cursor] == b'+' || bytes[cursor] == b'-') {
        cursor += 1;
    }
    while cursor < bytes.len() {
        let ch = bytes[cursor];
        if ch.is_ascii_digit() || ch == b'.' {
            cursor += 1;
        } else {
            break;
        }
    }
    let slice = &bytes[start..cursor];
    let text = std::str::from_utf8(slice)?;
    let value = text
        .parse::<f64>()
        .map_err(|err| anyhow!("invalid number {text}: {err}"))?;
    Ok((value, cursor))
}

fn parse_string(bytes: &[u8], mut cursor: usize) -> Result<(Vec<u8>, usize)> {
    let mut depth = 1usize;
    let mut out = Vec::new();
    while cursor < bytes.len() {
        let ch = bytes[cursor];
        cursor += 1;
        match ch {
            b'\\' => {
                if cursor < bytes.len() {
                    out.push(bytes[cursor]);
                    cursor += 1;
                }
            }
            b'(' => {
                depth += 1;
                out.push(ch);
            }
            b')' => {
                depth -= 1;
                if depth == 0 {
                    return Ok((out, cursor));
                }
                out.push(ch);
            }
            _ => out.push(ch),
        }
    }
    Err(anyhow!("unterminated string literal"))
}

fn parse_array(bytes: &[u8], mut cursor: usize) -> Result<(Vec<Operand>, usize)> {
    let mut items = Vec::new();
    loop {
        skip_whitespace(bytes, &mut cursor);
        if cursor >= bytes.len() {
            break;
        }
        let ch = bytes[cursor];
        match ch {
            b']' => {
                cursor += 1;
                break;
            }
            b'%' => skip_comment(bytes, &mut cursor),
            b'(' => {
                let (value, next) = parse_string(bytes, cursor + 1)?;
                items.push(Operand::String(value));
                cursor = next;
            }
            b'[' => {
                let (value, next) = parse_array(bytes, cursor + 1)?;
                items.push(Operand::Array(value));
                cursor = next;
            }
            b'/' => {
                let (name, next) = parse_name(bytes, cursor + 1)?;
                items.push(Operand::Name(name));
                cursor = next;
            }
            ch if is_number_start(ch) => {
                let (value, next) = parse_number(bytes, cursor)?;
                items.push(Operand::Number(value));
                cursor = next;
            }
            _ => {
                // Skip unknown token within array.
                let (_, next) = parse_operator(bytes, cursor)?;
                cursor = next;
            }
        }
    }
    Ok((items, cursor))
}

fn parse_operator(bytes: &[u8], mut cursor: usize) -> Result<(String, usize)> {
    let start = cursor;
    while cursor < bytes.len() {
        let ch = bytes[cursor];
        if ch.is_ascii_whitespace() || is_delimiter(ch) {
            break;
        }
        cursor += 1;
    }
    let text = std::str::from_utf8(&bytes[start..cursor])?.to_string();
    Ok((text, cursor))
}

fn is_delimiter(ch: u8) -> bool {
    matches!(
        ch,
        b'(' | b')' | b'<' | b'>' | b'[' | b']' | b'{' | b'}' | b'/' | b'%'
    )
}

fn is_operator_start(ch: u8) -> bool {
    ch.is_ascii_alphabetic() || matches!(ch, b'*')
}

fn is_number_start(ch: u8) -> bool {
    ch.is_ascii_digit() || matches!(ch, b'+' | b'-' | b'.')
}

#[cfg(test)]
mod tests {
    use super::*;

    fn token_labels(tokens: &[ContentToken]) -> Vec<Operator> {
        tokens.iter().map(|t| t.operator.clone()).collect()
    }

    #[test]
    fn parses_graphics_state_tokens() {
        let data = b"q 1 0 0 1 0 0 cm /Im1 Do Q";
        let tokens = tokenize_stream(data).expect("tokenize");
        let labels = token_labels(&tokens);
        assert_eq!(
            labels,
            vec![Operator::q, Operator::Cm, Operator::Do, Operator::Q,]
        );
        assert_eq!(tokens[1].operands.len(), 6);
        assert_eq!(tokens[2].operands.len(), 1);
    }

    #[test]
    fn parses_text_block() {
        let data = b"BT /F1 12 Tf 10 20 Td (Hi) Tj ET";
        let tokens = tokenize_stream(data).expect("tokenize");
        let labels = token_labels(&tokens);
        assert_eq!(
            labels,
            vec![
                Operator::BT,
                Operator::Tf,
                Operator::Td,
                Operator::Tj,
                Operator::ET,
            ]
        );
        assert_eq!(tokens[0].span.start, 0);
        assert_eq!(tokens[0].span.end, 2);
        assert_eq!(tokens[3].span.start, 18);
        assert_eq!(tokens[3].span.end, data.len() - 3);
    }

    #[test]
    fn parses_tj_array() {
        let data = b"BT [ (A) 120 (B) ] TJ ET";
        let tokens = tokenize_stream(data).expect("tokenize");
        assert_eq!(
            token_labels(&tokens)[..],
            [Operator::BT, Operator::TJ, Operator::ET]
        );
        if let Operand::Array(items) = &tokens[1].operands[0] {
            assert_eq!(items.len(), 3);
        } else {
            panic!("expected array operand");
        }
    }
}
