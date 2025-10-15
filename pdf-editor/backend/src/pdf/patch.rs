use crate::pdf::loader::StoreEntry;
use crate::types::PatchOp;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Serialize)]
pub struct PatchResponse {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_pdf: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remap: Option<HashMap<String, RemapEntry>>,
}

#[derive(Debug, Serialize)]
pub struct RemapEntry {
    #[serde(rename = "pdfRef")]
    pub pdf_ref: crate::types::PdfRef,
}

pub fn apply_patches(entry: &mut StoreEntry, ops: Vec<PatchOp>) -> anyhow::Result<PatchResponse> {
    // TODO: implement real patching logic. For the stub, we simply acknowledge the request
    // without mutating the underlying PDF.
    if ops.is_empty() {
        return Ok(PatchResponse { ok: true, updated_pdf: None, remap: None });
    }

    crate::pdf::write::incremental_update(entry)?;

    Ok(PatchResponse { ok: true, updated_pdf: None, remap: None })
}
