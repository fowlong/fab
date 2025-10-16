//! 2D affine matrix helpers used across the backend.

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Matrix2D {
    pub a: f64,
    pub b: f64,
    pub c: f64,
    pub d: f64,
    pub e: f64,
    pub f: f64,
}

impl Matrix2D {
    pub fn identity() -> Self {
        Self {
            a: 1.0,
            b: 0.0,
            c: 0.0,
            d: 1.0,
            e: 0.0,
            f: 0.0,
        }
    }

    pub fn multiply(self, other: Self) -> Self {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_is_neutral_element() {
        let transform = Matrix2D {
            a: 0.5,
            b: 0.1,
            c: -0.3,
            d: 1.2,
            e: 42.0,
            f: -7.0,
        };

        assert_eq!(transform.multiply(Matrix2D::identity()), transform);
        assert_eq!(Matrix2D::identity().multiply(transform), transform);
    }

    #[test]
    fn multiply_combines_scale_and_translation() {
        let scale = Matrix2D {
            a: 2.0,
            b: 0.0,
            c: 0.0,
            d: 2.0,
            e: 0.0,
            f: 0.0,
        };
        let translate = Matrix2D {
            a: 1.0,
            b: 0.0,
            c: 0.0,
            d: 1.0,
            e: 10.0,
            f: -5.0,
        };

        let combined = scale.multiply(translate);

        assert_eq!(combined.a, 2.0);
        assert_eq!(combined.d, 2.0);
        assert_eq!(combined.e, 20.0);
        assert_eq!(combined.f, -10.0);
    }
}
