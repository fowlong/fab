use anyhow::Result;

use crate::types::PatchOperation;

pub fn apply_patch(_ops: &[PatchOperation]) -> Result<()> {
    // TODO: mutate internal PDF representation based on patch operations
    Ok(())
}
