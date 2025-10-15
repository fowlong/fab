use anyhow::{Context, Result};
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use base64::Engine;

use crate::pdf::write;
use crate::types::{DocumentIr, PatchOperation, PatchResponse};

pub async fn apply_patch_ops(
    ir: &DocumentIr,
    ops: &[PatchOperation],
) -> Result<(DocumentIr, PatchResponse, Vec<u8>)> {
    // Placeholder: clone the IR and acknowledge the operations without
    // mutating the PDF. This keeps the API consistent for the frontend
    // while incremental features are added.
    let mut next_ir = ir.clone();
    let mut response = PatchResponse::default();
    response.ok = true;

    let updated_pdf = write::noop_incremental(ir).context("write incremental")?;
    response.updated_pdf = Some(format!(
        "data:application/pdf;base64,{}",
        BASE64_STANDARD.encode(&updated_pdf)
    ));

    Ok((next_ir, response, updated_pdf))
}
