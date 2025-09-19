//! Text encoding utilities

use crate::encoding::TextEncoding;

/// Decode bytes to string with specified encoding, with UTF-8 fallback on error
///
/// # Arguments
/// * `bytes` - The byte slice to decode
/// * `encoding` - The text encoding to use for decoding
///
/// # Returns
/// The decoded string. If the specified encoding fails, falls back to UTF-8 lossy decoding.
#[must_use]
pub fn decode_string_with_fallback(bytes: &[u8], encoding: TextEncoding) -> String {
    let (decoded, _encoding_used, had_errors) = encoding.to_encoding().decode(bytes);

    if had_errors {
        // If specified encoding decoding had errors, fallback to UTF-8
        String::from_utf8_lossy(bytes).to_string()
    } else {
        decoded.to_string()
    }
}

/// Encode string to bytes with specified encoding
///
/// # Arguments
/// * `string` - The string to encode
/// * `encoding` - The text encoding to use for encoding
///
/// # Returns
/// The encoded bytes
#[must_use]
pub fn encode_string(string: &str, encoding: TextEncoding) -> Vec<u8> {
    let (encoded, _encoding_used, _had_errors) = encoding.to_encoding().encode(string);
    encoded.to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_string_with_fallback_utf8() {
        let bytes = b"Hello World";
        let result = decode_string_with_fallback(bytes, TextEncoding::Utf8);
        assert_eq!(result, "Hello World");
    }

    #[test]
    fn test_decode_string_with_fallback_shift_jis() {
        let bytes = b"Hello World"; // ASCII characters work with both encodings
        let result = decode_string_with_fallback(bytes, TextEncoding::ShiftJis);
        assert_eq!(result, "Hello World");
    }

    #[test]
    fn test_decode_string_with_fallback_shift_jis_japanese() {
        // "テスト" in Shift_JIS encoding
        let bytes = &[0x83, 0x65, 0x83, 0x58, 0x83, 0x67];
        let result = decode_string_with_fallback(bytes, TextEncoding::ShiftJis);
        assert_eq!(result, "テスト");
    }

    #[test]
    fn test_decode_string_with_fallback_shift_jis_mixed() {
        // "HelloテストWorld" in Shift_JIS encoding
        let bytes = &[
            0x48, 0x65, 0x6C, 0x6C, 0x6F, 0x83, 0x65, 0x83, 0x58, 0x83, 0x67, 0x57, 0x6F, 0x72,
            0x6C, 0x64,
        ];
        let result = decode_string_with_fallback(bytes, TextEncoding::ShiftJis);
        assert_eq!(result, "HelloテストWorld");
    }

    #[test]
    fn test_decode_string_with_fallback_invalid_utf8() {
        let bytes = &[0xFF, 0xFE]; // Invalid UTF-8
        let result = decode_string_with_fallback(bytes, TextEncoding::Utf8);
        assert_eq!(result, ""); // UTF-8 lossy fallback
    }

    #[test]
    fn test_encode_string_utf8() {
        let string = "Hello World";
        let result = encode_string(string, TextEncoding::Utf8);
        assert_eq!(result, b"Hello World");
    }

    #[test]
    fn test_encode_string_shift_jis() {
        let string = "Hello World"; // ASCII characters work with both encodings
        let result = encode_string(string, TextEncoding::ShiftJis);
        assert_eq!(result, b"Hello World");
    }

    #[test]
    fn test_encode_string_shift_jis_japanese() {
        let string = "テスト";
        let result = encode_string(string, TextEncoding::ShiftJis);
        // "テスト" in Shift_JIS encoding
        assert_eq!(result, &[0x83, 0x65, 0x83, 0x58, 0x83, 0x67]);
    }

    #[test]
    fn test_encode_string_shift_jis_mixed() {
        let string = "HelloテストWorld";
        let result = encode_string(string, TextEncoding::ShiftJis);
        // "HelloテストWorld" in Shift_JIS encoding
        assert_eq!(
            result,
            &[
                0x48, 0x65, 0x6C, 0x6C, 0x6F, 0x83, 0x65, 0x83, 0x58, 0x83, 0x67, 0x57, 0x6F, 0x72,
                0x6C, 0x64
            ]
        );
    }

    #[test]
    fn test_roundtrip_shift_jis_japanese() {
        let original = "テストアラーム";
        let encoded = encode_string(original, TextEncoding::ShiftJis);
        let decoded = decode_string_with_fallback(&encoded, TextEncoding::ShiftJis);
        assert_eq!(decoded, original);
    }

    #[test]
    fn test_roundtrip_shift_jis_mixed() {
        let original = "HelloテストWorld123";
        let encoded = encode_string(original, TextEncoding::ShiftJis);
        let decoded = decode_string_with_fallback(&encoded, TextEncoding::ShiftJis);
        assert_eq!(decoded, original);
    }
}
