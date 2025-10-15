use anyhow::Result;

use crate::types::{IrObject, PdfState};

pub fn extract_ir(_pdf_bytes: &[u8]) -> Result<PdfState> {
    Ok(PdfState { pages: Vec::new() })
}

pub fn extract_objects_from_stream(_stream: &[u8]) -> Result<Vec<IrObject>> {
    Ok(Vec::new())
}
