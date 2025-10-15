//! Patch application for transform operations.

use anyhow::{anyhow, Context, Result};
use lopdf::ObjectId;

use crate::{
    pdf::extract::{ImageCache, PageCache, TextCache},
    types::{PatchOperation, TransformKind},
};

#[derive(Debug, Clone)]
pub struct PagePatch {
    pub page_id: ObjectId,
    pub new_stream_bytes: Vec<u8>,
}

pub fn apply_patches(page: &mut PageCache, ops: &[PatchOperation]) -> Result<Option<PagePatch>> {
    if ops.is_empty() {
        return Ok(None);
    }

    let mut changed = false;

    for op in ops {
        match op {
            PatchOperation::Transform {
                target,
                delta_matrix_pt,
                kind,
            } => {
                if target.page != page.page_index {
                    continue;
                }
                let delta = *delta_matrix_pt;
                match kind {
                    TransformKind::Text => {
                        let entry = page
                            .text_map
                            .get_mut(&target.id)
                            .context("text target missing from cache")?;
                        let shift = apply_text_transform(&target.id, page, entry, delta)?;
                        propagate_shift(page, &shift);
                        changed = true;
                    }
                    TransformKind::Image => {
                        let entry = page
                            .image_map
                            .get_mut(&target.id)
                            .context("image target missing from cache")?;
                        let shift = apply_image_transform(&target.id, page, entry, delta)?;
                        propagate_shift(page, &shift);
                        changed = true;
                    }
                }
            }
        }
    }

    if !changed {
        return Ok(None);
    }

    let mut combined = Vec::new();
    for (index, segment) in page.segments.iter().enumerate() {
        if index > 0 {
            combined.push(b'\n');
        }
        combined.extend_from_slice(&segment.bytes);
    }

    Ok(Some(PagePatch {
        page_id: page.page_id,
        new_stream_bytes: combined,
    }))
}

fn apply_text_transform(
    entry_id: &str,
    page: &mut PageCache,
    entry: &mut TextCache,
    delta: [f64; 6],
) -> Result<ShiftUpdate> {
    let segment = page
        .segments
        .iter_mut()
        .find(|segment| segment.id == entry.stream_id)
        .context("content stream for text run not found")?;
    let new_matrix = multiply(delta, entry.tm);
    let snippet = format_matrix(new_matrix, "Tm");

    let (start, original_len) = if let Some(range) = entry.tm_range.clone() {
        let original_len = range.end.saturating_sub(range.start);
        replace_bytes(&mut segment.bytes, range.clone(), &snippet);
        entry.tm_range = Some(range.start..range.start + snippet.len());
        (range.start, original_len)
    } else {
        let insert_at = entry.bt_range.end;
        if insert_at > segment.bytes.len() {
            return Err(anyhow!("BT span exceeds stream length"));
        }
        insert_bytes(&mut segment.bytes, insert_at, &snippet);
        entry.tm_range = Some(insert_at..insert_at + snippet.len());
        (insert_at, 0)
    };

    entry.tm = new_matrix;
    Ok(ShiftUpdate {
        stream_id: entry.stream_id,
        start,
        original_len,
        new_len: snippet.len(),
        skip_text: Some(entry_id.to_string()),
        skip_image: None,
    })
}

fn apply_image_transform(
    entry_id: &str,
    page: &mut PageCache,
    entry: &mut ImageCache,
    delta: [f64; 6],
) -> Result<ShiftUpdate> {
    let segment = page
        .segments
        .iter_mut()
        .find(|segment| segment.id == entry.stream_id)
        .context("content stream for image not found")?;
    let new_matrix = multiply(delta, entry.matrix);
    let snippet = format_matrix(new_matrix, "cm");

    let range = entry.cm_range.clone();
    replace_bytes(&mut segment.bytes, range.clone(), &snippet);
    entry.cm_range = range.start..range.start + snippet.len();
    entry.matrix = new_matrix;
    Ok(ShiftUpdate {
        stream_id: entry.stream_id,
        start: range.start,
        original_len: range.end.saturating_sub(range.start),
        new_len: snippet.len(),
        skip_text: None,
        skip_image: Some(entry_id.to_string()),
    })
}

#[derive(Debug, Clone)]
struct ShiftUpdate {
    stream_id: ObjectId,
    start: usize,
    original_len: usize,
    new_len: usize,
    skip_text: Option<String>,
    skip_image: Option<String>,
}

fn format_matrix(matrix: [f64; 6], operator: &str) -> Vec<u8> {
    let mut parts = Vec::new();
    for value in matrix {
        let normalised = if value.abs() < 1e-8 { 0.0 } else { value };
        parts.push(format!("{normalised:.6}"));
    }
    parts.push(operator.to_string());
    let joined = parts.join(" ");
    let mut bytes = joined.into_bytes();
    bytes.push(b'\n');
    bytes
}

fn replace_bytes(target: &mut Vec<u8>, range: std::ops::Range<usize>, replacement: &[u8]) {
    target.splice(range, replacement.iter().copied());
}

fn insert_bytes(target: &mut Vec<u8>, index: usize, snippet: &[u8]) {
    target.splice(index..index, snippet.iter().copied());
}

fn propagate_shift(page: &mut PageCache, shift: &ShiftUpdate) {
    let delta = shift.new_len as isize - shift.original_len as isize;
    if delta == 0 {
        return;
    }
    let skip_text = shift.skip_text.as_deref();
    let skip_image = shift.skip_image.as_deref();

    for (id, cache) in page.text_map.iter_mut() {
        if cache.stream_id != shift.stream_id {
            continue;
        }
        shift_range(&mut cache.bt_range, shift.start, delta);
        shift_range(&mut cache.et_range, shift.start, delta);
        if skip_text == Some(id.as_str()) {
            continue;
        }
        if let Some(range) = &mut cache.tm_range {
            shift_range(range, shift.start, delta);
        }
    }

    for (id, cache) in page.image_map.iter_mut() {
        if cache.stream_id != shift.stream_id {
            continue;
        }
        if skip_image == Some(id.as_str()) {
            continue;
        }
        shift_range(&mut cache.cm_range, shift.start, delta);
    }
}

fn shift_range(range: &mut std::ops::Range<usize>, start: usize, delta: isize) {
    if delta == 0 {
        return;
    }
    if range.start >= start {
        range.start = adjust_index(range.start, delta);
    }
    if range.end >= start {
        range.end = adjust_index(range.end, delta);
    }
}

fn adjust_index(value: usize, delta: isize) -> usize {
    if delta >= 0 {
        value + delta as usize
    } else {
        value.saturating_sub((-delta) as usize)
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
