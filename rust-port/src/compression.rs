//! Compression and decompression utilities for PSD files
//!
//! Supports RLE and ZIP compression methods used in PSD files.

use crate::error::{PsdError, Result};
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression as FlateCompression;
use std::io::{Read, Write};

/// Decompress RLE-compressed data
///
/// RLE (Run-Length Encoding) is used for layer image data in PSD files.
/// Each scanline is compressed separately.
pub fn decompress_rle(
    input: &[u8],
    output: &mut [u8],
    _width: usize,
    height: usize,
    byte_counts: &[u16],
) -> Result<()> {
    let mut input_pos: usize = 0;
    let mut output_pos: usize = 0;

    if byte_counts.len() < height {
        return Err(PsdError::Compression(
            "RLE: missing row byte counts".to_string(),
        ));
    }

    for row in 0..height {
        let byte_count = byte_counts[row] as usize;
        let row_end = input_pos
            .checked_add(byte_count)
            .ok_or_else(|| PsdError::Compression("RLE: row size overflow".to_string()))?;
        if row_end > input.len() {
            return Err(PsdError::Compression(
                "RLE: row exceeds input length".to_string(),
            ));
        }

        while input_pos < row_end && output_pos < output.len() {
            let header = input[input_pos];
            input_pos += 1;

            if header > 128 {
                // Repeat next byte (257 - header) times
                let count = 257usize - header as usize;
                if input_pos >= input.len() {
                    return Err(PsdError::Compression(
                        "RLE: unexpected end of input".to_string(),
                    ));
                }
                let value = input[input_pos];
                input_pos += 1;

                for _ in 0..count {
                    if output_pos >= output.len() {
                        return Err(PsdError::Compression("RLE: output overflow".to_string()));
                    }
                    output[output_pos] = value;
                    output_pos += 1;
                }
            } else if header < 128 {
                // Copy next (header + 1) bytes
                let count = header as usize + 1;
                for _ in 0..count {
                    if input_pos >= input.len() {
                        return Err(PsdError::Compression(
                            "RLE: unexpected end of input".to_string(),
                        ));
                    }
                    if output_pos >= output.len() {
                        return Err(PsdError::Compression("RLE: output overflow".to_string()));
                    }
                    output[output_pos] = input[input_pos];
                    input_pos += 1;
                    output_pos += 1;
                }
            } else {
                // 128 is a NOP in PackBits.
            }
        }

        // Skip any unread row bytes if malformed streams decode early.
        if input_pos < row_end {
            input_pos = row_end;
        }
    }

    Ok(())
}

/// Compress data using RLE
///
/// Returns the compressed data with byte counts for each scanline prepended.
pub fn compress_rle(data: &[u8], width: usize, height: usize) -> Result<Vec<u8>> {
    let mut result = Vec::new();
    let mut byte_counts = Vec::with_capacity(height);

    for y in 0..height {
        let row_start = y * width;
        let row_end = row_start + width;
        let row = &data[row_start..row_end];

        let compressed_row = compress_rle_row(row)?;
        byte_counts.push(compressed_row.len() as u16);
        result.extend_from_slice(&compressed_row);
    }

    // Prepend byte counts
    let mut output = Vec::with_capacity(byte_counts.len() * 2 + result.len());
    for count in byte_counts {
        output.extend_from_slice(&count.to_be_bytes());
    }
    output.extend_from_slice(&result);

    Ok(output)
}

/// Compress a single row using RLE
fn compress_rle_row(row: &[u8]) -> Result<Vec<u8>> {
    let mut result = Vec::new();
    let mut i = 0;

    while i < row.len() {
        // Try to encode a run first.
        let mut run_len = 1usize;
        while i + run_len < row.len() && row[i + run_len] == row[i] && run_len < 128 {
            run_len += 1;
        }
        if run_len >= 3 {
            result.push((1i16 - run_len as i16) as u8);
            result.push(row[i]);
            i += run_len;
            continue;
        }

        // Otherwise encode a literal block up to 128 bytes or until next run.
        let lit_start = i;
        let mut lit_len = 0usize;
        while i < row.len() && lit_len < 128 {
            if i + 2 < row.len() && row[i] == row[i + 1] && row[i] == row[i + 2] {
                break;
            }
            i += 1;
            lit_len += 1;
        }
        if lit_len == 0 {
            // Fallback safety; should never happen due run path above.
            lit_len = 1;
            i += 1;
        }
        result.push((lit_len - 1) as u8);
        result.extend_from_slice(&row[lit_start..lit_start + lit_len]);
    }

    Ok(result)
}

/// Decompress ZIP-compressed data
pub fn decompress_zip(input: &[u8], output_size: usize) -> Result<Vec<u8>> {
    let mut decoder = ZlibDecoder::new(input);
    let mut output = Vec::with_capacity(output_size);
    
    decoder
        .read_to_end(&mut output)
        .map_err(|e| PsdError::Compression(format!("ZIP decompression failed: {}", e)))?;

    Ok(output)
}

/// Compress data using ZIP
pub fn compress_zip(input: &[u8]) -> Result<Vec<u8>> {
    let mut encoder = ZlibEncoder::new(Vec::new(), FlateCompression::default());
    
    encoder
        .write_all(input)
        .map_err(|e| PsdError::Compression(format!("ZIP compression failed: {}", e)))?;
    
    encoder
        .finish()
        .map_err(|e| PsdError::Compression(format!("ZIP compression failed: {}", e)))
}

/// Decompress ZIP with prediction
///
/// Prediction is used to improve compression by storing differences between adjacent pixels.
pub fn decompress_zip_with_prediction(
    input: &[u8],
    width: usize,
    height: usize,
    channels: usize,
) -> Result<Vec<u8>> {
    let output_size = width * height * channels;
    let mut data = decompress_zip(input, output_size)?;
    
    // Reverse prediction
    reverse_prediction(&mut data, width, height, channels);
    
    Ok(data)
}

/// Compress data using ZIP with prediction
pub fn compress_zip_with_prediction(
    input: &[u8],
    width: usize,
    height: usize,
    channels: usize,
) -> Result<Vec<u8>> {
    let mut data = input.to_vec();
    
    // Apply prediction
    apply_prediction(&mut data, width, height, channels);
    
    compress_zip(&data)
}

/// Apply prediction filter to improve compression
fn apply_prediction(data: &mut [u8], width: usize, height: usize, channels: usize) {
    for y in 0..height {
        for c in 0..channels {
            let row_start = y * width * channels + c;
            
            for x in (1..width).rev() {
                let pos = row_start + x * channels;
                let prev_pos = row_start + (x - 1) * channels;
                
                if pos < data.len() && prev_pos < data.len() {
                    data[pos] = data[pos].wrapping_sub(data[prev_pos]);
                }
            }
        }
    }
}

/// Reverse prediction filter after decompression
fn reverse_prediction(data: &mut [u8], width: usize, height: usize, channels: usize) {
    for y in 0..height {
        for c in 0..channels {
            let row_start = y * width * channels + c;
            
            for x in 1..width {
                let pos = row_start + x * channels;
                let prev_pos = row_start + (x - 1) * channels;
                
                if pos < data.len() && prev_pos < data.len() {
                    data[pos] = data[pos].wrapping_add(data[prev_pos]);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compress_decompress_rle() {
        let data = vec![1, 1, 1, 2, 3, 4, 5, 5];
        let compressed = compress_rle(&data, 8, 1).unwrap();
        
        // Skip byte counts (2 bytes for 1 row)
        let byte_count = u16::from_be_bytes([compressed[0], compressed[1]]) as usize;
        let compressed_data = &compressed[2..];
        
        let mut output = vec![0u8; 8];
        decompress_rle(compressed_data, &mut output, 8, 1, &[byte_count as u16]).unwrap();
        
        assert_eq!(output, data);
    }

    #[test]
    fn test_compress_decompress_zip() {
        let data = b"Hello, World! This is a test of ZIP compression.";
        let compressed = compress_zip(data).unwrap();
        let decompressed = decompress_zip(&compressed, data.len()).unwrap();
        
        assert_eq!(&decompressed[..], &data[..]);
    }

    #[test]
    fn test_prediction() {
        let mut data = vec![10, 20, 30, 40, 50];
        let original = data.clone();
        
        apply_prediction(&mut data, 5, 1, 1);
        reverse_prediction(&mut data, 5, 1, 1);
        
        assert_eq!(data, original);
    }
}
