//! Patch application logic for transform operations.

use std::collections::HashMap;

use anyhow::{anyhow, Context, Result};
use lopdf::Document;

use crate::pdf::extract::{
    extract_page_zero, CachedObject, ImageMeta, PageCache, StreamCache, TextMeta,
};
use crate::pdf::write;
use crate::types::{Matrix6, PatchKind, PatchOperation};

#[derive(Debug)]
pub struct PatchOutcome {
    pub bytes: Vec<u8>,
    pub document: Document,
    pub page_cache: PageCache,
}

pub fn apply_patches(
    document: &Document,
    bytes: &[u8],
    cache: &PageCache,
    ops: &[PatchOperation],
) -> Result<Option<PatchOutcome>> {
    if ops.is_empty() {
        return Ok(None);
    }

    let mut combined: HashMap<String, CombinedOp> = HashMap::new();
    let mut order: Vec<String> = Vec::new();

    for op in ops {
        match op {
            PatchOperation::Transform {
                target,
                delta_matrix_pt,
                kind,
            } => {
                anyhow::ensure!(target.page == 0, "only page 0 is supported in MVP");
                let entry = combined.entry(target.id.clone()).or_insert_with(|| {
                    order.push(target.id.clone());
                    CombinedOp {
                        kind: kind.clone(),
                        delta: identity_matrix(),
                    }
                });
                anyhow::ensure!(entry.kind == *kind, "mixed patch kinds for the same object");
                entry.delta = multiply_matrices(*delta_matrix_pt, entry.delta);
            }
        }
    }

    if combined.is_empty() {
        return Ok(None);
    }

    let mut stream_plans: HashMap<u32, StreamPlan> = HashMap::new();

    for id in order {
        let op = combined
            .remove(&id)
            .expect("missing combined op while iterating order");
        let object = cache
            .objects
            .get(&id)
            .context("patch referenced unknown object")?;
        match (op.kind, object) {
            (PatchKind::Text, CachedObject::Text(text)) => {
                apply_text_transform(cache, &mut stream_plans, text, op.delta)?;
            }
            (PatchKind::Image, CachedObject::Image(image)) => {
                apply_image_transform(cache, &mut stream_plans, image, op.delta)?;
            }
            (PatchKind::Text, _) => return Err(anyhow!("patch type does not match cached object")),
            (PatchKind::Image, _) => {
                return Err(anyhow!("patch type does not match cached object"))
            }
        }
    }

    for plan in stream_plans.values_mut() {
        plan.apply_modifications();
    }

    let mut combined_stream = Vec::new();
    for (index, stream) in cache.streams.iter().enumerate() {
        if index > 0 {
            combined_stream.push(b'\n');
        }
        if let Some(plan) = stream_plans.get(&stream.id) {
            combined_stream.extend_from_slice(&plan.content);
        } else {
            combined_stream.extend_from_slice(&stream.content);
        }
    }

    let (bytes, document) =
        write::incremental_update(document, bytes, cache.page_id, combined_stream)?;
    let extraction = extract_page_zero(&document)?;
    Ok(Some(PatchOutcome {
        bytes,
        document,
        page_cache: extraction.page_cache,
    }))
}

#[derive(Debug)]
struct CombinedOp {
    kind: PatchKind,
    delta: Matrix6,
}

#[derive(Debug)]
struct StreamPlan {
    content: Vec<u8>,
    modifications: Vec<StreamModification>,
}

impl StreamPlan {
    fn new(stream: &StreamCache) -> Self {
        Self {
            content: stream.content.clone(),
            modifications: Vec::new(),
        }
    }

    fn apply_modifications(&mut self) {
        self.modifications.sort_by(|a, b| b.start().cmp(&a.start()));
        for modification in self.modifications.drain(..) {
            modification.apply(&mut self.content);
        }
    }
}

#[derive(Debug)]
enum StreamModification {
    Replace {
        start: usize,
        end: usize,
        bytes: Vec<u8>,
    },
    Insert {
        pos: usize,
        bytes: Vec<u8>,
    },
}

impl StreamModification {
    fn start(&self) -> usize {
        match self {
            StreamModification::Replace { start, .. } => *start,
            StreamModification::Insert { pos, .. } => *pos,
        }
    }

    fn apply(self, content: &mut Vec<u8>) {
        match self {
            StreamModification::Replace { start, end, bytes } => {
                content.splice(start..end, bytes.into_iter());
            }
            StreamModification::Insert { pos, bytes } => {
                content.splice(pos..pos, bytes.into_iter());
            }
        }
    }
}

fn apply_text_transform(
    cache: &PageCache,
    plans: &mut HashMap<u32, StreamPlan>,
    meta: &TextMeta,
    delta: Matrix6,
) -> Result<()> {
    if !plans.contains_key(&meta.stream_obj) {
        let stream = lookup_stream(cache, meta.stream_obj)
            .context("failed to locate stream for text transform")?;
        plans.insert(meta.stream_obj, StreamPlan::new(stream));
    }
    let plan = plans
        .get_mut(&meta.stream_obj)
        .expect("plan must exist after insertion");

    let new_matrix = multiply_matrices(delta, meta.tm_matrix);
    let numbers = format_matrix(&new_matrix);
    let snippet = format!("{numbers} Tm");
    if let Some(span) = meta.tm_span {
        let start = span.start as usize;
        let end = span.end as usize;
        plan.modifications.push(StreamModification::Replace {
            start,
            end,
            bytes: snippet.into_bytes(),
        });
    } else {
        let insert_pos = meta.bt_op_end as usize;
        let mut bytes = Vec::new();
        bytes.push(b' ');
        bytes.extend_from_slice(snippet.as_bytes());
        bytes.push(b'\n');
        plan.modifications.push(StreamModification::Insert {
            pos: insert_pos,
            bytes,
        });
    }
    Ok(())
}

fn apply_image_transform(
    cache: &PageCache,
    plans: &mut HashMap<u32, StreamPlan>,
    meta: &ImageMeta,
    delta: Matrix6,
) -> Result<()> {
    if !plans.contains_key(&meta.stream_obj) {
        let stream = lookup_stream(cache, meta.stream_obj)
            .context("failed to locate stream for image transform")?;
        plans.insert(meta.stream_obj, StreamPlan::new(stream));
    }
    let plan = plans
        .get_mut(&meta.stream_obj)
        .expect("plan must exist after insertion");
    let new_matrix = multiply_matrices(delta, meta.cm_matrix);
    let numbers = format_matrix(&new_matrix);
    let snippet = format!("{numbers} cm");
    let start = meta.cm_span.start as usize;
    let end = meta.cm_span.end as usize;
    plan.modifications.push(StreamModification::Replace {
        start,
        end,
        bytes: snippet.into_bytes(),
    });
    Ok(())
}

fn lookup_stream<'a>(cache: &'a PageCache, id: u32) -> Option<&'a StreamCache> {
    cache.streams.iter().find(|stream| stream.id == id)
}

fn identity_matrix() -> Matrix6 {
    [1.0, 0.0, 0.0, 1.0, 0.0, 0.0]
}

fn multiply_matrices(lhs: Matrix6, rhs: Matrix6) -> Matrix6 {
    [
        lhs[0] * rhs[0] + lhs[2] * rhs[1],
        lhs[1] * rhs[0] + lhs[3] * rhs[1],
        lhs[0] * rhs[2] + lhs[2] * rhs[3],
        lhs[1] * rhs[2] + lhs[3] * rhs[3],
        lhs[0] * rhs[4] + lhs[2] * rhs[5] + lhs[4],
        lhs[1] * rhs[4] + lhs[3] * rhs[5] + lhs[5],
    ]
}

fn format_matrix(matrix: &Matrix6) -> String {
    matrix
        .iter()
        .map(|value| format_number(*value))
        .collect::<Vec<_>>()
        .join(" ")
}

fn format_number(value: f64) -> String {
    let mut text = format!("{value:.6}");
    while text.contains('.') && text.ends_with('0') {
        text.pop();
    }
    if text.ends_with('.') {
        text.pop();
    }
    if text == "-0" {
        text = "0".into();
    }
    if text.is_empty() {
        text = "0".into();
    }
    text
}
