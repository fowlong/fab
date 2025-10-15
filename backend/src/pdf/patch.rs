use std::fs;

use anyhow::{anyhow, Result};

use crate::types::{PatchKind, PatchOperation};

use super::content::{multiply, Operand, Operation, Operator};
use super::loader::LoadedDocument;
use super::write::{incremental_update, StreamUpdate};

pub fn apply_patches(doc: &mut LoadedDocument, ops: &[PatchOperation]) -> Result<()> {
    for op in ops {
        apply_patch(doc, op)?;
    }
    fs::write(&doc.path, &doc.bytes)?;
    Ok(())
}

fn apply_patch(doc: &mut LoadedDocument, op: &PatchOperation) -> Result<()> {
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
                PatchKind::Text => apply_text_transform(doc, &target.id, delta_matrix_pt),
                PatchKind::Image => apply_image_transform(doc, &target.id, delta_matrix_pt),
            }?
        }
    }
    Ok(())
}

fn apply_text_transform(doc: &mut LoadedDocument, id: &str, delta: &[f64; 6]) -> Result<()> {
    let cached = match doc.lookup.get(id) {
        Some(super::extract::CachedObject::Text(info)) => info.clone(),
        _ => return Err(anyhow!("text object not found")),
    };
    let state = doc
        .streams
        .get_mut(&cached.stream_obj)
        .ok_or_else(|| anyhow!("missing stream"))?;

    let new_tm = multiply(delta, &cached.tm);

    if let Some(index) = cached.tm_token_index {
        let operation = state
            .operations
            .get_mut(index)
            .ok_or_else(|| anyhow!("Tm operation index out of range"))?;
        if !matches!(operation.operator, Operator::Tm) {
            return Err(anyhow!("expected Tm operator"));
        }
        operation.set_matrix(&new_tm);
    } else {
        insert_tm_after_bt(state, cached.bt_token_index, &new_tm)?;
    }

    commit_stream_update(doc, cached.stream_obj)
}

fn insert_tm_after_bt(
    state: &mut super::extract::StreamState,
    bt_index: usize,
    matrix: &[f64; 6],
) -> Result<()> {
    if bt_index >= state.operations.len() {
        return Err(anyhow!("BT token index out of range"));
    }
    if !matches!(state.operations[bt_index].operator, Operator::BT) {
        return Err(anyhow!("expected BT operator"));
    }
    let operands = matrix.iter().map(|value| Operand::Number(*value)).collect();
    let range_end = state.operations[bt_index].range.end;
    let operation = Operation {
        operator: Operator::Tm,
        operands,
        range: range_end..range_end,
    };
    state.operations.insert(bt_index + 1, operation);
    Ok(())
}

fn apply_image_transform(doc: &mut LoadedDocument, id: &str, delta: &[f64; 6]) -> Result<()> {
    let cached = match doc.lookup.get(id) {
        Some(super::extract::CachedObject::Image(info)) => info.clone(),
        _ => return Err(anyhow!("image object not found")),
    };
    let state = doc
        .streams
        .get_mut(&cached.stream_obj)
        .ok_or_else(|| anyhow!("missing stream"))?;

    let new_cm = multiply(delta, &cached.cm);

    if let Some(index) = cached.cm_token_index {
        let operation = state
            .operations
            .get_mut(index)
            .ok_or_else(|| anyhow!("cm operator index out of range"))?;
        if !matches!(operation.operator, Operator::Cm) {
            return Err(anyhow!("expected cm operator"));
        }
        operation.set_matrix(&new_cm);
    } else {
        insert_cm_before_do(state, cached.do_token_index, &new_cm)?;
    }

    commit_stream_update(doc, cached.stream_obj)
}

fn insert_cm_before_do(
    state: &mut super::extract::StreamState,
    do_index: usize,
    matrix: &[f64; 6],
) -> Result<()> {
    if do_index > state.operations.len() {
        return Err(anyhow!("Do token index out of range"));
    }
    let operands = matrix.iter().map(|value| Operand::Number(*value)).collect();
    let range_start = if do_index < state.operations.len() {
        state.operations[do_index].range.start
    } else {
        0
    };
    let operation = Operation {
        operator: Operator::Cm,
        operands,
        range: range_start..range_start,
    };
    state.operations.insert(do_index, operation);
    Ok(())
}

fn commit_stream_update(doc: &mut LoadedDocument, stream_id: u32) -> Result<()> {
    let state = doc
        .streams
        .get(&stream_id)
        .ok_or_else(|| anyhow!("stream state missing"))?;
    let bytes = super::content::serialize_operations(&state.operations);
    let updates = [StreamUpdate {
        stream_id,
        data: &bytes,
    }];
    let result = incremental_update(&doc.bytes, &doc.doc, &doc.page_zero, &updates)?;
    doc.bytes = result.bytes;
    doc.page_zero.contents = result.new_contents;
    doc.refresh()?;
    Ok(())
}
