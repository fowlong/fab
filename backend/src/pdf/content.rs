//! Content stream tokeniser and helpers.
//!
//! The MVP keeps this module lightweight and focused on defining the
//! shapes that later parsing code will fill in. The real implementation
//! will iterate over PDF graphics instructions and expose them as an
//! iterator. For now we provide a couple of stub types that describe the
//! intended direction.

use anyhow::Result;

use crate::types::Matrix;

#[derive(Debug, Clone)]
pub struct GraphicsState {
    pub ctm: Matrix,
}

impl GraphicsState {
    pub fn identity() -> Self {
        Self {
            ctm: Matrix([1.0, 0.0, 0.0, 1.0, 0.0, 0.0]),
        }
    }
}

#[allow(dead_code)]
pub fn parse_stream(_bytes: &[u8]) -> Result<Vec<GraphicsState>> {
    // Placeholder. The future implementation will return a sequence of
    // drawing operations paired with their graphics state snapshots.
    Ok(vec![GraphicsState::identity()])
}
