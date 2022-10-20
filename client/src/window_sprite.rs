use std::f64;

use common::get_normalized_radians_angle;

use crate::Raycasting;

#[derive(Debug)]
pub struct WindowSprite {
    pub sprite_sheet_index: usize,
    pub sprite_sheet_y_offset: u32,
    pub x: i16,
    pub distance: f64,
}

impl WindowSprite {
    pub fn new(sprite_sheet_index: usize, sprite_sheet_y_offset: u32, x: i16, distance: f64) -> Self {
        Self {
            sprite_sheet_index,
            sprite_sheet_y_offset,
            x,
            distance,
        }
    }
}

pub fn get_sorted_window_sprites(raycasting: &Raycasting) -> Vec<WindowSprite> {
    let mut window_sprites = Vec::new();
    for sprite in &raycasting.sprites {
        if let Some(client) = &raycasting.client {
            if sprite.id == client.id {
                continue;
            }
        }
        let angle_from_player = raycasting.player.get_angle_to(&sprite.position);
        if angle_from_player > raycasting.player.horizontal_fov
            || -raycasting.player.horizontal_fov > angle_from_player
        {
            continue;
        }
        let distance = nalgebra::distance(&sprite.position, &raycasting.player.position)
            * angle_from_player.cos();
        if distance < 1e-10 {
            continue;
        }
        let projection_plane_distance = (raycasting.window.width() as f64 / 2.0)
            / (raycasting.player.horizontal_fov / 2.0).tan();
        let x_center_offset = -angle_from_player.tan() * projection_plane_distance;
        let x = (raycasting.window.width() as f64 / 2.0 + x_center_offset).round() as i16;
        let sprite_sheet_y_offset = if let Some(angle) = sprite.angle {
            let angle_to_sprite = get_normalized_radians_angle(raycasting.player.angle + angle_from_player);
            let sprite_to_player_angle = get_normalized_radians_angle(angle_to_sprite + f64::consts::PI - angle);
            raycasting.sprite_sheets[sprite.sprite_sheet_index].get_y_offset(sprite_to_player_angle)
        } else {
            0
        };
        window_sprites.push(WindowSprite::new(sprite.sprite_sheet_index, sprite_sheet_y_offset, x, distance));
    }
    window_sprites.sort_unstable_by(|a, b| b.distance.partial_cmp(&a.distance).unwrap());
    window_sprites
}
