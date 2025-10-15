use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use lopdf::Document;

use super::{write, Result};
use crate::pdf::loader::StoredPdf;
use crate::types::{PatchOperation, PatchResponse};

pub fn apply_patch(doc: &mut StoredPdf, ops: &[PatchOperation]) -> Result<PatchResponse> {
    if ops.is_empty() {
        return Ok(PatchResponse {
            ok: true,
            ..Default::default()
        });
    }
    let mut document = Document::load_mem(&doc.bytes)?;
    // Placeholder: a real implementation would apply operations here.
    // For the scaffold we simply append an empty incremental update.
    write::append_incremental_save(&mut document)?;
    let mut buffer = Vec::new();
    document.save_to(&mut buffer)?;
    doc.bytes = buffer.clone();
    Ok(PatchResponse {
        ok: true,
        updated_pdf: Some(format!(
            "data:application/pdf;base64,{}",
            BASE64.encode(&buffer)
        )),
        ..Default::default()
    })
}
