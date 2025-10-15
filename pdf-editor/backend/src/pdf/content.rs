#![allow(dead_code)]

pub enum ContentToken {
    Operator(String),
    Number(f64),
    Name(String),
    String(String),
}

pub fn parse_stream(_bytes: &[u8]) -> Vec<ContentToken> {
    Vec::new()
}
