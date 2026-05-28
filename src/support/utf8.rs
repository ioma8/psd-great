//! UTF-8 encoding and decoding utilities
//!
//! Provides manual UTF-8 encoding/decoding for compatibility with PSD files.
//! Rust's standard library already handles UTF-8 correctly, but these utilities
//! are provided for cases where manual control is needed.

use crate::support::error::{PsdError, Result};

/// Threshold for using manual UTF-8 decoding vs built-in decoder
const UTF8_MANUAL_DECODE_THRESHOLD: usize = 1000;

/// Calculate the number of bytes needed to encode a Unicode code point in UTF-8
fn char_length_in_bytes(code: u32) -> usize {
    if code & 0xFFFF_FF80 == 0 {
        1
    } else if code & 0xFFFF_F800 == 0 {
        2
    } else if code & 0xFFFF_0000 == 0 {
        3
    } else {
        4
    }
}

/// Calculate the total number of bytes needed to encode a string in UTF-8
pub fn string_length_in_bytes(value: &str) -> usize {
    value.chars().map(|c| char_length_in_bytes(c as u32)).sum()
}

/// Write a single Unicode character to a buffer in UTF-8 encoding
fn write_character(buffer: &mut [u8], offset: usize, code: u32) -> Result<usize> {
    let length = char_length_in_bytes(code);

    if offset + length > buffer.len() {
        return Err(PsdError::InvalidUtf8);
    }

    match length {
        1 => {
            buffer[offset] = code as u8;
        }
        2 => {
            buffer[offset] = ((code >> 6) & 0x1F | 0xC0) as u8;
            buffer[offset + 1] = ((code & 0x3F) | 0x80) as u8;
        }
        3 => {
            buffer[offset] = ((code >> 12) & 0x0F | 0xE0) as u8;
            buffer[offset + 1] = ((code >> 6) & 0x3F | 0x80) as u8;
            buffer[offset + 2] = ((code & 0x3F) | 0x80) as u8;
        }
        4 => {
            buffer[offset] = ((code >> 18) & 0x07 | 0xF0) as u8;
            buffer[offset + 1] = ((code >> 12) & 0x3F | 0x80) as u8;
            buffer[offset + 2] = ((code >> 6) & 0x3F | 0x80) as u8;
            buffer[offset + 3] = ((code & 0x3F) | 0x80) as u8;
        }
        _ => return Err(PsdError::InvalidUtf8),
    }

    Ok(length)
}

/// Encode a string to a buffer at the specified offset, returning the new offset
pub fn encode_string_to(buffer: &mut [u8], offset: usize, value: &str) -> Result<usize> {
    let mut current_offset = offset;

    for c in value.chars() {
        let written = write_character(buffer, current_offset, c as u32)?;
        current_offset += written;
    }

    Ok(current_offset)
}

/// Encode a string to a new Vec<u8> in UTF-8 format
pub fn encode_string(value: &str) -> Vec<u8> {
    // Rust strings are already UTF-8, so we can just convert directly
    value.as_bytes().to_vec()
}

/// Read a continuation byte (10xxxxxx pattern)
fn continuation_byte(buffer: &[u8], index: usize) -> Result<u8> {
    if index >= buffer.len() {
        return Err(PsdError::InvalidUtf8);
    }

    let byte = buffer[index];

    if byte & 0xC0 == 0x80 {
        Ok(byte & 0x3F)
    } else {
        Err(PsdError::InvalidUtf8)
    }
}

/// Decode a UTF-8 byte array to a String
pub fn decode_string(value: &[u8]) -> Result<String> {
    // For most cases, use Rust's built-in UTF-8 validation and conversion
    if value.len() > UTF8_MANUAL_DECODE_THRESHOLD {
        return String::from_utf8(value.to_vec()).map_err(|_| PsdError::InvalidUtf8);
    }

    // Manual decoding for compatibility
    let mut result = String::new();
    let mut i = 0;

    while i < value.len() {
        let byte1 = value[i];
        i += 1;

        let code = if byte1 & 0x80 == 0 {
            // Single-byte character (0xxxxxxx)
            byte1 as u32
        } else if byte1 & 0xE0 == 0xC0 {
            // Two-byte character (110xxxxx 10xxxxxx)
            let byte2 = continuation_byte(value, i)?;
            i += 1;
            let code = ((byte1 & 0x1F) as u32) << 6 | byte2 as u32;

            if code < 0x80 {
                return Err(PsdError::InvalidUtf8);
            }
            code
        } else if byte1 & 0xF0 == 0xE0 {
            // Three-byte character (1110xxxx 10xxxxxx 10xxxxxx)
            let byte2 = continuation_byte(value, i)?;
            i += 1;
            let byte3 = continuation_byte(value, i)?;
            i += 1;
            let code = ((byte1 & 0x0F) as u32) << 12 | (byte2 as u32) << 6 | byte3 as u32;

            if code < 0x0800 {
                return Err(PsdError::InvalidUtf8);
            }

            if (0xD800..=0xDFFF).contains(&code) {
                return Err(PsdError::InvalidUtf8);
            }
            code
        } else if byte1 & 0xF8 == 0xF0 {
            // Four-byte character (11110xxx 10xxxxxx 10xxxxxx 10xxxxxx)
            let byte2 = continuation_byte(value, i)?;
            i += 1;
            let byte3 = continuation_byte(value, i)?;
            i += 1;
            let byte4 = continuation_byte(value, i)?;
            i += 1;
            let code = ((byte1 & 0x07) as u32) << 18
                | (byte2 as u32) << 12
                | (byte3 as u32) << 6
                | byte4 as u32;

            if code < 0x010000 || code > 0x10FFFF {
                return Err(PsdError::InvalidUtf8);
            }
            code
        } else {
            return Err(PsdError::InvalidUtf8);
        };

        if let Some(c) = char::from_u32(code) {
            result.push(c);
        } else {
            return Err(PsdError::InvalidUtf8);
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_char_length_in_bytes() {
        assert_eq!(char_length_in_bytes(0x41), 1); // 'A'
        assert_eq!(char_length_in_bytes(0xA9), 2); // '©'
        assert_eq!(char_length_in_bytes(0x20AC), 3); // '€'
        assert_eq!(char_length_in_bytes(0x1F600), 4); // '😀'
    }

    #[test]
    fn test_string_length_in_bytes() {
        assert_eq!(string_length_in_bytes("Hello"), 5);
        assert_eq!(string_length_in_bytes("©"), 2);
        assert_eq!(string_length_in_bytes("€"), 3);
        assert_eq!(string_length_in_bytes("😀"), 4);
        assert_eq!(string_length_in_bytes("Hello©€😀"), 5 + 2 + 3 + 4);
    }

    #[test]
    fn test_encode_decode_roundtrip() {
        let test_strings = vec![
            "Hello",
            "Hello, World!",
            "©2024",
            "Price: €100",
            "😀😃😄",
            "Mixed: Hello©€😀",
            "",
        ];

        for s in test_strings {
            let encoded = encode_string(s);
            let decoded = decode_string(&encoded).unwrap();
            assert_eq!(s, decoded);
        }
    }

    #[test]
    fn test_encode_string_to() {
        let mut buffer = vec![0u8; 100];
        let offset = encode_string_to(&mut buffer, 0, "Hello").unwrap();
        assert_eq!(offset, 5);
        assert_eq!(&buffer[0..5], b"Hello");
    }

    #[test]
    fn test_invalid_utf8() {
        // Invalid continuation byte
        assert!(decode_string(&[0xC0, 0x00]).is_err());

        // Lone surrogate
        assert!(decode_string(&[0xED, 0xA0, 0x80]).is_err());

        // Overlong encoding
        assert!(decode_string(&[0xC0, 0x80]).is_err());
    }
}
