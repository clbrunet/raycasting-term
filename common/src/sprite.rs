use nalgebra::Point2;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Sprite {
    pub id: u32,
    pub position: Point2<f64>,
    pub image_index: usize,
}

impl Sprite {
    pub fn new(id: u32, position: Point2<f64>, image_index: usize) -> Self {
        Self {
            id,
            position,
            image_index,
        }
    }
}
