use anyhow::Result;

use crate::types::{DocumentIr, PatchOperations};

pub fn apply_patch(_doc: &mut DocumentIr, _ops: &PatchOperations) -> Result<Vec<u8>> {
    // Placeholder incremental writer. A real implementation will rewrite the PDF
    // content streams and return the updated bytes.
    Ok(include_bytes!("../../../e2e/sample.pdf").to_vec())
}
