//! File control command handlers

use super::CommandHandler;
use crate::state::MockState;
use moto_hses_proto as proto;

/// Handler for file operations (0x00)
pub struct FileControlHandler;

impl CommandHandler for FileControlHandler {
    #[allow(clippy::too_many_lines)]
    fn handle(
        &self,
        message: &proto::HsesRequestMessage,
        state: &mut MockState,
    ) -> Result<Vec<u8>, proto::ProtocolError> {
        let service = message.sub_header.service;

        match service {
            0x01 => {
                // Get file list
                // Return a simple file list
                let file_list = "TEST.JOB\0";
                let file_list_bytes =
                    moto_hses_proto::encoding_utils::encode_string(file_list, state.text_encoding);
                Ok(file_list_bytes)
            }
            0x02 => {
                // Send file
                // Parse filename from payload
                if let Some(filename_pos) = message.payload.iter().position(|&b| b == 0) {
                    let filename = moto_hses_proto::encoding_utils::decode_string_with_fallback(
                        &message.payload[..filename_pos],
                        state.text_encoding,
                    );
                    let content = message.payload[filename_pos + 1..].to_vec();
                    state.set_file(filename, content);
                }
                Ok(vec![])
            }
            0x03 => {
                // Receive file
                // Parse filename from payload
                if let Some(filename_pos) = message.payload.iter().position(|&b| b == 0) {
                    let filename = moto_hses_proto::encoding_utils::decode_string_with_fallback(
                        &message.payload[..filename_pos],
                        state.text_encoding,
                    );
                    if let Some(content) = state.get_file(&filename) {
                        let mut response = moto_hses_proto::encoding_utils::encode_string(
                            &filename,
                            state.text_encoding,
                        );
                        response.push(0);
                        response.extend(content);
                        return Ok(response);
                    }
                }
                Ok(vec![])
            }
            0x04 => {
                // Delete file
                // Parse filename from payload
                if let Some(filename_pos) = message.payload.iter().position(|&b| b == 0) {
                    let filename = moto_hses_proto::encoding_utils::decode_string_with_fallback(
                        &message.payload[..filename_pos],
                        state.text_encoding,
                    );
                    state.delete_file(&filename);
                }
                Ok(vec![])
            }
            0x15 => {
                // Send file (Python client uses this)
                // Parse filename from payload
                if let Some(filename_pos) = message.payload.iter().position(|&b| b == 0) {
                    let filename = moto_hses_proto::encoding_utils::decode_string_with_fallback(
                        &message.payload[..filename_pos],
                        state.text_encoding,
                    );
                    let content = message.payload[filename_pos + 1..].to_vec();
                    let filename_clone = filename.clone();
                    let content_len = content.len();
                    state.set_file(filename, content);
                    debug!("File saved: {filename_clone} ({content_len} bytes)");
                }
                Ok(vec![])
            }
            0x32 => {
                // Get file list (Python client uses this)
                // Parse pattern from payload
                let pattern = if message.payload.is_empty() {
                    "*".to_string()
                } else {
                    moto_hses_proto::encoding_utils::decode_string_with_fallback(
                        &message.payload,
                        state.text_encoding,
                    )
                };

                let files = state.get_file_list(&pattern);
                let mut file_list = String::new();
                for file in files {
                    file_list.push_str(&file);
                    file_list.push('\0');
                }
                debug!("File list requested with pattern '{}', returning: {file_list:?}", pattern);
                let file_list_bytes =
                    moto_hses_proto::encoding_utils::encode_string(&file_list, state.text_encoding);
                Ok(file_list_bytes)
            }
            0x16 => {
                // Receive file (Python client uses this)
                // Parse filename from payload
                if let Some(filename_pos) = message.payload.iter().position(|&b| b == 0) {
                    let filename = moto_hses_proto::encoding_utils::decode_string_with_fallback(
                        &message.payload[..filename_pos],
                        state.text_encoding,
                    );
                    if let Some(content) = state.get_file(&filename) {
                        let mut response = moto_hses_proto::encoding_utils::encode_string(
                            &filename,
                            state.text_encoding,
                        );
                        response.push(0);
                        response.extend(content);
                        debug!("File requested: {} ({} bytes)", filename, content.len());
                        return Ok(response);
                    }
                    debug!("File not found: {filename}");
                }
                Ok(vec![])
            }
            0x09 => {
                // Delete file (Python client uses this)
                // Parse filename from payload
                if let Some(filename_pos) = message.payload.iter().position(|&b| b == 0) {
                    let filename = moto_hses_proto::encoding_utils::decode_string_with_fallback(
                        &message.payload[..filename_pos],
                        state.text_encoding,
                    );
                    let deleted = state.delete_file(&filename);
                    debug!("File deletion requested: {filename} (deleted: {deleted})");
                }
                Ok(vec![])
            }
            _ => Err(proto::ProtocolError::InvalidService),
        }
    }
}
