use anyhow::Result;

use super::loader::StoredDocument;

#[derive(Debug, Default, Clone)]
pub struct IncrementalUpdate {
    pub bytes: Vec<u8>,
}

pub fn append_incremental_update(doc: &StoredDocument) -> Result<IncrementalUpdate> {
    Ok(IncrementalUpdate {
        bytes: doc.latest_bytes().to_vec(),
    })
}
