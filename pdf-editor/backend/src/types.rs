use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct DocumentIR {
    pub pages: Vec<PageIR>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PageIR {
    pub index: usize,
}
