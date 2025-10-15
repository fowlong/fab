//! Harfbuzz shaping helpers (stub).

use anyhow::Result;

#[derive(Debug, Clone)]
pub struct ShapedGlyph {
    pub glyph_id: u32,
    pub advance_x: f32,
}

pub fn shape_text(_font_data: &[u8], text: &str) -> Result<Vec<ShapedGlyph>> {
    Ok(text
        .chars()
        .enumerate()
        .map(|(idx, _)| ShapedGlyph {
            glyph_id: idx as u32,
            advance_x: 500.0,
        })
        .collect())
}
