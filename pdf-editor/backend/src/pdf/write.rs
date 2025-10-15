use anyhow::Result;
use base64::Engine;

use super::loader::LoadedDocument;

/// Produce the latest PDF bytes encoded as a data URL.
///
/// In the MVP the method simply re-encodes the current in-memory PDF without
/// appending incremental updates. The scaffolding makes it straightforward to
/// append full incremental sections once the editing pipeline is implemented.
pub fn encode_data_url(doc: &LoadedDocument) -> Result<Option<String>> {
    let encoded = base64::engine::general_purpose::STANDARD.encode(doc.current_pdf());
    Ok(Some(format!("data:application/pdf;base64,{encoded}")))
}

/// Update the document with newly generated PDF bytes.
///
/// The helper stores the provided bytes as the current revision and returns a
/// data URL for convenience.
pub fn store_revision(doc: &mut LoadedDocument, bytes: Vec<u8>) -> Result<Option<String>> {
    doc.update_pdf_bytes(bytes);
    encode_data_url(doc)
}
