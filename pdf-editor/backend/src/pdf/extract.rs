//! Intermediate Representation (IR) extraction.

use anyhow::Result;

use crate::types::DocumentIr;

pub fn build_ir(_bytes: &[u8]) -> Result<DocumentIr> {
    // Placeholder implementation – returns an empty IR until the tokenizer is ready.
    Ok(DocumentIr::empty())
}
