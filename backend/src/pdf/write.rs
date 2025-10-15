use anyhow::{bail, Result};

pub fn incremental_save(_original: &[u8], _operations: &[u8]) -> Result<Vec<u8>> {
    bail!("incremental writer not implemented yet")
}
