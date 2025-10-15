//! Placeholder content stream tooling.

use anyhow::{bail, Result};

#[derive(Debug, Clone)]
pub struct ContentStream;

impl ContentStream {
    pub fn parse(_bytes: &[u8]) -> Result<Self> {
        Ok(Self)
    }

    pub fn splice_matrix<F>(&mut self, _offset: usize, mutator: F) -> Result<()>
    where
        F: FnOnce([f32; 6]) -> [f32; 6],
    {
        let _ = mutator([1.0, 0.0, 0.0, 1.0, 0.0, 0.0]);
        bail!("matrix splicing is not yet implemented")
    }
}
