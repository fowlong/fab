//! Harfbuzz shaping utilities (placeholder module).

use anyhow::Result;

pub fn shape_text(_font: &[u8], text: &str) -> Result<Vec<u32>> {
    Ok(text.chars().map(|c| c as u32).collect())
}
