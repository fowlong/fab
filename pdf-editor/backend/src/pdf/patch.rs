use std::collections::HashMap;

use anyhow::Result;

use crate::types::PatchOperation;

use super::write::{PdfUpdate, incremental_save};

pub fn apply_patch_operations(bytes: &[u8], _ops: &[PatchOperation]) -> Result<PdfUpdate> {
    // Placeholder: clone the original document until editing is implemented.
    Ok(PdfUpdate {
        updated_pdf: incremental_save(bytes),
        remap: Some(HashMap::new()),
    })
}
