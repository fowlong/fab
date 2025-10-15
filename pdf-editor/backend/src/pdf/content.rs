use anyhow::Result;

#[derive(Debug, Clone)]
pub enum PdfToken {
    Operator(String),
    Number(f64),
    Name(String),
    String(Vec<u8>),
}

pub fn parse_stream(_data: &[u8]) -> Result<Vec<PdfToken>> {
    // TODO: parse PDF content stream into tokens
    Ok(Vec::new())
}
