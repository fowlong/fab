use harfbuzz::Buffer;

pub fn shape_text(text: &str, font_name: &str) -> Vec<(u32, f32, f32)> {
    let mut buf = Buffer::new();
    buf.add_str(text);
    buf.set_direction(harfbuzz::Direction::LTR);
    let mut glyphs = Vec::new();
    for ch in text.chars() {
        glyphs.push((ch as u32, 0.0, 0.0));
    }
    glyphs
}
