use anyhow::Result;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use base64::Engine;

use crate::types::{DocumentIR, DocumentMeta, PageIR};

pub fn build_initial_ir(bytes: &[u8], file_name: &str) -> Result<DocumentIR> {
    let meta = DocumentMeta {
        file_name: file_name.to_string(),
        page_count: 1,
        original_pdf: Some(format!(
            "data:application/pdf;base64,{}",
            BASE64_STANDARD.encode(bytes)
        )),
    };

    let page = PageIR {
        index: 0,
        width_pt: 595.276,
        height_pt: 841.89,
        objects: Vec::new(),
    };

    Ok(DocumentIR {
        document_meta: meta,
        pages: vec![page],
    })
}
