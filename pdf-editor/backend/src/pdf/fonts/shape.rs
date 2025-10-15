//! Font shaping scaffolding.

use harfbuzz_rs::{Buffer, BufferFlags, Direction, Face, Font, Language, Tag, UnicodeBuffer};

/// Shapes text with HarfBuzz returning glyph ids and advances.
pub fn shape_text(face: &Face<'_>, text: &str) -> Vec<(u32, i32, i32)> {
    let mut buffer = UnicodeBuffer::new();
    buffer.add_str(text);
    let mut buffer: Buffer = buffer.into();
    buffer.set_direction(Direction::LTR);
    buffer.set_language(Language::from_opentype_language_tag(Tag::from_bytes(b"DFLT")));
    buffer.set_flags(BufferFlags::DEFAULT);

    let font = Font::new(face);
    let shaped = harfbuzz_rs::shape(&font, buffer, &[]);
    let infos = shaped.get_glyph_infos();
    let positions = shaped.get_glyph_positions();

    infos
        .iter()
        .zip(positions.iter())
        .map(|(info, pos)| (info.codepoint, pos.x_advance, pos.y_advance))
        .collect()
}
