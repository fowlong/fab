//! Axis-aligned bounding box utilities.

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BBox {
    pub min_x: f64,
    pub min_y: f64,
    pub max_x: f64,
    pub max_y: f64,
}

impl BBox {
    pub fn new(min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> Self {
        Self {
            min_x,
            min_y,
            max_x,
            max_y,
        }
    }

    pub fn width(&self) -> f64 {
        self.max_x - self.min_x
    }

    pub fn height(&self) -> f64 {
        self.max_y - self.min_y
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn width_and_height_are_positive() {
        let bbox = BBox::new(10.0, 20.0, 25.0, 45.0);
        assert_eq!(bbox.width(), 15.0);
        assert_eq!(bbox.height(), 25.0);
    }

    #[test]
    fn constructing_bbox_preserves_coordinates() {
        let bbox = BBox::new(-5.0, 2.5, 10.0, 8.0);
        assert_eq!(bbox.min_x, -5.0);
        assert_eq!(bbox.min_y, 2.5);
        assert_eq!(bbox.max_x, 10.0);
        assert_eq!(bbox.max_y, 8.0);
    }
}
