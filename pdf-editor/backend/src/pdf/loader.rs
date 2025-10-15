use anyhow::{Context, Result};
use lopdf::Document;

use crate::types::DocumentIr;

use super::extract;

/// Representation of a loaded PDF document held in memory.
#[derive(Clone)]
pub struct LoadedDocument {
    original_bytes: Vec<u8>,
    current_bytes: Vec<u8>,
    ir: DocumentIr,
}

impl LoadedDocument {
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self> {
        // Validate the PDF by parsing it once. Any structural errors are surfaced
        // immediately to the caller.
        let _parsed = Document::load_mem(&bytes).context("invalid PDF structure")?;
        Ok(Self {
            original_bytes: bytes.clone(),
            current_bytes: bytes,
            ir: DocumentIr::empty(),
        })
    }

    pub fn refresh_ir(&mut self) -> Result<()> {
        let parsed = Document::load_mem(&self.current_bytes).context("failed to reload pdf")?;
        self.ir = extract::extract_ir(&parsed)?;
        Ok(())
    }

    pub fn ir(&self) -> &DocumentIr {
        &self.ir
    }

    pub fn ir_mut(&mut self) -> &mut DocumentIr {
        &mut self.ir
    }

    pub fn current_pdf(&self) -> &[u8] {
        &self.current_bytes
    }

    pub fn update_pdf_bytes(&mut self, bytes: Vec<u8>) {
        self.current_bytes = bytes;
    }

    pub fn original_pdf(&self) -> &[u8] {
        &self.original_bytes
    }
}
