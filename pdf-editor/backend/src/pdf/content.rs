pub enum ContentToken {
    Operator(String),
    Number(f64),
    Name(String),
    String(Vec<u8>),
}

pub fn parse_stream(_data: &[u8]) -> Vec<ContentToken> {
    Vec::new()
}
