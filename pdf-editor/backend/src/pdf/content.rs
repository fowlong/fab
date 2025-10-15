//! Tokeniser and writer scaffolding for PDF content streams.
//!
//! The production editor will parse BT/ET blocks, graphics state operators,
//! and path instructions. The current MVP does not yet require these
//! capabilities, but the module exists to establish the final layout.

#[allow(dead_code)]
pub fn describe() -> &'static str {
    "Content stream helpers to be implemented"
}
