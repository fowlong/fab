use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::Result;
use lopdf::Document;

use crate::types::DocumentIR;

use super::extract::{extract_document, CachedObject, PageContents, PageZeroState, StreamState};

/// Load a PDF document from raw bytes.
pub fn parse_document(bytes: &[u8]) -> Result<Document> {
    let mut doc = Document::load_mem(bytes)?;
    doc.version = "1.7".to_string();
    Ok(doc)
}

#[derive(Debug, Clone)]
pub struct LoadedDocument {
    pub bytes: Vec<u8>,
    pub path: PathBuf,
    pub doc: Document,
    pub ir: DocumentIR,
    pub page_zero: PageZeroState,
    pub streams: HashMap<u32, StreamState>,
    pub lookup: HashMap<String, CachedObject>,
}

impl LoadedDocument {
    pub fn new(bytes: Vec<u8>, path: PathBuf) -> Result<Self> {
        let doc = parse_document(&bytes)?;
        let extraction = extract_document(&doc)?;
        Ok(Self {
            bytes,
            path,
            doc,
            ir: extraction.ir,
            page_zero: extraction.page_zero,
            streams: extraction.streams,
            lookup: extraction.lookup,
        })
    }

    pub fn refresh(&mut self) -> Result<()> {
        self.doc = parse_document(&self.bytes)?;
        let extraction = extract_document(&self.doc)?;
        self.ir = extraction.ir;
        self.page_zero = extraction.page_zero;
        self.streams = extraction.streams;
        self.lookup = extraction.lookup;
        Ok(())
    }
}

pub fn page_contents_clone(contents: &PageContents) -> PageContents {
    match contents {
        PageContents::Single(id) => PageContents::Single(*id),
        PageContents::Array(ids) => PageContents::Array(ids.clone()),
    }
}
