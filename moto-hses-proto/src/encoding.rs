//! Text encoding utilities for HSES protocol

use encoding_rs::{Encoding, SHIFT_JIS, UTF_8};

/// Supported text encodings for HSES protocol
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextEncoding {
    /// UTF-8 encoding (default)
    Utf8,
    /// `Shift_JIS` encoding (common for Japanese)
    ShiftJis,
}

impl Default for TextEncoding {
    fn default() -> Self {
        Self::Utf8
    }
}

impl TextEncoding {
    /// Get the corresponding `encoding_rs::Encoding`
    #[must_use]
    pub fn to_encoding(&self) -> &'static Encoding {
        match self {
            Self::Utf8 => UTF_8,
            Self::ShiftJis => SHIFT_JIS,
        }
    }
}
