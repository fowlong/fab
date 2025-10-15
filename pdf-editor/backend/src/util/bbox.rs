use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct BBox(pub [f32; 4]);

impl BBox {
    pub fn width(&self) -> f32 {
        let [x0, _, x1, _] = self.0;
        x1 - x0
    }

    pub fn height(&self) -> f32 {
        let [_, y0, _, y1] = self.0;
        y1 - y0
    }
}
