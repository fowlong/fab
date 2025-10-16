//! Patch application for transform operations.

use std::{collections::HashMap, ops::Range};

use anyhow::{anyhow, bail, Context, Result};

use crate::types::{PatchOperation, TransformKind};

use super::content::{self, Matrix};
use super::extract::{ImageCache, ObjectCache, PageCache, TextCache};

/// Apply a set of transform operations to the cached page.
///
/// Returns the new concatenated content stream for page zero.
pub fn apply_patches(page: &PageCache, ops: &[PatchOperation]) -> Result<Vec<u8>> {
    if ops.is_empty() {
        let mut combined = Vec::new();
        for stream_id in &page.order {
            if let Some(stream) = page.streams.get(stream_id) {
                combined.extend_from_slice(&stream.content);
                if !stream.content.ends_with(b"\n") {
                    combined.push(b'\n');
                }
            }
        }
        return Ok(combined);
    }

    let mut buffers: HashMap<u32, Vec<u8>> = page
        .streams
        .iter()
        .map(|(id, stream)| (*id, stream.content.clone()))
        .collect();

    for op in ops {
        match op {
            PatchOperation::Transform {
                target,
                delta_matrix_pt,
                kind,
            } => {
                if target.page != 0 {
                    bail!("only page zero is supported in the MVP");
                }
                let delta: Matrix = *delta_matrix_pt;
                let object = page
                    .objects
                    .get(&target.id)
                    .ok_or_else(|| anyhow!("object {} missing from cache", target.id))?;
                match (kind, object) {
                    (TransformKind::Text, ObjectCache::Text(text)) => {
                        let buffer = buffers
                            .get_mut(&text.stream_obj)
                            .context("stream missing from cache")?;
                        apply_text_transform(text, &delta, buffer)?;
                    }
                    (TransformKind::Image, ObjectCache::Image(image)) => {
                        let buffer = buffers
                            .get_mut(&image.stream_obj)
                            .context("stream missing from cache")?;
                        apply_image_transform(image, &delta, buffer)?;
                    }
                    (TransformKind::Text, _) => bail!("target object is not text"),
                    (TransformKind::Image, _) => bail!("target object is not image"),
                }
            }
        }
    }

    let mut combined = Vec::new();
    for stream_id in &page.order {
        if let Some(buffer) = buffers.get(stream_id) {
            combined.extend_from_slice(buffer);
            if !buffer.ends_with(b"\n") {
                combined.push(b'\n');
            }
        }
    }
    Ok(combined)
}

fn apply_text_transform(cache: &TextCache, delta: &Matrix, buffer: &mut Vec<u8>) -> Result<()> {
    let new_tm = content::multiply(delta, &cache.tm);
    let replacement = format!("{} Tm", content::format_matrix(&new_tm));
    if let Some(range) = &cache.tm_range {
        replace_with_suffix(buffer, range.clone(), &replacement);
    } else {
        let insert_at = cache.insert_after_bt.min(buffer.len());
        let mut inserted = replacement.into_bytes();
        inserted.push(b'\n');
        buffer.splice(insert_at..insert_at, inserted);
    }
    Ok(())
}

fn apply_image_transform(cache: &ImageCache, delta: &Matrix, buffer: &mut Vec<u8>) -> Result<()> {
    let new_cm = content::multiply(delta, &cache.cm);
    let replacement = format!("{} cm", content::format_matrix(&new_cm));
    replace_with_suffix(buffer, cache.cm_range.clone(), &replacement);
    Ok(())
}

fn replace_with_suffix(buffer: &mut Vec<u8>, range: Range<usize>, replacement: &str) {
    let start = range.start.min(buffer.len());
    let end = range.end.min(buffer.len());
    let original = &buffer[start..end];
    let suffix = trailing_whitespace(original);
    let mut bytes = replacement.as_bytes().to_vec();
    bytes.extend_from_slice(&suffix);
    buffer.splice(start..end, bytes);
}

fn trailing_whitespace(slice: &[u8]) -> Vec<u8> {
    let mut idx = slice.len();
    while idx > 0 && slice[idx - 1].is_ascii_whitespace() {
        idx -= 1;
    }
    slice[idx..].to_vec()
}
