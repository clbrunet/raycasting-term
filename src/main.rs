use std::f64;

use crossterm::{event::KeyCode, style::Color, Result};
use nalgebra::{Point2, Vector2};
use winterm::Window;

static MAP: [[u8; 8]; 8] = [
    [1, 1, 1, 1, 1, 1, 1, 1],
    [1, 0, 0, 0, 0, 0, 0, 1],
    [1, 0, 0, 0, 0, 2, 0, 1],
    [1, 0, 0, 0, 0, 0, 0, 1],
    [1, 0, 0, 0, 0, 0, 0, 1],
    [1, 0, 0, 0, 0, 0, 0, 1],
    [1, 0, 0, 0, 0, 0, 0, 1],
    [1, 1, 1, 1, 1, 1, 1, 1],
];

fn get_normalized_radians_angle(angle: f64) -> f64 {
    if angle.is_sign_negative() {
        angle % f64::consts::TAU + f64::consts::TAU
    } else {
        angle % f64::consts::TAU
    }
}

struct Player {
    position: Point2<f64>,
    angle: f64,
}

impl Player {
    fn translate(&mut self, x: f64, y: f64) {
        const DISPLACEMENT_FROM_WALL: f64 = 0.000001;

        self.position.x += x;
        if MAP[self.position.y as usize][self.position.x as usize] != 0 {
            if x > 0_f64 {
                self.position.x = self.position.x.floor() - DISPLACEMENT_FROM_WALL;
            } else {
                self.position.x = self.position.x.ceil() + DISPLACEMENT_FROM_WALL;
            }
        }

        self.position.y += y;
        if MAP[self.position.y as usize][self.position.x as usize] != 0 {
            if y > 0_f64 {
                self.position.y = self.position.y.floor() - DISPLACEMENT_FROM_WALL;
            } else {
                self.position.y = self.position.y.ceil() + DISPLACEMENT_FROM_WALL;
            }
        }
    }
}

struct Raycasting {
    window: Window,
    player: Player,
    horizontal_fov: f64,
    should_stop: bool,
}

impl Raycasting {
    fn new() -> Result<Self> {
        Ok(Raycasting {
            window: Window::new(45, 80)?,
            player: Player {
                position: Point2::new(3_f64, 4_f64),
                angle: 45_f64.to_radians(),
            },
            horizontal_fov: 60_f64.to_radians(),
            should_stop: false,
        })
    }

    fn instantaneous_update(&mut self) {
        if self.window.get_key(KeyCode::Esc) {
            self.should_stop = true;
        }
    }

    fn continuous_update(&mut self, delta_time: f64) {
        if self.window.get_key(KeyCode::Char('w')) {
            self.player.translate(
                self.player.angle.cos() * 5_f64 * delta_time,
                -self.player.angle.sin() * 5_f64 * delta_time,
            );
        }
        if self.window.get_key(KeyCode::Char('s')) {
            self.player.translate(
                -self.player.angle.cos() * 5_f64 * delta_time,
                self.player.angle.sin() * 5_f64 * delta_time,
            );
        }
        if self.window.get_key(KeyCode::Char('a')) {
            self.player.translate(
                (self.player.angle + f64::consts::FRAC_PI_2).cos() * 5_f64 * delta_time,
                -(self.player.angle + f64::consts::FRAC_PI_2).sin() * 5_f64 * delta_time,
            );
        }
        if self.window.get_key(KeyCode::Char('d')) {
            self.player.translate(
                -(self.player.angle + f64::consts::FRAC_PI_2).cos() * 5_f64 * delta_time,
                (self.player.angle + f64::consts::FRAC_PI_2).sin() * 5_f64 * delta_time,
            );
        }

        if self.window.get_key(KeyCode::Left) {
            self.player.angle += 50_f64.to_radians() * delta_time;
            self.player.angle = get_normalized_radians_angle(self.player.angle);
        }
        if self.window.get_key(KeyCode::Right) {
            self.player.angle -= 50_f64.to_radians() * delta_time;
            self.player.angle = get_normalized_radians_angle(self.player.angle);
        }
    }

    fn render_column(&mut self, x: u16, ray_angle: f64) -> Result<()> {
        let ray_direction = Vector2::new(ray_angle.cos(), ray_angle.sin());
        let mut map_coordinates = Vector2::new(
            self.player.position.x as usize,
            self.player.position.y as usize,
        );
        let mut map_coordinates_steps = Vector2::zeros();
        let mut distances = Vector2::zeros();
        if f64::consts::FRAC_PI_2 < ray_angle && ray_angle < 3_f64 * f64::consts::FRAC_PI_2 {
            map_coordinates_steps.x = -1;
            distances.x = self.player.position.x.fract() / (ray_angle - f64::consts::PI).cos();
        } else {
            map_coordinates_steps.x = 1;
            distances.x = (1_f64 - self.player.position.x.fract()) / ray_angle.cos();
        }
        if 0_f64 < ray_angle && ray_angle < f64::consts::PI {
            map_coordinates_steps.y = -1;
            distances.y =
                self.player.position.y.fract() / (ray_angle - f64::consts::FRAC_PI_2).cos();
        } else {
            map_coordinates_steps.y = 1;
            distances.y = (1_f64 - self.player.position.y.fract())
                / (ray_angle + f64::consts::FRAC_PI_2).cos();
        }
        let steps = Vector2::new(
            1_f64.hypot(ray_direction.y / ray_direction.x),
            1_f64.hypot(ray_direction.x / ray_direction.y),
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
        let distance = euclidian_distance * (self.player.angle - ray_angle).cos();
        let color = match MAP[map_coordinates.y][map_coordinates.x] {
            1 => {
                if is_vertical {
                    Color::Rgb { r: 0xa0, g: 0x00, b: 0x00 }
                } else {
                    Color::Rgb { r: 0x80, g: 0x00, b: 0x00 }
                }
            }
            2 => {
                if is_vertical {
                    Color::Rgb { r: 0x00, g: 0x00, b: 0xa0 }
                } else {
                    Color::Rgb { r: 0x00, g: 0x00, b: 0x80 }
                }
            }
            _ => Color::Black,
        };
        let height = std::cmp::min(
            (self.window.height() as f64 / distance).round() as u16,
            self.window.height(),
        );
        let wall_start = ((self.window.height() - height) as f32 / 2_f32).round() as u16;
        let wall_end = wall_start + height;
        for y in 0..wall_start {
            self.window.set_pixel(y, x, Color::Rgb { r: 0x64, g: 0x64, b: 0x64 });
        }
        for y in wall_start..wall_end {
            self.window.set_pixel(y, x, color);
        }
        for y in wall_end..self.window.height() {
            self.window.set_pixel(y, x, Color::Rgb { r: 0xBA, g: 0x92, b: 0x6C });
        }
        Ok(())
    }

    fn render(&mut self) -> Result<()> {
        let angle_increment = -(self.horizontal_fov / (self.window.width() - 1) as f64);
        let mut ray_angle =
            get_normalized_radians_angle(self.player.angle + self.horizontal_fov / 2_f64);
        for x in 0..self.window.width() {
            self.render_column(x, ray_angle)?;
            ray_angle = get_normalized_radians_angle(ray_angle + angle_increment);
        }
        self.window.draw()?;
        Ok(())
    }

    fn run(&mut self) -> Result<()> {
        while !self.should_stop {
            self.window.poll_events()?;
            self.instantaneous_update();
            self.continuous_update(0.02);
            self.render()?;
        }
        Ok(())
    }
}

fn main() -> Result<()> {
    let mut raycasting = Raycasting::new()?;
    raycasting.run()?;
    Ok(())
}
