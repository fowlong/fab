use crate::types::{DocumentIR, PatchOp};

pub fn apply_patch(
    bytes: &mut Vec<u8>,
    ir: &mut DocumentIR,
    ops: &[PatchOp],
) -> anyhow::Result<Vec<u8>> {
    // For now, this stub simply echoes the current bytes and does not mutate
    // the document. The infrastructure is in place so that future iterations
    // can implement true incremental updates.
    if ops.is_empty() {
        return Ok(bytes.clone());
    }

    Ok(bytes.clone())
}
