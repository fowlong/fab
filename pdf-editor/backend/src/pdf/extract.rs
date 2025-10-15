//! Intermediate representation (IR) extraction.
//!
//! This module will eventually traverse each page's content streams, tracking graphics state
//! and emitting high-level objects (text runs, images, paths). The scaffolding below documents
//! the intended entry points and data structures without providing a full implementation yet.

use anyhow::{anyhow, Result};

use crate::types::DocumentIR;

pub fn extract_ir_from_pdf(_bytes: &[u8]) -> Result<DocumentIR> {
    Err(anyhow!(
        "IR extraction not yet implemented; see module documentation for the planned algorithm"
    ))
}
