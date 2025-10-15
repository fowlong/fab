use anyhow::Result;
use std::sync::Arc;

use crate::types::DocumentIr;

#[derive(Clone)]
pub struct LoadedDocument {
    pub bytes: Arc<Vec<u8>>,
    pub ir: DocumentIr,
}

pub fn load_pdf(bytes: Vec<u8>) -> Result<LoadedDocument> {
    // Placeholder implementation: return a single empty page IR.
    let ir = DocumentIr { pages: Vec::new() };
    Ok(LoadedDocument {
        bytes: Arc::new(bytes),
        ir,
    })
}
