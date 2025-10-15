use anyhow::Result;

pub fn shape_text(_text: &str, _font_bytes: &[u8]) -> Result<Vec<u32>> {
    // TODO: shape text with harfbuzz
    Ok(Vec::new())
}
