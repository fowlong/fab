//! Font processing helpers.
//!
//! The module layout mirrors the specification: shaping, subsetting, and
//! embedding live in dedicated files. The MVP uses placeholder implementations
//! to keep the compilation pipeline intact while the detailed logic is built
//! incrementally.

pub mod embed;
pub mod shape;
pub mod subset;
