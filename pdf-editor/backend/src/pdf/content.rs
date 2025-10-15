use anyhow::{bail, Result};

#[derive(Debug, Clone)]
pub enum ContentToken {
  Operator(String),
  Number(f64),
  Name(String),
  String(Vec<u8>),
}

pub fn parse_stream(_data: &[u8]) -> Result<Vec<ContentToken>> {
  // Placeholder: real implementation required for full editing support.
  bail!("content parser not implemented")
}

pub fn serialize_stream(_tokens: &[ContentToken]) -> Vec<u8> {
  // Placeholder serialization.
  b"q\nQ\n".to_vec()
}
