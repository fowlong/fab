use anyhow::Result;

pub struct PdfDocument {
    pub bytes: Vec<u8>,
}

impl PdfDocument {
    pub fn open(bytes: Vec<u8>) -> Result<Self> {
        Ok(Self { bytes })
    }
}
