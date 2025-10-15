//! Placeholder shaping helpers.

pub fn shape_text(text: &str) -> Vec<(u32, f32, f32)> {
    text.chars()
        .enumerate()
        .map(|(idx, _)| (idx as u32, 500.0, 0.0))
        .collect()
}
