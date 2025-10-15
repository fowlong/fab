use anyhow::Context;

use crate::types::DocumentIr;

#[derive(Debug, Clone)]
pub struct LoadedDocument {
    pub ir: DocumentIr,
    pub bytes: Vec<u8>,
}

pub fn parse_document(bytes: &[u8]) -> anyhow::Result<LoadedDocument> {
    // Placeholder parser – just returns an empty IR until the real extraction is implemented.
    Ok(LoadedDocument {
        ir: DocumentIr::empty(),
        bytes: bytes.to_vec(),
    })
}

pub fn sanitize_pdf(_bytes: &[u8]) -> anyhow::Result<Vec<u8>> {
    // TODO: strip dangerous constructs (e.g. /OpenAction scripts) in a future revision.
    Ok(Vec::new())
}
