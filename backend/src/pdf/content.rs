use std::{fmt, ops::Range};

use anyhow::{anyhow, Result};

/// Affine matrix helper type `[a, b, c, d, e, f]`.
pub type Matrix = [f64; 6];

/// Operand that can appear in a PDF content operation.
#[derive(Debug, Clone, PartialEq)]
pub enum Operand {
    Number(f64),
    Name(String),
    Array(Vec<Operand>),
    String(Vec<u8>),
    Other(String),
}

impl Operand {
    pub fn as_number(&self) -> Option<f64> {
        match self {
            Operand::Number(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_name(&self) -> Option<&str> {
        match self {
            Operand::Name(name) => Some(name.as_str()),
            _ => None,
        }
    }

    pub fn to_pdf(&self) -> String {
        match self {
            Operand::Number(n) => format_number(*n),
            Operand::Name(name) => format!("/{}", name),
            Operand::Array(items) => {
                let inner = items
                    .iter()
                    .map(|item| item.to_pdf())
                    .collect::<Vec<_>>()
                    .join(" ");
                format!("[{}]", inner)
            }
            Operand::String(bytes) => format!("({})", escape_string(bytes)),
            Operand::Other(value) => value.clone(),
        }
    }
}

/// Supported operator subset.
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
            Operator::Other(name) => name.as_str(),
        }
    }
}

/// A single PDF content operation and the byte range that produced it.
#[derive(Debug, Clone)]
pub struct Operation {
    pub operator: Operator,
    pub operands: Vec<Operand>,
    pub range: Range<usize>,
}

impl Operation {
    pub fn matrix(&self) -> Option<Matrix> {
        matrix_from_operands(&self.operands)
    }

    pub fn set_matrix(&mut self, matrix: &Matrix) {
        self.operands = matrix.iter().map(|value| Operand::Number(*value)).collect();
    }
}

/// Tokenise a decoded PDF content stream.
pub fn tokenize_stream(bytes: &[u8]) -> Result<Vec<Operation>> {
    let mut cursor = 0usize;
    let mut operations = Vec::new();

    while cursor < bytes.len() {
        skip_ws(bytes, &mut cursor);
        if cursor >= bytes.len() {
            break;
        }

        let statement_start = cursor;
        let mut operands = Vec::new();

        loop {
            skip_ws(bytes, &mut cursor);
            if cursor >= bytes.len() {
                break;
            }

            let token = read_token(bytes, &mut cursor)?;
            match token {
                Token::Operand(value) => operands.push(value),
                Token::Operator(operator) => {
                    let range = statement_start..cursor;
                    operations.push(Operation {
                        operator,
                        operands,
                        range,
                    });
                    break;
                }
            }
        }
    }

    Ok(operations)
}

/// Serialise operations back into a PDF content stream (decoded form).
pub fn serialize_operations(ops: &[Operation]) -> Vec<u8> {
    let mut buffer = Vec::new();
    for op in ops {
        for (index, operand) in op.operands.iter().enumerate() {
            if index > 0 {
                buffer.push(b' ');
            }
            buffer.extend_from_slice(operand.to_pdf().as_bytes());
        }
        if !op.operands.is_empty() {
            buffer.push(b' ');
        }
        buffer.extend_from_slice(op.operator.as_str().as_bytes());
        buffer.push(b'\n');
    }
    buffer
}

/// Multiply two affine matrices (a × b).
pub fn multiply(a: &Matrix, b: &Matrix) -> Matrix {
    [
        a[0] * b[0] + a[2] * b[1],
        a[1] * b[0] + a[3] * b[1],
        a[0] * b[2] + a[2] * b[3],
        a[1] * b[2] + a[3] * b[3],
        a[0] * b[4] + a[2] * b[5] + a[4],
        a[1] * b[4] + a[3] * b[5] + a[5],
    ]
}

/// Invert an affine matrix. Returns identity when singular.
pub fn invert(m: &Matrix) -> Matrix {
    let det = m[0] * m[3] - m[1] * m[2];
    if det.abs() < f64::EPSILON {
        return [1.0, 0.0, 0.0, 1.0, 0.0, 0.0];
    }
    let inv_det = 1.0 / det;
    let a = m[0] * inv_det;
    let b = m[1] * inv_det;
    let c = m[2] * inv_det;
    let d = m[3] * inv_det;
    let e = -(a * m[4] + c * m[5]);
    let f = -(b * m[4] + d * m[5]);
    [d, -b, -c, a, e, f]
}

/// Extract matrix values from operands.
pub fn matrix_from_operands(operands: &[Operand]) -> Option<Matrix> {
    if operands.len() != 6 {
        return None;
    }
    let mut result = [0.0; 6];
    for (index, operand) in operands.iter().enumerate() {
        result[index] = operand.as_number()?;
    }
    Some(result)
}

fn format_number(value: f64) -> String {
    if (value - value.round()).abs() < 1e-7 {
        format!("{:.0}", value.round())
    } else {
        let mut s = format!("{:.6}", value);
        while s.contains('.') && s.ends_with('0') {
            s.pop();
        }
        if s.ends_with('.') {
            s.pop();
        }
        s
    }
}

fn escape_string(bytes: &[u8]) -> String {
    let mut escaped = String::new();
    for &byte in bytes {
        match byte {
            b'(' | b')' | b'\\' => {
                escaped.push('\\');
                escaped.push(byte as char);
            }
            b'\n' => escaped.push_str("\\n"),
            b'\r' => escaped.push_str("\\r"),
            b'\t' => escaped.push_str("\\t"),
            _ => escaped.push(byte as char),
        }
    }
    escaped
}

fn skip_ws(bytes: &[u8], cursor: &mut usize) {
    while *cursor < bytes.len() {
        match bytes[*cursor] {
            b' ' | b'\t' | b'\r' | b'\n' | 0x0C | 0x00 => *cursor += 1,
            b'%' => {
                while *cursor < bytes.len() && bytes[*cursor] != b'\n' {
                    *cursor += 1;
                }
            }
            _ => break,
        }
    }
}

#[derive(Debug, Clone)]
enum Token {
    Operator(Operator),
    Operand(Operand),
}

fn read_token(bytes: &[u8], cursor: &mut usize) -> Result<Token> {
    skip_ws(bytes, cursor);
    if *cursor >= bytes.len() {
        return Err(anyhow!("unexpected end of stream"));
    }

    let byte = bytes[*cursor];
    if byte == b'/' {
        parse_name(bytes, cursor).map(Token::Operand)
    } else if byte == b'[' {
        parse_array(bytes, cursor).map(Token::Operand)
    } else if byte == b'(' {
        parse_string(bytes, cursor).map(Token::Operand)
    } else if is_number_start(byte) {
        parse_number(bytes, cursor).map(Token::Operand)
    } else {
        parse_operator(bytes, cursor).map(Token::Operator)
    }
}

fn parse_name(bytes: &[u8], cursor: &mut usize) -> Result<Operand> {
    *cursor += 1; // skip '/'
    let start = *cursor;
    while *cursor < bytes.len() && !is_delimiter(bytes[*cursor]) {
        *cursor += 1;
    }
    let name = std::str::from_utf8(&bytes[start..*cursor])?.to_string();
    Ok(Operand::Name(name))
}

fn parse_array(bytes: &[u8], cursor: &mut usize) -> Result<Operand> {
    *cursor += 1; // skip '['
    let mut items = Vec::new();
    loop {
        skip_ws(bytes, cursor);
        if *cursor >= bytes.len() {
            return Err(anyhow!("unterminated array"));
        }
        if bytes[*cursor] == b']' {
            *cursor += 1;
            break;
        }
        items.push(match read_token(bytes, cursor)? {
            Token::Operand(op) => op,
            Token::Operator(op) => Operand::Other(op.as_str().to_string()),
        });
    }
    Ok(Operand::Array(items))
}

fn parse_string(bytes: &[u8], cursor: &mut usize) -> Result<Operand> {
    *cursor += 1; // skip '('
    let mut depth = 1;
    let mut output = Vec::new();
    while *cursor < bytes.len() {
        let byte = bytes[*cursor];
        *cursor += 1;
        match byte {
            b'\\' => {
                if *cursor < bytes.len() {
                    let escaped = bytes[*cursor];
                    *cursor += 1;
                    match escaped {
                        b'n' => output.push(b'\n'),
                        b'r' => output.push(b'\r'),
                        b't' => output.push(b'\t'),
                        other => output.push(other),
                    }
                }
            }
            b'(' => {
                depth += 1;
                output.push(byte);
            }
            b')' => {
                depth -= 1;
                if depth == 0 {
                    break;
                }
                output.push(byte);
            }
            _ => output.push(byte),
        }
    }
    if depth != 0 {
        return Err(anyhow!("unterminated string literal"));
    }
    Ok(Operand::String(output))
}

fn parse_number(bytes: &[u8], cursor: &mut usize) -> Result<Operand> {
    let start = *cursor;
    *cursor += 1;
    while *cursor < bytes.len() && is_number_body(bytes[*cursor]) {
        *cursor += 1;
    }
    let slice = &bytes[start..*cursor];
    let number = std::str::from_utf8(slice)?.parse::<f64>()?;
    Ok(Operand::Number(number))
}

fn parse_operator(bytes: &[u8], cursor: &mut usize) -> Result<Operator> {
    let start = *cursor;
    *cursor += 1;
    while *cursor < bytes.len() && !is_delimiter(bytes[*cursor]) {
        *cursor += 1;
    }
    let word = std::str::from_utf8(&bytes[start..*cursor])?;
    Ok(match word {
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
    })
}

fn is_number_start(byte: u8) -> bool {
    matches!(byte, b'+' | b'-' | b'.' | b'0'..=b'9')
}

fn is_number_body(byte: u8) -> bool {
    matches!(byte, b'+' | b'-' | b'.' | b'0'..=b'9' | b'E' | b'e')
}

fn is_delimiter(byte: u8) -> bool {
    matches!(
        byte,
        b' ' | b'\t'
            | b'\r'
            | b'\n'
            | 0x0C
            | 0x00
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

/// Apply matrix to point `(x, y)`.
pub fn apply_matrix(matrix: &Matrix, x: f64, y: f64) -> (f64, f64) {
    (
        matrix[0] * x + matrix[2] * y + matrix[4],
        matrix[1] * x + matrix[3] * y + matrix[5],
    )
}

/// Axis-aligned bounding box of transformed unit rectangle.
pub fn rect_from_matrix(matrix: &Matrix, width: f64, height: f64) -> [f64; 4] {
    let points = [
        apply_matrix(matrix, 0.0, 0.0),
        apply_matrix(matrix, width, 0.0),
        apply_matrix(matrix, 0.0, height),
        apply_matrix(matrix, width, height),
    ];
    let (mut min_x, mut min_y) = points[0];
    let (mut max_x, mut max_y) = points[0];
    for &(x, y) in &points[1..] {
        min_x = min_x.min(x);
        min_y = min_y.min(y);
        max_x = max_x.max(x);
        max_y = max_y.max(y);
    }
    [min_x, min_y, max_x, max_y]
}

impl fmt::Display for Operator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "q 1 0 0 1 0 0 cm /Im0 Do Q\n";

    #[test]
    fn parses_cm_and_do() {
        let ops = tokenize_stream(SAMPLE.as_bytes()).unwrap();
        assert_eq!(ops.len(), 4);
        assert!(matches!(ops[0].operator, Operator::q));
        assert!(matches!(ops[1].operator, Operator::Cm));
        assert_eq!(ops[1].operands.len(), 6);
        assert!(matches!(ops[2].operator, Operator::Do));
    }

    #[test]
    fn parses_bt_et() {
        let data = b"BT /F1 12 Tf 10 20 Td (Hello) Tj ET";
        let ops = tokenize_stream(data).unwrap();
        assert_eq!(ops.len(), 6);
        assert!(matches!(ops[0].operator, Operator::BT));
        assert!(matches!(ops[5].operator, Operator::ET));
    }

    #[test]
    fn parses_tj_and_tj_array() {
        let data = b"BT (Hi) Tj [ (A) 50 (B) ] TJ ET";
        let ops = tokenize_stream(data).unwrap();
        assert!(matches!(ops[1].operator, Operator::Tj));
        assert!(matches!(ops[2].operator, Operator::TJ));
        if let Operand::Array(items) = &ops[2].operands[0] {
            assert_eq!(items.len(), 3);
        } else {
            panic!("expected array operand");
        }
    }
}
