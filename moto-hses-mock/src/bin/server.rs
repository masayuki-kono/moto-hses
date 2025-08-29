//! Simple mock HSES UDP server (placeholder).
//! Usage: cargo run -p moto-hses-mock -- [addr:port]
//! Default: 127.0.0.1:12222

use std::net::SocketAddr;
use tokio::net::UdpSocket;
use moto_hses_proto as proto;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let bind: SocketAddr = std::env::args().nth(1).unwrap_or_else(|| "127.0.0.1:12222".to_string()).parse()?;
    let sock = UdpSocket::bind(bind).await?;
    eprintln!("moto-hses-mock listening on {}", bind);

    let mut buf = vec![0u8; 2048];
    loop {
        let (n, src) = sock.recv_from(&mut buf).await?;
        if n < 32 { continue; }

        // Parse HSES message
        let message = match proto::HsesMessage::decode(&buf[..n]) {
            Ok(msg) => msg,
            Err(_) => continue,
        };
        
        let command = message.sub_header.command;
        
        // Build response header
        let _response_header = proto::HsesHeader::new(
            message.header.division,
            0x01, // ACK
            message.header.request_id,
            0
        );
        
        // Craft a canned payload by command
        let payload: Vec<u8> = match command {
            0x72 => vec![0x01, 0x00, 0x40, 0x00], // ReadStatus - running and servo on
            0x75 => vec![0u8; 52], // ReadCurrentPosition - placeholder
            0x7a => vec![0x01, 0x00, 0x00, 0x00], // ReadVar<u8> - value 1
            0x7b => vec![0x64, 0x00, 0x00, 0x00], // ReadVar<i32> - value 100
            0x7d => vec![0x00, 0x00, 0x20, 0x41], // ReadVar<f32> - value 10.0
            0x7f => vec![0u8; 52], // ReadVar<Position> - placeholder
            _ => vec![],
        };
        
        // Create response message
        let response_message = proto::HsesMessage::new(
            message.header.division,
            0x01, // ACK
            message.header.request_id,
            command,
            message.sub_header.instance,
            message.sub_header.attribute,
            message.sub_header.service,
            payload
        );
        
        let response_data = response_message.encode();
        let _ = sock.send_to(&response_data, src).await?;
    }
}