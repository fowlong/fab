//! Minimal TrueType subsetter scaffolding.

use anyhow::{anyhow, Result};

pub fn subset_font(_font_data: &[u8], _glyphs: &[u32]) -> Result<Vec<u8>> {
    Err(anyhow!(
        "font subsetting is not yet implemented; this stub captures the desired return type"
    ))
}
