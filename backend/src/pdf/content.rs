//! Content stream tokenisation helpers used by the MVP extractor and
//! patcher. The implementation intentionally focuses on a small subset of
//! the PDF instruction set that is required for text and image placement.

use std::{
    fmt,
    ops::Range,
    str::{self, FromStr},
};

use anyhow::{anyhow, bail, Context, Result};

/// Affine matrix represented in PDF order `[a, b, c, d, e, f]`.
pub type Matrix = [f64; 6];

/// Supported operator kinds emitted by the tokenizer. Operators outside this
/// list are ignored by higher level code but retained in the token stream so
/// that byte ranges remain accurate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

impl Operator {
    fn from_bytes(bytes: &[u8]) -> Self {
        match bytes {
            b"Q" => Operator::Q,
            b"q" => Operator::q,
            b"cm" => Operator::Cm,
            b"BT" => Operator::Bt,
            b"ET" => Operator::Et,
            b"Tf" => Operator::Tf,
            b"Tm" => Operator::Tm,
            b"Td" => Operator::Td,
            b"T*" => Operator::TStar,
            b"Tj" => Operator::Tj,
            b"TJ" => Operator::TjArray,
            b"Do" => Operator::Do,
            _ => Operator::Other,
        }
    }
}

/// Operand representation for a content operation. Only the types required by
/// the MVP (numbers, names, strings and arrays) are modelled explicitly. The
/// rest are captured as opaque raw strings to maintain byte accounting.
#[derive(Debug, Clone, PartialEq)]
pub enum Operand {
    Number(f64),
    Name(String),
    Str(String),
    Array(Vec<Operand>),
    Raw(String),
}

/// Parsed content operation with byte span information relative to the
/// decompressed stream.
#[derive(Debug, Clone)]
pub struct ContentOp {
    pub operator: Operator,
    pub operator_raw: String,
    pub operands: Vec<Operand>,
    pub range: Range<usize>,
}

impl fmt::Display for ContentOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.operator_raw)?;
        for operand in &self.operands {
            write!(f, " {}", operand)?;
        }
        Ok(())
    }
}

impl fmt::Display for Operand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Operand::Number(n) => {
                if n.fract() == 0.0 {
                    write!(f, "{:.0}", n)
                } else {
                    write!(f, "{n}")
                }
            }
            Operand::Name(name) => write!(f, "/{name}"),
            Operand::Str(value) => write!(f, "({value})"),
            Operand::Array(items) => {
                write!(f, "[")?;
                let mut first = true;
                for item in items {
                    if !first {
                        write!(f, " ")?;
                    }
                    write!(f, "{item}")?;
                    first = false;
                }
                write!(f, "]")
            }
            Operand::Raw(raw) => write!(f, "{raw}"),
        }
    }
}

/// Tokenise a PDF content stream while recording byte ranges.
pub fn tokenize_stream(bytes: &[u8]) -> Result<Vec<ContentOp>> {
    let mut index = 0usize;
    let mut ops = Vec::new();
    let mut operands = Vec::new();
    let mut span_start: Option<usize> = None;

    while index < bytes.len() {
        skip_ws_and_comments(bytes, &mut index);
        if index >= bytes.len() {
            break;
        }

        let token_start = index;
        let (token, token_end) = parse_token(bytes, index)?;
        index = token_end;
        match token {
            ParsedToken::Operator(raw) => {
                let op = Operator::from_bytes(raw.as_slice());
                let start = span_start.unwrap_or(token_start);
                let range = start..token_end;
                ops.push(ContentOp {
                    operator: op,
                    operator_raw: String::from_utf8_lossy(raw.as_slice()).into_owned(),
                    operands: std::mem::take(&mut operands),
                    range,
                });
                span_start = None;
            }
            ParsedToken::Operand(operand) => {
                if span_start.is_none() {
                    span_start = Some(token_start);
                }
                operands.push(operand);
            }
        }
    }

    Ok(ops)
}

enum ParsedToken {
    Operator(Vec<u8>),
    Operand(Operand),
}

fn skip_ws_and_comments(bytes: &[u8], index: &mut usize) {
    while *index < bytes.len() {
        match bytes[*index] {
            b'\n' | b'\r' | b'\t' | b'\x0c' | b'\x00' | b' ' => *index += 1,
            b'%' => {
                while *index < bytes.len() && !matches!(bytes[*index], b'\n' | b'\r') {
                    *index += 1;
                }
            }
            _ => break,
        }
    }
}

fn parse_token(bytes: &[u8], index: usize) -> Result<(ParsedToken, usize)> {
    if index >= bytes.len() {
        return Err(anyhow!("unexpected end of stream"));
    }
    match bytes[index] {
        b'(' => parse_literal_string(bytes, index)
            .map(|(s, end)| (ParsedToken::Operand(Operand::Str(s)), end)),
        b'<' => {
            if index + 1 < bytes.len() && bytes[index + 1] == b'<' {
                // Dictionary start – treat as raw token for stability.
                parse_raw(bytes, index)
            } else {
                parse_hex_string(bytes, index)
                    .map(|(s, end)| (ParsedToken::Operand(Operand::Str(s)), end))
            }
        }
        b'[' => parse_array(bytes, index)
            .map(|(arr, end)| (ParsedToken::Operand(Operand::Array(arr)), end)),
        b'/' => parse_name(bytes, index)
            .map(|(name, end)| (ParsedToken::Operand(Operand::Name(name)), end)),
        b'+' | b'-' | b'.' | b'0'..=b'9' => parse_number(bytes, index)
            .map(|(value, end)| (ParsedToken::Operand(Operand::Number(value)), end)),
        _ => {
            let (raw, end) = parse_keyword(bytes, index)?;
            Ok((ParsedToken::Operator(raw), end))
        }
    }
}

fn parse_keyword(bytes: &[u8], mut index: usize) -> Result<(Vec<u8>, usize)> {
    let start = index;
    while index < bytes.len()
        && !matches!(
            bytes[index],
            b'\n' | b'\r' | b'\t' | b'\x0c' | b'\x00' | b' ' | b'[' | b']'
        )
    {
        index += 1;
    }
    if start == index {
        bail!("empty keyword");
    }
    Ok((bytes[start..index].to_vec(), index))
}

fn parse_raw(bytes: &[u8], mut index: usize) -> Result<(ParsedToken, usize)> {
    let start = index;
    while index < bytes.len()
        && !matches!(
            bytes[index],
            b'\n' | b'\r' | b'\t' | b'\x0c' | b'\x00' | b' '
        )
    {
        index += 1;
    }
    Ok((
        ParsedToken::Operand(Operand::Raw(
            String::from_utf8_lossy(&bytes[start..index]).into_owned(),
        )),
        index,
    ))
}

fn parse_number(bytes: &[u8], mut index: usize) -> Result<(f64, usize)> {
    let start = index;
    while index < bytes.len() && matches!(bytes[index], b'+' | b'-' | b'.' | b'0'..=b'9') {
        index += 1;
    }
    let slice = &bytes[start..index];
    let text = str::from_utf8(slice).context("invalid number token")?;
    let value = f64::from_str(text).context("failed to parse number")?;
    Ok((value, index))
}

fn parse_name(bytes: &[u8], mut index: usize) -> Result<(String, usize)> {
    index += 1; // skip leading '/'
    let start = index;
    while index < bytes.len()
        && !matches!(
            bytes[index],
            b'\n' | b'\r' | b'\t' | b'\x0c' | b'\x00' | b' ' | b'/' | b'[' | b']'
        )
    {
        index += 1;
    }
    let text = String::from_utf8(bytes[start..index].to_vec()).context("invalid name token")?;
    Ok((text, index))
}

fn parse_literal_string(bytes: &[u8], mut index: usize) -> Result<(String, usize)> {
    index += 1; // skip '('
    let start = index;
    let mut depth = 1usize;
    let mut result = Vec::new();
    while index < bytes.len() {
        let byte = bytes[index];
        match byte {
            b'\\' => {
                index += 1;
                if index < bytes.len() {
                    result.push(bytes[index]);
                }
            }
            b'(' => {
                depth += 1;
                result.push(byte);
            }
            b')' => {
                depth -= 1;
                if depth == 0 {
                    index += 1;
                    break;
                }
                result.push(byte);
            }
            _ => result.push(byte),
        }
        index += 1;
    }
    if depth != 0 {
        bail!("unterminated string starting at byte {start}");
    }
    let text = String::from_utf8_lossy(&result).into_owned();
    Ok((text, index))
}

fn parse_hex_string(bytes: &[u8], mut index: usize) -> Result<(String, usize)> {
    index += 1; // skip '<'
    let start = index;
    let mut hex = Vec::new();
    while index < bytes.len() && bytes[index] != b'>' {
        if !matches!(
            bytes[index],
            b'\n' | b'\r' | b'\t' | b'\x0c' | b'\x00' | b' '
        ) {
            hex.push(bytes[index]);
        }
        index += 1;
    }
    if index >= bytes.len() {
        bail!("unterminated hex string at byte {start}");
    }
    index += 1; // skip '>'
    if hex.len() % 2 == 1 {
        hex.push(b'0');
    }
    let mut bytes_out = Vec::new();
    for chunk in hex.chunks(2) {
        let text = str::from_utf8(chunk).context("invalid hex digit")?;
        let value = u8::from_str_radix(text, 16).context("failed to parse hex digit")?;
        bytes_out.push(value);
    }
    let text = String::from_utf8_lossy(&bytes_out).into_owned();
    Ok((text, index))
}

fn parse_array(bytes: &[u8], mut index: usize) -> Result<(Vec<Operand>, usize)> {
    index += 1; // skip '['
    let mut items = Vec::new();
    loop {
        skip_ws_and_comments(bytes, &mut index);
        if index >= bytes.len() {
            bail!("unterminated array");
        }
        if bytes[index] == b']' {
            index += 1;
            break;
        }
        let (token, end) = parse_token(bytes, index)?;
        index = end;
        match token {
            ParsedToken::Operand(op) => items.push(op),
            ParsedToken::Operator(raw) => {
                items.push(Operand::Raw(String::from_utf8_lossy(&raw).into_owned()));
            }
        }
    }
    Ok((items, index))
}

/// Parse a matrix from operands. The slice must contain exactly six numbers.
pub fn parse_matrix(operands: &[Operand]) -> Result<Matrix> {
    if operands.len() != 6 {
        bail!("matrix requires six operands");
    }
    let mut matrix = [0.0; 6];
    for (idx, operand) in operands.iter().enumerate() {
        matrix[idx] = match operand {
            Operand::Number(value) => *value,
            Operand::Raw(text) => {
                f64::from_str(text).context("invalid numeric literal in matrix")?
            }
            other => bail!("unexpected operand {other:?} in matrix"),
        };
    }
    Ok(matrix)
}

/// Multiply two affine matrices.
pub fn multiply(lhs: &Matrix, rhs: &Matrix) -> Matrix {
    [
        lhs[0] * rhs[0] + lhs[2] * rhs[1],
        lhs[1] * rhs[0] + lhs[3] * rhs[1],
        lhs[0] * rhs[2] + lhs[2] * rhs[3],
        lhs[1] * rhs[2] + lhs[3] * rhs[3],
        lhs[0] * rhs[4] + lhs[2] * rhs[5] + lhs[4],
        lhs[1] * rhs[4] + lhs[3] * rhs[5] + lhs[5],
    ]
}

/// Compute the inverse of an affine matrix.
pub fn invert(matrix: &Matrix) -> Result<Matrix> {
    let det = matrix[0] * matrix[3] - matrix[1] * matrix[2];
    if det.abs() < 1e-9 {
        bail!("matrix is singular");
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

/// Format a matrix using trimmed floats suitable for reinserting into the
/// content stream. Small fractional parts are rounded to avoid noisy output.
pub fn format_matrix(matrix: &Matrix) -> String {
    matrix
        .iter()
        .map(|value| {
            if (value.fract()).abs() < 1e-6 {
                format!("{:.0}", value.round())
            } else {
                format!("{value:.6}")
                    .trim_end_matches('0')
                    .trim_end_matches('.')
                    .to_string()
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"
        q
        1 0 0 1 0 0 cm
        BT
        /F1 12 Tf
        10 0 0 10 50 750 Tm
        (Hello) Tj
        T*
        (World) Tj
        ET
        Q
        /Im1 Do
    "#;

    #[test]
    fn tokenize_basic_stream() {
        let bytes = SAMPLE.as_bytes();
        let ops = tokenize_stream(bytes).expect("tokenise stream");
        let tf = ops.iter().find(|op| op.operator == Operator::Tf).unwrap();
        assert_eq!(tf.operands.len(), 2);
        match &tf.operands[0] {
            Operand::Name(name) => assert_eq!(name, "F1"),
            other => panic!("expected name operand, got {:?}", other),
        }
        match &tf.operands[1] {
            Operand::Number(value) => assert_eq!(*value, 12.0),
            other => panic!("expected numeric operand, got {:?}", other),
        }

        let tm = ops.iter().find(|op| op.operator == Operator::Tm).unwrap();
        let matrix = parse_matrix(&tm.operands).expect("parse matrix");
        assert!((matrix[4] - 50.0).abs() < 1e-6);
        assert!(tm.range.start < tm.range.end);

        let tj_ops: Vec<_> = ops
            .iter()
            .filter(|op| op.operator == Operator::Tj)
            .collect();
        assert_eq!(tj_ops.len(), 2);
        for op in tj_ops {
            assert_eq!(op.operands.len(), 1);
        }

        let do_op = ops.iter().find(|op| op.operator == Operator::Do).unwrap();
        assert_eq!(do_op.operands.len(), 1);
    }

    #[test]
    fn invert_and_multiply_roundtrip() {
        let a: Matrix = [1.1, 0.2, 0.1, 0.9, 10.0, -5.0];
        let inv = invert(&a).expect("invert");
        let ident = multiply(&a, &inv);
        assert!((ident[0] - 1.0).abs() < 1e-9);
        assert!((ident[3] - 1.0).abs() < 1e-9);
        assert!(ident[1].abs() < 1e-9);
        assert!(ident[2].abs() < 1e-9);
        assert!(ident[4].abs() < 1e-9);
        assert!(ident[5].abs() < 1e-9);
    }
}
