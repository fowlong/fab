use anyhow::Result;

use crate::types::{DocumentIr, PageIr};

pub fn extract_ir(_bytes: &[u8]) -> Result<DocumentIr> {
    Ok(DocumentIr {
        pages: vec![PageIr {
            index: 0,
            width_pt: 595.276,
            height_pt: 841.89,
            objects: Vec::new(),
        }],
    })
}
