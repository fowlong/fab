//! Axis aligned bounding boxes helpers.

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BBox {
    pub min_x: f32,
    pub min_y: f32,
    pub max_x: f32,
    pub max_y: f32,
}

impl BBox {
    #[must_use]
    pub fn from_array(b: [f32; 4]) -> Self {
        Self {
            min_x: b[0],
            min_y: b[1],
            max_x: b[2],
            max_y: b[3],
        }
    }

    #[must_use]
    pub fn to_array(self) -> [f32; 4] {
        [self.min_x, self.min_y, self.max_x, self.max_y]
    }
}
