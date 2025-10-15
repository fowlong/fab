use anyhow::Result;

use crate::types::{PatchOperation, PatchResponse};

pub fn apply_patch(_ops: &[PatchOperation]) -> Result<PatchResponse> {
    Ok(PatchResponse {
        ok: true,
        updated_pdf: None,
        remap: None,
    })
}
