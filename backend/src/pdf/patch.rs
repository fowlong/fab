//! Apply transform patches to PDF content streams.

use std::collections::HashMap;

use anyhow::{anyhow, Result};

use crate::types::{DocumentIR, PatchOperation, TransformKind};

use super::extract::{build_document_ir, extract_page0, PageCache};
use super::loader::LoadedDoc;
use super::write::incremental_update;

pub struct PatchResult {
    pub bytes: Vec<u8>,
    pub ir: DocumentIR,
}

pub fn apply_patches(doc: &mut LoadedDoc, ops: &[PatchOperation]) -> Result<PatchResult> {
    let mut overrides: HashMap<u32, Vec<u8>> = HashMap::new();
    if ops.is_empty() {
        let cache = ensure_page_cache(doc, &overrides)?;
        doc.cache.page0 = Some(cache.clone());
        return Ok(PatchResult {
            bytes: doc.bytes.clone(),
            ir: build_document_ir(&cache),
        });
    }

    for op in ops {
        let cache = ensure_page_cache(doc, &overrides)?;
        match op {
            PatchOperation::Transform {
                target,
                delta_matrix_pt,
                kind,
            } => {
                if target.page != 0 {
                    return Err(anyhow!("only page 0 is supported in the MVP"));
                }
                match kind {
                    TransformKind::Text => {
                        apply_text_transform(
                            &cache,
                            target.id.clone(),
                            *delta_matrix_pt,
                            &mut overrides,
                        )?;
                    }
                    TransformKind::Image => {
                        apply_image_transform(
                            &cache,
                            target.id.clone(),
                            *delta_matrix_pt,
                            &mut overrides,
                        )?;
                    }
                }
            }
        }
    }

    let pre_update_cache = ensure_page_cache(doc, &overrides)?;
    let combined_stream = combine_streams(&pre_update_cache, &overrides);
    let (bytes, document) = incremental_update(doc, combined_stream)?;
    doc.bytes = bytes.clone();
    doc.document = document;
    doc.page0.contents = doc.document.get_page_contents(doc.page0.page_id);
    let refreshed_cache = extract_page0(&doc.document, &doc.page0, &HashMap::new())?;
    doc.cache.page0 = Some(refreshed_cache.clone());

    Ok(PatchResult {
        bytes,
        ir: build_document_ir(&refreshed_cache),
    })
}

fn ensure_page_cache(doc: &LoadedDoc, overrides: &HashMap<u32, Vec<u8>>) -> Result<PageCache> {
    extract_page0(&doc.document, &doc.page0, overrides)
}

fn apply_text_transform(
    cache: &PageCache,
    id: String,
    delta: [f64; 6],
    overrides: &mut HashMap<u32, Vec<u8>>,
) -> Result<()> {
    let meta = cache
        .text
        .get(&id)
        .ok_or_else(|| anyhow!("text object {id} not found"))?;
    let stream = cache
        .streams
        .get(meta.stream_index)
        .ok_or_else(|| anyhow!("content stream missing for {id}"))?;
    let stream_id = stream.object_id.0;
    let mut bytes = overrides
        .get(&stream_id)
        .cloned()
        .unwrap_or_else(|| stream.bytes.clone());
    let new_tm = multiply(delta, meta.tm);
    let serialized = serialize_matrix(new_tm, "Tm");
    if let Some(tm_token) = &meta.tm_token {
        replace_range(
            &mut bytes,
            tm_token.start,
            tm_token.end,
            serialized.as_bytes(),
        );
    } else {
        let insert_pos = meta.bt_token.end;
        let mut snippet = Vec::new();
        snippet.extend_from_slice(b"\n");
        snippet.extend_from_slice(serialized.as_bytes());
        snippet.extend_from_slice(b"\n");
        bytes.splice(insert_pos..insert_pos, snippet);
    }
    overrides.insert(stream_id, bytes);
    Ok(())
}

fn apply_image_transform(
    cache: &PageCache,
    id: String,
    delta: [f64; 6],
    overrides: &mut HashMap<u32, Vec<u8>>,
) -> Result<()> {
    let meta = cache
        .images
        .get(&id)
        .ok_or_else(|| anyhow!("image object {id} not found"))?;
    let cm_token = meta
        .cm_token
        .as_ref()
        .ok_or_else(|| anyhow!("image {id} is missing a cm operator"))?;
    let stream = cache
        .streams
        .get(meta.stream_index)
        .ok_or_else(|| anyhow!("content stream missing for image {id}"))?;
    let stream_id = stream.object_id.0;
    let mut bytes = overrides
        .get(&stream_id)
        .cloned()
        .unwrap_or_else(|| stream.bytes.clone());
    let new_cm = multiply(delta, meta.cm);
    let serialized = serialize_matrix(new_cm, "cm");
    replace_range(
        &mut bytes,
        cm_token.start,
        cm_token.end,
        serialized.as_bytes(),
    );
    overrides.insert(stream_id, bytes);
    Ok(())
}

fn combine_streams(cache: &PageCache, overrides: &HashMap<u32, Vec<u8>>) -> Vec<u8> {
    let mut combined = Vec::new();
    for stream in &cache.streams {
        let bytes = overrides
            .get(&stream.object_id.0)
            .cloned()
            .unwrap_or_else(|| stream.bytes.clone());
        if !combined.is_empty() {
            combined.push(b'\n');
        }
        combined.extend_from_slice(&bytes);
    }
    combined
}

fn replace_range(bytes: &mut Vec<u8>, start: usize, end: usize, replacement: &[u8]) {
    bytes.splice(start..end, replacement.iter().copied());
}

fn serialize_matrix(matrix: [f64; 6], op: &str) -> String {
    format!(
        "{:.6} {:.6} {:.6} {:.6} {:.6} {:.6} {}",
        matrix[0], matrix[1], matrix[2], matrix[3], matrix[4], matrix[5], op
    )
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
