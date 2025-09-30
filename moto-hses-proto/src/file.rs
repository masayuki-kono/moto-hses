//! File control commands for HSES protocol

use crate::commands::Command;
use crate::error::ProtocolError;

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
    pub encoding: crate::encoding::TextEncoding,
}

impl SendFile {
    #[must_use]
    pub const fn new(
        filename: String,
        content: Vec<u8>,
        encoding: crate::encoding::TextEncoding,
    ) -> Self {
        Self { filename, content, encoding }
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
        let filename_bytes = crate::encoding_utils::encode_string(&self.filename, self.encoding);
        let mut payload = filename_bytes;
        payload.push(0); // Null terminator
        payload.extend_from_slice(&self.content);
        Ok(payload)
    }
}

/// Receive file command
#[derive(Debug, Clone)]
pub struct ReceiveFile {
    pub filename: String,
    pub encoding: crate::encoding::TextEncoding,
}

impl ReceiveFile {
    #[must_use]
    pub const fn new(filename: String, encoding: crate::encoding::TextEncoding) -> Self {
        Self { filename, encoding }
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
        let filename_bytes = crate::encoding_utils::encode_string(&self.filename, self.encoding);
        let mut payload = filename_bytes;
        payload.push(0); // Null terminator
        Ok(payload)
    }
}

/// Delete file command
#[derive(Debug, Clone)]
pub struct DeleteFile {
    pub filename: String,
    pub encoding: crate::encoding::TextEncoding,
}

impl DeleteFile {
    #[must_use]
    pub const fn new(filename: String, encoding: crate::encoding::TextEncoding) -> Self {
        Self { filename, encoding }
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
        let filename_bytes = crate::encoding_utils::encode_string(&self.filename, self.encoding);
        let mut payload = filename_bytes;
        payload.push(0); // Null terminator
        Ok(payload)
    }
}

/// File operation response parsers
pub mod response {
    use super::ProtocolError;

    /// Parse file list response with specified text encoding
    ///
    /// # Errors
    ///
    /// Returns an error if parsing fails
    pub fn parse_file_list(
        data: &[u8],
        encoding: crate::encoding::TextEncoding,
    ) -> Result<Vec<String>, ProtocolError> {
        let content = crate::encoding_utils::decode_string_with_fallback(data, encoding);
        let files: Vec<String> = content
            .split('\0')
            .filter(|s| !s.is_empty())
            .map(std::string::ToString::to_string)
            .collect();
        Ok(files)
    }

    /// Parse file content response
    ///
    /// # Errors
    ///
    /// Returns an error if parsing fails
    pub fn parse_file_content(data: &[u8]) -> Result<Vec<u8>, ProtocolError> {
        // Extract file content from response
        // Response format: filename\0content
        data.iter().position(|&b| b == 0).map_or_else(
            || Ok(data.to_vec()),
            |null_pos| {
                let content = data[null_pos + 1..].to_vec();
                Ok(content)
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_read_file_list_serialization() {
        let cmd = ReadFileList;
        let data = cmd.serialize().unwrap();
        assert_eq!(data, vec![]);
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_read_file_list_deserialization() {
        let data = b"file1.job\0file2.job\0";
        let files = response::parse_file_list(data, crate::encoding::TextEncoding::Utf8).unwrap();
        assert_eq!(files, vec!["file1.job", "file2.job"]);
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_send_file_serialization() {
        let cmd = SendFile::new(
            "test.job".to_string(),
            b"content".to_vec(),
            crate::encoding::TextEncoding::Utf8,
        );
        let data = cmd.serialize().unwrap();
        let expected = b"test.job\0content".to_vec();
        assert_eq!(data, expected);
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_receive_file_serialization() {
        let cmd = ReceiveFile::new("test.job".to_string(), crate::encoding::TextEncoding::Utf8);
        let data = cmd.serialize().unwrap();
        let expected = b"test.job\0".to_vec();
        assert_eq!(data, expected);
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_receive_file_deserialization() {
        let data = b"test.job\0file content";
        let content = response::parse_file_content(data).unwrap();
        assert_eq!(content, b"file content".to_vec());
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_delete_file_serialization() {
        let cmd = DeleteFile::new("test.job".to_string(), crate::encoding::TextEncoding::Utf8);
        let data = cmd.serialize().unwrap();
        let expected = b"test.job\0".to_vec();
        assert_eq!(data, expected);
    }
}
