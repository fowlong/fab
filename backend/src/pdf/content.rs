//! Content stream tokenisation and matrix utilities.

use std::ops::Range;

use anyhow::{anyhow, Result};

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub operator: Operator,
    pub operands: Vec<Operand>,
    pub span: Range<usize>,
}

#[derive(Debug, Clone, PartialEq)]
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

#[derive(Debug, Clone, PartialEq)]
pub enum Operand {
    Number(f64),
    Name(String),
    String(Vec<u8>),
    Array(Vec<Operand>),
    Other(String),
}

pub type Matrix = [f64; 6];

pub fn identity() -> Matrix {
    [1.0, 0.0, 0.0, 1.0, 0.0, 0.0]
}

pub fn multiply(a: Matrix, b: Matrix) -> Matrix {
    [
        a[0] * b[0] + a[2] * b[1],
        a[1] * b[0] + a[3] * b[1],
        a[0] * b[2] + a[2] * b[3],
        a[1] * b[2] + a[3] * b[3],
        a[0] * b[4] + a[2] * b[5] + a[4],
        a[1] * b[4] + a[3] * b[5] + a[5],
    ]
}

pub fn translation(tx: f64, ty: f64) -> Matrix {
    [1.0, 0.0, 0.0, 1.0, tx, ty]
}

pub fn invert(matrix: Matrix) -> Result<Matrix> {
    let det = matrix[0] * matrix[3] - matrix[1] * matrix[2];
    if det.abs() < 1e-8 {
        return Err(anyhow!("matrix not invertible"));
    }
    let inv_det = 1.0 / det;
    let a = matrix[3] * inv_det;
    let b = -matrix[1] * inv_det;
    let c = -matrix[2] * inv_det;
    let d = matrix[0] * inv_det;
    let e = -(a * matrix[4] + c * matrix[5]);
    let f = -(b * matrix[4] + d * matrix[5]);
    Ok([a, b, c, d, e, f])
}

pub fn apply(matrix: Matrix, x: f64, y: f64) -> (f64, f64) {
    (
        matrix[0] * x + matrix[2] * y + matrix[4],
        matrix[1] * x + matrix[3] * y + matrix[5],
    )
}

pub fn format_matrix(matrix: Matrix) -> String {
    matrix
        .iter()
        .map(|v| format_number(*v))
        .collect::<Vec<_>>()
        .join(" ")
}

fn format_number(value: f64) -> String {
    if value.abs() < 1e-8 {
        return "0".into();
    }
    let mut s = format!("{value:.6}");
    while s.contains('.') && s.ends_with('0') {
        s.pop();
    }
    if s.ends_with('.') {
        s.pop();
    }
    s
}

pub fn tokenize_stream(bytes: &[u8]) -> Result<Vec<Token>> {
    let mut tokens = Vec::new();
    let mut operands: Vec<Operand> = Vec::new();
    let mut token_start: Option<usize> = None;
    let mut i = 0;

    while i < bytes.len() {
        skip_whitespace(bytes, &mut i);
        if i >= bytes.len() {
            break;
        }
        if bytes[i] == b'%' {
            skip_comment(bytes, &mut i);
            continue;
        }
        if token_start.is_none() {
            token_start = Some(i);
        }

        match bytes[i] {
            b'(' => {
                let (value, next) = parse_string(bytes, i)?;
                operands.push(Operand::String(value));
                i = next;
            }
            b'[' => {
                let (value, next) = parse_array(bytes, i)?;
                operands.push(Operand::Array(value));
                i = next;
            }
            b'/' => {
                let (name, next) = parse_name(bytes, i + 1)?;
                operands.push(Operand::Name(name));
                i = next;
            }
            b'+' | b'-' | b'.' | b'0'..=b'9' => {
                let (number, next) = parse_number(bytes, i)?;
                operands.push(Operand::Number(number));
                i = next;
            }
            _ => {
                let (word, next) = parse_word(bytes, i)?;
                i = next;
                if word.is_empty() {
                    continue;
                }
                let operator = Operator::from_word(&word);
                let start = token_start.take().unwrap_or(i);
                let span = start..i;
                tokens.push(Token {
                    operator,
                    operands,
                    span,
                });
                operands = Vec::new();
                continue;
            }
        }

        skip_whitespace(bytes, &mut i);
        if i < bytes.len() && bytes[i] == b'%' {
            skip_comment(bytes, &mut i);
        }
    }

    Ok(tokens)
}

fn skip_whitespace(bytes: &[u8], index: &mut usize) {
    while *index < bytes.len() && is_whitespace(bytes[*index]) {
        *index += 1;
    }
}

fn skip_comment(bytes: &[u8], index: &mut usize) {
    while *index < bytes.len() && bytes[*index] != b'\n' {
        *index += 1;
    }
    if *index < bytes.len() {
        *index += 1;
    }
}

fn is_whitespace(byte: u8) -> bool {
    matches!(byte, b' ' | b'\n' | b'\r' | b'\t' | b'\x0c' | b'\x00')
}

fn parse_word(bytes: &[u8], mut index: usize) -> Result<(String, usize)> {
    let start = index;
    while index < bytes.len() && !is_whitespace(bytes[index]) && !is_delimiter(bytes[index]) {
        index += 1;
    }
    let word = std::str::from_utf8(&bytes[start..index])?.to_string();
    Ok((word, index))
}

fn is_delimiter(byte: u8) -> bool {
    matches!(
        byte,
        b'(' | b')' | b'<' | b'>' | b'[' | b']' | b'{' | b'}' | b'/'
    )
}

fn parse_number(bytes: &[u8], mut index: usize) -> Result<(f64, usize)> {
    let start = index;
    if bytes[index] == b'+' || bytes[index] == b'-' {
        index += 1;
    }
    while index < bytes.len() && bytes[index].is_ascii_digit() {
        index += 1;
    }
    if index < bytes.len() && bytes[index] == b'.' {
        index += 1;
        while index < bytes.len() && bytes[index].is_ascii_digit() {
            index += 1;
        }
    }
    if index < bytes.len() && (bytes[index] == b'e' || bytes[index] == b'E') {
        index += 1;
        if index < bytes.len() && (bytes[index] == b'+' || bytes[index] == b'-') {
            index += 1;
        }
        while index < bytes.len() && bytes[index].is_ascii_digit() {
            index += 1;
        }
    }
    let number = std::str::from_utf8(&bytes[start..index])?.parse::<f64>()?;
    Ok((number, index))
}

fn parse_name(bytes: &[u8], mut index: usize) -> Result<(String, usize)> {
    let start = index;
    while index < bytes.len() && !is_whitespace(bytes[index]) && !is_delimiter(bytes[index]) {
        index += 1;
    }
    let name = std::str::from_utf8(&bytes[start..index])?.to_string();
    Ok((name, index))
}

fn parse_string(bytes: &[u8], mut index: usize) -> Result<(Vec<u8>, usize)> {
    index += 1; // skip '('
    let mut depth = 1;
    let mut output = Vec::new();
    while index < bytes.len() {
        let byte = bytes[index];
        match byte {
            b'\\' => {
                if index + 1 < bytes.len() {
                    output.push(bytes[index + 1]);
                    index += 2;
                } else {
                    index += 1;
                }
            }
            b'(' => {
                depth += 1;
                output.push(byte);
                index += 1;
            }
            b')' => {
                depth -= 1;
                if depth == 0 {
                    index += 1;
                    break;
                }
                output.push(byte);
                index += 1;
            }
            _ => {
                output.push(byte);
                index += 1;
            }
        }
    }
    Ok((output, index))
}

fn parse_array(bytes: &[u8], mut index: usize) -> Result<(Vec<Operand>, usize)> {
    index += 1; // skip '['
    let mut items = Vec::new();
    loop {
        skip_whitespace(bytes, &mut index);
        if index >= bytes.len() {
            break;
        }
        match bytes[index] {
            b']' => {
                index += 1;
                break;
            }
            b'(' => {
                let (value, next) = parse_string(bytes, index)?;
                items.push(Operand::String(value));
                index = next;
            }
            b'[' => {
                let (value, next) = parse_array(bytes, index)?;
                items.push(Operand::Array(value));
                index = next;
            }
            b'/' => {
                let (name, next) = parse_name(bytes, index + 1)?;
                items.push(Operand::Name(name));
                index = next;
            }
            b'+' | b'-' | b'.' | b'0'..=b'9' => {
                let (number, next) = parse_number(bytes, index)?;
                items.push(Operand::Number(number));
                index = next;
            }
            _ => {
                let (word, next) = parse_word(bytes, index)?;
                if word.is_empty() {
                    index = next;
                    continue;
                }
                items.push(Operand::Other(word));
                index = next;
            }
        }
    }
    Ok((items, index))
}

impl Operator {
    fn from_word(word: &str) -> Self {
        match word {
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
            other => Operator::Other(other.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenises_cm() {
        let data = b"1 0 0 1 10 20 cm q";
        let tokens = tokenize_stream(data).unwrap();
        assert_eq!(tokens.len(), 2);
        assert!(matches!(tokens[0].operator, Operator::Cm));
        assert_eq!(tokens[0].operands.len(), 6);
        assert!(matches!(tokens[1].operator, Operator::q));
    }

    #[test]
    fn tokenises_text_block() {
        let data = b"BT /F1 12 Tf 1 0 0 1 100 200 Tm (Hello) Tj ET";
        let tokens = tokenize_stream(data).unwrap();
        assert_eq!(tokens.len(), 6);
        assert!(matches!(tokens[0].operator, Operator::BT));
        assert!(matches!(tokens[1].operator, Operator::Tf));
        assert!(matches!(tokens[2].operator, Operator::Tm));
        assert!(matches!(tokens[3].operator, Operator::Tj));
        assert!(matches!(tokens[5].operator, Operator::ET));
    }

    #[test]
    fn tokenises_tj_array() {
        let data = b"BT [(Hi) 120 (There)] TJ ET";
        let tokens = tokenize_stream(data).unwrap();
        assert_eq!(tokens.len(), 4);
        assert!(matches!(tokens[1].operator, Operator::TJ));
        if let Operand::Array(items) = &tokens[1].operands[0] {
            assert_eq!(items.len(), 3);
        } else {
            panic!("expected array operand");
        }
    }

    #[test]
    fn tokenises_do() {
        let data = b"q 100 0 0 100 10 20 cm /Im0 Do Q";
        let tokens = tokenize_stream(data).unwrap();
        assert_eq!(tokens.len(), 4);
        assert!(matches!(tokens[2].operator, Operator::Do));
        assert!(matches!(tokens[2].operands[0], Operand::Name(ref n) if n == "Im0"));
    }

    #[test]
    fn tokenises_graphics_state_wrappers() {
        let data = b"q 2 0 0 2 0 0 cm q 1 0 0 1 10 10 cm Q Q";
        let tokens = tokenize_stream(data).unwrap();
        let ops: Vec<_> = tokens.iter().map(|t| &t.operator).collect();
        assert_eq!(ops.len(), 6);
        assert!(matches!(ops[0], Operator::q));
        assert!(matches!(ops[1], Operator::Cm));
        assert!(matches!(ops[2], Operator::q));
        assert!(matches!(ops[3], Operator::Cm));
        assert!(matches!(ops[4], Operator::Q));
        assert!(matches!(ops[5], Operator::Q));
    }

    #[test]
    fn token_spans_cover_original_bytes() {
        let data = b"BT /F1 9 Tf\n1 0 0 1 15 25 Tm\n(Hey) Tj\nET";
        let tokens = tokenize_stream(data).unwrap();
        assert_eq!(tokens.len(), 5);
        let snippets: Vec<_> = tokens
            .iter()
            .map(|t| std::str::from_utf8(&data[t.span.clone()]).unwrap())
            .collect();
        assert_eq!(snippets, vec!["BT", "/F1 9 Tf", "1 0 0 1 15 25 Tm", "(Hey) Tj", "ET"]);
    }

    #[test]
    fn skips_comments_and_blank_lines() {
        let data = b"BT\n% licenced matrix tweak\n/F2 14 Tf\n1 0 Td\nT*\n(Hi) Tj\nET";
        let tokens = tokenize_stream(data).unwrap();
        assert_eq!(tokens.len(), 6);
        assert!(matches!(tokens[1].operator, Operator::Tf));
        assert!(matches!(tokens[2].operator, Operator::Td));
        assert!(matches!(tokens[3].operator, Operator::TStar));
    }
}
