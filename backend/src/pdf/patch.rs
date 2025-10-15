//! Application of transform patches to the cached page data.

use std::collections::HashSet;

use anyhow::{anyhow, Result};

use crate::{
    pdf::extract::{CachedObject, ImageLocator, PageCache, TextLocator},
    types::{PageIR, PageObject, PatchKind, PatchOperation},
};

pub struct PatchOutcome {
    pub touched_streams: HashSet<u32>,
}

pub fn apply_patches(
    page: &PageIR,
    cache: &mut PageCache,
    ops: &[PatchOperation],
) -> Result<PatchOutcome> {
    let mut touched = HashSet::new();
    for op in ops {
        match op {
            PatchOperation::Transform {
                target,
                delta_matrix_pt,
                kind,
            } => {
                if target.page != page.index {
                    return Err(anyhow!("patch refers to unsupported page"));
                }
                let delta = *delta_matrix_pt;
                match kind {
                    PatchKind::Text => {
                        apply_text_transform(page, cache, &target.id, delta, &mut touched)?;
                    }
                    PatchKind::Image => {
                        apply_image_transform(page, cache, &target.id, delta, &mut touched)?;
                    }
                }
            }
        }
    }
    Ok(PatchOutcome {
        touched_streams: touched,
    })
}

fn apply_text_transform(
    page: &PageIR,
    cache: &mut PageCache,
    id: &str,
    delta: [f64; 6],
    touched: &mut HashSet<u32>,
) -> Result<()> {
    let text = page
        .objects
        .iter()
        .find_map(|obj| match obj {
            PageObject::Text(text) if text.id == id => Some(text),
            _ => None,
        })
        .ok_or_else(|| anyhow!("text object {id} not found"))?;

    let locator = match cache.objects.get(id) {
        Some(CachedObject::Text(locator)) => locator.clone(),
        _ => return Err(anyhow!("text locator missing for {id}")),
    };

    let stream = cache
        .streams
        .get_mut(&locator.stream_obj)
        .ok_or_else(|| anyhow!("stream {} missing", locator.stream_obj))?;

    let new_matrix = multiply(delta, text.tm);
    let tm_snippet = format_matrix(new_matrix, "Tm");
    let mut updated = Vec::new();
    let bytes = &stream.bytes;

    if let Some(range) = locator.tm_span {
        if range.end > bytes.len() {
            return Err(anyhow!("Tm span exceeds stream length"));
        }
        updated.extend_from_slice(&bytes[..range.start]);
        updated.extend_from_slice(tm_snippet.as_bytes());
        updated.extend_from_slice(&bytes[range.end..]);
    } else {
        let insert_pos = locator.insert_after_bt.min(bytes.len());
        updated.extend_from_slice(&bytes[..insert_pos]);
        if insert_pos > 0 && !bytes[insert_pos - 1].is_ascii_whitespace() {
            updated.push(b' ');
        }
        updated.extend_from_slice(tm_snippet.as_bytes());
        updated.push(b'\n');
        updated.extend_from_slice(&bytes[insert_pos..]);
    }

    stream.bytes = updated;
    touched.insert(locator.stream_obj);
    Ok(())
}

fn apply_image_transform(
    page: &PageIR,
    cache: &mut PageCache,
    id: &str,
    delta: [f64; 6],
    touched: &mut HashSet<u32>,
) -> Result<()> {
    let image = page
        .objects
        .iter()
        .find_map(|obj| match obj {
            PageObject::Image(image) if image.id == id => Some(image),
            _ => None,
        })
        .ok_or_else(|| anyhow!("image object {id} not found"))?;

    let locator = match cache.objects.get(id) {
        Some(CachedObject::Image(locator)) => locator.clone(),
        _ => return Err(anyhow!("image locator missing for {id}")),
    };

    let stream = cache
        .streams
        .get_mut(&locator.stream_obj)
        .ok_or_else(|| anyhow!("stream {} missing", locator.stream_obj))?;

    let new_matrix = multiply(delta, image.cm);
    let cm_snippet = format_matrix(new_matrix, "cm");
    let bytes = &stream.bytes;
    let range = locator.cm_span;
    if range.end > bytes.len() {
        return Err(anyhow!("cm span exceeds stream length"));
    }
    let mut updated = Vec::new();
    updated.extend_from_slice(&bytes[..range.start]);
    updated.extend_from_slice(cm_snippet.as_bytes());
    updated.extend_from_slice(&bytes[range.end..]);
    stream.bytes = updated;
    touched.insert(locator.stream_obj);
    Ok(())
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

fn format_matrix(matrix: [f64; 6], operator: &str) -> String {
    let mut parts = Vec::with_capacity(6);
    for value in matrix {
        parts.push(format_number(value));
    }
    let mut snippet = parts.join(" ");
    snippet.push(' ');
    snippet.push_str(operator);
    snippet
}

fn format_number(value: f64) -> String {
    let mut text = format!("{value:.6}");
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

#[cfg(test)]
mod tests {
    use super::format_number;

    #[test]
    fn trims_trailing_zeroes() {
        assert_eq!(format_number(12.3400), "12.34");
        assert_eq!(format_number(5.0), "5");
    }
}
