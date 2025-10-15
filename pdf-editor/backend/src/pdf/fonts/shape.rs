//! Font shaping placeholder built on harfbuzz-rs.

use anyhow::Result;

#[allow(unused_variables)]
pub fn shape_text(text: &str, font_bytes: &[u8]) -> Result<()> {
    // TODO: integrate harfbuzz shaping pipeline.
    let _ = (text, font_bytes);
    Ok(())
}
