use anyhow::Result;

use crate::types::{DocumentIr, PatchOperations};

pub fn apply_operations(doc: &mut DocumentIr, ops: &PatchOperations) -> Result<()> {
    doc.last_patch = Some(ops.clone());
    Ok(())
}
