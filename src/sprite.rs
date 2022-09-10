use std::f64;

use nalgebra::Point2;
use winterm::Window;

use crate::player::Player;

pub trait UpdateExt {
    fn update(&mut self, player: &Player, window: &Window);
}

#[derive(Debug)]
pub struct Sprite {
    pub position: Point2<f64>,
    pub image_index: usize,
    pub angle_from_player: f64,
    pub center_x: i16,
    pub distance: f64,
}

impl Sprite {
    pub fn new(
        position: Point2<f64>,
        image_index: usize,
        player: &Player,
        window: &Window,
    ) -> Self {
        let mut sprite = Self {
            position,
            image_index,
            angle_from_player: 0.0,
            center_x: 0,
            distance: 0.0,
        };
        sprite.update(player, window);
        sprite
    }
}

impl UpdateExt for Sprite {
    fn update(&mut self, player: &Player, window: &Window) {
        self.angle_from_player = player.get_angle_to(&self.position);
        let projection_plane_distance =
            (window.width() as f64 / 2.0) / (player.horizontal_fov / 2.0).tan();
        let sprite_center_x_from_center = -self.angle_from_player.tan() * projection_plane_distance;
        self.center_x = (window.width() as f64 / 2.0 + sprite_center_x_from_center).round() as i16;
        self.distance =
            nalgebra::distance(&self.position, &player.position) * self.angle_from_player.cos();
    }
}

impl UpdateExt for Vec<Sprite> {
    fn update(&mut self, player: &Player, window: &Window) {
        for sprite in self.iter_mut() {
            sprite.update(player, window);
        }
        self.sort_unstable_by(|a, b| b.distance.partial_cmp(&a.distance).unwrap());
    }
}
