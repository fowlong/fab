pub fn shape_text(text: &str) -> Vec<u16> {
    text.encode_utf16().collect()
}
