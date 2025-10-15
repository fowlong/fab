//! Font embedding helpers (stub).

use anyhow::Result;
use lopdf::{Dictionary, Object};

pub fn build_font_dictionary(_subset_bytes: &[u8]) -> Result<Dictionary> {
    let mut dict = Dictionary::new();
    dict.set("Type", Object::Name(b"Font".to_vec()));
    dict.set("Subtype", Object::Name(b"Type0".to_vec()));
    Ok(dict)
}
