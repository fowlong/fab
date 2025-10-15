use anyhow::Result;

use crate::types::DocumentIr;

pub fn extract_ir(_bytes: &[u8]) -> Result<DocumentIr> {
    Ok(DocumentIr::sample())
}
