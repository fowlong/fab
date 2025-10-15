use std::{ops::Range, str::FromStr};

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
    TjArray,
    Do,
}

impl Operator {
    fn parse(keyword: &str) -> Option<Self> {
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
            "TJ" => Some(Operator::TjArray),
            "Do" => Some(Operator::Do),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ContentToken {
    pub operator: Operator,
    pub operands: Vec<Operand>,
    pub span: Range<usize>,
}

pub fn tokenize_stream(bytes: &[u8]) -> Result<Vec<ContentToken>> {
    let mut tokens = Vec::new();
    let mut pos = 0usize;
    let len = bytes.len();

    while pos < len {
        skip_ws(bytes, &mut pos);
        if pos >= len {
            break;
        }

        let mut operands: Vec<(Operand, Range<usize>)> = Vec::new();
        let mut token_start: Option<usize> = None;

        loop {
            skip_ws(bytes, &mut pos);
            if pos >= len {
                break;
            }
            if let Some((keyword, span)) = read_keyword(bytes, pos) {
                if let Some(op) = Operator::parse(&keyword) {
                    let start = token_start.unwrap_or(span.start);
                    let mut operand_values = Vec::with_capacity(operands.len());
                    for (operand, _) in &operands {
                        operand_values.push(operand.clone());
                    }
                    tokens.push(ContentToken {
                        operator: op,
                        operands: operand_values,
                        span: start..span.end,
                    });
                    pos = span.end;
                    break;
                }
            }

            match read_operand(bytes, pos)? {
                Some((operand, span)) => {
                    if token_start.is_none() {
                        token_start = Some(span.start);
                    }
                    pos = span.end;
                    operands.push((operand, span));
                }
                None => {
                    // Consume keyword as operator even if unrecognised to progress.
                    if let Some((_, span)) = read_keyword(bytes, pos) {
                        pos = span.end;
                    } else {
                        pos += 1;
                    }
                    break;
                }
            }
        }
    }

    Ok(tokens)
}

fn skip_ws(bytes: &[u8], pos: &mut usize) {
    while *pos < bytes.len() {
        match bytes[*pos] {
            b'\n' | b'\r' | b'\t' | b'\x0c' | b' ' => *pos += 1,
            b'%' => {
                while *pos < bytes.len() && bytes[*pos] != b'\n' {
                    *pos += 1;
                }
            }
            _ => break,
        }
    }
}

fn read_keyword(bytes: &[u8], mut pos: usize) -> Option<(String, Range<usize>)> {
    if pos >= bytes.len() {
        return None;
    }
    let start = pos;
    let mut buf = Vec::new();
    while pos < bytes.len() {
        let ch = bytes[pos];
        if is_delimiter(ch) {
            break;
        }
        buf.push(ch);
        pos += 1;
    }
    if buf.is_empty() {
        None
    } else {
        Some((String::from_utf8_lossy(&buf).into_owned(), start..pos))
    }
}

fn read_operand(bytes: &[u8], pos: usize) -> Result<Option<(Operand, Range<usize>)>> {
    if pos >= bytes.len() {
        return Ok(None);
    }
    match bytes[pos] {
        b'/' => read_name(bytes, pos).map(Some),
        b'(' => read_string(bytes, pos).map(Some),
        b'[' => read_array(bytes, pos).map(Some),
        b'+' | b'-' | b'.' | b'0'..=b'9' => read_number(bytes, pos).map(Some),
        _ => Ok(None),
    }
}

fn read_name(bytes: &[u8], mut pos: usize) -> Result<(Operand, Range<usize>)> {
    let start = pos;
    pos += 1; // skip '/'
    let mut buf = Vec::new();
    while pos < bytes.len() {
        let ch = bytes[pos];
        if is_delimiter(ch) {
            break;
        }
        buf.push(ch);
        pos += 1;
    }
    let name = String::from_utf8_lossy(&buf).into_owned();
    Ok((Operand::Name(name), start..pos))
}

fn read_string(bytes: &[u8], mut pos: usize) -> Result<(Operand, Range<usize>)> {
    let start = pos;
    pos += 1; // skip '('
    let mut buf = Vec::new();
    let mut depth = 1i32;
    while pos < bytes.len() {
        let ch = bytes[pos];
        match ch {
            b'\\' => {
                if pos + 1 < bytes.len() {
                    buf.push(bytes[pos + 1]);
                    pos += 2;
                } else {
                    pos += 1;
                }
            }
            b'(' => {
                depth += 1;
                buf.push(ch);
                pos += 1;
            }
            b')' => {
                depth -= 1;
                if depth == 0 {
                    pos += 1;
                    break;
                }
                buf.push(ch);
                pos += 1;
            }
            _ => {
                buf.push(ch);
                pos += 1;
            }
        }
    }
    if depth != 0 {
        return Err(anyhow!("unterminated string literal"));
    }
    Ok((Operand::String(buf), start..pos))
}

fn read_array(bytes: &[u8], mut pos: usize) -> Result<(Operand, Range<usize>)> {
    let start = pos;
    pos += 1; // skip '['
    let mut items = Vec::new();
    loop {
        skip_ws(bytes, &mut pos);
        if pos >= bytes.len() {
            return Err(anyhow!("unterminated array"));
        }
        if bytes[pos] == b']' {
            pos += 1;
            break;
        }
        if let Some((operand, span)) = read_operand(bytes, pos)? {
            pos = span.end;
            items.push(operand);
        } else if let Some((keyword, span)) = read_keyword(bytes, pos) {
            if let Some(op) = Operator::parse(&keyword) {
                // Treat inline operators as names to keep structure.
                items.push(Operand::Name(format!("op:{op:?}")));
                pos = span.end;
            } else {
                items.push(Operand::Name(keyword));
                pos = span.end;
            }
        } else {
            pos += 1;
        }
    }
    Ok((Operand::Array(items), start..pos))
}

fn read_number(bytes: &[u8], mut pos: usize) -> Result<(Operand, Range<usize>)> {
    let start = pos;
    if bytes[pos] == b'+' || bytes[pos] == b'-' {
        pos += 1;
    }
    while pos < bytes.len() {
        let ch = bytes[pos];
        if !matches!(ch, b'0'..=b'9') {
            break;
        }
        pos += 1;
    }
    if pos < bytes.len() && bytes[pos] == b'.' {
        pos += 1;
        while pos < bytes.len() {
            let ch = bytes[pos];
            if !matches!(ch, b'0'..=b'9') {
                break;
            }
            pos += 1;
        }
    }
    let value = f64::from_str(std::str::from_utf8(&bytes[start..pos])?)?;
    Ok((Operand::Number(value), start..pos))
}

fn is_delimiter(ch: u8) -> bool {
    matches!(
        ch,
        b'\0'
            | b'\t'
            | b'\n'
            | b'\x0c'
            | b'\r'
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenize_cm_sequence() {
        let data = b"1 0 0 1 10 20 cm q";
        let tokens = tokenize_stream(data).unwrap();
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].operator, Operator::Cm);
        assert_eq!(tokens[1].operator, Operator::q);

        assert_eq!(
            std::str::from_utf8(&data[tokens[0].span.clone()]).unwrap(),
            "1 0 0 1 10 20 cm"
        );
        assert_eq!(
            tokens[0].operands,
            vec![
                Operand::Number(1.0),
                Operand::Number(0.0),
                Operand::Number(0.0),
                Operand::Number(1.0),
                Operand::Number(10.0),
                Operand::Number(20.0),
            ]
        );

        assert_eq!(
            std::str::from_utf8(&data[tokens[1].span.clone()]).unwrap(),
            "q"
        );
        assert!(tokens[1].operands.is_empty());
    }

    #[test]
    fn tokenize_bt_block() {
        let data = b"BT /F1 12 Tf 10 20 Td (Hello) Tj ET";
        let tokens = tokenize_stream(data).unwrap();
        assert_eq!(tokens.len(), 6);
        assert_eq!(tokens[0].operator, Operator::Bt);
        assert_eq!(tokens[1].operator, Operator::Tf);
        assert_eq!(tokens[2].operator, Operator::Td);
        assert_eq!(tokens[3].operator, Operator::Tj);
        assert_eq!(tokens[5].operator, Operator::Et);

        assert_eq!(
            std::str::from_utf8(&data[tokens[0].span.clone()]).unwrap(),
            "BT"
        );
        assert_eq!(
            std::str::from_utf8(&data[tokens[1].span.clone()]).unwrap(),
            "/F1 12 Tf"
        );
        assert_eq!(
            std::str::from_utf8(&data[tokens[2].span.clone()]).unwrap(),
            "10 20 Td"
        );
        assert_eq!(
            std::str::from_utf8(&data[tokens[3].span.clone()]).unwrap(),
            "(Hello) Tj"
        );
        assert_eq!(
            tokens[3].operands,
            vec![Operand::String(b"Hello".to_vec())]
        );
        assert_eq!(
            std::str::from_utf8(&data[tokens[5].span.clone()]).unwrap(),
            "ET"
        );
    }

    #[test]
    fn tokenize_tj_array() {
        let data = b"BT [ (Hello) 120 (World) ] TJ ET";
        let tokens = tokenize_stream(data).unwrap();
        assert_eq!(tokens.len(), 4);
        assert_eq!(tokens[1].operator, Operator::TjArray);
        assert_eq!(
            std::str::from_utf8(&data[tokens[1].span.clone()]).unwrap(),
            "[ (Hello) 120 (World) ] TJ"
        );
        assert_eq!(
            tokens[1].operands,
            vec![Operand::Array(vec![
                Operand::String(b"Hello".to_vec()),
                Operand::Number(120.0),
                Operand::String(b"World".to_vec()),
            ])]
        );
    }

    #[test]
    fn tokenize_do_operator() {
        let data = b"q 200 0 0 200 10 20 cm /Im1 Do Q";
        let tokens = tokenize_stream(data).unwrap();
        assert_eq!(tokens.len(), 5);
        assert_eq!(tokens[3].operator, Operator::Do);
        assert_eq!(
            std::str::from_utf8(&data[tokens[3].span.clone()]).unwrap(),
            "/Im1 Do"
        );
        assert_eq!(tokens[3].operands, vec![Operand::Name("Im1".into())]);
    }
}
