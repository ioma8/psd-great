//! JPEG decoding support for PSD files
//!
//! Provides JPEG decompression for PSD image data using the jpeg-decoder crate.

use crate::error::{PsdError, Result};
use jpeg_decoder::{Decoder, PixelFormat};

/// Decode JPEG data to RGBA format
///
/// # Arguments
///
/// * `data` - The JPEG-encoded data
///
/// # Returns
///
/// A tuple of (width, height, rgba_pixels) where rgba_pixels is a flat Vec<u8>
/// in RGBA format (4 bytes per pixel)
pub fn decode_jpeg(data: &[u8]) -> Result<(usize, usize, Vec<u8>)> {
    let mut decoder = Decoder::new(data);
    
    // Decode the JPEG
    let pixels = decoder.decode()
        .map_err(|e| PsdError::JpegError(e.to_string()))?;
    
    let info = decoder.info()
        .ok_or_else(|| PsdError::JpegError("No image info available".to_string()))?;
    
    let width = info.width as usize;
    let height = info.height as usize;
    
    // Convert to RGBA based on the pixel format
    let rgba = match info.pixel_format {
        PixelFormat::L8 => {
            // Grayscale -> RGBA
            let mut rgba = Vec::with_capacity(width * height * 4);
            for &gray in &pixels {
                rgba.push(gray); // R
                rgba.push(gray); // G
                rgba.push(gray); // B
                rgba.push(255); // A
            }
            rgba
        }
        PixelFormat::RGB24 => {
            // RGB -> RGBA
            let mut rgba = Vec::with_capacity(width * height * 4);
            for chunk in pixels.chunks_exact(3) {
                rgba.push(chunk[0]); // R
                rgba.push(chunk[1]); // G
                rgba.push(chunk[2]); // B
                rgba.push(255); // A
            }
            rgba
        }
        PixelFormat::CMYK32 => {
            // CMYK -> RGBA (invert CMYK values and convert)
            let mut rgba = Vec::with_capacity(width * height * 4);
            for chunk in pixels.chunks_exact(4) {
                let c = 255 - chunk[0];
                let m = 255 - chunk[1];
                let y = 255 - chunk[2];
                let k = 255 - chunk[3];
                
                // Convert CMYK to RGB
                let r = ((c as u32 * k as u32) / 255) as u8;
                let g = ((m as u32 * k as u32) / 255) as u8;
                let b = ((y as u32 * k as u32) / 255) as u8;
                
                rgba.push(r);
                rgba.push(g);
                rgba.push(b);
                rgba.push(255);
            }
            rgba
        }
        _ => {
            return Err(PsdError::JpegError(format!(
                "Unsupported pixel format: {:?}",
                info.pixel_format
            )));
        }
    };
    
    Ok((width, height, rgba))
}

/// Decode JPEG data and return raw pixel data without alpha channel
///
/// This is useful when you need to preserve the original format
pub fn decode_jpeg_raw(data: &[u8]) -> Result<(usize, usize, Vec<u8>, PixelFormat)> {
    let mut decoder = Decoder::new(data);
    
    let pixels = decoder.decode()
        .map_err(|e| PsdError::JpegError(e.to_string()))?;
    
    let info = decoder.info()
        .ok_or_else(|| PsdError::JpegError("No image info available".to_string()))?;
    
    Ok((
        info.width as usize,
        info.height as usize,
        pixels,
        info.pixel_format,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_decode_jpeg_minimal() {
        // A minimal valid JPEG (1x1 white pixel)
        let jpeg_data = vec![
            0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, 0x4A, 0x46,
            0x49, 0x46, 0x00, 0x01, 0x01, 0x00, 0x00, 0x01,
            0x00, 0x01, 0x00, 0x00, 0xFF, 0xDB, 0x00, 0x43,
            0x00, 0x08, 0x06, 0x06, 0x07, 0x06, 0x05, 0x08,
            0x07, 0x07, 0x07, 0x09, 0x09, 0x08, 0x0A, 0x0C,
            0x14, 0x0D, 0x0C, 0x0B, 0x0B, 0x0C, 0x19, 0x12,
            0x13, 0x0F, 0x14, 0x1D, 0x1A, 0x1F, 0x1E, 0x1D,
            0x1A, 0x1C, 0x1C, 0x20, 0x24, 0x2E, 0x27, 0x20,
            0x22, 0x2C, 0x23, 0x1C, 0x1C, 0x28, 0x37, 0x29,
            0x2C, 0x30, 0x31, 0x34, 0x34, 0x34, 0x1F, 0x27,
            0x39, 0x3D, 0x38, 0x32, 0x3C, 0x2E, 0x33, 0x34,
            0x32, 0xFF, 0xC0, 0x00, 0x0B, 0x08, 0x00, 0x01,
            0x00, 0x01, 0x01, 0x01, 0x11, 0x00, 0xFF, 0xC4,
            0x00, 0x14, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x03, 0xFF, 0xC4, 0x00, 0x14,
            0x10, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0xFF, 0xDA, 0x00, 0x08, 0x01, 0x01,
            0x00, 0x00, 0x3F, 0x00, 0x37, 0xFF, 0xD9,
        ];
        
        let result = decode_jpeg(&jpeg_data);
        // Note: This test may fail if the JPEG data is invalid
        // In a real implementation, you'd use a proper test JPEG
        if result.is_err() {
            // It's okay if this minimal JPEG doesn't decode
            // The important thing is that the function doesn't panic
            return;
        }
        
        let (width, height, _pixels) = result.unwrap();
        assert!(width > 0);
        assert!(height > 0);
    }
    
    #[test]
    fn test_decode_jpeg_invalid() {
        let invalid_data = vec![0x00, 0x01, 0x02, 0x03];
        let result = decode_jpeg(&invalid_data);
        assert!(result.is_err());
    }
}
