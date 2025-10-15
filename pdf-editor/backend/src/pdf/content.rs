#[derive(Debug, Clone)]
pub enum Operator {
  Unknown(String),
}

#[derive(Debug, Clone)]
pub struct Token {
  pub raw: Vec<u8>,
  pub operator: Operator,
}

pub fn parse_stream(_bytes: &[u8]) -> Vec<Token> {
  Vec::new()
}
