use std::f64;

use common::{get_normalized_radians_angle, MAP};
use nalgebra::Point2;

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

    /// Get angle to position in [-pi; pi[
    pub fn get_angle_to(&self, position: &Point2<f64>) -> f64 {
        let player_to_position = position - self.position;
        let player_to_position_world_angle = (-player_to_position.y).atan2(player_to_position.x);
        let angle_from_player =
            get_normalized_radians_angle(player_to_position_world_angle - self.angle);
        if angle_from_player >= f64::consts::PI {
            angle_from_player - f64::consts::TAU
        } else {
            angle_from_player
        }
    }
}
