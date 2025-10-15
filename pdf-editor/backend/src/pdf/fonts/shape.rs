//! Stub harfbuzz integration. The scaffolding exposes a friendly API for
//! shaping unicode strings into positioned glyphs.

use anyhow::Result;

#[derive(Debug, Clone)]
pub struct ShapedGlyph {
    pub gid: u32,
    pub advance: f32,
}

pub fn shape_text(_font_data: &[u8], text: &str) -> Result<Vec<ShapedGlyph>> {
    Ok(text
        .chars()
        .map(|ch| ShapedGlyph {
            gid: ch as u32,
            advance: 500.0,
        })
        .collect())
}
