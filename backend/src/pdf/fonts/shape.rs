//! HarfBuzz shaping stub.

use anyhow::Result;

use crate::types::TextGlyph;

pub fn shape_text(_text: &str) -> Result<Vec<TextGlyph>> {
    Ok(Vec::new())
}
