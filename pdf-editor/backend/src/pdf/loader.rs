//! Utilities for opening and caching PDF documents.

/// Returns a baked-in demo PDF for the frontend scaffold.
#[must_use]
pub fn demo_pdf() -> Vec<u8> {
    include_bytes!("../../../e2e/sample.pdf").to_vec()
}
