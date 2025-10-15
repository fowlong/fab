//! Content stream token utilities (placeholder).

use anyhow::Result;

#[derive(Debug, Clone)]
pub struct ContentStream {
    pub bytes: Vec<u8>,
}

impl ContentStream {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self { bytes }
    }

    pub fn rewrite(&mut self, _offset: usize, _payload: &[u8]) -> Result<()> {
        // Placeholder for future stream manipulation logic.
        Ok(())
    }
}
