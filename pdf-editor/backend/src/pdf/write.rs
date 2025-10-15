use anyhow::Result;

use crate::types::DocumentIR;

pub fn incremental_save(_ir: &DocumentIR, current_pdf: &[u8]) -> Result<Vec<u8>> {
    // Placeholder implementation: forward existing bytes.
    Ok(current_pdf.to_vec())
}
