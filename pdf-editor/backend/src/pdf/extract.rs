use crate::types::{DocumentIR, PageIR};

pub fn extract_ir() -> anyhow::Result<DocumentIR> {
    Ok(DocumentIR {
        pages: vec![PageIR::default()],
    })
}
