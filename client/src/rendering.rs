use std::{cmp, f64};

use common::{get_normalized_radians_angle, MAP};
use crossterm::{style::Color, Result};
use image::GenericImageView;
use nalgebra::Vector2;

use crate::{
    window_sprite::{get_sorted_window_sprites, WindowSprite},
    Raycasting,
};

fn render_column(raycasting: &mut Raycasting, x: u16, ray_angle: f64) -> Result<()> {
    let ray_direction = Vector2::new(ray_angle.cos(), ray_angle.sin());
    let mut map_coordinates = Vector2::new(
        raycasting.player.position.x as usize,
        raycasting.player.position.y as usize,
    );
    let mut map_coordinates_steps = Vector2::zeros();
    let mut distances = Vector2::zeros();
    if f64::consts::FRAC_PI_2 < ray_angle && ray_angle < 3.0 * f64::consts::FRAC_PI_2 {
        map_coordinates_steps.x = -1;
        distances.x = raycasting.player.position.x.fract() / (ray_angle - f64::consts::PI).cos();
    } else {
        map_coordinates_steps.x = 1;
        distances.x = (1.0 - raycasting.player.position.x.fract()) / ray_angle.cos();
    }
    if 0.0 < ray_angle && ray_angle < f64::consts::PI {
        map_coordinates_steps.y = -1;
        distances.y =
            raycasting.player.position.y.fract() / (ray_angle - f64::consts::FRAC_PI_2).cos();
    } else {
        map_coordinates_steps.y = 1;
        distances.y = (1_f64 - raycasting.player.position.y.fract())
            / (ray_angle + f64::consts::FRAC_PI_2).cos();
    }
    let steps = Vector2::new(
        1.0_f64.hypot(ray_direction.y / ray_direction.x),
        1.0_f64.hypot(ray_direction.x / ray_direction.y),
    );
    let (euclidian_distance, is_vertical) = loop {
        if distances.x < distances.y {
            map_coordinates.x = (map_coordinates.x as i32 + map_coordinates_steps.x) as usize;
            if MAP[map_coordinates.y][map_coordinates.x] != 0 {
                break (distances.x, true);
            }
            distances.x += steps.x;
        } else {
            map_coordinates.y = (map_coordinates.y as i32 + map_coordinates_steps.y) as usize;
            if MAP[map_coordinates.y][map_coordinates.x] != 0 {
                break (distances.y, false);
            }
            distances.y += steps.y;
        }
    };
    let distance = euclidian_distance * (raycasting.player.angle - ray_angle).cos();
    raycasting.z_buffer[x as usize] = distance;
    let color = match MAP[map_coordinates.y][map_coordinates.x] {
        1 => {
            if is_vertical {
                Color::Rgb {
                    r: 0xa0,
                    g: 0x00,
                    b: 0x00,
                }
            } else {
                Color::Rgb {
                    r: 0x80,
                    g: 0x00,
                    b: 0x00,
                }
            }
        }
        2 => {
            if is_vertical {
                Color::Rgb {
                    r: 0x00,
                    g: 0x00,
                    b: 0xa0,
                }
            } else {
                Color::Rgb {
                    r: 0x00,
                    g: 0x00,
                    b: 0x80,
                }
            }
        }
        _ => Color::Black,
    };
    let height = (raycasting.window.height() as f64 / distance).round() as u16;
    let wall_start = cmp::max(
        0,
        ((raycasting.window.height() as i32 - height as i32) as f32 / 2_f32).round() as u16,
    );
    let wall_end = cmp::min(
        raycasting.window.height(),
        ((raycasting.window.height() as i32 + height as i32) as f32 / 2_f32).round() as u16,
    );
    for y in 0..wall_start {
        raycasting.window.set_pixel(
            y,
            x,
            Color::Rgb {
                r: 0x64,
                g: 0x64,
                b: 0x64,
            },
        );
    }
    for y in wall_start..wall_end {
        raycasting.window.set_pixel(y, x, color);
    }
    for y in wall_end..raycasting.window.height() {
        raycasting.window.set_pixel(
            y,
            x,
            Color::Rgb {
                r: 0xBA,
                g: 0x92,
                b: 0x6C,
            },
        );
    }
    Ok(())
}

fn render_window_sprite(window_sprite: &WindowSprite, raycasting: &mut Raycasting) -> Result<()> {
    let sprite_sheet = &raycasting.sprite_sheets[window_sprite.sprite_sheet_index];
    let height = (raycasting.window.height() as f64 / window_sprite.distance).round() as u16;
    let start_y = cmp::max(
        0,
        ((raycasting.window.height() as i32 - height as i32) as f32 / 2.0).round() as u16,
    );
    let end_y = cmp::min(
        raycasting.window.height(),
        ((raycasting.window.height() + height) as f32 / 2.0).round() as u16,
    );
    let image_y_step = sprite_sheet.size() as f64 / height as f64;
    let start_image_y = f64::max(
        0.0,
        -((raycasting.window.height() as i32 - height as i32) as f64 / 2_f64).round()
            * image_y_step,
    );
    let width =
        (height as f32 * sprite_sheet.size() as f32 / sprite_sheet.size() as f32).round() as u16;
    let start_x = cmp::max(
        0,
        (window_sprite.x as f32 - (width as f32 / 2.0) + 0.1).round() as u16,
    );
    let end_x = Ord::clamp(
        (window_sprite.x as f32 + (width as f32 / 2.0)).round() as i16,
        0,
        raycasting.window.width() as i16,
    ) as u16;
    let image_x_step = sprite_sheet.size() as f64 / width as f64;
    let mut image_x = f64::max(
        0.0,
        -(window_sprite.x as f64 - (width as f64 / 2_f64) + 0.1).round() * image_x_step,
    );
    for x in start_x..end_x {
        if window_sprite.distance > raycasting.z_buffer[x as usize] {
            image_x += image_x_step;
            continue;
        } else {
            raycasting.z_buffer[x as usize] = window_sprite.distance;
        }
        let mut image_y = start_image_y;
        for y in start_y..end_y {
            let color = &sprite_sheet
                .image()
                .get_pixel(image_x as u32, image_y as u32 + window_sprite.sprite_sheet_y_offset);
            if color[3] != u8::MAX {
                image_y += image_y_step;
                continue;
            }
            let terminal_color = Color::Rgb {
                r: color[0],
                g: color[1],
                b: color[2],
            };
            raycasting.window.set_pixel(y, x, terminal_color);
            image_y += image_y_step;
        }
        image_x += image_x_step;
    }
    Ok(())
}

pub fn render(raycasting: &mut Raycasting) -> Result<()> {
    let angle_increment =
        -(raycasting.player.horizontal_fov / (raycasting.window.width() - 1) as f64);
    let mut ray_angle = get_normalized_radians_angle(
        raycasting.player.angle + raycasting.player.horizontal_fov / 2.0,
    );
    for x in 0..raycasting.window.width() {
        render_column(raycasting, x, ray_angle)?;
        ray_angle = get_normalized_radians_angle(ray_angle + angle_increment);
    }
    let sorted_window_sprites = get_sorted_window_sprites(raycasting);
    for window_sprite in sorted_window_sprites {
        render_window_sprite(&window_sprite, raycasting)?;
    }
    raycasting.window.redraw()?;
    Ok(())
}
