//! Content stream tokenisation utilities.
//! The implementation is left as a stub; methods expose the desired API surface for future work.

use anyhow::{anyhow, Result};

#[derive(Debug, Clone)]
pub enum Operator {
    MoveTo,
    LineTo,
    CurveTo,
    ClosePath,
    Stroke,
    Fill,
    Save,
    Restore,
    ConcatMatrix,
    BeginText,
    EndText,
    SetTextMatrix,
    ShowText,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub operator: Operator;
    pub operands: Vec<String>;
}

pub fn parse_stream(_data: &[u8]) -> Result<Vec<Token>> {
    Err(anyhow!("content stream parsing not yet implemented"))
}

pub fn write_stream(_tokens: &[Token]) -> Result<Vec<u8>> {
    Err(anyhow!("content stream serialisation not yet implemented"))
}
