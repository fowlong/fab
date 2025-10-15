//! Text shaping helpers built on top of `harfbuzz_rs`.

use anyhow::{anyhow, Result};

pub struct ShapedGlyph {
    pub id: u32,
    pub advance_x: f32,
    pub advance_y: f32,
}

pub fn shape_text(_font_data: &[u8], _text: &str) -> Result<Vec<ShapedGlyph>> {
    Err(anyhow!("text shaping not yet implemented"))
}
