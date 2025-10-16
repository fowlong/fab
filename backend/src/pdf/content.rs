//! Content stream tokenisation utilities for PDF content streams.

use std::ops::Range;

use anyhow::{bail, Result};

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
    let mut lexer = Lexer::new(bytes);
    let mut tokens = Vec::new();
    let mut operands = Vec::new();
    let mut operand_start: Option<usize> = None;

    while let Some(item) = lexer.next_item()? {
        match item {
            Lexeme::Operand(op, span) => {
                if operand_start.is_none() {
                    operand_start = Some(span.start);
                }
                operands.push(op);
            }
            Lexeme::Operator(op, span) => {
                let start = operand_start.unwrap_or(span.start);
                let range = start..span.end;
                tokens.push(ContentToken {
                    operands: std::mem::take(&mut operands),
                    operator: op,
                    span: range,
                });
                operand_start = None;
            }
        }
    }

    Ok(tokens)
}

#[derive(Debug)]
struct Lexer<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> Lexer<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, pos: 0 }
    }

    fn next_item(&mut self) -> Result<Option<Lexeme>> {
        self.skip_ws();
        if self.pos >= self.bytes.len() {
            return Ok(None);
        }
        let ch = self.bytes[self.pos];
        match ch {
            b'%' => {
                self.skip_comment();
                self.next_item()
            }
            b'(' => {
                let (value, span) = self.read_string()?;
                Ok(Some(Lexeme::Operand(Operand::String(value), span)))
            }
            b'[' => {
                let (value, span) = self.read_array()?;
                Ok(Some(Lexeme::Operand(Operand::Array(value), span)))
            }
            b'<' => {
                if self.peek_char(1) == Some(b'<') {
                    // Dictionary start acts as separator; advance and continue.
                    self.pos += 2;
                    Ok(Some(Lexeme::Operator(
                        Operator::Other("<<".into()),
                        self.pos - 2..self.pos,
                    )))
                } else {
                    let (value, span) = self.read_hex_string()?;
                    Ok(Some(Lexeme::Operand(Operand::String(value), span)))
                }
            }
            b'/' => {
                let (value, span) = self.read_name()?;
                Ok(Some(Lexeme::Operand(Operand::Name(value), span)))
            }
            b'-' | b'+' | b'.' | b'0'..=b'9' => {
                let (value, span) = self.read_number()?;
                Ok(Some(Lexeme::Operand(Operand::Number(value), span)))
            }
            b']' => {
                let start = self.pos;
                self.pos += 1;
                Ok(Some(Lexeme::Operator(
                    Operator::Other("]".into()),
                    start..self.pos,
                )))
            }
            _ => {
                let (word, span) = self.read_word()?;
                let operator = match word.as_str() {
                    "q" => Operator::q,
                    "Q" => Operator::Q,
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
                };
                Ok(Some(Lexeme::Operator(operator, span)))
            }
        }
    }

    fn skip_ws(&mut self) {
        while let Some(&b) = self.bytes.get(self.pos) {
            if matches!(b, b'\t' | b'\n' | 0x0C | b'\r' | b' ') {
                self.pos += 1;
                continue;
            }
            break;
        }
    }

    fn skip_comment(&mut self) {
        while let Some(&b) = self.bytes.get(self.pos) {
            self.pos += 1;
            if b == b'\n' || b == b'\r' {
                break;
            }
        }
    }

    fn peek_char(&self, offset: usize) -> Option<u8> {
        self.bytes.get(self.pos + offset).copied()
    }

    fn read_word(&mut self) -> Result<(String, Range<usize>)> {
        let start = self.pos;
        while let Some(&b) = self.bytes.get(self.pos) {
            if is_delimiter(b) {
                break;
            }
            self.pos += 1;
        }
        if start == self.pos {
            bail!("expected word at position {start}");
        }
        let slice = &self.bytes[start..self.pos];
        Ok((String::from_utf8(slice.to_vec())?, start..self.pos))
    }

    fn read_number(&mut self) -> Result<(f64, Range<usize>)> {
        let start = self.pos;
        let mut end = self.pos;
        while let Some(&b) = self.bytes.get(end) {
            if matches!(b, b'0'..=b'9' | b'.' | b'+' | b'-') {
                end += 1;
            } else {
                break;
            }
        }
        if end == start {
            bail!("expected number at position {start}");
        }
        let slice = &self.bytes[start..end];
        let value = std::str::from_utf8(slice)?.parse::<f64>()?;
        self.pos = end;
        Ok((value, start..end))
    }

    fn read_name(&mut self) -> Result<(String, Range<usize>)> {
        let start = self.pos;
        self.pos += 1; // skip '/'
        while let Some(&b) = self.bytes.get(self.pos) {
            if is_delimiter(b) {
                break;
            }
            self.pos += 1;
        }
        let slice = &self.bytes[start + 1..self.pos];
        Ok((String::from_utf8(slice.to_vec())?, start..self.pos))
    }

    fn read_string(&mut self) -> Result<(Vec<u8>, Range<usize>)> {
        let start = self.pos;
        self.pos += 1; // skip '('
        let mut depth = 1usize;
        let mut output = Vec::new();
        while self.pos < self.bytes.len() {
            let b = self.bytes[self.pos];
            self.pos += 1;
            match b {
                b'\\' => {
                    if let Some(&next) = self.bytes.get(self.pos) {
                        let value = match next {
                            b'n' => b'\n',
                            b'r' => b'\r',
                            b't' => b'\t',
                            b'b' => 0x08,
                            b'f' => 0x0c,
                            b'(' => b'(',
                            b')' => b')',
                            b'\\' => b'\\',
                            other => other,
                        };
                        output.push(value);
                        self.pos += 1;
                    }
                }
                b'(' => {
                    depth += 1;
                    output.push(b);
                }
                b')' => {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                    output.push(b);
                }
                _ => output.push(b),
            }
        }
        if depth != 0 {
            bail!("unterminated string literal");
        }
        Ok((output, start..self.pos))
    }

    fn read_hex_string(&mut self) -> Result<(Vec<u8>, Range<usize>)> {
        let start = self.pos;
        self.pos += 1; // skip '<'
        let mut value = Vec::new();
        let mut buffer = Vec::new();
        while let Some(&b) = self.bytes.get(self.pos) {
            if b == b'>' {
                self.pos += 1;
                break;
            }
            if b.is_ascii_hexdigit() {
                buffer.push(b);
                if buffer.len() == 2 {
                    let byte = u8::from_str_radix(std::str::from_utf8(&buffer)?, 16)?;
                    value.push(byte);
                    buffer.clear();
                }
            }
            self.pos += 1;
        }
        if !buffer.is_empty() {
            buffer.push(b'0');
            let byte = u8::from_str_radix(std::str::from_utf8(&buffer[..2])?, 16)?;
            value.push(byte);
        }
        Ok((value, start..self.pos))
    }

    fn read_array(&mut self) -> Result<(Vec<Operand>, Range<usize>)> {
        let start = self.pos;
        self.pos += 1; // skip '['
        let mut values = Vec::new();
        loop {
            self.skip_ws();
            if self.pos >= self.bytes.len() {
                bail!("unterminated array");
            }
            if self.bytes[self.pos] == b']' {
                self.pos += 1;
                break;
            }
            let (operand, _) = self.read_operand()?;
            values.push(operand);
        }
        Ok((values, start..self.pos))
    }

    fn read_operand(&mut self) -> Result<(Operand, Range<usize>)> {
        self.skip_ws();
        if self.pos >= self.bytes.len() {
            bail!("unexpected end of stream while reading operand");
        }
        let ch = self.bytes[self.pos];
        match ch {
            b'(' => self
                .read_string()
                .map(|(v, span)| (Operand::String(v), span)),
            b'[' => self.read_array(),
            b'<' => {
                if self.peek_char(1) == Some(b'<') {
                    let start = self.pos;
                    self.pos += 2;
                    Ok((Operand::Name("<<".into()), start..self.pos))
                } else {
                    self.read_hex_string()
                        .map(|(v, span)| (Operand::String(v), span))
                }
            }
            b'/' => self.read_name().map(|(v, span)| (Operand::Name(v), span)),
            b'-' | b'+' | b'.' | b'0'..=b'9' => self
                .read_number()
                .map(|(v, span)| (Operand::Number(v), span)),
            _ => {
                let (word, span) = self.read_word()?;
                if let Ok(number) = word.parse::<f64>() {
                    Ok((Operand::Number(number), span))
                } else {
                    Ok((Operand::Name(word), span))
                }
            }
        }
    }
}

enum Lexeme {
    Operand(Operand, Range<usize>),
    Operator(Operator, Range<usize>),
}

fn is_delimiter(byte: u8) -> bool {
    matches!(
        byte,
        0x00 | b'\t'
            | b'\n'
            | 0x0C
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

    fn span(token: &ContentToken) -> (usize, usize) {
        (token.span.start, token.span.end)
    }

    #[test]
    fn parses_cm_sequence() {
        let input = b"q 1 0 0 1 10 20 cm /Im1 Do Q";
        let tokens = tokenize_stream(input).unwrap();
        assert_eq!(tokens.len(), 5);
        assert!(matches!(tokens[0].operator, Operator::q));
        assert!(matches!(tokens[1].operator, Operator::Cm));
        assert!(matches!(tokens[2].operator, Operator::Do));
        assert!(matches!(tokens[3].operator, Operator::Q));
        assert_eq!(span(&tokens[1]), (2, 20));
    }

    #[test]
    fn parses_text_block() {
        let input = b"BT /F1 12 Tf 100 200 Td (Hello) Tj ET";
        let tokens = tokenize_stream(input).unwrap();
        assert_eq!(tokens.len(), 6);
        assert!(matches!(tokens[0].operator, Operator::BT));
        assert!(matches!(tokens[1].operator, Operator::Tf));
        assert!(matches!(tokens[2].operator, Operator::Td));
        assert!(matches!(tokens[3].operator, Operator::Tj));
        assert!(matches!(tokens[5].operator, Operator::ET));
        if let Operand::String(bytes) = &tokens[3].operands[0] {
            assert_eq!(bytes, b"Hello");
        } else {
            panic!("expected string operand");
        }
    }

    #[test]
    fn parses_tj_array() {
        let input = b"BT (A) Tj [(B) 120 (C)] TJ ET";
        let tokens = tokenize_stream(input).unwrap();
        assert_eq!(tokens.len(), 6);
        assert!(matches!(tokens[2].operator, Operator::TJ));
        match &tokens[2].operands[0] {
            Operand::Array(items) => {
                assert_eq!(items.len(), 3);
            }
            _ => panic!("expected array"),
        }
    }
}
