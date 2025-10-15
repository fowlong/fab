//! Harfbuzz shaping integration (placeholder).

use anyhow::Result;

pub fn shape_text(_font_data: &[u8], _text: &str) -> Result<()> {
    // TODO: integrate harfbuzz_rs once glyph rewriting is implemented.
    Ok(())
}
