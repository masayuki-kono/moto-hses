//! HSES message structures and operations

use bytes::{Buf, BufMut, BytesMut};
use crate::error::ProtocolError;

// HSES Message Structure
#[derive(Debug, Clone)]
pub struct HsesHeader {
    pub magic: [u8; 4],
    pub header_size: u16,
    pub payload_size: u16,
    pub reserved_magic: u8,
    pub division: u8,
    pub ack: u8,
    pub request_id: u8,
    pub block_number: u32,
    pub reserved: [u8; 8],
}

impl HsesHeader {
    pub fn new(division: u8, ack: u8, request_id: u8, payload_size: u16) -> Self {
        Self {
            magic: *b"YERC",
            header_size: 0x20,
            payload_size,
            reserved_magic: 0x03,
            division,
            ack,
            request_id,
            block_number: if ack == 0x01 { 0x80000000 } else { 0 },
            reserved: *b"99999999",
        }
    }

    pub fn encode(&self, dst: &mut BytesMut) {
        dst.extend_from_slice(&self.magic);
        dst.put_u16_le(self.header_size);
        dst.put_u16_le(self.payload_size);
        dst.put_u8(self.reserved_magic);
        dst.put_u8(self.division);
        dst.put_u8(self.ack);
        dst.put_u8(self.request_id);
        dst.put_u32_le(self.block_number);
        dst.extend_from_slice(&self.reserved);
    }

    pub fn decode(src: &mut &[u8]) -> Result<Self, ProtocolError> {
        if src.len() < 24 {
            return Err(ProtocolError::Underflow);
        }

        let mut buf = *src;
        let magic = [buf.get_u8(), buf.get_u8(), buf.get_u8(), buf.get_u8()];
        if magic != *b"YERC" {
            return Err(ProtocolError::InvalidHeader);
        }

        let header_size = buf.get_u16_le();
        let payload_size = buf.get_u16_le();
        let reserved_magic = buf.get_u8();
        let division = buf.get_u8();
        let ack = buf.get_u8();
        let request_id = buf.get_u8();
        let block_number = buf.get_u32_le();
        let mut reserved = [0u8; 8];
        buf.copy_to_slice(&mut reserved);

        *src = &buf[..];

        Ok(Self {
            magic,
            header_size,
            payload_size,
            reserved_magic,
            division,
            ack,
            request_id,
            block_number,
            reserved,
        })
    }
}

#[derive(Debug, Clone)]
pub struct HsesSubHeader {
    pub command: u16,
    pub instance: u16,
    pub attribute: u8,
    pub service: u8,
    pub padding: u16,
}

impl HsesSubHeader {
    pub fn new(command: u16, instance: u16, attribute: u8, service: u8) -> Self {
        Self {
            command,
            instance,
            attribute,
            service,
            padding: 0,
        }
    }

    pub fn encode(&self, dst: &mut BytesMut) {
        dst.put_u16_le(self.command);
        dst.put_u16_le(self.instance);
        dst.put_u8(self.attribute);
        dst.put_u8(self.service);
        dst.put_u16_le(self.padding);
    }

    pub fn decode(src: &mut &[u8]) -> Result<Self, ProtocolError> {
        if src.len() < 8 {
            return Err(ProtocolError::Underflow);
        }

        let mut buf = *src;
        let command = buf.get_u16_le();
        let instance = buf.get_u16_le();
        let attribute = buf.get_u8();
        let service = buf.get_u8();
        let padding = buf.get_u16_le();

        *src = &buf[..];

        Ok(Self {
            command,
            instance,
            attribute,
            service,
            padding,
        })
    }
}

#[derive(Debug, Clone)]
pub struct HsesMessage {
    pub header: HsesHeader,
    pub sub_header: HsesSubHeader,
    pub payload: Vec<u8>,
}

impl HsesMessage {
    pub fn new(division: u8, ack: u8, request_id: u8, command: u16, instance: u16, attribute: u8, service: u8, payload: Vec<u8>) -> Self {
        let header = HsesHeader::new(division, ack, request_id, payload.len() as u16);
        let sub_header = HsesSubHeader::new(command, instance, attribute, service);
        Self { header, sub_header, payload }
    }

    pub fn encode(&self) -> BytesMut {
        let mut buf = BytesMut::with_capacity(32 + self.payload.len());
        self.header.encode(&mut buf);
        self.sub_header.encode(&mut buf);
        buf.extend_from_slice(&self.payload);
        buf
    }

    pub fn decode(src: &[u8]) -> Result<Self, ProtocolError> {
        let mut buf = src;
        let header = HsesHeader::decode(&mut buf)?;
        let sub_header = HsesSubHeader::decode(&mut buf)?;
        let payload = buf.to_vec();
        
        Ok(Self { header, sub_header, payload })
    }
}
