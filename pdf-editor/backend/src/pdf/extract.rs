//! Conversion from lopdf documents into the JSON IR shared with the frontend.

use anyhow::Result;
use lopdf::Document;

use crate::types::{DocumentIr, PageIr};

pub fn extract_ir(_doc: &Document) -> Result<DocumentIr> {
    // TODO: Traverse PDF and build IR.
    Ok(DocumentIr {
        pages: vec![PageIr {
            index: 0,
            width_pt: 595.0,
            height_pt: 842.0,
            objects: Vec::new(),
        }],
        source_pdf_url: None,
    })
}
