use nalgebra::Point2;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Sprite {
    pub id: u32,
    pub position: Point2<f64>,
    pub sprite_sheet_index: usize,
    pub angle: Option<f64>,
}

impl Sprite {
    pub fn new(id: u32, position: Point2<f64>, image_index: usize, angle: Option<f64>) -> Self {
        Self {
            id,
            position,
            sprite_sheet_index: image_index,
            angle,
        }
    }
}
