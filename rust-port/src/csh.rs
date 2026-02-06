//! Custom Shape (CSH) file format support
//!
//! Provides reading of Adobe Photoshop Custom Shape files.

use crate::error::{PsdError, Result};
use crate::layer::{BezierKnot, BezierPath};
use crate::types::BooleanOperation;
use byteorder::{BigEndian, ReadBytesExt};
use std::io::{Cursor, Read};

/// Custom shape definition
#[derive(Debug, Clone)]
pub struct CustomShape {
    pub name: String,
    pub id: String,
    pub width: u32,
    pub height: u32,
    pub paths: Vec<BezierPath>,
}

/// CSH file structure
#[derive(Debug, Clone)]
pub struct Csh {
    pub shapes: Vec<CustomShape>,
}

/// Read a CSH file from a reader
pub fn read_csh<R: Read>(mut reader: R) -> Result<Csh> {
    // Read entire file into memory and use a cursor for position tracking
    let mut buffer = Vec::new();
    reader.read_to_end(&mut buffer)
        .map_err(|e| PsdError::InvalidCsh(format!("Failed to read CSH: {}", e)))?;
    
    let mut cursor = Cursor::new(buffer);
    
    // Read signature
    let mut sig = [0u8; 4];
    cursor.read_exact(&mut sig)
        .map_err(|e| PsdError::InvalidCsh(format!("Failed to read signature: {}", e)))?;
    
    if &sig != b"cush" {
        return Err(PsdError::InvalidCsh(format!(
            "Invalid signature: expected 'cush', got {:?}",
            String::from_utf8_lossy(&sig)
        )));
    }

    // Read version
    let version = cursor.read_u32::<BigEndian>()
        .map_err(|e| PsdError::InvalidCsh(format!("Failed to read version: {}", e)))?;
    
    if version != 2 {
        return Err(PsdError::InvalidCsh(format!(
            "Unsupported version: {} (expected 2)",
            version
        )));
    }

    // Read shape count
    let count = cursor.read_u32::<BigEndian>()
        .map_err(|e| PsdError::InvalidCsh(format!("Failed to read count: {}", e)))?;

    let mut shapes = Vec::with_capacity(count as usize);

    for _ in 0..count {
        // Read Unicode name
        let name = read_unicode_string(&mut cursor)?;
        
        // Align to 4-byte boundary
        align_to_4bytes(&mut cursor)?;

        // Read shape version
        let shape_version = cursor.read_u32::<BigEndian>()
            .map_err(|e| PsdError::InvalidCsh(format!("Failed to read shape version: {}", e)))?;
        
        if shape_version != 1 {
            return Err(PsdError::InvalidCsh(format!(
                "Unsupported shape version: {}",
                shape_version
            )));
        }

        // Read size
        let size = cursor.read_u32::<BigEndian>()
            .map_err(|e| PsdError::InvalidCsh(format!("Failed to read size: {}", e)))?;
        
        let end_offset = size as usize;

        // Read ID (Pascal string)
        let id = read_pascal_string(&mut cursor, 1)?;

        // Read bounds
        let y1 = cursor.read_u32::<BigEndian>()
            .map_err(|e| PsdError::InvalidCsh(format!("Failed to read y1: {}", e)))?;
        let x1 = cursor.read_u32::<BigEndian>()
            .map_err(|e| PsdError::InvalidCsh(format!("Failed to read x1: {}", e)))?;
        let y2 = cursor.read_u32::<BigEndian>()
            .map_err(|e| PsdError::InvalidCsh(format!("Failed to read y2: {}", e)))?;
        let x2 = cursor.read_u32::<BigEndian>()
            .map_err(|e| PsdError::InvalidCsh(format!("Failed to read x2: {}", e)))?;

        let width = x2 - x1;
        let height = y2 - y1;

        // Read vector mask data
        let paths = read_vector_mask_paths(&mut cursor, width, height, end_offset)?;

        shapes.push(CustomShape {
            name,
            id,
            width,
            height,
            paths,
        });
    }

    Ok(Csh { shapes })
}

/// Read a Unicode string (length-prefixed UTF-16 big-endian)
fn read_unicode_string<R: Read>(reader: &mut R) -> Result<String> {
    let length = reader.read_u32::<BigEndian>()
        .map_err(|e| PsdError::InvalidCsh(format!("Failed to read string length: {}", e)))?;
    
    if length == 0 {
        return Ok(String::new());
    }

    let mut buffer = Vec::with_capacity(length as usize);
    
    for _ in 0..length {
        let ch = reader.read_u16::<BigEndian>()
            .map_err(|e| PsdError::InvalidCsh(format!("Failed to read unicode char: {}", e)))?;
        if ch != 0 {
            buffer.push(ch);
        }
    }
    
    String::from_utf16(&buffer)
        .map_err(|e| PsdError::InvalidCsh(format!("Invalid UTF-16: {}", e)))
}

/// Read a Pascal string (length byte + characters)
fn read_pascal_string<R: Read>(reader: &mut R, pad_to: usize) -> Result<String> {
    let length = reader.read_u8()
        .map_err(|e| PsdError::InvalidCsh(format!("Failed to read pascal string length: {}", e)))?;
    
    let mut buffer = vec![0u8; length as usize];
    reader.read_exact(&mut buffer)
        .map_err(|e| PsdError::InvalidCsh(format!("Failed to read pascal string: {}", e)))?;
    
    // Read padding
    let total = 1 + length as usize;
    let padding = (pad_to - (total % pad_to)) % pad_to;
    
    for _ in 0..padding {
        reader.read_u8()
            .map_err(|e| PsdError::InvalidCsh(format!("Failed to read padding: {}", e)))?;
    }
    
    String::from_utf8(buffer)
        .map_err(|e| PsdError::InvalidCsh(format!("Invalid UTF-8: {}", e)))
}

/// Align cursor position to 4-byte boundary
fn align_to_4bytes(cursor: &mut Cursor<Vec<u8>>) -> Result<()> {
    let pos = cursor.position() as usize;
    let padding = (4 - (pos % 4)) % 4;
    if padding > 0 {
        let mut padding_buf = vec![0u8; padding];
        cursor.read_exact(&mut padding_buf)
            .map_err(|e| PsdError::InvalidCsh(format!("Failed to read padding: {}", e)))?;
    }
    Ok(())
}

/// Read vector mask paths
fn read_vector_mask_paths(
    cursor: &mut Cursor<Vec<u8>>,
    _width: u32,
    _height: u32,
    _size: usize,
) -> Result<Vec<BezierPath>> {
    let mut paths = Vec::new();

    // Read path count
    let path_count = cursor.read_u32::<BigEndian>()
        .map_err(|e| PsdError::InvalidCsh(format!("Failed to read path count: {}", e)))?;

    for _ in 0..path_count {
        // Read path record type and count
        let record_type = cursor.read_u16::<BigEndian>()
            .map_err(|e| PsdError::InvalidCsh(format!("Failed to read record type: {}", e)))?;

        match record_type {
            0 => {
                // Closed path length record
                let num_points = cursor.read_u16::<BigEndian>()
                    .map_err(|e| PsdError::InvalidCsh(format!("Failed to read num points: {}", e)))?;
                
                let mut knots = Vec::with_capacity(num_points as usize);
                
                for _ in 0..num_points {
                    let knot = read_bezier_knot(cursor)?;
                    knots.push(knot);
                }
                
                paths.push(BezierPath {
                    open: false,
                    operation: Some(BooleanOperation::Combine),
                    knots,
                    fill_rule: "nonzero".to_string(),
                });
            }
            3 => {
                // Open path length record
                let num_points = cursor.read_u16::<BigEndian>()
                    .map_err(|e| PsdError::InvalidCsh(format!("Failed to read num points: {}", e)))?;
                
                let mut knots = Vec::with_capacity(num_points as usize);
                
                for _ in 0..num_points {
                    let knot = read_bezier_knot(cursor)?;
                    knots.push(knot);
                }
                
                paths.push(BezierPath {
                    open: true,
                    operation: Some(BooleanOperation::Combine),
                    knots,
                    fill_rule: "nonzero".to_string(),
                });
            }
            _ => {
                // Skip unknown record types
                continue;
            }
        }
    }

    Ok(paths)
}

/// Read a Bezier knot
fn read_bezier_knot(cursor: &mut Cursor<Vec<u8>>) -> Result<BezierKnot> {
    // Each knot has 6 coordinates (control points + anchor)
    // They are stored as 32-bit fixed point values
    
    let mut points = Vec::with_capacity(6);
    
    for _ in 0..6 {
        let value = cursor.read_i32::<BigEndian>()
            .map_err(|e| PsdError::InvalidCsh(format!("Failed to read knot point: {}", e)))?;
        
        // Convert from fixed point (8.24) to floating point
        let float_value = value as f64 / (1 << 24) as f64;
        points.push(float_value);
    }
    
    Ok(BezierKnot {
        linked: true,
        points,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_read_csh_invalid_signature() {
        let data = b"INVALID";
        let cursor = Cursor::new(data);
        let result = read_csh(cursor);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_pascal_string() {
        let data = vec![5, b'H', b'e', b'l', b'l', b'o', 0]; // length=5, "Hello", padding
        let mut cursor = Cursor::new(data);
        let result = read_pascal_string(&mut cursor, 2).unwrap();
        assert_eq!(result, "Hello");
    }
}
