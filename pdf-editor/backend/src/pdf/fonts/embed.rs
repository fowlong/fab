use lopdf::{Dictionary, Object, Stream};

pub struct EmbeddedFont {
  pub font_dict: Dictionary,
  pub descendant_dict: Dictionary,
  pub stream: Stream,
}

/// Builds PDF dictionaries for a Type0 font embedding (simplified stub).
pub fn embed_simple_font(font_name: &str, font_data: Vec<u8>) -> anyhow::Result<EmbeddedFont> {
  let font_stream = Stream::new(Dictionary::new(), font_data);

  let mut descendant = Dictionary::new();
  descendant.set("Type", Object::Name(b"Font".to_vec()));
  descendant.set("Subtype", Object::Name(b"CIDFontType2".to_vec()));
  descendant.set("BaseFont", Object::Name(font_name.as_bytes().to_vec()));

  let mut cid_info = Dictionary::new();
  cid_info.set("Registry", Object::string_literal("Adobe"));
  cid_info.set("Ordering", Object::string_literal("Identity"));
  cid_info.set("Supplement", Object::Integer(0));
  descendant.set("CIDSystemInfo", Object::Dictionary(cid_info));

  let mut font = Dictionary::new();
  font.set("Type", Object::Name(b"Font".to_vec()));
  font.set("Subtype", Object::Name(b"Type0".to_vec()));
  font.set("BaseFont", Object::Name(font_name.as_bytes().to_vec()));
  font.set("Encoding", Object::Name(b"Identity-H".to_vec()));
  font.set(
    "DescendantFonts",
    Object::Array(vec![Object::Dictionary(descendant.clone())]),
  );

  Ok(EmbeddedFont {
    font_dict: font,
    descendant_dict: descendant,
    stream: font_stream,
  })
}
