use crate::types::{PageIr, PatchOp};

pub fn apply_noop(pages: &mut [PageIr], ops: &[PatchOp]) {
    tracing::info!(page_count = pages.len(), ops = ops.len(), "received patch operations (noop mode)");
}
