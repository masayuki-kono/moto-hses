//! Convenience methods for HSES client

use crate::types::{ClientError, HsesClient};

impl HsesClient {
    /// Read an 8-bit unsigned integer variable (B variable)
    ///
    /// # Errors
    ///
    /// Returns an error if communication fails
    pub async fn read_u8(&self, index: u16) -> Result<u8, ClientError> {
        self.read_variable::<u8>(index).await
    }

    /// Write an 8-bit unsigned integer variable (B variable)
    ///
    /// # Errors
    ///
    /// Returns an error if communication fails
    pub async fn write_u8(&self, index: u16, value: u8) -> Result<(), ClientError> {
        self.write_variable(index, value).await
    }

    /// Read a 16-bit integer variable (I variable)
    ///
    /// # Errors
    ///
    /// Returns an error if communication fails
    pub async fn read_i16(&self, index: u16) -> Result<i16, ClientError> {
        self.read_variable::<i16>(index).await
    }

    /// Write a 16-bit integer variable (I variable)
    ///
    /// # Errors
    ///
    /// Returns an error if communication fails
    pub async fn write_i16(&self, index: u16, value: i16) -> Result<(), ClientError> {
        self.write_variable(index, value).await
    }

    /// Read a 32-bit integer variable (D variable)
    ///
    /// # Errors
    ///
    /// Returns an error if communication fails
    pub async fn read_i32(&self, index: u16) -> Result<i32, ClientError> {
        self.read_variable::<i32>(index).await
    }

    /// Write a 32-bit integer variable (D variable)
    ///
    /// # Errors
    ///
    /// Returns an error if communication fails
    pub async fn write_i32(&self, index: u16, value: i32) -> Result<(), ClientError> {
        self.write_variable(index, value).await
    }

    /// Read a 32-bit float variable (R variable)
    ///
    /// # Errors
    ///
    /// Returns an error if communication fails
    pub async fn read_f32(&self, index: u16) -> Result<f32, ClientError> {
        self.read_variable::<f32>(index).await
    }

    /// Write a 32-bit float variable (R variable)
    ///
    /// # Errors
    ///
    /// Returns an error if communication fails
    pub async fn write_f32(&self, index: u16, value: f32) -> Result<(), ClientError> {
        self.write_variable(index, value).await
    }

    /// Read a string variable (S variable) with encoding support
    ///
    /// Uses `read_variable<String>` which handles encoding/decoding in the proto layer.
    ///
    /// # Errors
    ///
    /// Returns an error if communication fails
    pub async fn read_string(&self, index: u16) -> Result<String, ClientError> {
        self.read_variable::<String>(index).await
    }

    /// Write a string variable (S variable) with encoding support
    ///
    /// Uses `write_string_variable` which handles encoding with client's text encoding setting.
    ///
    /// # Errors
    ///
    /// Returns an error if communication fails or if string exceeds 16 bytes when encoded
    pub async fn write_string(&self, index: u16, value: String) -> Result<(), ClientError> {
        self.write_string_variable(index, value).await
    }

    /// Read multiple u8 variables (B)
    ///
    /// # Errors
    /// Returns an error if communication fails or parameters are invalid
    pub async fn read_multiple_u8(
        &self,
        start_variable_number: u16,
        count: u32,
    ) -> Result<Vec<u8>, ClientError> {
        self.read_multiple_variables::<u8>(start_variable_number, count).await
    }

    /// Write multiple u8 variables (B)
    ///
    /// # Errors
    /// Returns an error if communication fails or parameters are invalid
    pub async fn write_multiple_u8(
        &self,
        start_variable_number: u16,
        values: Vec<u8>,
    ) -> Result<(), ClientError> {
        self.write_multiple_variables(start_variable_number, values).await
    }

    /// Read multiple i16 variables (I)
    ///
    /// # Errors
    /// Returns an error if communication fails or parameters are invalid
    pub async fn read_multiple_i16(
        &self,
        start_variable_number: u16,
        count: u32,
    ) -> Result<Vec<i16>, ClientError> {
        self.read_multiple_variables::<i16>(start_variable_number, count).await
    }

    /// Write multiple i16 variables (I)
    ///
    /// # Errors
    /// Returns an error if communication fails or parameters are invalid
    pub async fn write_multiple_i16(
        &self,
        start_variable_number: u16,
        values: Vec<i16>,
    ) -> Result<(), ClientError> {
        self.write_multiple_variables(start_variable_number, values).await
    }

    /// Read multiple i32 variables (D)
    ///
    /// # Errors
    /// Returns an error if communication fails or parameters are invalid
    pub async fn read_multiple_i32(
        &self,
        start_variable_number: u16,
        count: u32,
    ) -> Result<Vec<i32>, ClientError> {
        self.read_multiple_variables::<i32>(start_variable_number, count).await
    }

    /// Write multiple i32 variables (D)
    ///
    /// # Errors
    /// Returns an error if communication fails or parameters are invalid
    pub async fn write_multiple_i32(
        &self,
        start_variable_number: u16,
        values: Vec<i32>,
    ) -> Result<(), ClientError> {
        self.write_multiple_variables(start_variable_number, values).await
    }

    /// Read multiple f32 variables (R)
    ///
    /// # Errors
    /// Returns an error if communication fails or parameters are invalid
    pub async fn read_multiple_f32(
        &self,
        start_variable_number: u16,
        count: u32,
    ) -> Result<Vec<f32>, ClientError> {
        self.read_multiple_variables::<f32>(start_variable_number, count).await
    }

    /// Write multiple f32 variables (R)
    ///
    /// # Errors
    /// Returns an error if communication fails or parameters are invalid
    pub async fn write_multiple_f32(
        &self,
        start_variable_number: u16,
        values: Vec<f32>,
    ) -> Result<(), ClientError> {
        self.write_multiple_variables(start_variable_number, values).await
    }

    /// Read multiple string variables (S) with encoding support
    ///
    /// Uses `read_multiple_variables<String>` which handles encoding/decoding in the proto layer.
    ///
    /// # Errors
    /// Returns an error if communication fails
    pub async fn read_multiple_strings(
        &self,
        start_variable_number: u16,
        count: u32,
    ) -> Result<Vec<String>, ClientError> {
        self.read_multiple_variables::<String>(start_variable_number, count).await
    }

    /// Write multiple string variables (S) with encoding support
    ///
    /// Uses `write_multiple_string_variables` which handles encoding with client's text encoding setting.
    ///
    /// # Errors
    /// Returns an error if communication fails or if any string exceeds 16 bytes when encoded
    pub async fn write_multiple_strings(
        &self,
        start_variable_number: u16,
        values: Vec<String>,
    ) -> Result<(), ClientError> {
        self.write_multiple_string_variables(start_variable_number, values).await
    }
}
