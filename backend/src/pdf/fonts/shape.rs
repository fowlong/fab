use anyhow::Result;

pub fn shape_text(_font_data: &[u8], text: &str) -> Result<Vec<ShapedGlyph>> {
    Ok(text.chars().enumerate().map(|(i, _)| ShapedGlyph { glyph_id: i as u32, advance: 500.0 }).collect())
}

#[derive(Debug, Clone)]
pub struct ShapedGlyph {
    pub glyph_id: u32,
    pub advance: f32,
}
