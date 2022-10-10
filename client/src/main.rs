use clap::Parser;
use crossterm::{event::KeyCode, Result};
use nalgebra::{ArrayStorage, Const, Matrix, Point2, Vector3, Vector4};
use std::io;
use std::net::{ToSocketAddrs, UdpSocket};
use std::{f64, time::Instant};
type Matrix16<T> = Matrix<T, Const<16>, Const<16>, ArrayStorage<T, 16, 16>>;
use winterm::Window;

mod player;
mod rendering;
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

pub struct Client {
    socket: UdpSocket,
    id: u32,
}

impl Client {
    fn new<A: ToSocketAddrs>(addr: A, position: &Point2<f64>) -> io::Result<Self> {
        let socket = UdpSocket::bind("0.0.0.0:0")?;
        socket.connect(addr)?;
        socket.send(&bincode::serialize(&position).unwrap())?;
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
    images: Vec<Matrix16<Vector4<u8>>>,
    z_buffer: Vec<f64>,
    should_stop: bool,
}

impl Raycasting {
    fn new(server_address: Option<&str>) -> Result<Self> {
        let height = 45;
        let width = 80;
        let mut image = Matrix16::zeros();
        fill_test_image(&mut image);
        let position = Point2::new(3.0, 4.0);
        let (client, sprites) = match server_address {
            Some(addr) => (Some(Client::new(addr, &position)?), vec![]),
            None => (
                None,
                vec![
                    Sprite::new(0, Point2::new(4.0, 6.0), 0),
                    Sprite::new(1, Point2::new(6.9, 4.0), 0),
                ],
            ),
        };
        Ok(Self {
            window: Window::new(height, width)?,
            player: Player::new(position, 180.0_f64.to_radians(), 60.0_f64.to_radians()),
            client,
            sprites,
            images: vec![image],
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
                .send(&bincode::serialize(&self.player.position).unwrap())?;
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

fn main() -> Result<()> {
    let args = Args::parse();

    let mut raycasting = Raycasting::new(args.server_address.as_deref())?;
    raycasting.run()?;
    Ok(())
}
