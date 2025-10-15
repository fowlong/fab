use anyhow::Result;

#[derive(Debug, Clone)]
pub struct ContentStream;

pub fn parse_content(_bytes: &[u8]) -> Result<ContentStream> {
    Ok(ContentStream)
}
