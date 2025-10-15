//! Incremental PDF writer.
//!
//! This module will manage appending new/updated objects to an existing PDF while keeping
//! original objects intact. The stub exposes a placeholder API for future work.

use anyhow::{anyhow, Result};

use crate::types::{DocumentId, PatchOperation};

pub fn apply_patch_incremental(
    _doc_id: &DocumentId,
    _ops: &[PatchOperation],
    _original_pdf: &[u8],
) -> Result<Vec<u8>> {
    Err(anyhow!(
        "incremental PDF writing is not yet implemented; follow the design notes to add support"
    ))
}
