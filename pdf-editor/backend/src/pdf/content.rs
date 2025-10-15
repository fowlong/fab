pub struct ContentStream;

impl ContentStream {
    pub fn parse(_bytes: &[u8]) -> anyhow::Result<Self> {
        Ok(Self)
    }
}
