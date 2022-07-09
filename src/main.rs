use std::time::Duration;

use crossterm::{
    event::{poll, read, Event::Key, KeyCode, KeyEvent},
    style::Color,
    Result,
};
use winterm::Window;

pub struct Vec2<T> {
    pub x: T,
    pub y: T,
}

impl<T> Vec2<T> {
    pub fn new(x: T, y: T) -> Self {
        Vec2 { x, y }
    }
}

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

pub struct Raycasting {
    window: Window,
    should_stop: bool,
    player: (Vec2<f64>, f64),
    horizontal_fov: f64,
}

impl Raycasting {
    fn new() -> Result<Self> {
        Ok(Raycasting {
            window: Window::new(58, 72)?,
            should_stop: false,
            player: (Vec2::new(4., 4.), 0.),
            horizontal_fov: 60f64.to_radians(),
        })
    }

    fn on_key_event(&mut self, key_event: KeyEvent) {
        if key_event.code == KeyCode::Esc {
            self.should_stop = true;
        }
        if key_event.code == KeyCode::Char('w') {
            self.player.0.x += self.player.1.cos() * 0.1;
            self.player.0.y += self.player.1.sin() * 0.1;
        }
        if key_event.code == KeyCode::Char('s') {
            self.player.0.x -= self.player.1.cos() * 0.1;
            self.player.0.y -= self.player.1.sin() * 0.1;
        }
        if key_event.code == KeyCode::Left {
            self.player.1 -= 1f64.to_radians();
        }
        if key_event.code == KeyCode::Right {
            self.player.1 += 1f64.to_radians();
        }
    }

    fn render(&mut self) {
        let angle_increment = self.horizontal_fov / (self.window.width() - 1) as f64;
        let mut ray_angle = self.player.1 - self.horizontal_fov / 2.;
        for x in 0..self.window.width() {
            let mut hit = Vec2::new(self.player.0.x, self.player.0.y);
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
            distance *= (self.player.1 - ray_angle).cos();
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
    }

    fn run(&mut self) -> Result<()> {
        loop {
            while poll(Duration::from_secs(0))? {
                let event = read()?;
                if let Key(key_event) = event {
                    self.on_key_event(key_event);
                }
            }
            if self.should_stop {
                break;
            }
            self.render();
            self.window.draw()?;
        }
        Ok(())
    }
}

fn main() -> Result<()> {
    let mut raycasting = Raycasting::new()?;
    raycasting.run()?;
    Ok(())
}
