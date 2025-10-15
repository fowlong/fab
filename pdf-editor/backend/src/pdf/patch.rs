use anyhow::Result;
use tracing::info;

use crate::types::{DocumentIR, PatchOp};

pub fn apply_patch(current_pdf: &[u8], _ir: &DocumentIR, ops: &[PatchOp]) -> Result<Vec<u8>> {
    info!(
        "received_patch_ops" = ops.len(),
        "Applying patch ops in placeholder implementation"
    );
    // Placeholder: real implementation will edit the PDF in-place.
    let mut updated = current_pdf.to_vec();
    // Emit a no-op incremental save to keep the pipeline flowing.
    if !ops.is_empty() {
        updated.extend_from_slice(b"% patched placeholder\n");
    }
    Ok(updated)
}
