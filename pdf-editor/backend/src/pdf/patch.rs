use anyhow::{bail, Result};

use crate::types::{DocumentIR, PatchOp};

pub fn apply_patches(_ir: &mut DocumentIR, ops: &[PatchOp]) -> Result<()> {
    if ops.is_empty() {
        return Ok(());
    }
    bail!("patch application not yet implemented")
}
