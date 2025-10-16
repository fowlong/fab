//! Utilities for loading and caching PDFs.

use std::path::PathBuf;

use anyhow::{anyhow, Result};
use lopdf::{Document, ObjectId};

use super::extract::PageCache;

#[derive(Default)]
pub struct DocCache {
    pub page0: Option<PageCache>,
}

pub struct PageInfo {
    pub page_id: ObjectId,
    pub contents: Vec<ObjectId>,
}

pub struct LoadedDoc {
    pub bytes: Vec<u8>,
    pub document: Document,
    pub page0: PageInfo,
    pub cache: DocCache,
    pub path: PathBuf,
}

pub fn load_document(bytes: Vec<u8>, path: PathBuf) -> Result<LoadedDoc> {
    let document = Document::load_mem(&bytes)?;
    let mut pages = document.page_iter();
    let page_id = pages
        .next()
        .ok_or_else(|| anyhow!("document has no pages"))?;
    let contents = document.get_page_contents(page_id);
    if contents.is_empty() {
        return Err(anyhow!("page 0 has no content streams"));
    }
    Ok(LoadedDoc {
        bytes,
        document,
        page0: PageInfo { page_id, contents },
        cache: DocCache::default(),
        path,
    })
}
