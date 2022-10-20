use clap::Parser;
use crossterm::{event::KeyCode, Result};
use nalgebra::Point2;
use sprite_sheet::SpriteSheet;
use std::io::{self, Cursor};
use std::net::{ToSocketAddrs, UdpSocket};
use std::{f64, time::Instant};
use winterm::Window;

mod player;
mod rendering;
mod sprite_sheet;
mod window_sprite;

use common::{get_normalized_radians_angle, sprite::Sprite};
use player::Player;
use rendering::render;

/// raycasting-term client
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Server address (eg. "127.0.0.1:4242")
    server_address: Option<String>,
}

pub struct Client {
    socket: UdpSocket,
    id: u32,
}

impl Client {
    fn new<A: ToSocketAddrs>(addr: A, position: &Point2<f64>, angle: f64) -> io::Result<Self> {
        let socket = UdpSocket::bind("0.0.0.0:0")?;
        socket.connect(addr)?;
        socket.send(&bincode::serialize(&(position, angle)).unwrap())?;
        // TODO: resend until id is received
        let mut buf = [0; 128];
        let len = socket.recv(&mut buf)?;
        let id = bincode::deserialize(&buf[..len]).unwrap();
        socket.set_nonblocking(true)?;
        Ok(Self { socket, id })
    }
}

pub struct Raycasting {
    window: Window,
    client: Option<Client>,
    player: Player,
    sprites: Vec<Sprite>,
    sprite_sheets: Vec<SpriteSheet>,
    z_buffer: Vec<f64>,
    should_stop: bool,
}

impl Raycasting {
    fn new(server_address: Option<&str>) -> Result<Self> {
        let height = 45;
        let width = 80;
        let position = Point2::new(3.0, 4.0);
        let angle = 180.0_f64.to_radians();
        let (client, sprites) = match server_address {
            Some(addr) => (Some(Client::new(addr, &position, angle)?), vec![]),
            None => (
                None,
                vec![
                    Sprite::new(0, Point2::new(4.0, 6.0), 0, Some(90.0_f64.to_radians())),
                    Sprite::new(1, Point2::new(6.9, 4.0), 0, None),
                ],
            ),
        };
        Ok(Self {
            window: Window::new(height, width)?,
            player: Player::new(position, angle, 60.0_f64.to_radians()),
            client,
            sprites,
            sprite_sheets: vec![SpriteSheet::new(Cursor::new(include_bytes!(
                "../assets/penguin.png"
            )))],
            z_buffer: vec![0.0; width.into()],
            should_stop: false,
        })
    }

    fn instantaneous_update(&mut self) -> Result<()> {
        if self.window.get_key(KeyCode::Esc) {
            self.should_stop = true;
        }
        if let Some(client) = &self.client {
            let mut buf = [0; 4096];
            match client.socket.recv(&mut buf) {
                Ok(len) => {
                    self.sprites = bincode::deserialize(&buf[..len]).unwrap();
                    Ok(())
                }
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => Ok(()),
                Err(e) => Err(e),
            }?;
            client
                .socket
                .send(&bincode::serialize(&(&self.player.position, self.player.angle)).unwrap())?;
        }
        Ok(())
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
                -(self.player.angle + f64::consts::FRAC_PI_2).sin()
                    * TRANSLATION_SPEED
                    * delta_time,
            );
        }
        if self.window.get_key(KeyCode::Char('d')) {
            self.player.translate(
                -(self.player.angle + f64::consts::FRAC_PI_2).cos()
                    * TRANSLATION_SPEED
                    * delta_time,
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

    fn run(&mut self) -> Result<()> {
        let start_time = Instant::now();
        let mut consumned_seconds = 0.0;
        let max_elapsed_time = 0.03;
        while !self.should_stop {
            self.window.poll_events()?;
            self.instantaneous_update()?;
            let elapsed_time = start_time.elapsed().as_secs_f64() - consumned_seconds;
            consumned_seconds += elapsed_time;
            let delta_time = if elapsed_time < max_elapsed_time {
                elapsed_time
            } else {
                max_elapsed_time
            };
            self.continuous_update(delta_time);
            render(self)?;
        }
        Ok(())
    }
}

fn raycasting(args: Args) -> Result<()> {
    let mut raycasting = Raycasting::new(args.server_address.as_deref())?;
    raycasting.run()?;
    Ok(())
}

fn main() -> Result<()> {
    let args = Args::parse();
    raycasting(args)
}
