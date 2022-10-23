use clap::Parser;
use std::{
    collections::HashMap,
    io,
    net::{SocketAddr, ToSocketAddrs, UdpSocket},
    time::{Duration, Instant},
};

use common::sprite::Sprite;

/// raycasting-term server
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Server port
    port: u16,
}

struct Server {
    socket: UdpSocket,
    clients: HashMap<SocketAddr, Sprite>,
    next_id: u32,
}

impl Server {
    fn new<A: ToSocketAddrs>(addr: A) -> io::Result<Self> {
        let socket = UdpSocket::bind(addr)?;
        socket.set_nonblocking(true)?;
        Ok(Self {
            socket,
            clients: HashMap::new(),
            next_id: 0,
        })
    }

    fn handle_message(&mut self, buf: &[u8], addr: SocketAddr) -> std::io::Result<()> {
        let (position, angle) = bincode::deserialize(buf).unwrap();
        match self.clients.get_mut(&addr) {
            None => {
                self.clients
                    .insert(addr, Sprite::new(self.next_id, position, 0, Some(angle)));
                self.socket
                    .send_to(&bincode::serialize(&self.next_id).unwrap(), addr)?;
                // TODO: resend until id is received
                self.next_id += 1;
            }
            Some(sprite) => {
                sprite.position = position;
                sprite.angle = Some(angle);
            }
        }
        Ok(())
    }

    fn run(&mut self) -> io::Result<()> {
        let mut buf = [0; 128];
        let mut time = Instant::now();
        let duration = Duration::from_secs_f64(1.0 / 30.0);

        println!("Server running on {}", self.socket.local_addr()?);
        loop { // TODO: remove clients if their last update exceeds a time limit
            match self.socket.recv_from(&mut buf) {
                Ok((len, addr)) => self.handle_message(&buf[..len], addr),
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => Ok(()),
                Err(e) => Err(e),
            }?;
            if time.elapsed() >= duration {
                let serialized_sprites =
                    &bincode::serialize(&Vec::from_iter(self.clients.values())).unwrap();
                for addr in self.clients.keys() {
                    self.socket.send_to(serialized_sprites, addr)?;
                }
                time = Instant::now();
            }
        }
    }
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();

    let mut server = Server::new(String::from("0.0.0.0:") + &args.port.to_string())?;
    server.run()
}
