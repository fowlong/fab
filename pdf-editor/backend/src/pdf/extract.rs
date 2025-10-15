use crate::types::{DocumentIr, PageIr};

pub fn extract_ir(_bytes: &[u8]) -> anyhow::Result<DocumentIr> {
    // TODO: implement real PDF parsing pipeline.
    let page = PageIr { index: 0, width_pt: 595.0, height_pt: 842.0, objects: vec![] };
    Ok(DocumentIr { pages: vec![page] })
}
