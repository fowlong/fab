use anyhow::Result;

use crate::types::{PatchOperation, PatchResponse};

use super::loader::StoredDocument;

pub struct PatchOutcome {
    pub updated_bytes: Vec<u8>,
    pub response: PatchResponse,
}

pub fn apply_patch(doc: &StoredDocument, _ops: &[PatchOperation]) -> Result<PatchOutcome> {
    Ok(PatchOutcome {
        updated_bytes: doc.current_bytes.clone(),
        response: PatchResponse {
            ok: true,
            ..Default::default()
        },
    })
}
