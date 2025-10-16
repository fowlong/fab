//! Incremental PDF writer placeholder.

use anyhow::Result;

use crate::types::DocumentIR;

/// Write a new revision of the PDF. Currently returns the original bytes
/// untouched until the incremental writer is implemented.
pub fn incremental_update(_ir: &DocumentIR, original: &[u8]) -> Result<Vec<u8>> {
    Ok(original.to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn incremental_update_returns_identical_bytes() {
        let ir = DocumentIR::sample();
        let original = b"%PDF-1.7 stub content";

        let updated = incremental_update(&ir, original).expect("update should succeed");

        assert_eq!(updated, original);
        assert_ne!(updated.as_ptr(), original.as_ptr());
    }

    #[test]
    fn incremental_update_does_not_modify_ir() {
        let ir = DocumentIR::sample();
        let original = b"%PDF-1.4";

        let before = ir.clone();
        let result = incremental_update(&ir, original).expect("update should succeed");

        assert_eq!(result, original);
        assert_eq!(ir, before);
    }
}
