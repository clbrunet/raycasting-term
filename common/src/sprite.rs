use nalgebra::Point2;

#[derive(Debug)]
pub struct Sprite {
    pub position: Point2<f64>,
    pub image_index: usize,
}

impl Sprite {
    pub fn new(position: Point2<f64>, image_index: usize) -> Self {
        Self {
            position,
            image_index,
        }
    }
}
