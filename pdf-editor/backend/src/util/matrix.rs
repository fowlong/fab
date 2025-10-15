#[derive(Debug, Clone, Copy, Default)]
pub struct Matrix2D {
  pub a: f32,
  pub b: f32,
  pub c: f32,
  pub d: f32,
  pub e: f32,
  pub f: f32,
}

impl Matrix2D {
  pub fn identity() -> Self {
    Self {
      a: 1.0,
      d: 1.0,
      ..Self::default()
    }
  }

  pub fn multiply(&self, other: &Self) -> Self {
    Self {
      a: self.a * other.a + self.c * other.b,
      b: self.b * other.a + self.d * other.b,
      c: self.a * other.c + self.c * other.d,
      d: self.b * other.c + self.d * other.d,
      e: self.a * other.e + self.c * other.f + self.e,
      f: self.b * other.e + self.d * other.f + self.f,
    }
  }
}
