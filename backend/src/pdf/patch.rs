use std::ops::Range;

use anyhow::{anyhow, Context, Result};
use lopdf::Document;

use crate::types::{PatchOperation, TransformKind};

use super::extract::{CachedObject, PageCache};
use super::loader::LoadedDoc;
use super::write::incremental_update;

pub struct PatchOutcome {
    pub updated_bytes: Vec<u8>,
}

pub fn apply_patches(doc: &mut LoadedDoc, ops: &[PatchOperation]) -> Result<PatchOutcome> {
    if ops.is_empty() {
        return Ok(PatchOutcome {
            updated_bytes: doc.bytes.clone(),
        });
    }

    let page_extraction = doc.ensure_page0_mut()?;
    let page_cache = &mut page_extraction.cache;

    for op in ops {
        match op {
            PatchOperation::Transform {
                target,
                delta_matrix_pt,
                kind,
            } => {
                if target.page != 0 {
                    return Err(anyhow!("only page 0 supported in MVP"));
                }
                match kind {
                    TransformKind::Text => {
                        apply_text_transform(page_cache, &target.id, *delta_matrix_pt)?
                    }
                    TransformKind::Image => {
                        apply_image_transform(page_cache, &target.id, *delta_matrix_pt)?
                    }
                }
            }
        }
    }

    let mut merged = Vec::new();
    for (index, stream) in page_cache.streams.iter().enumerate() {
        merged.extend_from_slice(&stream.bytes);
        if index + 1 < page_cache.streams.len() {
            merged.push(b'\n');
        }
    }

    let updated = incremental_update(&doc.bytes, &doc.document, page_cache.page_id, merged)?;

    doc.bytes = updated.clone();
    std::fs::write(&doc.path, &doc.bytes)?;
    doc.document = Document::load_mem(&doc.bytes)?;
    doc.invalidate_page0();
    let _ = doc.ensure_page0()?; // rebuild cache and IR

    Ok(PatchOutcome {
        updated_bytes: updated,
    })
}

fn apply_text_transform(cache: &mut PageCache, id: &str, delta: [f64; 6]) -> Result<()> {
    let entry = cache
        .objects
        .get_mut(id)
        .ok_or_else(|| anyhow!("text object {id} not found"))?;
    let text = match entry {
        CachedObject::Text(text) => text,
        _ => return Err(anyhow!("expected text object")),
    };

    let stream = cache
        .streams
        .get_mut(text.stream_index)
        .context("stream index invalid")?;

    let new_tm = multiply(delta, text.tm);
    let tm_string = format_matrix(new_tm, "Tm");
    let tm_bytes = tm_string.as_bytes();

    if let Some(span) = text.tm_span.clone() {
        replace_range(&mut stream.bytes, span.clone(), tm_bytes);
        text.tm_span = Some(span.start..span.start + tm_bytes.len());
    } else {
        let bt_token = stream
            .tokens
            .get(text.bt_token_index)
            .ok_or_else(|| anyhow!("BT token missing"))?;
        let insert_pos = bt_token.span.end;
        insert_bytes(&mut stream.bytes, insert_pos, tm_bytes);
        text.tm_span = Some(insert_pos..insert_pos + tm_bytes.len());
    }

    text.tm = new_tm;

    Ok(())
}

fn apply_image_transform(cache: &mut PageCache, id: &str, delta: [f64; 6]) -> Result<()> {
    let entry = cache
        .objects
        .get_mut(id)
        .ok_or_else(|| anyhow!("image object {id} not found"))?;
    let image = match entry {
        CachedObject::Image(image) => image,
        _ => return Err(anyhow!("expected image object")),
    };

    let stream = cache
        .streams
        .get_mut(image.stream_index)
        .context("stream index invalid")?;

    let new_cm = multiply(delta, image.cm);
    let cm_string = format_matrix(new_cm, "cm");
    let cm_bytes = cm_string.as_bytes();

    let span = image.cm_span.clone();
    replace_range(&mut stream.bytes, span.clone(), cm_bytes);
    image.cm_span = span.start..span.start + cm_bytes.len();
    image.cm = new_cm;

    Ok(())
}

fn format_matrix(matrix: [f64; 6], operator: &str) -> String {
    format!(
        "{} {} {} {} {} {} {}",
        fmt_number(matrix[0]),
        fmt_number(matrix[1]),
        fmt_number(matrix[2]),
        fmt_number(matrix[3]),
        fmt_number(matrix[4]),
        fmt_number(matrix[5]),
        operator
    )
}

fn fmt_number(value: f64) -> String {
    if value.abs() < 1e-6 {
        "0".into()
    } else {
        format!("{value:.6}")
    }
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

fn replace_range(target: &mut Vec<u8>, span: Range<usize>, replacement: &[u8]) {
    let mut cursor = span.start;
    for byte in replacement {
        if cursor < span.end {
            target[cursor] = *byte;
            cursor += 1;
        } else {
            target.insert(cursor, *byte);
            cursor += 1;
        }
    }
    if cursor < span.end {
        target.drain(cursor..span.end);
    }
}

fn insert_bytes(target: &mut Vec<u8>, index: usize, content: &[u8]) {
    target.splice(index..index, content.iter().copied());
}
