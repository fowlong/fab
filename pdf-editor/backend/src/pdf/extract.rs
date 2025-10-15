use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct DocumentIR {
    pub pages: Vec<PageIR>,
}

#[derive(Debug, Serialize)]
pub struct PageIR {
    pub index: usize,
}

pub fn extract_ir() -> DocumentIR {
    DocumentIR { pages: Vec::new() }
}
