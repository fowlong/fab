use crate::types::{PatchOp, PatchResponse};
use anyhow::Result;

pub fn apply_patches(data: &mut Vec<u8>, _ops: &[PatchOp]) -> Result<PatchResponse> {
  // Placeholder behaviour simply echoes the original document as a base64 data URI.
  let encoded = base64::engine::general_purpose::STANDARD.encode(data);
  Ok(PatchResponse {
    ok: true,
    updated_pdf: Some(format!("data:application/pdf;base64,{encoded}")),
    remap: None,
  })
}
