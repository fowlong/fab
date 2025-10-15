//! Incremental writer placeholder.

use crate::types::PatchResponse;

/// In the MVP skeleton the writer simply signals success without mutating the
/// stored bytes. Future revisions will append incremental updates and return the
/// updated PDF buffer encoded as base64.
pub fn build_response() -> PatchResponse {
    PatchResponse {
        ok: true,
        updated_pdf: None,
        remap: None,
        error: None,
    }
}
