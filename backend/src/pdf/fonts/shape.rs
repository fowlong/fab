use anyhow::{bail, Result};

pub fn shape_text(_text: &str, _font: &str) -> Result<()> {
    bail!("harfbuzz shaping not yet implemented")
}
