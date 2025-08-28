//! moto-hses-proto (placeholder)

use bytes::{Buf, BufMut, BytesMut};
use thiserror::Error;

pub const DEFAULT_PORT: u16 = 12222;

#[derive(Debug, Clone, Copy)]
#[repr(u16)]
pub enum CommandId {
    ReadStatus = 0x0101,
    ReadPositions = 0x0102,
    ReadIo = 0x0103,
    WriteIo = 0x0201,
}

#[derive(Debug, Error)]
pub enum ProtoError {
    #[error("buffer underflow")]
    Underflow,
    #[error("invalid header")]
    InvalidHeader,
    #[error("unknown command 0x{0:04X}")]
    UnknownCommand(u16),
    #[error("unsupported")]
    Unsupported,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Header {
    pub seq: u16,
    pub cmd: u16,
    pub size: u16,
    pub reserved: u16,
}

impl Header {
    pub fn encode(&self, dst: &mut BytesMut) {
        dst.put_u16(self.seq);
        dst.put_u16(self.cmd);
        dst.put_u16(self.size);
        dst.put_u16(self.reserved);
    }

    pub fn decode(src: &mut &[u8]) -> Result<Self, ProtoError> {
        if src.len() < 8 { return Err(ProtoError::Underflow); }
        let mut buf = *src;
        let seq = buf.get_u16();
        let cmd = buf.get_u16();
        let size = buf.get_u16();
        let reserved = buf.get_u16();
        *src = &buf[..];
        Ok(Self { seq, cmd, size, reserved })
    }
}

#[derive(Debug, Clone)]
pub struct Request {
    pub header: Header,
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct Response {
    pub header: Header,
    pub payload: Vec<u8>,
}

impl Request {
    pub fn new(seq: u16, cmd: CommandId, payload: Vec<u8>) -> Self {
        let size = payload.len() as u16;
        Self { header: Header { seq, cmd: cmd as u16, size, reserved: 0 }, payload }
    }
    pub fn encode(&self) -> BytesMut {
        let mut buf = BytesMut::with_capacity(8 + self.payload.len());
        self.header.encode(&mut buf);
        buf.extend_from_slice(&self.payload);
        buf
    }
}

impl Response {
    pub fn decode(mut src: &[u8]) -> Result<Self, ProtoError> {
        let header = Header::decode(&mut src)?;
        let payload = src.to_vec();
        Ok(Self { header, payload })
    }
}