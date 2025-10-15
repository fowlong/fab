use anyhow::Result;

use crate::types::{IrDocument, IrObject, IrPage, TextObject};

/// Extracts a simplified IR from the provided PDF bytes.
///
/// The current implementation is a stub that returns a single empty page.
pub fn extract_document_ir(_bytes: &[u8]) -> Result<IrDocument> {
    let page = IrPage {
        index: 0,
        width_pt: 595.276,
        height_pt: 841.89,
        objects: vec![IrObject::Text(TextObject {
            id: "t:stub".to_string(),
            unicode: "Placeholder text".to_string(),
            bbox: [100.0, 700.0, 250.0, 720.0],
        })],
    };

    Ok(IrDocument { pages: vec![page] })
}
