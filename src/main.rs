use std::{
    cmp::{self, Ord},
    f64,
    time::Instant,
};

use crossterm::{event::KeyCode, style::Color, Result};
use nalgebra::{ArrayStorage, Const, Matrix, Point2, Vector2, Vector3, Vector4};
use winterm::Window;

type Matrix16<T> = Matrix<T, Const<16>, Const<16>, ArrayStorage<T, 16, 16>>;

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
    horizontal_fov: f64,
}

impl Player {
    fn new(x: f64, y: f64, angle: f64, horizontal_fov: f64) -> Self {
        Self {
            position: Point2::new(x, y),
            angle,
            horizontal_fov,
        }
    }

    fn translate(&mut self, x: f64, y: f64) {
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

#[derive(Debug)]
struct Sprite {
    position: Point2<f64>,
    image_index: usize,
}

impl Sprite {
    fn new(position: Point2<f64>, image_index: usize) -> Self {
        Self {
            position,
            image_index,
        }
    }

    fn get_sorted_seen_sprites<'a>(
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
struct SeenSprite<'a> {
    sprite: &'a Sprite,
    center_x: i16,
    distance: f64,
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

struct Raycasting {
    window: Window,
    player: Player,
    sprites: Vec<Sprite>,
    images: Vec<Matrix16<Vector4<u8>>>,
    z_buffer: Vec<f64>,
    should_stop: bool,
}

fn fill_test_image(image: &mut Matrix16<Vector4<u8>>) {
    let colors = vec![
        Vector3::new(255.0, 0.0, 0.0),
        Vector3::new(255.0, 63.0, 0.0),
        Vector3::new(255.0, 127.0, 0.0),
        Vector3::new(255.0, 191.0, 0.0),
        Vector3::new(255.0, 255.0, 0.0),
        Vector3::new(191.0, 255.0, 0.0),
        Vector3::new(127.0, 255.0, 0.0),
        Vector3::new(63.0, 255.0, 0.0),
        Vector3::new(0.0, 255.0, 0.0),
        Vector3::new(0.0, 255.0, 127.0),
        Vector3::new(0.0, 255.0, 255.0),
        Vector3::new(0.0, 127.0, 255.0),
        Vector3::new(0.0, 0.0, 255.0),
        Vector3::new(127.0, 0.0, 255.0),
        Vector3::new(255.0, 0.0, 255.0),
        Vector3::new(255.0, 0.0, 127.0),
    ];
    for x in 0..image.ncols() {
        let multiplier = ((x + 1) as f64 / image.ncols() as f64) * 0.75 + 0.25;
        for y in 0..image.nrows() {
            let color = colors[y] * multiplier;
            image[(y, x)] = Vector4::new(color.x as u8, color.y as u8, color.z as u8, 255);
        }
    }
}

impl Raycasting {
    fn new() -> Result<Self> {
        let height = 45;
        let width = 80;
        let mut image = Matrix16::zeros();
        fill_test_image(&mut image);
        Ok(Self {
            window: Window::new(height, width)?,
            player: Player::new(3.0, 4.0, 180.0_f64.to_radians(), 60.0_f64.to_radians()),
            sprites: vec![
                Sprite::new(Point2::new(4.0, 6.0), 0),
                Sprite::new(Point2::new(2.0, 4.0), 0),
            ],
            images: vec![image],
            z_buffer: vec![0.0; width.into()],
            should_stop: false,
        })
    }

    fn instantaneous_update(&mut self) {
        if self.window.get_key(KeyCode::Esc) {
            self.should_stop = true;
        }
    }

    fn continuous_update(&mut self, delta_time: f64) {
        const TRANSLATION_SPEED: f64 = 30.0;
        let rotation_speed: f64 = 300.0_f64.to_radians();

        if self.window.get_key(KeyCode::Char('w')) {
            self.player.translate(
                self.player.angle.cos() * TRANSLATION_SPEED * delta_time,
                -self.player.angle.sin() * TRANSLATION_SPEED * delta_time,
            );
        } else {
        }
        if self.window.get_key(KeyCode::Char('s')) {
            self.player.translate(
                -self.player.angle.cos() * TRANSLATION_SPEED * delta_time,
                self.player.angle.sin() * TRANSLATION_SPEED * delta_time,
            );
        }
        if self.window.get_key(KeyCode::Char('a')) {
            self.player.translate(
                (self.player.angle + f64::consts::FRAC_PI_2).cos() * TRANSLATION_SPEED * delta_time,
                -(self.player.angle + f64::consts::FRAC_PI_2).sin() * TRANSLATION_SPEED * delta_time,
            );
        }
        if self.window.get_key(KeyCode::Char('d')) {
            self.player.translate(
                -(self.player.angle + f64::consts::FRAC_PI_2).cos() * TRANSLATION_SPEED * delta_time,
                (self.player.angle + f64::consts::FRAC_PI_2).sin() * TRANSLATION_SPEED * delta_time,
            );
        }

        if self.window.get_key(KeyCode::Left) {
            self.player.angle += rotation_speed * delta_time;
            self.player.angle = get_normalized_radians_angle(self.player.angle);
        }
        if self.window.get_key(KeyCode::Right) {
            self.player.angle -= rotation_speed * delta_time;
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
        if f64::consts::FRAC_PI_2 < ray_angle && ray_angle < 3.0 * f64::consts::FRAC_PI_2 {
            map_coordinates_steps.x = -1;
            distances.x = self.player.position.x.fract() / (ray_angle - f64::consts::PI).cos();
        } else {
            map_coordinates_steps.x = 1;
            distances.x = (1.0 - self.player.position.x.fract()) / ray_angle.cos();
        }
        if 0.0 < ray_angle && ray_angle < f64::consts::PI {
            map_coordinates_steps.y = -1;
            distances.y =
                self.player.position.y.fract() / (ray_angle - f64::consts::FRAC_PI_2).cos();
        } else {
            map_coordinates_steps.y = 1;
            distances.y = (1_f64 - self.player.position.y.fract())
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
        let distance = euclidian_distance * (self.player.angle - ray_angle).cos();
        self.z_buffer[x as usize] = distance;
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
        let height = (self.window.height() as f64 / distance).round() as u16;
        let wall_start = cmp::max(
            0,
            ((self.window.height() as i32 - height as i32) as f32 / 2_f32).round() as u16,
        );
        let wall_end = cmp::min(
            self.window.height(),
            ((self.window.height() as i32 + height as i32) as f32 / 2_f32).round() as u16,
        );
        for y in 0..wall_start {
            self.window.set_pixel(
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
            self.window.set_pixel(y, x, color);
        }
        for y in wall_end..self.window.height() {
            self.window.set_pixel(
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

    fn render(&mut self) -> Result<()> {
        let angle_increment = -(self.player.horizontal_fov / (self.window.width() - 1) as f64);
        let mut ray_angle =
            get_normalized_radians_angle(self.player.angle + self.player.horizontal_fov / 2.0);
        for x in 0..self.window.width() {
            self.render_column(x, ray_angle)?;
            ray_angle = get_normalized_radians_angle(ray_angle + angle_increment);
        }
        for seen_sprite in
            Sprite::get_sorted_seen_sprites(&self.sprites, &self.player, &self.window)
        {
            let image = &self.images[seen_sprite.sprite.image_index];
            let height = (self.window.height() as f64 / seen_sprite.distance).round() as u16;
            let start_y = cmp::max(
                0,
                ((self.window.height() as i32 - height as i32) as f32 / 2.0).round() as u16,
            );
            let end_y = cmp::min(
                self.window.height(),
                ((self.window.height() + height) as f32 / 2.0).round() as u16,
            );
            let image_y_step = image.nrows() as f64 / height as f64;
            let start_image_y = f64::max(
                0.0,
                -((self.window.height() as i32 - height as i32) as f64 / 2_f64).round()
                    * image_y_step,
            );
            let width =
                (height as f32 * image.ncols() as f32 / image.nrows() as f32).round() as u16;
            let start_x = cmp::max(
                0,
                (seen_sprite.center_x as f32 - (width as f32 / 2.0) + 0.1).round() as u16,
            );
            let end_x = Ord::clamp(
                (seen_sprite.center_x as f32 + (width as f32 / 2.0)).round() as i16,
                0,
                self.window.width() as i16,
            ) as u16;
            let image_x_step = image.ncols() as f64 / width as f64;
            let mut image_x = f64::max(
                0.0,
                -(seen_sprite.center_x as f64 - (width as f64 / 2_f64) + 0.1).round()
                    * image_x_step,
            );
            for x in start_x..end_x {
                if seen_sprite.distance > self.z_buffer[x as usize] {
                    image_x += image_x_step;
                    continue;
                } else {
                    self.z_buffer[x as usize] = seen_sprite.distance;
                }
                let mut image_y = start_image_y;
                for y in start_y..end_y {
                    let color = &image[(image_y as usize, image_x as usize)];
                    if color.w != u8::MAX {
                        continue;
                    }
                    let terminal_color = Color::Rgb {
                        r: color.x,
                        g: color.y,
                        b: color.z,
                    };
                    self.window.set_pixel(y, x, terminal_color);
                    image_y += image_y_step;
                }
                image_x += image_x_step;
            }
        }
        self.window.redraw()?;
        Ok(())
    }

    fn run(&mut self) -> Result<()> {
        let start_time = Instant::now();
        let mut consumned_seconds = 0.0;
        let max_elapsed_time = 0.03;
        while !self.should_stop {
            self.window.poll_events()?;
            self.instantaneous_update();
            let elapsed_time = start_time.elapsed().as_secs_f64() - consumned_seconds;
            consumned_seconds += elapsed_time;
            let delta_time = if elapsed_time < max_elapsed_time {
                elapsed_time
            } else {
                max_elapsed_time
            };
            self.continuous_update(delta_time);
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
