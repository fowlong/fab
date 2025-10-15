//! Utilities for loading and caching PDFs.

use std::path::PathBuf;

use anyhow::Result;
use lopdf::Document;

use super::extract::Page0Cache;

#[derive(Debug)]
pub struct LoadedDoc {
    pub path: PathBuf,
    pub bytes: Vec<u8>,
    pub doc: Document,
    pub page0_cache: Option<Page0Cache>,
}

impl LoadedDoc {
    pub fn new(path: PathBuf, bytes: Vec<u8>, doc: Document) -> Self {
        Self {
            path,
            bytes,
            doc,
            page0_cache: None,
        }
    }
}

pub fn parse_document(bytes: &[u8]) -> Result<Document> {
    let mut doc = Document::load_mem(bytes)?;
    if doc.version.is_empty() {
        doc.version = "1.7".to_string();
    }
    Ok(doc)
}

pub fn load(bytes: Vec<u8>, path: PathBuf) -> Result<LoadedDoc> {
    let doc = parse_document(&bytes)?;
    Ok(LoadedDoc::new(path, bytes, doc))
}
