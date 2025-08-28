//! moto-hses-client (placeholder)

use std::net::SocketAddr;
use tokio::net::UdpSocket;
use tokio::time::{timeout, Duration};
use thiserror::Error;
use moto_hses_proto as proto;

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("protocol error: {0}")]
    Proto(#[from] proto::ProtoError),
    #[error("timeout")]
    Timeout,
}

pub struct HsesClient {
    socket: UdpSocket,
    controller: SocketAddr,
    seq: u16,
}

impl HsesClient {
    pub async fn connect(controller: SocketAddr) -> Result<Self, ClientError> {
        let local: SocketAddr = if controller.is_ipv4() { "0.0.0.0:0".parse().unwrap() } else { "[::]:0".parse().unwrap() };
        let socket = UdpSocket::bind(local).await?;
        Ok(Self { socket, controller, seq: 1 })
    }

    fn next_seq(&mut self) -> u16 { let s = self.seq; self.seq = self.seq.wrapping_add(1); s }

    pub async fn request(&mut self, cmd: proto::CommandId, payload: Vec<u8>) -> Result<proto::Response, ClientError> {
        let req = proto::Request::new(self.next_seq(), cmd, payload);
        let buf = req.encode();
        self.socket.send_to(&buf, self.controller).await?;
        let mut recv = vec![0u8; 2048];
        let n = timeout(Duration::from_millis(500), self.socket.recv(&mut recv)).await
            .map_err(|_| ClientError::Timeout)??;
        recv.truncate(n);
        let resp = proto::Response::decode(&recv)?;
        Ok(resp)
    }

    pub async fn read_status(&mut self) -> Result<Vec<u8>, ClientError> {
        let resp = self.request(proto::CommandId::ReadStatus, Vec::new()).await?;
        Ok(resp.payload)
    }
}