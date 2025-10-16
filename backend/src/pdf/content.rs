use std::{fmt, ops::Range};

use anyhow::{bail, Result};

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
    TjArray,
    Do,
    Other,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ContentToken {
    pub operands: Vec<Operand>,
    pub operator: Operator,
    pub span: Range<usize>,
}

impl ContentToken {
    pub fn new(operands: Vec<Operand>, operator: Operator, span: Range<usize>) -> Self {
        Self {
            operands,
            operator,
            span,
        }
    }
}

pub fn tokenize_stream(bytes: &[u8]) -> Result<Vec<ContentToken>> {
    let mut tokens = Vec::new();
    let mut pos = 0usize;
    while let Some(start) = skip_whitespace(bytes, pos) {
        pos = start;
        if bytes[pos] == b'%' {
            pos = skip_comment(bytes, pos);
            continue;
        }

        let token_start = pos;
        let mut operands = Vec::new();
        loop {
            pos = match skip_whitespace(bytes, pos) {
                Some(p) => p,
                None => break,
            };
            if pos >= bytes.len() {
                break;
            }
            if bytes[pos] == b'%' {
                pos = skip_comment(bytes, pos);
                continue;
            }

            match classify_next(bytes[pos]) {
                NextKind::Operator => {
                    let (op, end) = parse_operator(bytes, pos)?;
                    let operator = map_operator(&op);
                    let span = token_start..end;
                    tokens.push(ContentToken::new(operands, operator, span));
                    pos = end;
                    break;
                }
                NextKind::Operand => {
                    let (operand, next_pos) = parse_operand(bytes, pos)?;
                    operands.push(operand);
                    pos = next_pos;
                }
                NextKind::Delimiter => {
                    // stray delimiter, advance to avoid infinite loop
                    pos += 1;
                }
            }
        }
    }
    Ok(tokens)
}

fn skip_whitespace(bytes: &[u8], mut pos: usize) -> Option<usize> {
    while pos < bytes.len() {
        let b = bytes[pos];
        if matches!(b, b' ' | b'\t' | b'\r' | b'\n' | 0x0c | 0x00) {
            pos += 1;
            continue;
        }
        return Some(pos);
    }
    None
}

fn skip_comment(bytes: &[u8], mut pos: usize) -> usize {
    while pos < bytes.len() && bytes[pos] != b'\n' {
        pos += 1;
    }
    pos
}

enum NextKind {
    Operator,
    Operand,
    Delimiter,
}

fn classify_next(byte: u8) -> NextKind {
    match byte {
        b'a'..=b'z' | b'A'..=b'Z' | b'*' => NextKind::Operator,
        b'[' | b'(' | b'/' | b'+' | b'-' | b'.' | b'0'..=b'9' => NextKind::Operand,
        _ => NextKind::Delimiter,
    }
}

fn parse_operator(bytes: &[u8], mut pos: usize) -> Result<(String, usize)> {
    let start = pos;
    while pos < bytes.len() {
        let b = bytes[pos];
        if matches!(b, b'a'..=b'z' | b'A'..=b'Z' | b'*') {
            pos += 1;
        } else {
            break;
        }
    }
    if start == pos {
        bail!("expected operator at position {start}");
    }
    let op = std::str::from_utf8(&bytes[start..pos])?.to_string();
    Ok((op, pos))
}

fn parse_operand(bytes: &[u8], pos: usize) -> Result<(Operand, usize)> {
    if pos >= bytes.len() {
        bail!("operand beyond end of stream");
    }
    match bytes[pos] {
        b'/' => parse_name(bytes, pos),
        b'(' => parse_string(bytes, pos),
        b'[' => parse_array(bytes, pos),
        b'+' | b'-' | b'.' | b'0'..=b'9' => parse_number(bytes, pos),
        other => bail!("unsupported operand prefix: {other:02x}"),
    }
}

fn parse_name(bytes: &[u8], mut pos: usize) -> Result<(Operand, usize)> {
    pos += 1; // skip '/'
    let start = pos;
    while pos < bytes.len() {
        let b = bytes[pos];
        if matches!(
            b,
            b' ' | b'\t' | b'\r' | b'\n' | 0x0c | 0x00 | b'/' | b'[' | b']' | b'(' | b')'
        ) {
            break;
        }
        pos += 1;
    }
    let name = std::str::from_utf8(&bytes[start..pos])?.to_string();
    Ok((Operand::Name(name), pos))
}

fn parse_number(bytes: &[u8], mut pos: usize) -> Result<(Operand, usize)> {
    let start = pos;
    pos += 1;
    while pos < bytes.len() {
        let b = bytes[pos];
        if matches!(b, b'+' | b'-' | b'.' | b'0'..=b'9') {
            pos += 1;
        } else {
            break;
        }
    }
    let num_str = std::str::from_utf8(&bytes[start..pos])?;
    let value: f64 = num_str.parse()?;
    Ok((Operand::Number(value), pos))
}

fn parse_string(bytes: &[u8], mut pos: usize) -> Result<(Operand, usize)> {
    pos += 1; // skip opening '('
    let mut depth = 1usize;
    let mut content = Vec::new();
    while pos < bytes.len() {
        let b = bytes[pos];
        pos += 1;
        match b {
            b'\\' => {
                if pos >= bytes.len() {
                    bail!("unterminated escape in string");
                }
                let escaped = bytes[pos];
                pos += 1;
                content.push(match escaped {
                    b'n' => b'\n',
                    b'r' => b'\r',
                    b't' => b'\t',
                    b'b' => 0x08,
                    b'f' => 0x0c,
                    b'(' => b'(',
                    b')' => b')',
                    b'\\' => b'\\',
                    other => other,
                });
            }
            b'(' => {
                depth += 1;
                content.push(b'(');
            }
            b')' => {
                depth -= 1;
                if depth == 0 {
                    break;
                }
                content.push(b')');
            }
            other => content.push(other),
        }
    }
    if depth != 0 {
        bail!("unterminated literal string");
    }
    Ok((Operand::String(content), pos))
}

fn parse_array(bytes: &[u8], mut pos: usize) -> Result<(Operand, usize)> {
    pos += 1; // skip '['
    let mut items = Vec::new();
    loop {
        pos = match skip_whitespace(bytes, pos) {
            Some(p) => p,
            None => bail!("unterminated array"),
        };
        if pos >= bytes.len() {
            bail!("unterminated array");
        }
        if bytes[pos] == b']' {
            pos += 1;
            break;
        }
        if bytes[pos] == b'%' {
            pos = skip_comment(bytes, pos);
            continue;
        }
        let (operand, next_pos) = parse_operand(bytes, pos)?;
        items.push(operand);
        pos = next_pos;
    }
    Ok((Operand::Array(items), pos))
}

fn map_operator(op: &str) -> Operator {
    match op {
        "q" => Operator::q,
        "Q" => Operator::Q,
        "cm" => Operator::Cm,
        "BT" => Operator::Bt,
        "ET" => Operator::Et,
        "Tf" => Operator::Tf,
        "Tm" => Operator::Tm,
        "Td" => Operator::Td,
        "T*" => Operator::TStar,
        "Tj" => Operator::Tj,
        "TJ" => Operator::TjArray,
        "Do" => Operator::Do,
        _ => Operator::Other,
    }
}

impl fmt::Display for Operator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            Operator::Q => "Q",
            Operator::q => "q",
            Operator::Cm => "cm",
            Operator::Bt => "BT",
            Operator::Et => "ET",
            Operator::Tf => "Tf",
            Operator::Tm => "Tm",
            Operator::Td => "Td",
            Operator::TStar => "T*",
            Operator::Tj => "Tj",
            Operator::TjArray => "TJ",
            Operator::Do => "Do",
            Operator::Other => "?",
        };
        write!(f, "{text}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn span(token: &ContentToken) -> (usize, usize) {
        (token.span.start, token.span.end)
    }

    #[test]
    fn parse_cm_operator() {
        let input = b"1 0 0 1 10 20 cm\n";
        let tokens = tokenize_stream(input).unwrap();
        assert_eq!(tokens.len(), 1);
        let token = &tokens[0];
        assert_eq!(token.operator, Operator::Cm);
        assert_eq!(token.operands.len(), 6);
        assert_eq!(span(token), (0, input.len()));
    }

    #[test]
    fn parse_bt_et_block() {
        let input = b"BT /F1 12 Tf 10 20 Td (Hello) Tj ET";
        let tokens = tokenize_stream(input).unwrap();
        assert_eq!(tokens.len(), 6);
        assert!(matches!(tokens[0].operator, Operator::Bt));
        assert!(matches!(tokens[1].operator, Operator::Tf));
        assert!(matches!(tokens[2].operator, Operator::Td));
        assert!(matches!(tokens[3].operator, Operator::Tj));
        assert!(matches!(tokens[5].operator, Operator::Et));
    }

    #[test]
    fn parse_tj_array() {
        let input = b"BT [ (Hel) -120 (lo) ] TJ ET";
        let tokens = tokenize_stream(input).unwrap();
        assert_eq!(tokens.len(), 4);
        assert!(matches!(tokens[2].operator, Operator::TjArray));
        if let Operand::Array(items) = &tokens[2].operands[0] {
            assert_eq!(items.len(), 3);
        } else {
            panic!("expected array operand");
        }
    }

    #[test]
    fn parse_do_operator() {
        let input = b"q 1 0 0 1 10 10 cm /Im0 Do Q";
        let tokens = tokenize_stream(input).unwrap();
        assert_eq!(tokens.len(), 5);
        assert!(matches!(tokens[0].operator, Operator::q));
        assert!(matches!(tokens[1].operator, Operator::Cm));
        assert!(matches!(tokens[2].operator, Operator::Do));
        assert!(matches!(tokens[4].operator, Operator::Q));
    }
}
