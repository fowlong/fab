use harfbuzz::Buffer;

pub fn shape_text(text: &str) -> anyhow::Result<Vec<u32>> {
    // TODO: integrate harfbuzz shaping with font data.
    let mut buffer = Buffer::new();
    buffer.add_str(text);
    Ok(buffer.glyph_infos().iter().map(|info| info.codepoint).collect())
}
