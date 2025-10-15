use std::path::{Path, PathBuf};

use anyhow::Result;
use lopdf::Document;

use super::extract::{extract_page_zero, PageExtraction};

#[derive(Default)]
pub struct DocCache {
    pub page0: Option<PageExtraction>,
}

pub struct LoadedDoc {
    pub path: PathBuf,
    pub bytes: Vec<u8>,
    pub document: Document,
    pub cache: DocCache,
}

impl LoadedDoc {
    pub fn new(bytes: Vec<u8>, path: PathBuf) -> Result<Self> {
        let document = Document::load_mem(&bytes)?;
        Ok(Self {
            path,
            bytes,
            document,
            cache: DocCache::default(),
        })
    }

    pub fn ensure_page0(&mut self) -> Result<&PageExtraction> {
        if self.cache.page0.is_none() {
            let extraction = extract_page_zero(&self.document)?;
            self.cache.page0 = Some(extraction);
        }
        Ok(self.cache.page0.as_ref().expect("page0 cached"))
    }

    pub fn ensure_page0_mut(&mut self) -> Result<&mut PageExtraction> {
        if self.cache.page0.is_none() {
            let extraction = extract_page_zero(&self.document)?;
            self.cache.page0 = Some(extraction);
        }
        Ok(self.cache.page0.as_mut().expect("page0 cached"))
    }

    pub fn invalidate_page0(&mut self) {
        self.cache.page0 = None;
    }
}

pub fn persist_temp(doc_id: &str, bytes: &[u8]) -> Result<PathBuf> {
    let dir = Path::new("/tmp/fab");
    std::fs::create_dir_all(dir)?;
    let path = dir.join(format!("{doc_id}.pdf"));
    std::fs::write(&path, bytes)?;
    Ok(path)
}
