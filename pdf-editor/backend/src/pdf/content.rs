use anyhow::{anyhow, Result};

#[derive(Debug, Clone)]
pub struct ContentStream {
    pub raw: Vec<u8>,
}

impl ContentStream {
    pub fn parse(bytes: &[u8]) -> Result<Self> {
        Ok(Self {
            raw: bytes.to_vec(),
        })
    }

    pub fn apply_transform(&mut self, _matrix: [f32; 6]) -> Result<()> {
        // Placeholder: the MVP implementation will splice PDF operators here.
        Err(anyhow!("content manipulation not yet implemented"))
    }
}
