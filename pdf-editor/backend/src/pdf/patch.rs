use std::collections::HashMap;

use anyhow::Result;
use tracing::info;

use crate::types::{PatchOp, RemappedObject};

use super::{loader::LoadedDocument, write};

#[derive(Debug, Default)]
pub struct PatchOutcome {
    pub updated_pdf: Option<String>,
    pub remap: HashMap<String, RemappedObject>,
}

/// Apply a sequence of patch operations to the in-memory document.
///
/// The current implementation records the requested operations in the logs and
/// returns the unchanged PDF bytes. The scaffolding ensures the HTTP contract is
/// satisfied while the detailed PDF mutation logic is implemented.
pub fn apply_ops(doc: &mut LoadedDocument, ops: &[PatchOp]) -> Result<PatchOutcome> {
    if ops.is_empty() {
        return Ok(PatchOutcome {
            updated_pdf: write::encode_data_url(doc)?,
            remap: HashMap::new(),
        });
    }

    for op in ops {
        info!(?op, "received patch operation");
    }

    let data_url = write::store_revision(doc, doc.current_pdf().to_vec())?;

    Ok(PatchOutcome {
        updated_pdf: data_url,
        remap: HashMap::new(),
    })
}
