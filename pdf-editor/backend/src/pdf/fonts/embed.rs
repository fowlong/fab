//! Minimal embedding utilities for the MVP. The generated data structures are
//! placeholders that satisfy the rest of the pipeline while keeping the API
//! easy to extend.

use anyhow::Result;
use lopdf::{Dictionary, Object, Stream};

pub struct EmbeddedFont {
    pub font_dict: Dictionary,
    pub stream: Stream,
}

pub fn embed_subset_font(font_bytes: Vec<u8>, font_name: &str) -> Result<EmbeddedFont> {
    let mut dict = Dictionary::new();
    dict.set("Type", Object::Name(b"Font".to_vec()));
    dict.set("Subtype", Object::Name(b"Type0".to_vec()));
    dict.set("BaseFont", Object::Name(font_name.as_bytes().to_vec()));
    let stream = Stream::new(Dictionary::new(), font_bytes);
    Ok(EmbeddedFont {
        font_dict: dict,
        stream,
    })
}
