use crate::pdf::extract::IrDocument;

pub enum PatchOp {
  Transform,
  EditText,
  SetStyle,
}

#[allow(unused)]
pub fn apply_patch(_doc: &mut IrDocument, _ops: &[PatchOp]) {}
