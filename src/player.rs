use nalgebra::Point2;

use crate::MAP;

pub struct Player {
    pub position: Point2<f64>,
    pub angle: f64,
    pub horizontal_fov: f64,
}

impl Player {
    pub fn new(x: f64, y: f64, angle: f64, horizontal_fov: f64) -> Self {
        Self {
            position: Point2::new(x, y),
            angle,
            horizontal_fov,
        }
    }

    pub fn translate(&mut self, x: f64, y: f64) {
        const DISPLACEMENT_FROM_WALL: f64 = 0.000001;

        self.position.x += x;
        if MAP[self.position.y as usize][self.position.x as usize] != 0 {
            if x > 0.0 {
                self.position.x = self.position.x.floor() - DISPLACEMENT_FROM_WALL;
            } else {
                self.position.x = self.position.x.ceil() + DISPLACEMENT_FROM_WALL;
            }
        }

        self.position.y += y;
        if MAP[self.position.y as usize][self.position.x as usize] != 0 {
            if y > 0.0 {
                self.position.y = self.position.y.floor() - DISPLACEMENT_FROM_WALL;
            } else {
                self.position.y = self.position.y.ceil() + DISPLACEMENT_FROM_WALL;
            }
        }
    }
}
