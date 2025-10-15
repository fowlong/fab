use bytes::Bytes;
use uuid::Uuid;

use crate::types::{PatchOp, PatchResponse};

pub fn apply_patch(_doc_id: Uuid, _ops: &[PatchOp]) -> anyhow::Result<PatchResponse> {
    Ok(PatchResponse {
        ok: true,
        updated_pdf: None,
        remap: None,
    })
}

pub fn incremental_save(_doc_id: Uuid) -> anyhow::Result<Bytes> {
    Ok(Bytes::new())
}
