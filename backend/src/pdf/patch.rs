use anyhow::{bail, Result};

use crate::types::{DocumentIR, PatchOperation};

pub fn apply_patches(_doc: &mut DocumentIR, ops: &[PatchOperation]) -> Result<()> {
    if ops
        .iter()
        .any(|op| !matches!(op, PatchOperation::Transform(_)))
    {
        bail!("only transform operations are currently supported in the stub backend");
    }
    // The stub does not yet mutate the document. Future work should update matrices here.
    Ok(())
}
