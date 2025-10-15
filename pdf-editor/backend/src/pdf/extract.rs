#![allow(dead_code)]

use anyhow::Result;

use crate::types::{DocumentIr, PageIr};

pub fn build_ir(_bytes: &[u8]) -> Result<DocumentIr> {
    // Stub IR with an empty single page to keep the UI functional.
    Ok(DocumentIr {
        pages: vec![PageIr {
            index: 0,
            width_pt: 595.0,
            height_pt: 842.0,
            objects: Vec::new(),
        }],
    })
}
