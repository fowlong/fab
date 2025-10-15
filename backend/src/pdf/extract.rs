use crate::types::DocumentIR;

pub fn extract_ir(_bytes: &[u8]) -> anyhow::Result<DocumentIR> {
    // Placeholder extraction that returns an empty document IR until the
    // full parser is implemented. This keeps the API surface stable for the
    // frontend scaffolding.
    Ok(DocumentIR { pages: Vec::new() })
}
