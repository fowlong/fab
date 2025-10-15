use anyhow::{bail, Result};

pub fn shape_text(_font_data: &[u8], _text: &str) -> Result<()> {
    bail!("harfbuzz shaping not yet implemented")
}
