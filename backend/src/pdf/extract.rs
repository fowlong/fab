use crate::types::{DocumentIR, PageIR};

pub fn extract_ir() -> DocumentIR {
    DocumentIR { pages: vec![] }
}

pub fn extract_page_stub(index: usize) -> PageIR {
    PageIR {
        index,
        width_pt: 595.0,
        height_pt: 842.0,
        objects: vec![],
    }
}
