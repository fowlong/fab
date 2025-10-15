//! Content stream helpers (placeholder implementation).

use crate::types::PatchOperation;

/// Represents a parsed PDF content stream.
#[derive(Debug, Default, Clone)]
pub struct ContentStream {
    pub raw_bytes: Vec<u8>,
}

impl ContentStream {
    #[must_use]
    pub fn new(raw_bytes: Vec<u8>) -> Self {
        Self { raw_bytes }
    }

    /// Applies a patch to the in-memory stream.
    pub fn apply_patch(&mut self, _patch: &PatchOperation) {
        // Real implementation will parse operators and splice bytes.
    }
}
