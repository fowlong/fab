use anyhow::{anyhow, Result};

use crate::types::PatchOperation;

pub fn apply_patch(_bytes: &mut Vec<u8>, ops: &[PatchOperation]) -> Result<()> {
    if !ops.is_empty() {
        return Err(anyhow!("patch application not yet implemented"));
    }
    Ok(())
}
