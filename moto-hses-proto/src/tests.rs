//! Tests for HSES protocol

use bytes::BytesMut;
use crate::message::HsesHeader;
use crate::position::{Position, PulsePosition};
use crate::status::Status;
use crate::types::VariableType;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hses_header_creation() {
        let header = HsesHeader::new(1, 0, 1, 0);
        assert_eq!(header.magic, *b"YERC");
        assert_eq!(header.header_size, 0x20);
        assert_eq!(header.division, 1);
        assert_eq!(header.ack, 0);
        assert_eq!(header.request_id, 1);
    }

    #[test]
    fn test_hses_header_encode_decode() {
        let header = HsesHeader::new(1, 0, 1, 0);
        let mut buf = BytesMut::new();
        header.encode(&mut buf);
        
        let mut data = &buf[..];
        let decoded = HsesHeader::decode(&mut data).unwrap();
        
        assert_eq!(header.magic, decoded.magic);
        assert_eq!(header.header_size, decoded.header_size);
        assert_eq!(header.division, decoded.division);
        assert_eq!(header.ack, decoded.ack);
        assert_eq!(header.request_id, decoded.request_id);
    }

    #[test]
    fn test_status_from_bytes() {
        let data = vec![0x01, 0x00, 0x40, 0x00];
        let status = Status::from_bytes(&data).unwrap();
        assert!(status.step);
        assert!(status.servo_on);
        assert!(!status.running);
    }

    #[test]
    fn test_position_serialization() {
        let position = Position::Pulse(PulsePosition::new([1000, 2000, 3000, 0, 0, 0, 0, 0], 1));
        let serialized = position.serialize().unwrap();
        let deserialized = Position::deserialize(&serialized).unwrap();
        assert_eq!(position, deserialized);
    }

    #[test]
    fn test_variable_type_serialization() {
        let value: u8 = 42;
        let serialized = value.serialize().unwrap();
        let deserialized = u8::deserialize(&serialized).unwrap();
        assert_eq!(value, deserialized);
    }
}
