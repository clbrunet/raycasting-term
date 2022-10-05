use std::f64;

use crate::Raycasting;

#[derive(Debug)]
pub struct WindowSprite {
    pub image_index: usize,
    pub x: i16,
    pub distance: f64,
}

impl WindowSprite {
    pub fn new(image_index: usize, x: i16, distance: f64) -> Self {
        Self {
            image_index,
            x,
            distance,
        }
    }
}

pub fn get_sorted_window_sprites(raycasting: &Raycasting) -> Vec<WindowSprite> {
    let mut window_sprites = Vec::new();
    for sprite in &raycasting.sprites {
        let angle_from_player = raycasting.player.get_angle_to(&sprite.position);
        if angle_from_player > raycasting.player.horizontal_fov
            || -raycasting.player.horizontal_fov > angle_from_player
        {
            continue;
        }
        let projection_plane_distance = (raycasting.window.width() as f64 / 2.0)
            / (raycasting.player.horizontal_fov / 2.0).tan();
        let sprite_center_x_from_center = -angle_from_player.tan() * projection_plane_distance;
        let x =
            (raycasting.window.width() as f64 / 2.0 + sprite_center_x_from_center).round() as i16;
        let distance = nalgebra::distance(&sprite.position, &raycasting.player.position)
            * angle_from_player.cos();
        window_sprites.push(WindowSprite::new(sprite.image_index, x, distance));
    }
    window_sprites.sort_unstable_by(|a, b| b.distance.partial_cmp(&a.distance).unwrap());
    window_sprites
}
