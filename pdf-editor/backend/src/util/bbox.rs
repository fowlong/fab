#[derive(Debug, Clone, Copy, Default)]
pub struct BBox {
  pub min_x: f32,
  pub min_y: f32,
  pub max_x: f32,
  pub max_y: f32,
}

impl BBox {
  pub fn from_points(points: &[(f32, f32)]) -> Option<Self> {
    let mut iter = points.iter();
    let &(mut min_x, mut min_y) = iter.next()?;
    let mut max_x = min_x;
    let mut max_y = min_y;

    for &(x, y) in iter {
      min_x = min_x.min(x);
      min_y = min_y.min(y);
      max_x = max_x.max(x);
      max_y = max_y.max(y);
    }

    Some(Self {
      min_x,
      min_y,
      max_x,
      max_y,
    })
  }

  pub fn to_array(self) -> [f32; 4] {
    [self.min_x, self.min_y, self.max_x, self.max_y]
  }
}
