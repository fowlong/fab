use anyhow::Result;

use crate::types::{IrDocument, IrPage};

pub fn extract_ir(bytes: &[u8]) -> Result<IrDocument> {
    // Placeholder implementation: expose a single empty page so the frontend can render.
    let page = IrPage {
        index: 0,
        width_pt: 595.0,
        height_pt: 842.0,
        objects: Vec::new(),
    };
    Ok(IrDocument { pages: vec![page] })
}
