//! Simple mock HSES UDP server (placeholder).
//! Usage: cargo run -p moto-hses-mock -- [addr:port]
//! Default: 127.0.0.1:12222

use std::net::SocketAddr;
use tokio::net::UdpSocket;
use moto_hses_proto as proto;
use bytes::{BytesMut, BufMut};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let bind: SocketAddr = std::env::args().nth(1).unwrap_or_else(|| "127.0.0.1:12222".to_string()).parse()?;
    let sock = UdpSocket::bind(bind).await?;
    eprintln!("moto-hses-mock listening on {}", bind);

    let mut buf = vec![0u8; 2048];
    loop {
        let (n, src) = sock.recv_from(&mut buf).await?;
        if n < 8 { continue; }

        let mut s = &buf[..n];
        // Parse placeholder header
        let hdr = match proto::Header::decode(&mut s) {
            Ok(h) => h,
            Err(_) => continue,
        };
        let cmd = hdr.cmd;
        // Build response
        let mut out = BytesMut::with_capacity(8 + 64);
        let mut rh = proto::Header { seq: hdr.seq, cmd, size: 0, reserved: 0 };
        // Craft a canned payload by command
        let payload: Vec<u8> = match cmd {
            x if x == proto::CommandId::ReadStatus as u16 => vec![0xAA, 0x55],
            x if x == proto::CommandId::ReadPositions as u16 => vec![0u8; 24], // placeholder
            x if x == proto::CommandId::ReadIo as u16 => vec![0x01], // e.g., ON
            _ => vec![],
        };
        rh.size = payload.len() as u16;
        rh.encode(&mut out);
        out.put_slice(&payload);

        let _ = sock.send_to(&out, src).await?;
    }
}