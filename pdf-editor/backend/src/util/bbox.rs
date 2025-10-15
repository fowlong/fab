#[derive(Debug, Clone, Copy, Default)]
pub struct BBox {
    pub min_x: f64,
    pub min_y: f64,
    pub max_x: f64,
    pub max_y: f64,
}

impl BBox {
    pub fn new() -> Self {
        Self {
            min_x: f64::INFINITY,
            min_y: f64::INFINITY,
            max_x: f64::NEG_INFINITY,
            max_y: f64::NEG_INFINITY,
        }
    }

    pub fn expand(&mut self, x: f64, y: f64) {
        self.min_x = self.min_x.min(x);
        self.min_y = self.min_y.min(y);
        self.max_x = self.max_x.max(x);
        self.max_y = self.max_y.max(y);
    }

    pub fn to_array(self) -> [f64; 4] {
        if self.min_x.is_infinite() {
            [0.0, 0.0, 0.0, 0.0]
        } else {
            [self.min_x, self.min_y, self.max_x, self.max_y]
        }
    }
}
