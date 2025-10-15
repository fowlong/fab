use anyhow::Context;
use lopdf::Document;

/// Placeholder for incremental writer. Currently returns the original document bytes.
pub fn incremental_save(doc: &Document) -> anyhow::Result<Vec<u8>> {
  let mut buffer = Vec::new();
  doc.save_to(&mut buffer).context("serialize pdf")?;
  Ok(buffer)
}
