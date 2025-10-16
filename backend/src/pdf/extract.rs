//! Extraction pipeline stub.

use anyhow::Result;

use crate::types::DocumentIR;

/// Produce an intermediate representation from a PDF document.
/// Currently returns the sample IR used by the mocked backend routes.
pub fn extract_ir(_pdf_bytes: &[u8]) -> Result<DocumentIR> {
    Ok(DocumentIR::sample())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_ir_returns_sample_ir() {
        let bytes = include_bytes!("../../../e2e/sample.pdf");
        let ir = extract_ir(bytes).expect("extracting IR should succeed");
        assert_eq!(ir, DocumentIR::sample());
    }
}
