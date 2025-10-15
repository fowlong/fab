use crate::types::{PatchOperation, PatchResponse};

pub fn apply_patch_ops(_ops: &[PatchOperation]) -> anyhow::Result<PatchResponse> {
    Ok(PatchResponse::ok())
}
