use anyhow::Result;
use base64::Engine;

use crate::types::{DocumentIr, PatchOperation, PatchResponse};

pub fn apply(
    _ir: &mut DocumentIr,
    bytes: &mut Vec<u8>,
    _ops: Vec<PatchOperation>,
) -> Result<PatchResponse> {
    // Incremental updates are not implemented yet; just echo the original PDF.
    let encoded = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, bytes);
    Ok(PatchResponse {
        ok: true,
        updated_pdf: format!("data:application/pdf;base64,{}", encoded),
        remap: None,
    })
}
