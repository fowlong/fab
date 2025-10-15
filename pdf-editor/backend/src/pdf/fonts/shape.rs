use anyhow::{anyhow, Result};

pub fn shape_text(_text: &str, _font_bytes: &[u8]) -> Result<()> {
    Err(anyhow!("harfbuzz integration pending"))
}
