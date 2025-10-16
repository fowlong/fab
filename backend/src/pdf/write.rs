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
    fn incremental_update_returns_clone() {
        let ir = DocumentIR::sample();
        let original = b"%PDF-1.7\n";
        let updated = incremental_update(&ir, original).expect("write should succeed");

        assert_eq!(updated, original);
        assert_ne!(updated.as_ptr(), original.as_ptr());
    }
}
