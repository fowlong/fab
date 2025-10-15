//! Incremental PDF writer placeholder.

use anyhow::Result;

use crate::types::DocumentIR;

/// Write a new revision of the PDF. Currently returns the original bytes
/// untouched until the incremental writer is implemented.
pub fn incremental_update(_ir: &DocumentIR, original: &[u8]) -> Result<Vec<u8>> {
    Ok(original.to_vec())
}
