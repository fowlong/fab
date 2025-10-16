//! Apply transform patches to the cached PDF representation.

use std::collections::HashMap;

use anyhow::{anyhow, Result};
use lopdf::{Document, ObjectId};

use crate::{
    pdf::{
        content::{self, Matrix},
        extract::{DocumentCache, ImageCache, PageCache, StreamCache, TextCache},
        write,
    },
    types::{PatchOperation, TransformKind},
};

/// Apply the supplied patch operations to the cached document and return
/// updated PDF bytes when a modification is performed.
pub fn apply_patches(
    document: &Document,
    cache: &DocumentCache,
    original_bytes: &[u8],
    ops: &[PatchOperation],
) -> Result<Option<Vec<u8>>> {
    let mut tasks: HashMap<String, TransformTask> = HashMap::new();

    for op in ops {
        if let PatchOperation::Transform(transform) = op {
            if transform.target.page != 0 {
                continue;
            }
            let entry = tasks
                .entry(transform.target.id.clone())
                .or_insert_with(|| TransformTask {
                    kind: transform.kind.clone(),
                    delta: IDENTITY_MATRIX,
                });

            if entry.kind != transform.kind {
                return Err(anyhow!("conflicting transform kinds for target"));
            }

            let delta = matrix_from_array(&transform.delta_matrix_pt);
            entry.delta = content::multiply(&delta, &entry.delta);
        }
    }

    if tasks.is_empty() {
        return Ok(None);
    }

    let page = &cache.page0;
    let mut edits: HashMap<u32, Vec<StreamEdit>> = HashMap::new();

    for (id, task) in tasks {
        match task.kind {
            TransformKind::Text => apply_text_edit(page, &id, &task.delta, &mut edits)?,
            TransformKind::Image => apply_image_edit(page, &id, &task.delta, &mut edits)?,
        }
    }

    if edits.is_empty() {
        return Ok(None);
    }

    let mut stream_updates: HashMap<ObjectId, Vec<u8>> = HashMap::new();

    for (object_number, stream_edits) in edits {
        let stream = page
            .streams
            .get(&object_number)
            .ok_or_else(|| anyhow!("missing stream {object_number}"))?;
        let mut buffer = stream.data.clone();
        apply_edits(&mut buffer, stream_edits);
        stream_updates.insert(stream.object_id, buffer);
    }

    let new_bytes = write::incremental_update(
        original_bytes,
        document,
        page.page_id,
        &page.contents_order,
        page.contents_is_array,
        &stream_updates,
    )?;

    Ok(Some(new_bytes))
}

const IDENTITY_MATRIX: Matrix = [1.0, 0.0, 0.0, 1.0, 0.0, 0.0];

#[derive(Clone)]
struct TransformTask {
    kind: TransformKind,
    delta: Matrix,
}

#[derive(Clone)]
struct StreamEdit {
    range: std::ops::Range<usize>,
    replacement: Vec<u8>,
}

fn apply_text_edit(
    page: &PageCache,
    id: &str,
    delta: &Matrix,
    edits: &mut HashMap<u32, Vec<StreamEdit>>,
) -> Result<()> {
    let text = page
        .text_objects
        .get(id)
        .ok_or_else(|| anyhow!("text object {id} not found"))?;
    let stream = page
        .streams
        .get(&text.stream_id.0)
        .ok_or_else(|| anyhow!("stream {:?} missing for text", text.stream_id))?;

    let new_tm = content::multiply(delta, &text.base_tm);
    let tm_line = format_matrix(&new_tm, "Tm");

    if let Some(range) = &text.tm_range {
        add_edit(edits, stream, range.clone(), tm_line.into_bytes());
    } else {
        let insertion = format!("\n{}", tm_line);
        let insert_range = text.tm_insert_pos..text.tm_insert_pos;
        add_edit(edits, stream, insert_range, insertion.into_bytes());
    }

    Ok(())
}

fn apply_image_edit(
    page: &PageCache,
    id: &str,
    delta: &Matrix,
    edits: &mut HashMap<u32, Vec<StreamEdit>>,
) -> Result<()> {
    let image = page
        .image_objects
        .get(id)
        .ok_or_else(|| anyhow!("image object {id} not found"))?;
    let stream = page
        .streams
        .get(&image.stream_id.0)
        .ok_or_else(|| anyhow!("stream {:?} missing for image", image.stream_id))?;

    let new_cm = content::multiply(delta, &image.cm_matrix);
    let cm_line = format_matrix(&new_cm, "cm");
    add_edit(edits, stream, image.cm_span.clone(), cm_line.into_bytes());
    Ok(())
}

fn add_edit(
    edits: &mut HashMap<u32, Vec<StreamEdit>>,
    stream: &StreamCache,
    range: std::ops::Range<usize>,
    replacement: Vec<u8>,
) {
    edits
        .entry(stream.object_number)
        .or_default()
        .push(StreamEdit { range, replacement });
}

fn apply_edits(buffer: &mut Vec<u8>, mut edits: Vec<StreamEdit>) {
    edits.sort_by(|a, b| b.range.start.cmp(&a.range.start));
    for edit in edits {
        buffer.splice(edit.range, edit.replacement.iter().copied());
    }
}

fn matrix_from_array(values: &[f64; 6]) -> Matrix {
    [
        values[0], values[1], values[2], values[3], values[4], values[5],
    ]
}

fn format_matrix(matrix: &Matrix, operator: &str) -> String {
    let coeffs: Vec<String> = matrix.iter().map(|value| format_number(*value)).collect();
    format!("{} {}", coeffs.join(" "), operator)
}

fn format_number(value: f64) -> String {
    if value.abs() < 1e-7 {
        return "0".into();
    }
    let formatted = format!("{:.6}", value);
    formatted
        .trim_end_matches('0')
        .trim_end_matches('.')
        .to_string()
}
