use std::collections::HashMap;

use anyhow::{anyhow, Result};
use lopdf::Document;

use crate::{
    pdf::{
        extract::{self, Matrix, PageExtraction, TokenLocator},
        write,
    },
    types::{PatchOperation, TransformKind},
};

pub struct PatchResult {
    pub bytes: Vec<u8>,
    pub extraction: PageExtraction,
}

pub fn apply_patches(
    document: &Document,
    original_bytes: &[u8],
    extraction: &PageExtraction,
    ops: &[PatchOperation],
) -> Result<PatchResult> {
    if ops.is_empty() {
        return Err(anyhow!("no patch operations supplied"));
    }

    let mut overrides: HashMap<_, Vec<u8>> = HashMap::new();
    let mut current_bytes = extraction.cache.stream_bytes.clone();
    let mut current_extraction = extraction.clone();

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
                        apply_text_transform(
                            &mut current_bytes,
                            &current_extraction.cache.text_lookup,
                            &target.id,
                            delta_matrix_pt,
                        )?;
                    }
                    TransformKind::Image => {
                        apply_image_transform(
                            &mut current_bytes,
                            &current_extraction.cache.image_lookup,
                            &target.id,
                            delta_matrix_pt,
                        )?;
                    }
                }
                overrides.insert(current_extraction.cache.stream_obj, current_bytes.clone());
                current_extraction = extract::extract_page0(document, Some(&overrides))?;
                current_bytes = current_extraction.cache.stream_bytes.clone();
            }
        }
    }

    let updated_bytes = write::incremental_update(
        document,
        original_bytes,
        current_extraction.cache.page_id,
        &current_bytes,
    )?;

    Ok(PatchResult {
        bytes: updated_bytes,
        extraction: current_extraction,
    })
}

fn apply_text_transform(
    stream: &mut Vec<u8>,
    lookup: &HashMap<String, extract::TextCache>,
    id: &str,
    delta: &[f64; 6],
) -> Result<()> {
    let entry = lookup
        .get(id)
        .ok_or_else(|| anyhow!("text object {id} not found"))?;
    let new_matrix = multiply(*delta, entry.tm);
    if let Some(locator) = &entry.tm_token {
        splice_token(stream, locator, &matrix_command(&new_matrix, "Tm"));
    } else {
        insert_after(
            stream,
            entry.insertion_point,
            &matrix_command(&new_matrix, "Tm"),
        );
    }
    Ok(())
}

fn apply_image_transform(
    stream: &mut Vec<u8>,
    lookup: &HashMap<String, extract::ImageCache>,
    id: &str,
    delta: &[f64; 6],
) -> Result<()> {
    let entry = lookup
        .get(id)
        .ok_or_else(|| anyhow!("image object {id} not found"))?;
    let locator = entry
        .cm_token
        .as_ref()
        .ok_or_else(|| anyhow!("image object lacks cm token"))?;
    let new_matrix = multiply(*delta, entry.cm);
    splice_token(stream, locator, &matrix_command(&new_matrix, "cm"));
    Ok(())
}

fn splice_token(stream: &mut Vec<u8>, locator: &TokenLocator, replacement: &str) {
    stream.splice(
        locator.range.start..locator.range.end,
        replacement.as_bytes().iter().copied(),
    );
}

fn insert_after(stream: &mut Vec<u8>, position: usize, replacement: &str) {
    stream.splice(position..position, replacement.as_bytes().iter().copied());
}

fn matrix_command(matrix: &Matrix, operator: &str) -> String {
    let values: Vec<String> = matrix.iter().map(|value| format_number(*value)).collect();
    let mut command = values.join(" ");
    command.push(' ');
    command.push_str(operator);
    command.push('\n');
    command
}

fn format_number(value: f64) -> String {
    let mut formatted = format!("{value:.6}");
    if formatted.contains('.') {
        while formatted.ends_with('0') {
            formatted.pop();
        }
        if formatted.ends_with('.') {
            formatted.push('0');
        }
    }
    if formatted == "-0" {
        formatted = "0".to_string();
    }
    formatted
}

fn multiply(a: Matrix, b: Matrix) -> Matrix {
    [
        a[0] * b[0] + a[2] * b[1],
        a[1] * b[0] + a[3] * b[1],
        a[0] * b[2] + a[2] * b[3],
        a[1] * b[2] + a[3] * b[3],
        a[0] * b[4] + a[2] * b[5] + a[4],
        a[1] * b[4] + a[3] * b[5] + a[5],
    ]
}
