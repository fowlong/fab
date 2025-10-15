use std::collections::HashMap;

use crate::types::ObjectRemap;

pub struct PdfUpdate {
    pub updated_pdf: Vec<u8>,
    pub remap: Option<HashMap<String, ObjectRemap>>,
}

pub fn incremental_save(original: &[u8]) -> Vec<u8> {
    // Placeholder: cloning original bytes for now.
    original.to_vec()
}
