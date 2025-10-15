use anyhow::Result;

use crate::types::{PatchOperation, PatchResponse};

/// Apply a list of patch operations to the stored document. The MVP skeleton does
/// not yet modify the PDF content; it simply echoes success when at least one
/// operation is received.
pub fn apply_patch(ops: &[PatchOperation]) -> Result<PatchResponse> {
    if ops.is_empty() {
        return Ok(PatchResponse {
            ok: false,
            updated_pdf: None,
            remap: None,
            error: Some("no operations provided".into()),
        });
    }
    Ok(crate::pdf::write::build_response())
}
