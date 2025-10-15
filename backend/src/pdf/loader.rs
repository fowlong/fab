//! Utilities for loading, caching, and persisting PDFs.

use std::{path::Path, path::PathBuf};

use anyhow::{Context, Result};
use lopdf::Document;

use super::extract::PageExtraction;

#[derive(Debug, Default, Clone)]
pub struct DocCache {
    pub page0: Option<PageExtraction>,
}

#[derive(Debug, Clone)]
pub struct LoadedDoc {
    pub bytes: Vec<u8>,
    pub doc: Document,
    pub cache: DocCache,
    pub path: PathBuf,
}

pub fn parse_document(bytes: &[u8]) -> Result<Document> {
    let mut doc = Document::load_mem(bytes)?;
    if doc.version.is_empty() {
        doc.version = "1.7".to_string();
    }
    Ok(doc)
}

pub async fn persist_bytes(tmp_dir: &Path, doc_id: &str, bytes: &[u8]) -> Result<PathBuf> {
    tokio::fs::create_dir_all(tmp_dir)
        .await
        .with_context(|| format!("failed to create temp dir {:?}", tmp_dir))?;
    let path = tmp_dir.join(format!("{doc_id}.pdf"));
    tokio::fs::write(&path, bytes)
        .await
        .with_context(|| format!("failed to persist PDF for {doc_id}"))?;
    Ok(path)
}

impl LoadedDoc {
    pub fn new(bytes: Vec<u8>, doc: Document, path: PathBuf) -> Self {
        Self {
            bytes,
            doc,
            cache: DocCache::default(),
            path,
        }
    }

    pub fn invalidate_page0(&mut self) {
        self.cache.page0 = None;
    }
}
