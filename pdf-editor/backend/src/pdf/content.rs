//! Tokeniser and helpers for PDF content streams.

use anyhow::Result;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Operator {
    /// Generic placeholder operator variant so we have a compile-time type.
    Raw(String),
}

#[derive(Debug, Clone)]
pub struct Token {
    pub offset: usize,
    pub operator: Operator,
}

#[derive(Debug, Default, Clone)]
pub struct ContentStream {
    pub tokens: Vec<Token>,
}

impl ContentStream {
    pub fn parse(_bytes: &[u8]) -> Result<Self> {
        // TODO: implement BT/ET, path ops, etc.
        Ok(Self { tokens: Vec::new() })
    }

    pub fn rewrite(&self) -> Vec<u8> {
        // TODO: serialize tokens back to a PDF stream.
        let mut output = Vec::new();
        for token in &self.tokens {
            output.extend_from_slice(token.operator_name().as_bytes());
            output.push(b'\n');
        }
        output
    }
}

impl Operator {
    fn operator_name(&self) -> &str {
        match self {
            Operator::Raw(name) => name.as_str(),
        }
    }
}
