use crate::util::bbox::BBox;
use crate::util::matrix::Matrix;

#[derive(Debug, Clone)]
pub enum IrObject {
  Text,
  Image,
  Path,
}

#[derive(Debug, Clone)]
pub struct IrPage {
  pub index: usize,
  pub width_pt: f64,
  pub height_pt: f64,
  pub objects: Vec<IrObject>,
}

#[derive(Debug, Clone)]
pub struct IrDocument {
  pub pages: Vec<IrPage>,
}

#[allow(unused)]
pub fn extract_document(_bytes: &[u8]) -> IrDocument {
  IrDocument { pages: Vec::new() }
}

#[allow(unused)]
pub fn compute_bbox(_matrix: Matrix) -> BBox {
  BBox::new(0.0, 0.0, 0.0, 0.0)
}
