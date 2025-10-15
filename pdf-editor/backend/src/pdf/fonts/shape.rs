#![allow(dead_code)]

use anyhow::Result;

pub struct ShapedGlyph {
    pub gid: u32,
    pub advance: f32,
}

pub fn shape_text(_font_data: &[u8], text: &str) -> Result<Vec<ShapedGlyph>> {
    Ok(text
        .chars()
        .enumerate()
        .map(|(i, _)| ShapedGlyph {
            gid: i as u32,
            advance: 500.0,
        })
        .collect())
}
