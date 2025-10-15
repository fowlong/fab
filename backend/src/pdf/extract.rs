//! Extraction pipeline stub.

use anyhow::Result;

use crate::types::DocumentIR;

/// Produce an intermediate representation from a PDF document.
/// Currently returns the sample IR used by the mocked backend routes.
pub fn extract_ir(_pdf_bytes: &[u8]) -> Result<DocumentIR> {
    Ok(DocumentIR::sample())
}
