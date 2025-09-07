//! Convenience methods for HSES client

use crate::types::{ClientError, HsesClient};

impl HsesClient {
    /// Read an integer variable
    pub async fn read_int(&self, index: u8) -> Result<i32, ClientError> {
        self.read_variable::<i32>(index).await
    }

    /// Write an integer variable
    pub async fn write_int(&self, index: u8, value: i32) -> Result<(), ClientError> {
        self.write_variable(index, value).await
    }

    /// Read a float variable
    pub async fn read_float(&self, index: u8) -> Result<f32, ClientError> {
        self.read_variable::<f32>(index).await
    }

    /// Write a float variable
    pub async fn write_float(&self, index: u8, value: f32) -> Result<(), ClientError> {
        self.write_variable(index, value).await
    }

    /// Read a byte variable
    pub async fn read_byte(&self, index: u8) -> Result<u8, ClientError> {
        self.read_variable::<u8>(index).await
    }

    /// Write a byte variable
    pub async fn write_byte(&self, index: u8, value: u8) -> Result<(), ClientError> {
        self.write_variable(index, value).await
    }
}
