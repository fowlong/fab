//! JSON patch application for matrix transforms.

use std::collections::HashMap;
use std::fs;
use std::ops::Range;

use anyhow::{anyhow, Result};
use lopdf::Document;

use crate::types::PatchOperation;

use super::extract::CachedObject;
use super::loader::{self, LoadedDoc};
use super::write;

pub fn apply_patches(doc: &mut LoadedDoc, ops: &[PatchOperation]) -> Result<()> {
    for op in ops {
        match op {
            PatchOperation::Transform {
                target,
                delta_matrix_pt,
                kind,
            } => {
                if target.page != 0 {
                    return Err(anyhow!("only page 0 is supported in the MVP"));
                }
                let delta = *delta_matrix_pt;
                match kind.as_str() {
                    "text" => apply_text_transform(doc, &target.id, delta)?,
                    "image" => apply_image_transform(doc, &target.id, delta)?,
                    other => return Err(anyhow!(format!("unsupported transform kind: {other}"))),
                }
            }
        }
    }
    Ok(())
}

fn apply_text_transform(doc: &mut LoadedDoc, id: &str, delta: [f64; 6]) -> Result<()> {
    let cached = doc
        .cache
        .objects
        .get(id)
        .cloned()
        .ok_or_else(|| anyhow!("text object {id} not found"))?;
    let text = match cached {
        CachedObject::Text(text) => text,
        _ => return Err(anyhow!("target object was not text")),
    };
    let stream = doc
        .cache
        .streams
        .get(text.stream_index)
        .ok_or_else(|| anyhow!("stream index out of range"))?;
    let new_tm = multiply(delta, text.tm);
    let formatted = format!("{} Tm", format_matrix(&new_tm));

    let mut new_bytes = if let Some(tm_index) = text.tm_index {
        let token = stream
            .tokens
            .get(tm_index)
            .ok_or_else(|| anyhow!("Tm token missing"))?;
        splice_bytes(&stream.bytes, token.span.clone(), formatted.as_bytes())
    } else {
        let bt_token = stream
            .tokens
            .get(text.bt_index)
            .ok_or_else(|| anyhow!("BT token missing"))?;
        insert_bytes(
            &stream.bytes,
            bt_token.span.end,
            format!("\n{formatted}").as_bytes(),
        )
    };
    if !new_bytes.ends_with(b"\n") {
        new_bytes.push(b'\n');
    }
    persist_update(doc, stream.object_id, new_bytes)
}

fn apply_image_transform(doc: &mut LoadedDoc, id: &str, delta: [f64; 6]) -> Result<()> {
    let cached = doc
        .cache
        .objects
        .get(id)
        .cloned()
        .ok_or_else(|| anyhow!("image object {id} not found"))?;
    let image = match cached {
        CachedObject::Image(image) => image,
        _ => return Err(anyhow!("target object was not an image")),
    };
    let stream = doc
        .cache
        .streams
        .get(image.stream_index)
        .ok_or_else(|| anyhow!("stream index out of range"))?;
    let new_cm = multiply(delta, image.cm);
    let token = stream
        .tokens
        .get(image.cm_index)
        .ok_or_else(|| anyhow!("cm token missing"))?;
    let formatted = format!("{} cm", format_matrix(&new_cm));
    let mut new_bytes = splice_bytes(&stream.bytes, token.span.clone(), formatted.as_bytes());
    if !new_bytes.ends_with(b"\n") {
        new_bytes.push(b'\n');
    }
    persist_update(doc, stream.object_id, new_bytes)
}

fn persist_update(doc: &mut LoadedDoc, object_id: (u32, u16), bytes: Vec<u8>) -> Result<()> {
    let mut updates = HashMap::new();
    updates.insert(object_id, bytes);
    let updated = write::incremental_update(doc, &updates)?;
    fs::write(&doc.path, &updated)?;
    doc.bytes = updated;
    doc.startxref = loader::find_startxref(&doc.bytes)?;
    doc.doc = reload_document(&doc.bytes)?;
    loader::refresh_cache(doc)?;
    Ok(())
}

fn reload_document(bytes: &[u8]) -> Result<Document> {
    let mut doc = Document::load_mem(bytes)?;
    doc.version = "1.7".to_string();
    Ok(doc)
}

fn splice_bytes(original: &[u8], span: Range<usize>, replacement: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(original.len() + replacement.len());
    out.extend_from_slice(&original[..span.start]);
    out.extend_from_slice(replacement);
    out.extend_from_slice(&original[span.end..]);
    out
}

fn insert_bytes(original: &[u8], index: usize, insertion: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(original.len() + insertion.len());
    out.extend_from_slice(&original[..index]);
    out.extend_from_slice(insertion);
    out.extend_from_slice(&original[index..]);
    out
}

fn multiply(a: [f64; 6], b: [f64; 6]) -> [f64; 6] {
    [
        a[0] * b[0] + a[2] * b[1],
        a[1] * b[0] + a[3] * b[1],
        a[0] * b[2] + a[2] * b[3],
        a[1] * b[2] + a[3] * b[3],
        a[0] * b[4] + a[2] * b[5] + a[4],
        a[1] * b[4] + a[3] * b[5] + a[5],
    ]
}

fn format_matrix(m: &[f64; 6]) -> String {
    m.iter()
        .map(|v| format_number(*v))
        .collect::<Vec<_>>()
        .join(" ")
}

fn format_number(value: f64) -> String {
    if value.abs() < 1e-8 {
        "0".into()
    } else {
        let formatted = format!("{value:.6}");
        formatted
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_string()
    }
}
