//! Incremental writer placeholder.

use crate::types::PatchOperation;

/// Applies a list of patch operations and returns updated bytes.
#[must_use]
pub fn incremental_save(original: &[u8], patches: &[PatchOperation]) -> Vec<u8> {
    let _ = patches;
    original.to_vec()
}
