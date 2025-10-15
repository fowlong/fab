//! Font helpers for shaping and embedding.
//!
//! The submodules currently expose lightweight stubs so the rest of the
//! backend can compile before the full font pipeline is implemented.

pub use embed::EmbeddedFont;
pub use shape::{ShapePlan, ShapedGlyph};

pub mod embed;
pub mod shape;
pub mod subset;
