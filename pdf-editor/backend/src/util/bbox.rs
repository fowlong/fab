//! Axis-aligned bounding box helpers.

#[derive(Debug, Clone, Copy, Default)]
pub struct BBox {
    pub min_x: f64,
    pub min_y: f64,
    pub max_x: f64,
    pub max_y: f64,
}

impl BBox {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn include(&mut self, x: f64, y: f64) {
        if self.min_x == 0.0 && self.max_x == 0.0 && self.min_y == 0.0 && self.max_y == 0.0 {
            self.min_x = x;
            self.max_x = x;
            self.min_y = y;
            self.max_y = y;
        } else {
            if x < self.min_x {
                self.min_x = x;
            }
            if x > self.max_x {
                self.max_x = x;
            }
            if y < self.min_y {
                self.min_y = y;
            }
            if y > self.max_y {
                self.max_y = y;
            }
        }
    }

    pub fn to_array(self) -> [f64; 4] {
        [self.min_x, self.min_y, self.max_x, self.max_y]
    }
}
