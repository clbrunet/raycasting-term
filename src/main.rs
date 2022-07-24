use crossterm::{event::KeyCode, style::Color, Result};
use nalgebra::{Point2, Vector2};
use winterm::Window;

static MAP: [[u8; 8]; 8] = [
    [1, 1, 1, 1, 1, 1, 1, 1],
    [1, 0, 0, 0, 0, 0, 0, 1],
    [1, 0, 0, 0, 0, 0, 0, 1],
    [1, 0, 0, 0, 0, 2, 0, 1],
    [1, 0, 0, 0, 0, 0, 0, 1],
    [1, 0, 0, 0, 0, 0, 0, 1],
    [1, 0, 0, 0, 0, 0, 0, 1],
    [1, 1, 1, 1, 1, 1, 1, 1],
];

struct Player {
    position: Point2<f64>,
    angle: f64,
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
                position: Point2::new(4., 4.),
                angle: 0.,
            },
            horizontal_fov: 60f64.to_radians(),
            should_stop: false,
        })
    }

    fn instantaneous_update(&mut self) {
        if self.window.get_key(KeyCode::Esc) {
            self.should_stop = true;
        }
    }

    fn continuous_update(&mut self) {
        if self.window.get_key(KeyCode::Left) {
            self.player.angle -= 1f64.to_radians();
        }
        if self.window.get_key(KeyCode::Right) {
            self.player.angle += 1f64.to_radians();
        }

        let mut movement: Vector2<f64> = Vector2::zeros();
        if self.window.get_key(KeyCode::Char('w')) {
            movement.x += self.player.angle.cos();
            movement.y += self.player.angle.sin();
        }
        if self.window.get_key(KeyCode::Char('s')) {
            movement.x -= self.player.angle.cos();
            movement.y -= self.player.angle.sin();
        }
        if movement == Vector2::zeros() {
            return;
        }
        movement.set_magnitude(0.1);
        self.player.position += movement;
    }

    fn render(&mut self) -> Result<()> {
        let angle_increment = self.horizontal_fov / (self.window.width() - 1) as f64;
        let mut ray_angle = self.player.angle - self.horizontal_fov / 2.;
        for x in 0..self.window.width() {
            let mut hit = self.player.position;
            hit.x += ray_angle.cos() * 0.05;
            hit.y += ray_angle.sin() * 0.05;
            let mut distance = 0.05;
            while distance < 100. {
                if MAP[hit.y as usize][hit.x as usize] != 0 {
                    break;
                }
                hit.x += ray_angle.cos() * 0.05;
                hit.y += ray_angle.sin() * 0.05;
                distance += 0.05;
            }
            distance *= (self.player.angle - ray_angle).cos();
            let height = ((50f64 / distance).round() as u16).min(self.window.height());
            let ceiling_border = ((self.window.height() - height) as f32 / 2.).round() as u16;
            for y in 0..self.window.height() {
                self.window.set_pixel(y, x, Color::Black);
            }
            let color = match MAP[hit.y as usize][hit.x as usize] {
                1 => Color::White,
                2 => Color::DarkYellow,
                _ => Color::Black,
            };
            for y in ceiling_border..(ceiling_border + height) {
                self.window.set_pixel(y, x, color);
            }
            ray_angle += angle_increment;
        }
        self.window.draw()?;
        Ok(())
    }

    fn run(&mut self) -> Result<()> {
        while self.should_stop == false {
            self.window.poll_events()?;
            self.instantaneous_update();
            self.continuous_update();
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
