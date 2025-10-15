use anyhow::Result;

use crate::types::DocumentIR;

/// Extract the intermediate representation from a PDF file.
///
/// The current implementation returns the IR supplied by the loader verbatim.
/// Future iterations will walk the content streams and populate `PageObject` entries.
pub fn extract_ir(ir: DocumentIR) -> Result<DocumentIR> {
    Ok(ir)
}
