//! File control command handlers

use super::CommandHandler;
use crate::state::MockState;
use moto_hses_proto as proto;

/// Handler for file operations (0x00)
pub struct FileControlHandler;

impl CommandHandler for FileControlHandler {
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
                Ok(file_list.as_bytes().to_vec())
            }
            0x02 => {
                // Send file
                // Parse filename from payload
                if let Some(filename_pos) = message.payload.iter().position(|&b| b == 0) {
                    let filename =
                        String::from_utf8_lossy(&message.payload[..filename_pos]).to_string();
                    let content = message.payload[filename_pos + 1..].to_vec();
                    state.set_file(filename, content);
                }
                Ok(vec![])
            }
            0x03 => {
                // Receive file
                // Parse filename from payload
                if let Some(filename_pos) = message.payload.iter().position(|&b| b == 0) {
                    let filename =
                        String::from_utf8_lossy(&message.payload[..filename_pos]).to_string();
                    if let Some(content) = state.get_file(&filename) {
                        let mut response = filename.as_bytes().to_vec();
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
                    let filename =
                        String::from_utf8_lossy(&message.payload[..filename_pos]).to_string();
                    state.delete_file(&filename);
                }
                Ok(vec![])
            }
            0x15 => {
                // Send file (Python client uses this)
                // Parse filename from payload
                if let Some(filename_pos) = message.payload.iter().position(|&b| b == 0) {
                    let filename =
                        String::from_utf8_lossy(&message.payload[..filename_pos]).to_string();
                    let content = message.payload[filename_pos + 1..].to_vec();
                    let filename_clone = filename.clone();
                    let content_len = content.len();
                    state.set_file(filename, content);
                    eprintln!("File saved: {} ({} bytes)", filename_clone, content_len);
                }
                Ok(vec![])
            }
            0x32 => {
                // Get file list (Python client uses this)
                // Return actual file list from state
                let files = state.get_file_list("*");
                let mut file_list = String::new();
                for file in files {
                    file_list.push_str(&file);
                    file_list.push('\0');
                }
                eprintln!("File list requested, returning: {:?}", file_list);
                Ok(file_list.as_bytes().to_vec())
            }
            0x16 => {
                // Receive file (Python client uses this)
                // Parse filename from payload
                if let Some(filename_pos) = message.payload.iter().position(|&b| b == 0) {
                    let filename =
                        String::from_utf8_lossy(&message.payload[..filename_pos]).to_string();
                    if let Some(content) = state.get_file(&filename) {
                        let mut response = filename.as_bytes().to_vec();
                        response.push(0);
                        response.extend(content);
                        eprintln!("File requested: {} ({} bytes)", filename, content.len());
                        return Ok(response);
                    } else {
                        eprintln!("File not found: {}", filename);
                    }
                }
                Ok(vec![])
            }
            0x09 => {
                // Delete file (Python client uses this)
                // Parse filename from payload
                if let Some(filename_pos) = message.payload.iter().position(|&b| b == 0) {
                    let filename =
                        String::from_utf8_lossy(&message.payload[..filename_pos]).to_string();
                    let deleted = state.delete_file(&filename);
                    eprintln!("File deletion requested: {} (deleted: {})", filename, deleted);
                }
                Ok(vec![])
            }
            _ => Err(proto::ProtocolError::InvalidService),
        }
    }
}
