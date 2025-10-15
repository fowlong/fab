use anyhow::{Context, Result};
use lopdf::Document;

use crate::types::DocumentIr;

use super::extract;

pub struct LoadedDocument {
    pub doc: Document,
    pub ir: DocumentIr,
}

pub fn load_from_bytes(bytes: &[u8]) -> Result<LoadedDocument> {
    let mut doc = Document::load_mem(bytes)?;
    doc.version = "1.7".to_string();
    let ir = extract::extract_ir(&doc).context("failed to build IR")?;
    Ok(LoadedDocument { doc, ir })
}
