//! Patch orchestration placeholder.

use crate::types::{DocumentIr, PatchOperation};

/// Applies patches to the in-memory IR. Returns a copy for now.
#[must_use]
pub fn apply_patches(ir: &DocumentIr, patches: &[PatchOperation]) -> DocumentIr {
    let _ = patches;
    ir.clone()
}
