use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct PatchOperation {
    pub op: String,
}

pub fn apply_patch(_ops: &[PatchOperation]) {}
