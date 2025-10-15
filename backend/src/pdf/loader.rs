//! Utilities for loading PDFs into the in-memory document store.

use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};
use lopdf::Document;

use crate::types::DocumentIR;

use super::extract::{extract_page_zero, ExtractedDocument, PageCache};

pub struct LoadedDoc {
    pub path: PathBuf,
    pub bytes: Vec<u8>,
    pub doc: Document,
    pub ir: DocumentIR,
    pub cache: PageCache,
    pub startxref: usize,
}

pub fn load(bytes: Vec<u8>, path: PathBuf) -> Result<LoadedDoc> {
    let mut doc = Document::load_mem(&bytes).context("failed to parse PDF bytes")?;
    doc.version = "1.7".to_string();
    let ExtractedDocument { ir, cache } = extract_page_zero(&doc)?;
    let startxref = find_startxref(&bytes)?;
    Ok(LoadedDoc {
        path,
        bytes,
        doc,
        ir,
        cache,
        startxref,
    })
}

pub fn refresh_cache(doc: &mut LoadedDoc) -> Result<()> {
    let ExtractedDocument { ir, cache } = extract_page_zero(&doc.doc)?;
    doc.ir = ir;
    doc.cache = cache;
    Ok(())
}

pub fn find_startxref(bytes: &[u8]) -> Result<usize> {
    let marker = b"startxref";
    let pos = bytes
        .windows(marker.len())
        .rposition(|window| window == marker)
        .ok_or_else(|| anyhow!("startxref not found"))?;
    let mut index = pos + marker.len();
    while index < bytes.len() && bytes[index].is_ascii_whitespace() {
        index += 1;
    }
    let mut end = index;
    while end < bytes.len() && bytes[end].is_ascii_digit() {
        end += 1;
    }
    let value = std::str::from_utf8(&bytes[index..end])?.parse::<usize>()?;
    Ok(value)
}
