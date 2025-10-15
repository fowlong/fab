#[derive(Debug, Clone)]
pub struct ContentToken {
    pub operator: String,
    pub operands: Vec<String>,
}

impl ContentToken {
    pub fn new(operator: impl Into<String>, operands: Vec<String>) -> Self {
        Self { operator: operator.into(), operands }
    }
}

pub fn tokenize_stream(_bytes: &[u8]) -> Vec<ContentToken> {
    // Placeholder: a full tokenizer will be implemented later.
    Vec::new()
}
