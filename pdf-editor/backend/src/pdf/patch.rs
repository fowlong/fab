use anyhow::Result;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use base64::Engine;
use serde_json::json;

use crate::types::{PatchOp, PatchResponse};

pub fn apply_patches(current: &mut Vec<u8>, ops: &[PatchOp]) -> Result<PatchResponse> {
    if !ops.is_empty() {
        println!("Received {} patch op(s).", ops.len());
    }
    let encoded = BASE64_STANDARD.encode(&current);
    let data_uri = format!("data:application/pdf;base64,{}", encoded);
    Ok(PatchResponse {
        ok: true,
        updated_pdf: Some(data_uri),
        remap: Some(json!({})),
    })
}
