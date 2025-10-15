use anyhow::{anyhow, Result};

use crate::types::PatchOp;

/// Performs lightweight validation on incoming patch operations.
pub fn validate_patch_ops(ops: &[PatchOp]) -> Result<()> {
    if ops.is_empty() {
        return Err(anyhow!("no patch operations supplied"));
    }

    // Additional validation can be added here as the patch logic is implemented.
    Ok(())
}
