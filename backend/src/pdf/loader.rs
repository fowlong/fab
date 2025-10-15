use std::path::Path;

use anyhow::Result;

pub fn load_pdf_bytes(path: impl AsRef<Path>) -> Result<Vec<u8>> {
    Ok(std::fs::read(path)?)
}
