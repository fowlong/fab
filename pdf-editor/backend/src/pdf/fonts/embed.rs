use anyhow::Result;

pub struct EmbeddedFont {
    pub font_object_id: u32,
}

pub fn embed_font(_subset: &[u8]) -> Result<EmbeddedFont> {
    // Placeholder for font embedding logic.
    Ok(EmbeddedFont { font_object_id: 0 })
}
