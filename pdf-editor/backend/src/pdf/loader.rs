//! Utilities for loading PDFs from disk or memory and caching them in memory.

use std::path::Path;

use anyhow::{Context, Result};
use lopdf::Document;

/// Load a PDF document from raw bytes.
pub fn load_from_bytes(bytes: &[u8]) -> Result<Document> {
    Document::load_mem(bytes).context("failed to load PDF bytes")
}

/// Load a PDF document from the filesystem.
pub fn load_from_path(path: impl AsRef<Path>) -> Result<Document> {
    Document::load(path).context("failed to load PDF file")
}
