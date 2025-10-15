use std::ops::Range;

use anyhow::{anyhow, bail, Result};
use lopdf::Document;

use crate::{
    types::{PatchKind, PatchOperation},
    util::matrix::Matrix2D,
};

use super::{
    extract::{self, DocumentCache, PageState},
    loader, write,
};

pub struct PatchOutcome {
    pub bytes: Vec<u8>,
    pub document: Document,
    pub cache: DocumentCache,
}

pub fn apply_patches(
    original_bytes: &[u8],
    original_doc: &Document,
    cache: &DocumentCache,
    ops: &[PatchOperation],
) -> Result<PatchOutcome> {
    if ops.is_empty() {
        return Ok(PatchOutcome {
            bytes: original_bytes.to_vec(),
            document: original_doc.clone(),
            cache: cache.clone(),
        });
    }

    let mut page_state = cache
        .page0
        .clone()
        .ok_or_else(|| anyhow!("page cache unavailable"))?;
    let mut working_content = page_state.content_bytes.clone();

    for op in ops {
        match op {
            PatchOperation::Transform {
                target,
                delta_matrix_pt,
                kind,
            } => {
                if target.page != 0 {
                    bail!("only page 0 is supported in the MVP");
                }
                match kind {
                    PatchKind::Text => {
                        let context_snapshot = page_state.context.clone();
                        apply_text_transform(
                            &mut working_content,
                            &mut page_state,
                            &context_snapshot,
                            &target.id,
                            *delta_matrix_pt,
                        )?;
                    }
                    PatchKind::Image => {
                        let context_snapshot = page_state.context.clone();
                        apply_image_transform(
                            &mut working_content,
                            &mut page_state,
                            &context_snapshot,
                            &target.id,
                            *delta_matrix_pt,
                        )?;
                    }
                }
            }
        }
    }

    let updated_bytes = write::incremental_update(
        original_bytes,
        original_doc,
        page_state.context.page_id,
        &working_content,
    )?;
    let updated_document = loader::parse_document(&updated_bytes)?;
    let updated_cache = extract::extract_document(&updated_document)?;

    Ok(PatchOutcome {
        bytes: updated_bytes,
        document: updated_document,
        cache: updated_cache,
    })
}

fn apply_text_transform(
    content: &mut Vec<u8>,
    page_state: &mut PageState,
    context: &extract::PageContext,
    target_id: &str,
    delta: [f64; 6],
) -> Result<()> {
    let meta = page_state
        .text_objects
        .get(target_id)
        .cloned()
        .ok_or_else(|| anyhow!("text object {target_id} missing from cache"))?;
    let original_tm = matrix_from_array(meta.tm);
    let delta_matrix = matrix_from_array(delta);
    let updated_tm = delta_matrix.multiply(original_tm);
    let serialized = format_matrix(updated_tm);

    if let Some(range) = meta.tm_range.clone() {
        replace_range(content, range, serialized.as_bytes())?;
    } else {
        let insertion = format!("{} Tm\n", serialized);
        insert_bytes(content, meta.insert_after_bt, insertion.as_bytes())?;
    }

    let (_, new_state) = extract::inspect_page_with_context(context, content)?;
    page_state.content_bytes = new_state.content_bytes;
    page_state.text_objects = new_state.text_objects;
    page_state.image_objects = new_state.image_objects;
    page_state.context = new_state.context;
    Ok(())
}

fn apply_image_transform(
    content: &mut Vec<u8>,
    page_state: &mut PageState,
    context: &extract::PageContext,
    target_id: &str,
    delta: [f64; 6],
) -> Result<()> {
    let meta = page_state
        .image_objects
        .get(target_id)
        .cloned()
        .ok_or_else(|| anyhow!("image object {target_id} missing from cache"))?;
    let original_cm = matrix_from_array(meta.cm);
    let delta_matrix = matrix_from_array(delta);
    let updated_cm = delta_matrix.multiply(original_cm);
    let serialized = format_matrix(updated_cm);
    replace_range(content, meta.cm_range.clone(), serialized.as_bytes())?;

    let (_, new_state) = extract::inspect_page_with_context(context, content)?;
    page_state.content_bytes = new_state.content_bytes;
    page_state.text_objects = new_state.text_objects;
    page_state.image_objects = new_state.image_objects;
    page_state.context = new_state.context;
    Ok(())
}

fn matrix_from_array(values: [f64; 6]) -> Matrix2D {
    Matrix2D {
        a: values[0],
        b: values[1],
        c: values[2],
        d: values[3],
        e: values[4],
        f: values[5],
    }
}

fn format_matrix(matrix: Matrix2D) -> String {
    let values = [matrix.a, matrix.b, matrix.c, matrix.d, matrix.e, matrix.f];
    values
        .iter()
        .map(|value| format_number(*value))
        .collect::<Vec<_>>()
        .join(" ")
}

fn format_number(value: f64) -> String {
    let mut s = format!("{value:.6}");
    while s.contains('.') && s.ends_with('0') {
        s.pop();
    }
    if s.ends_with('.') {
        s.push('0');
    }
    s
}

fn replace_range(buffer: &mut Vec<u8>, range: Range<u32>, replacement: &[u8]) -> Result<()> {
    let start = range.start as usize;
    let end = range.end as usize;
    if start > end || end > buffer.len() {
        bail!("replacement range outside bounds");
    }
    buffer.splice(start..end, replacement.iter().copied());
    Ok(())
}

fn insert_bytes(buffer: &mut Vec<u8>, offset: u32, data: &[u8]) -> Result<()> {
    let index = offset as usize;
    if index > buffer.len() {
        bail!("insertion point outside buffer");
    }
    buffer.splice(index..index, data.iter().copied());
    Ok(())
}
