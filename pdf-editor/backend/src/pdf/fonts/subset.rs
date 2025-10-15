use anyhow::Result;

#[derive(Debug, Clone)]
pub struct FontSubset {
    pub glyphs: Vec<u32>,
    pub data: Vec<u8>,
}

pub fn subset_font(_glyphs: &[u32], _font_data: &[u8]) -> Result<FontSubset> {
    Ok(FontSubset {
        glyphs: Vec::new(),
        data: Vec::new(),
    })
}
