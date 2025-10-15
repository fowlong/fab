use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;

use crate::pdf::loader::Document;
use crate::types::{PatchOperation, PatchResponse};

pub fn apply_patches(_doc_id: &str, doc: &mut Document, ops: &[PatchOperation]) -> anyhow::Result<PatchResponse> {
  if ops.is_empty() {
    return Ok(PatchResponse {
      ok: true,
      updated_pdf: None,
      remap: serde_json::Value::Null,
    });
  }

  // For the scaffold we do not yet modify the PDF bytes. Instead, we
  // return the current document as a data URI so the frontend can display
  // the incremental update cycle during development.
  let encoded = BASE64.encode(&doc.bytes);
  let data_uri = format!("data:application/pdf;base64,{}", encoded);

  Ok(PatchResponse {
    ok: true,
    updated_pdf: Some(data_uri),
    remap: serde_json::Value::Null,
  })
}
