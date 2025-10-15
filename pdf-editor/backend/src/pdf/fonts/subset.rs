//! Font subsetting helpers (stub).

use anyhow::Result;

pub fn subset_font(_font_bytes: &[u8], _required_glyphs: &[u32]) -> Result<Vec<u8>> {
    Ok(Vec::new())
}
