//! Placeholder module for incremental PDF writing.
//!
//! The MVP backend currently echoes the uploaded PDF without mutation.
//! Future iterations will replace this module with routines that append
//! incremental updates to the document's cross-reference table and streams.

#[allow(dead_code)]
pub fn describe() -> &'static str {
    "Incremental writer pending implementation"
}
