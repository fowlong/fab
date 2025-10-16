//! Patch application for matrix transforms.

use std::ops::Range;

use anyhow::{anyhow, Context, Result};
use lopdf::{Document, ObjectId};

use crate::{
    pdf::extract::extract_page_zero,
    pdf::extract::{CachedObject, ImageCache, PageCache, StreamCache, TextCache},
    pdf::write,
    types::{DocumentIR, PatchOperation, TransformKind},
    util::matrix::Matrix2D,
};

/// State held while applying a batch of patch operations.
#[derive(Debug)]
pub struct PatchState {
    pub document: Document,
    pub bytes: Vec<u8>,
    pub ir: DocumentIR,
    pub page_cache: PageCache,
}

impl PatchState {
    pub fn new(document: Document, bytes: Vec<u8>, ir: DocumentIR, page_cache: PageCache) -> Self {
        Self {
            document,
            bytes,
            ir,
            page_cache,
        }
    }
}

/// Apply a list of operations, returning the updated PDF state.
pub fn apply_patches(mut state: PatchState, ops: &[PatchOperation]) -> Result<PatchState> {
    for op in ops {
        state = apply_single(state, op)?;
    }
    Ok(state)
}

fn apply_single(mut state: PatchState, op: &PatchOperation) -> Result<PatchState> {
    match op {
        PatchOperation::Transform {
            target,
            delta_matrix_pt,
            kind,
        } => {
            if target.page != 0 {
                return Err(anyhow!("only page 0 is supported in this stage"));
            }
            let cached = state
                .page_cache
                .objects
                .get(&target.id)
                .cloned()
                .ok_or_else(|| anyhow!("object {} not found", target.id))?;
            match (kind, cached) {
                (TransformKind::Text, CachedObject::Text(text_cache)) => {
                    state = apply_text_transform(state, &text_cache, *delta_matrix_pt)?;
                }
                (TransformKind::Image, CachedObject::Image(image_cache)) => {
                    state = apply_image_transform(state, &image_cache, *delta_matrix_pt)?;
                }
                _ => {
                    return Err(anyhow!("kind {:?} does not match cached object", kind));
                }
            }
        }
    }
    Ok(state)
}

fn apply_text_transform(
    state: PatchState,
    cache: &TextCache,
    delta_matrix: [f64; 6],
) -> Result<PatchState> {
    let stream_cache = find_stream(&state.page_cache, cache.stream_obj)?;
    let mut stream_bytes = stream_cache.bytes.clone();

    let delta = Matrix2D::from_array(delta_matrix);
    let base_tm = Matrix2D::from_array(cache.base_tm);
    let new_tm = delta.multiply(base_tm);
    let matrix_words = format_matrix(new_tm);

    if let Some(tm_locator) = &cache.tm {
        replace_range(
            &mut stream_bytes,
            tm_locator.byte_range.clone(),
            &with_original_whitespace(
                &stream_cache.bytes[tm_locator.byte_range.clone()],
                &format!("{} Tm", matrix_words),
            ),
        );
    } else {
        let insertion = format!("\n{} Tm\n", matrix_words);
        insert_bytes(
            &mut stream_bytes,
            cache.bt.byte_range.end,
            insertion.as_bytes(),
        );
    }

    commit_stream_update(state, cache.stream_obj, stream_bytes)
}

fn apply_image_transform(
    state: PatchState,
    cache: &ImageCache,
    delta_matrix: [f64; 6],
) -> Result<PatchState> {
    let stream_cache = find_stream(&state.page_cache, cache.stream_obj)?;
    let mut stream_bytes = stream_cache.bytes.clone();

    let delta = Matrix2D::from_array(delta_matrix);
    let base_cm = Matrix2D::from_array(cache.current_cm);
    let new_cm = delta.multiply(base_cm);
    let matrix_words = format_matrix(new_cm);

    if let Some(cm_locator) = &cache.cm {
        replace_range(
            &mut stream_bytes,
            cm_locator.byte_range.clone(),
            &with_original_whitespace(
                &stream_cache.bytes[cm_locator.byte_range.clone()],
                &format!("{} cm", matrix_words),
            ),
        );
    } else {
        let insertion = format!("{} cm\n", matrix_words);
        insert_bytes(
            &mut stream_bytes,
            cache.do_token.byte_range.start,
            insertion.as_bytes(),
        );
    }

    commit_stream_update(state, cache.stream_obj, stream_bytes)
}

fn commit_stream_update(
    state: PatchState,
    target_stream: ObjectId,
    new_stream_bytes: Vec<u8>,
) -> Result<PatchState> {
    let page_id = state.page_cache.page_id;

    let (updated_bytes, updated_document) = write::append_revision(
        &state.bytes,
        &state.document,
        page_id,
        target_stream,
        new_stream_bytes,
    )?;

    let extraction = extract_page_zero(&updated_document)?;

    Ok(PatchState {
        document: updated_document,
        bytes: updated_bytes,
        ir: extraction.ir,
        page_cache: extraction.page_cache,
    })
}

fn find_stream<'a>(page_cache: &'a PageCache, stream_id: ObjectId) -> Result<&'a StreamCache> {
    page_cache
        .streams
        .iter()
        .find(|stream| stream.object_id == stream_id)
        .ok_or_else(|| anyhow!("stream {:?} missing from cache", stream_id))
}

fn replace_range(bytes: &mut Vec<u8>, range: Range<usize>, replacement: &[u8]) {
    bytes.splice(range, replacement.iter().copied());
}

fn insert_bytes(bytes: &mut Vec<u8>, offset: usize, data: &[u8]) {
    bytes.splice(offset..offset, data.iter().copied());
}

fn with_original_whitespace(original: &[u8], replacement: &str) -> Vec<u8> {
    let trailing_ws_len = original
        .iter()
        .rev()
        .take_while(|&&b| matches!(b, b' ' | b'\t' | b'\r' | b'\n'))
        .count();
    let mut result = replacement.as_bytes().to_vec();
    if trailing_ws_len > 0 {
        let start = original.len() - trailing_ws_len;
        result.extend_from_slice(&original[start..]);
    }
    result
}

fn format_matrix(matrix: Matrix2D) -> String {
    let values = matrix.to_array();
    values
        .iter()
        .map(|value| format_number(*value))
        .collect::<Vec<_>>()
        .join(" ")
}

fn format_number(value: f64) -> String {
    let mut text = format!("{:.6}", value);
    while text.contains('.') && text.ends_with('0') {
        text.pop();
    }
    if text.ends_with('.') {
        text.pop();
    }
    if text.is_empty() || text == "-" {
        "0".into()
    } else {
        text
    }
}
