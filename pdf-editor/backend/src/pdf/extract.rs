use anyhow::Result;

use crate::types::{IrDocument, IrPage};

pub fn extract_ir(_bytes: &[u8]) -> Result<IrDocument> {
    // TODO: convert PDF into IR representation
    Ok(IrDocument { pages: Vec::new() })
}

pub fn extract_page(_index: usize) -> Result<IrPage> {
    // TODO: parse individual page
    Ok(IrPage {
        index: 0,
        width_pt: 0.0,
        height_pt: 0.0,
        objects: Vec::new(),
    })
}
