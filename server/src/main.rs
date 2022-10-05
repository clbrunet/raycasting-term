use std::net::UdpSocket;

use clap::Parser;
use nalgebra::Point2;

/// raycasting-term server
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Server port
    port: u16,
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();

    let server_addr = String::from("127.0.0.1:") + &args.port.to_string();
    let socket = UdpSocket::bind(&server_addr)?;
    eprintln!("Server running on {}", server_addr);
    let mut buf = [0; 128];
    loop {
        let (len, addr) = socket.recv_from(&mut buf)?;
        dbg!(len);
        dbg!(addr);
        dbg!(&buf[..len]);
        let ttt: Point2<f64> = bincode::deserialize(&buf[..len]).unwrap();
        dbg!(ttt);
    }
}
