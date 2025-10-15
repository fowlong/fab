//! Placeholder font embedding helpers.

use super::subset;

pub fn embed_subset_font(_subset: &[u8]) -> Vec<u8> {
    subset::subset_font(&[], &[])
}
