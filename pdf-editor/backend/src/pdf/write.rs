#![allow(dead_code)]

use anyhow::Result;

use crate::types::DocumentIr;

pub fn incremental_save(original: &[u8], _ir: &DocumentIr) -> Result<Vec<u8>> {
    // Placeholder: return original bytes for now.
    Ok(original.to_vec())
}
