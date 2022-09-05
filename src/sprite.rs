use std::f64;

use nalgebra::Point2;
use winterm::Window;

use crate::{get_normalized_radians_angle, player::Player};

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

    pub fn get_sorted_seen_sprites<'a>(
        sprites: &'a Vec<Self>,
        player: &Player,
        window: &Window,
    ) -> Vec<SeenSprite<'a>> {
        let mut seen_sprites = Vec::new();
        for sprite in sprites {
            let player_to_sprite = sprite.position - player.position;
            let player_to_sprite_angle =
                get_normalized_radians_angle((-player_to_sprite.y).atan2(player_to_sprite.x));
            let mut angle = get_normalized_radians_angle(player_to_sprite_angle - player.angle);
            if angle > f64::consts::PI {
                angle -= f64::consts::TAU;
            }
            angle = -angle;
            let projection_plane_distance =
                (window.width() as f64 / 2.0) / (player.horizontal_fov / 2.0).tan();
            let center_x_from_center = angle.tan() * projection_plane_distance;
            let center_x = (window.width() as f64 / 2.0 + center_x_from_center).round();
            if -f64::consts::FRAC_PI_2 < angle && angle < f64::consts::FRAC_PI_2 {
                seen_sprites.push(SeenSprite::new(
                    sprite,
                    center_x as i16,
                    player_to_sprite.magnitude() * angle.cos(),
                ));
            }
        }
        seen_sprites.sort_unstable_by(|a, b| b.distance.partial_cmp(&a.distance).unwrap());
        seen_sprites
    }
}

#[derive(Debug)]
pub struct SeenSprite<'a> {
    pub sprite: &'a Sprite,
    pub center_x: i16,
    pub distance: f64,
}

impl<'a> SeenSprite<'a> {
    fn new(sprite: &'a Sprite, center_x: i16, distance: f64) -> Self {
        Self {
            sprite,
            center_x,
            distance,
        }
    }
}
