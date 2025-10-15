#![allow(dead_code)]

use anyhow::Result;

use crate::types::{DocumentIr, PatchOperation};

pub fn apply_patch(ir: &mut DocumentIr, _ops: &[PatchOperation]) -> Result<()> {
    // Placeholder that leaves IR untouched.
    let _ = ir;
    Ok(())
}
