//! Content stream helpers for the PDF editor backend.
//!
//! The MVP implementation exposes only a handful of utilities necessary for unit
//! testing and higher level prototyping. Future iterations will expand the
//! tokenizer and writer to cover the full PDF operator set required by the
//! application.

use anyhow::{anyhow, Result};
use lopdf::content::Content;

/// Parse raw content bytes into a lopdf [`Content`] structure.
pub fn parse_content(bytes: &[u8]) -> Result<Content> {
    Content::decode(bytes).map_err(|err| anyhow!(err))
}
