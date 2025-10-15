//! Text shaping helpers built on top of harfbuzz.

use anyhow::Result;

pub struct ShapedGlyph {
    pub glyph_id: u32,
    pub x_advance: i32,
    pub y_advance: i32,
}

pub fn shape_text(_font_data: &[u8], _text: &str) -> Result<Vec<ShapedGlyph>> {
    // TODO: integrate harfbuzz shaping.
    Ok(Vec::new())
}
