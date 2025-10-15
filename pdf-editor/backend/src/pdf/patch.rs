use anyhow::Result;
use serde_json::json;

use crate::types::{IrDocument, PatchOperation, PatchRequest};

pub struct PatchOutcome {
    pub updated_pdf: Vec<u8>,
    pub updated_ir: IrDocument,
    pub remap: serde_json::Value,
}

/// Applies patch operations to the PDF bytes and IR.
///
/// This MVP implementation simply echoes the existing bytes and IR without
/// modification, but wires the function so the rest of the stack can evolve
/// gradually.
pub fn apply_patch(_pdf: &[u8], ir: &IrDocument, ops: &PatchRequest) -> Result<PatchOutcome> {
    let mut updated_ir = ir.clone();

    if !ops.is_empty() {
        // As a placeholder we only log that we received operations by adding a
        // synthetic text object to the first page.
        if let Some(page) = updated_ir.pages.first_mut() {
            page.objects
                .push(crate::types::IrObject::Text(crate::types::TextObject {
                    id: format!("t:patched:{}", page.objects.len()),
                    unicode: format!("Applied {} op(s)", ops.len()),
                    bbox: [50.0, 650.0, 200.0, 670.0],
                }));
        }
    }

    Ok(PatchOutcome {
        updated_pdf: Vec::new(),
        updated_ir,
        remap: json!({}),
    })
}

pub fn apply_single_op(_op: &PatchOperation) {
    // placeholder for future logic
}
