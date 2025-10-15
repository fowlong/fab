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
