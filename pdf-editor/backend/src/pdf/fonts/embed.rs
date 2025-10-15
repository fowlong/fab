use anyhow::Result;

#[derive(Debug, Clone)]
pub struct EmbeddedFont {
    pub name: String,
    pub data: Vec<u8>,
}

pub fn embed_font(_family: &str, _data: &[u8]) -> Result<EmbeddedFont> {
    Ok(EmbeddedFont {
        name: "StubFont".into(),
        data: Vec::new(),
    })
}
