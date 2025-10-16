//! JSON patch application placeholder.

use anyhow::Result;

use crate::types::{DocumentIR, PatchOperation};

/// Apply a list of patch operations to the supplied IR.
/// The stubbed implementation simply logs the operations.
pub fn apply_patches(ir: &mut DocumentIR, ops: &[PatchOperation]) -> Result<()> {
    for op in ops {
        tracing::info!(?op, "received patch op");
    }
    let _ = ir; // suppress unused warning until real logic lands
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_patches_is_noop_in_stub() {
        let mut ir = DocumentIR::sample();
        let original = ir.clone();
        let ops = vec![PatchOperation::SetStyle {
            target: crate::types::PatchTarget {
                page: 0,
                id: "t:42".into(),
            },
            style: crate::types::StylePayload {
                fill_color: Some([1.0, 0.0, 0.0]),
                ..Default::default()
            },
        }];

        apply_patches(&mut ir, &ops).expect("patch application should succeed");
        assert_eq!(ir, original);
    }

    #[test]
    fn apply_patches_accepts_empty_operations() {
        let mut ir = DocumentIR::sample();
        apply_patches(&mut ir, &[]).expect("empty patch list should succeed");
    }
}
