//! File control commands for HSES protocol

use crate::error::ProtocolError;
use crate::types::Command;

/// File list request command
#[derive(Debug, Clone)]
pub struct ReadFileList;

impl Command for ReadFileList {
    type Response = Vec<String>;

    fn command_id() -> u16 {
        0x0000
    }

    fn instance(&self) -> u16 {
        0
    }

    fn attribute(&self) -> u8 {
        0
    }

    fn service(&self) -> u8 {
        0x32
    }

    fn serialize(&self) -> Result<Vec<u8>, ProtocolError> {
        Ok(vec![])
    }
}

/// Send file command
#[derive(Debug, Clone)]
pub struct SendFile {
    pub filename: String,
    pub content: Vec<u8>,
}

impl SendFile {
    pub fn new(filename: String, content: Vec<u8>) -> Self {
        Self { filename, content }
    }
}

impl Command for SendFile {
    type Response = ();

    fn command_id() -> u16 {
        0x0000
    }

    fn instance(&self) -> u16 {
        0
    }

    fn attribute(&self) -> u8 {
        0
    }

    fn service(&self) -> u8 {
        0x15
    }

    fn serialize(&self) -> Result<Vec<u8>, ProtocolError> {
        let mut payload = self.filename.as_bytes().to_vec();
        payload.push(0); // Null terminator
        payload.extend_from_slice(&self.content);
        Ok(payload)
    }
}

/// Receive file command
#[derive(Debug, Clone)]
pub struct ReceiveFile {
    pub filename: String,
}

impl ReceiveFile {
    pub fn new(filename: String) -> Self {
        Self { filename }
    }
}

impl Command for ReceiveFile {
    type Response = Vec<u8>;

    fn command_id() -> u16 {
        0x0000
    }

    fn instance(&self) -> u16 {
        0
    }

    fn attribute(&self) -> u8 {
        0
    }

    fn service(&self) -> u8 {
        0x16
    }

    fn serialize(&self) -> Result<Vec<u8>, ProtocolError> {
        let mut payload = self.filename.as_bytes().to_vec();
        payload.push(0); // Null terminator
        Ok(payload)
    }
}

/// Delete file command
#[derive(Debug, Clone)]
pub struct DeleteFile {
    pub filename: String,
}

impl DeleteFile {
    pub fn new(filename: String) -> Self {
        Self { filename }
    }
}

impl Command for DeleteFile {
    type Response = ();

    fn command_id() -> u16 {
        0x0000
    }

    fn instance(&self) -> u16 {
        0
    }

    fn attribute(&self) -> u8 {
        0
    }

    fn service(&self) -> u8 {
        0x09
    }

    fn serialize(&self) -> Result<Vec<u8>, ProtocolError> {
        let mut payload = self.filename.as_bytes().to_vec();
        payload.push(0); // Null terminator
        Ok(payload)
    }
}

/// File operation response parsers
pub mod response {
    use super::*;

    /// Parse file list response
    pub fn parse_file_list(data: &[u8]) -> Result<Vec<String>, ProtocolError> {
        let content = String::from_utf8_lossy(data);
        let files: Vec<String> =
            content.split('\0').filter(|s| !s.is_empty()).map(|s| s.to_string()).collect();
        Ok(files)
    }

    /// Parse file content response
    pub fn parse_file_content(data: &[u8]) -> Result<Vec<u8>, ProtocolError> {
        // Extract file content from response
        // Response format: filename\0content
        if let Some(null_pos) = data.iter().position(|&b| b == 0) {
            let content = data[null_pos + 1..].to_vec();
            Ok(content)
        } else {
            Ok(data.to_vec())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_file_list_serialization() {
        let cmd = ReadFileList;
        let data = cmd.serialize().unwrap();
        assert_eq!(data, vec![]);
    }

    #[test]
    fn test_read_file_list_deserialization() {
        let data = b"file1.job\0file2.job\0";
        let files = response::parse_file_list(data).unwrap();
        assert_eq!(files, vec!["file1.job", "file2.job"]);
    }

    #[test]
    fn test_send_file_serialization() {
        let cmd = SendFile::new("test.job".to_string(), b"content".to_vec());
        let data = cmd.serialize().unwrap();
        let expected = b"test.job\0content".to_vec();
        assert_eq!(data, expected);
    }

    #[test]
    fn test_receive_file_serialization() {
        let cmd = ReceiveFile::new("test.job".to_string());
        let data = cmd.serialize().unwrap();
        let expected = b"test.job\0".to_vec();
        assert_eq!(data, expected);
    }

    #[test]
    fn test_receive_file_deserialization() {
        let data = b"test.job\0file content";
        let content = response::parse_file_content(data).unwrap();
        assert_eq!(content, b"file content".to_vec());
    }

    #[test]
    fn test_delete_file_serialization() {
        let cmd = DeleteFile::new("test.job".to_string());
        let data = cmd.serialize().unwrap();
        let expected = b"test.job\0".to_vec();
        assert_eq!(data, expected);
    }
}
