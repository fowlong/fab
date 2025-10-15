//! PDF content stream helpers.

use anyhow::Result;

/// Placeholder content stream tokenizer.
/// The MVP keeps this module minimal until the full operator stack is implemented.
#[allow(dead_code)]
pub fn parse_stream(_bytes: &[u8]) -> Result<()> {
    Ok(())
}
