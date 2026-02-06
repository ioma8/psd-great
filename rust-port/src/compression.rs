//! Compression and decompression utilities for PSD files
//!
//! Supports RLE and ZIP compression methods used in PSD files.

use crate::error::{PsdError, Result};
use crate::types::{PixelData, Compression};
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
    width: usize,
    height: usize,
    byte_counts: &[u16],
) -> Result<()> {
    let mut input_pos = 0;
    let mut output_pos = 0;

    for row in 0..height {
        let byte_count = byte_counts[row] as usize;
        let row_end = input_pos + byte_count;

        while input_pos < row_end && output_pos < output.len() {
            let len = input[input_pos] as i8;
            input_pos += 1;

            if len < 0 {
                // Repeat next byte (-len + 1) times
                let count = (-len + 1) as usize;
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
            } else {
                // Copy next (len + 1) bytes
                let count = (len + 1) as usize;
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
            }
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
        let start = i;
        
        // Check for run
        if i + 2 < row.len() && row[i] == row[i + 1] && row[i] == row[i + 2] {
            let value = row[i];
            let mut count = 1;
            i += 1;
            
            while i < row.len() && row[i] == value && count < 128 {
                count += 1;
                i += 1;
            }
            
            // Write run
            result.push((1 - count) as u8);
            result.push(value);
        } else {
            // Collect literal bytes
            let mut literal = Vec::new();
            
            while i < row.len() && literal.len() < 128 {
                literal.push(row[i]);
                i += 1;
                
                // Check if we should stop (run ahead)
                if i + 2 < row.len() 
                    && row[i] == row[i + 1] 
                    && row[i] == row[i + 2] {
                    i -= 1;
                    literal.pop();
                    break;
                }
            }
            
            if !literal.is_empty() {
                result.push((literal.len() - 1) as u8);
                result.extend_from_slice(&literal);
            }
        }
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
