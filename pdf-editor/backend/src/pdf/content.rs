//! Content stream tokenisation utilities.
//!
//! The real implementation will parse PDF operators (e.g. `BT`, `Tf`, `cm`) and
//! expose helpers to rewrite individual spans. For now, the module only defines
//! lightweight placeholders so that the crate compiles while the rest of the
//! architecture is built out.

use anyhow::Result;

#[derive(Debug, Clone)]
pub struct ContentStream<'a> {
    pub data: &'a [u8],
}

impl<'a> ContentStream<'a> {
    pub fn from_bytes(data: &'a [u8]) -> Result<Self> {
        Ok(Self { data })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Matrix(pub [f32; 6]);

impl Matrix {
    pub fn identity() -> Self {
        Self([1.0, 0.0, 0.0, 1.0, 0.0, 0.0])
    }
}
