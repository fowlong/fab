//! Incremental PDF writer.
//!
//! The real writer will patch content streams and append new objects without
//! rewriting the entire file. At this stage the module simply exposes placeholders
//! so the rest of the crate can compile and tests can exercise the API surface.

use anyhow::Result;

pub fn apply_noop_incremental_update(bytes: &[u8]) -> Result<Vec<u8>> {
    Ok(bytes.to_vec())
}
