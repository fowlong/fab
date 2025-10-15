//! Embed shaped font data into a PDF document.

use anyhow::Result;
use lopdf::{Dictionary, Document, Object, Stream};

pub fn embed_font(_doc: &mut Document, font_name: &str, font_bytes: &[u8]) -> Result<Object> {
    // TODO: create Font dictionary and ToUnicode map.
    let stream = Stream::new(Dictionary::new(), font_bytes.to_vec());
    Ok(Object::Stream(stream))
}
