use anyhow::Result;

#[derive(Debug, Clone)]
pub struct EmbeddedFont {
    pub name: String,
    pub data: Vec<u8>,
}

impl EmbeddedFont {
    pub fn new(name: impl Into<String>, data: Vec<u8>) -> Self {
        Self {
            name: name.into(),
            data,
        }
    }
}

#[allow(dead_code)]
pub fn embed_font(_font: &EmbeddedFont) -> Result<()> {
    // Placeholder for future integration with lopdf writer logic.
    Ok(())
}
