//! Font pipeline placeholder module.
//!
//! The MVP ships with stubs that describe the intended API surface. Real text shaping
//! and font embedding logic can be implemented later using harfbuzz-rs and ttf-parser.

pub mod embed;
pub mod shape;
pub mod subset;
