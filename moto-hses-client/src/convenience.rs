//! Convenience methods for HSES client

use crate::types::{ClientError, HsesClient};

impl HsesClient {
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

    /// Read a string variable (S variable) with encoding support
    ///
    /// Reads raw byte array using `read_variable<Vec<u8>>` and converts it to string
    /// using the client's text encoding configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if communication fails
    pub async fn read_string(&self, index: u16) -> Result<String, ClientError> {
        // Read raw byte array from protocol layer
        let byte_array = self.read_variable::<[u8; 16]>(index).await?;

        // Find null terminator
        let trimmed_bytes = byte_array
            .iter()
            .position(|&b| b == 0)
            .map_or(&byte_array[..], |pos| &byte_array[..pos]);

        // Decode using client's text encoding
        let string = moto_hses_proto::encoding_utils::decode_string_with_fallback(
            trimmed_bytes,
            self.config.text_encoding,
        );

        Ok(string)
    }

    /// Write a string variable (S variable) with encoding support
    ///
    /// Converts string to raw byte array using the client's text encoding configuration,
    /// then writes it using `write_variable<[u8; 16]>`.
    ///
    /// # Errors
    ///
    /// Returns an error if communication fails or if string exceeds 16 bytes when encoded
    pub async fn write_string(&self, index: u16, value: String) -> Result<(), ClientError> {
        // Convert string to byte array with encoding
        let encoded_bytes =
            moto_hses_proto::encoding_utils::encode_string(&value, self.config.text_encoding);

        if encoded_bytes.len() > 16 {
            return Err(ClientError::SystemError(format!(
                "String exceeds 16 bytes when encoded: {} bytes",
                encoded_bytes.len()
            )));
        }

        // Pad to 16 bytes with null terminator
        let mut byte_array = [0u8; 16];
        byte_array[..encoded_bytes.len()].copy_from_slice(&encoded_bytes);

        // Write raw byte array using protocol layer
        self.write_variable(index, byte_array).await
    }

    // Multiple variable operations

    /// Read multiple byte variables (B)
    ///
    /// # Errors
    /// Returns an error if communication fails
    pub async fn read_multiple_u8(
        &self,
        start_variable_number: u16,
        count: u32,
    ) -> Result<Vec<u8>, ClientError> {
        self.read_multiple_variables::<u8>(start_variable_number, count).await
    }

    /// Write multiple byte variables (B)
    ///
    /// # Errors
    /// Returns an error if communication fails
    pub async fn write_multiple_u8(
        &self,
        start_variable_number: u16,
        values: Vec<u8>,
    ) -> Result<(), ClientError> {
        self.write_multiple_variables_u8(start_variable_number, values).await
    }

    /// Read multiple integer variables (I)
    ///
    /// # Errors
    /// Returns an error if communication fails
    pub async fn read_multiple_i16(
        &self,
        start_variable_number: u16,
        count: u32,
    ) -> Result<Vec<i16>, ClientError> {
        self.read_multiple_variables::<i16>(start_variable_number, count).await
    }

    /// Write multiple integer variables (I)
    ///
    /// # Errors
    /// Returns an error if communication fails
    pub async fn write_multiple_i16(
        &self,
        start_variable_number: u16,
        values: Vec<i16>,
    ) -> Result<(), ClientError> {
        self.write_multiple_variables_i16(start_variable_number, values).await
    }

    /// Read multiple double precision integer variables (D)
    ///
    /// # Errors
    /// Returns an error if communication fails
    pub async fn read_multiple_i32(
        &self,
        start_variable_number: u16,
        count: u32,
    ) -> Result<Vec<i32>, ClientError> {
        self.read_multiple_variables::<i32>(start_variable_number, count).await
    }

    /// Write multiple double precision integer variables (D)
    ///
    /// # Errors
    /// Returns an error if communication fails
    pub async fn write_multiple_i32(
        &self,
        start_variable_number: u16,
        values: Vec<i32>,
    ) -> Result<(), ClientError> {
        self.write_multiple_variables_i32(start_variable_number, values).await
    }

    /// Read multiple real type variables (R)
    ///
    /// # Errors
    /// Returns an error if communication fails
    pub async fn read_multiple_f32(
        &self,
        start_variable_number: u16,
        count: u32,
    ) -> Result<Vec<f32>, ClientError> {
        self.read_multiple_variables::<f32>(start_variable_number, count).await
    }

    /// Write multiple real type variables (R)
    ///
    /// # Errors
    /// Returns an error if communication fails
    pub async fn write_multiple_f32(
        &self,
        start_variable_number: u16,
        values: Vec<f32>,
    ) -> Result<(), ClientError> {
        self.write_multiple_variables_f32(start_variable_number, values).await
    }

    /// Read multiple string variables (S) with encoding support
    ///
    /// Reads raw byte arrays using `read_multiple_variables<[u8; 16]>` and converts them to strings
    /// using the client's text encoding configuration.
    ///
    /// # Errors
    /// Returns an error if communication fails
    pub async fn read_multiple_strings(
        &self,
        start_variable_number: u16,
        count: u32,
    ) -> Result<Vec<String>, ClientError> {
        // Read raw byte arrays from protocol layer
        let byte_arrays =
            self.read_multiple_variables::<[u8; 16]>(start_variable_number, count).await?;

        // Convert byte arrays to strings with encoding
        let mut strings = Vec::with_capacity(byte_arrays.len());
        for byte_array in byte_arrays {
            // Find null terminator
            let trimmed_bytes = byte_array
                .iter()
                .position(|&b| b == 0)
                .map_or(&byte_array[..], |pos| &byte_array[..pos]);

            // Decode using client's text encoding
            let string = moto_hses_proto::encoding_utils::decode_string_with_fallback(
                trimmed_bytes,
                self.config.text_encoding,
            );
            strings.push(string);
        }

        Ok(strings)
    }

    /// Write multiple string variables (S) with encoding support
    ///
    /// Converts strings to raw byte arrays using the client's text encoding configuration,
    /// then writes them using `write_multiple_variables<[u8; 16]>`.
    ///
    /// # Errors
    /// Returns an error if communication fails or if any string exceeds 16 bytes when encoded
    pub async fn write_multiple_strings(
        &self,
        start_variable_number: u16,
        values: Vec<String>,
    ) -> Result<(), ClientError> {
        // Convert strings to byte arrays with encoding
        let mut byte_arrays = Vec::with_capacity(values.len());
        for (i, value) in values.iter().enumerate() {
            let encoded_bytes =
                moto_hses_proto::encoding_utils::encode_string(value, self.config.text_encoding);

            if encoded_bytes.len() > 16 {
                return Err(ClientError::SystemError(format!(
                    "String at index {i} exceeds 16 bytes when encoded: {} bytes",
                    encoded_bytes.len()
                )));
            }

            let mut byte_array = [0u8; 16];
            byte_array[..encoded_bytes.len()].copy_from_slice(&encoded_bytes);
            byte_arrays.push(byte_array);
        }

        // Write raw byte arrays using protocol layer
        self.write_multiple_variables_string_bytes(start_variable_number, byte_arrays).await
    }
}
