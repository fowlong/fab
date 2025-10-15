use anyhow::Result;

use crate::types::{DocumentIR, PatchOperation, PatchResponse};

pub struct PatchOutcome {
    pub response: PatchResponse,
    pub updated_bytes: Option<Vec<u8>>,
}

pub fn apply_patches(
    _ir: &mut DocumentIR,
    _bytes: &[u8],
    operations: &[PatchOperation],
) -> Result<PatchOutcome> {
    let mut response = PatchResponse::default();
    response.ok = true;
    if operations.is_empty() {
        return Ok(PatchOutcome {
            response,
            updated_bytes: None,
        });
    }

    Ok(PatchOutcome {
        response,
        updated_bytes: None,
    })
}
