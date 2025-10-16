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

    pub fn from_array(values: [f64; 6]) -> Self {
        Self {
            a: values[0],
            b: values[1],
            c: values[2],
            d: values[3],
            e: values[4],
            f: values[5],
        }
    }

    pub fn to_array(self) -> [f64; 6] {
        [self.a, self.b, self.c, self.d, self.e, self.f]
    }

    pub fn concat(transforms: &[Self]) -> Self {
        transforms
            .iter()
            .copied()
            .fold(Self::identity(), |acc, item| acc.multiply(item))
    }

    pub fn inverse(self) -> Option<Self> {
        let det = self.a * self.d - self.b * self.c;
        if det.abs() < f64::EPSILON {
            return None;
        }

        let inv_det = 1.0 / det;
        Some(Self {
            a: self.d * inv_det,
            b: -self.b * inv_det,
            c: -self.c * inv_det,
            d: self.a * inv_det,
            e: (self.c * self.f - self.d * self.e) * inv_det,
            f: (self.b * self.e - self.a * self.f) * inv_det,
        })
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

    #[test]
    fn concat_applies_matrices_in_sequence() {
        let translate = Matrix2D::from_array([1.0, 0.0, 0.0, 1.0, 5.0, -3.0]);
        let scale = Matrix2D::from_array([2.0, 0.0, 0.0, 0.5, 0.0, 0.0]);
        let rotate = Matrix2D::from_array([0.0, 1.0, -1.0, 0.0, 0.0, 0.0]);

        let combined = Matrix2D::concat(&[translate, scale, rotate]);
        let manual = translate.multiply(scale).multiply(rotate);

        assert_eq!(combined, manual);
    }

    #[test]
    fn inverse_reverts_affine_transform() {
        let transform = Matrix2D::from_array([1.2, 0.3, -0.4, 0.9, 5.0, -7.5]);
        let inverse = transform.inverse().expect("matrix should be invertible");

        let identity = transform.multiply(inverse);
        assert!((identity.a - 1.0).abs() < 1e-9);
        assert!((identity.d - 1.0).abs() < 1e-9);
        assert!(identity.b.abs() < 1e-9);
        assert!(identity.c.abs() < 1e-9);
        assert!(identity.e.abs() < 1e-9);
        assert!(identity.f.abs() < 1e-9);
    }

    #[test]
    fn inverse_returns_none_for_singular_matrix() {
        let transform = Matrix2D::from_array([1.0, 2.0, 2.0, 4.0, 0.0, 0.0]);
        assert!(transform.inverse().is_none());
    }
}
