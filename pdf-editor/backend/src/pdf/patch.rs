use anyhow::Result;
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;

use crate::types::{DocumentIr, PatchOp, PatchResponse};

pub fn apply_patches(
    _ir: &mut DocumentIr,
    pdf_bytes: &[u8],
    _ops: &[PatchOp],
) -> Result<PatchResponse> {
    let data_url = format!("data:application/pdf;base64,{}", BASE64.encode(pdf_bytes));
    Ok(PatchResponse {
        ok: true,
        updated_pdf: Some(data_url),
        remap: None,
        message: Some("Patch processing not yet implemented; returning current PDF".into()),
    })
}
