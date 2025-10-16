//! Font embedding stub.

use anyhow::Result;

pub fn build_font_stream(_subset: &[u8]) -> Result<Vec<u8>> {
    Ok(Vec::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_font_stream_returns_empty_placeholder() {
        let stream = build_font_stream(&[1, 2, 3]).expect("embedding should succeed");
        assert!(stream.is_empty());
    }
}
