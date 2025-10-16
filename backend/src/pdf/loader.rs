use std::{fs, path::PathBuf};

use anyhow::{Context, Result};
use lopdf::Document;

pub fn parse_document(bytes: &[u8]) -> Result<Document> {
    Document::load_mem(bytes).context("failed to parse PDF bytes")
}

pub fn persist_original(doc_id: &str, bytes: &[u8]) -> Result<PathBuf> {
    let root = PathBuf::from("/tmp/fab");
    fs::create_dir_all(&root).context("unable to create temp directory")?;
    let path = root.join(format!("{doc_id}.pdf"));
    fs::write(&path, bytes).context("unable to persist original PDF")?;
    Ok(path)
}
