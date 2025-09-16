//! Convenience methods for HSES client

use crate::types::{ClientError, HsesClient};

impl HsesClient {
    /// Read a 16-bit integer variable (I variable)
    ///
    /// # Errors
    ///
    /// Returns an error if communication fails
    pub async fn read_i16(&self, index: u8) -> Result<i16, ClientError> {
        self.read_variable::<i16>(index).await
    }

    /// Write a 16-bit integer variable (I variable)
    ///
    /// # Errors
    ///
    /// Returns an error if communication fails
    pub async fn write_i16(&self, index: u8, value: i16) -> Result<(), ClientError> {
        self.write_variable(index, value).await
    }

    /// Read a 32-bit integer variable (D variable)
    ///
    /// # Errors
    ///
    /// Returns an error if communication fails
    pub async fn read_i32(&self, index: u8) -> Result<i32, ClientError> {
        self.read_variable::<i32>(index).await
    }

    /// Write a 32-bit integer variable (D variable)
    ///
    /// # Errors
    ///
    /// Returns an error if communication fails
    pub async fn write_i32(&self, index: u8, value: i32) -> Result<(), ClientError> {
        self.write_variable(index, value).await
    }

    /// Read a 32-bit float variable (R variable)
    ///
    /// # Errors
    ///
    /// Returns an error if communication fails
    pub async fn read_f32(&self, index: u8) -> Result<f32, ClientError> {
        self.read_variable::<f32>(index).await
    }

    /// Write a 32-bit float variable (R variable)
    ///
    /// # Errors
    ///
    /// Returns an error if communication fails
    pub async fn write_f32(&self, index: u8, value: f32) -> Result<(), ClientError> {
        self.write_variable(index, value).await
    }

    /// Read an 8-bit unsigned integer variable (B variable)
    ///
    /// # Errors
    ///
    /// Returns an error if communication fails
    pub async fn read_u8(&self, index: u8) -> Result<u8, ClientError> {
        self.read_variable::<u8>(index).await
    }

    /// Write an 8-bit unsigned integer variable (B variable)
    ///
    /// # Errors
    ///
    /// Returns an error if communication fails
    pub async fn write_u8(&self, index: u8, value: u8) -> Result<(), ClientError> {
        self.write_variable(index, value).await
    }

    /// Read a string variable (S variable)
    ///
    /// # Errors
    ///
    /// Returns an error if communication fails
    pub async fn read_string(&self, index: u8) -> Result<Vec<u8>, ClientError> {
        self.read_variable::<Vec<u8>>(index).await
    }

    /// Write a string variable (S variable)
    ///
    /// # Errors
    ///
    /// Returns an error if communication fails
    pub async fn write_string(&self, index: u8, value: Vec<u8>) -> Result<(), ClientError> {
        self.write_variable(index, value).await
    }
}
