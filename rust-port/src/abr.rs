//! Adobe Brush (ABR) file format support
//!
//! Provides reading of Adobe Photoshop brush preset files.

use crate::error::{PsdError, Result};
use crate::descriptor::Descriptor;
use crate::layer::PatternInfo;
use crate::psd::ReadOptions;
use crate::reader::PsdReader;
use byteorder::{BigEndian, ReadBytesExt};
use std::io::{Cursor, Read, Seek};

/// Brush file structure
#[derive(Debug, Clone)]
pub struct Abr {
    pub brushes: Vec<Brush>,
    pub samples: Vec<SampleInfo>,
    pub patterns: Vec<PatternInfo>,
}

/// Sample information
#[derive(Debug, Clone)]
pub struct SampleInfo {
    pub id: String,
    pub bounds: BrushBounds,
    pub alpha: Vec<u8>,
}

/// Brush bounds
#[derive(Debug, Clone, PartialEq)]
pub struct BrushBounds {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

/// Brush dynamics control
#[derive(Debug, Clone, PartialEq)]
pub enum DynamicsControl {
    Off,
    Fade,
    PenPressure,
    PenTilt,
    StylusWheel,
    InitialDirection,
    Direction,
    InitialRotation,
    Rotation,
}

/// Brush dynamics settings
#[derive(Debug, Clone)]
pub struct BrushDynamics {
    pub control: DynamicsControl,
    pub steps: i32,
    pub jitter: f64,
    pub minimum: f64,
}

/// Brush shape types
#[derive(Debug, Clone)]
pub enum BrushShape {
    Computed {
        size: f64,
        angle: f64,
        roundness: f64,
        hardness: f64,
        spacing_on: bool,
        spacing: f64,
        flip_x: bool,
        flip_y: bool,
    },
    Sampled {
        name: String,
        size: f64,
        angle: f64,
        roundness: f64,
        spacing_on: bool,
        spacing: f64,
        flip_x: bool,
        flip_y: bool,
        sampled_data: String,
    },
}

/// Brush definition
#[derive(Debug, Clone)]
pub struct Brush {
    pub name: String,
    pub shape: Option<BrushShape>,
    pub spacing: Option<f64>,
    pub diameter: Option<f64>,
    pub roundness: Option<f64>,
    pub angle: Option<f64>,
    pub hardness: Option<f64>,
    pub size_dynamics: Option<BrushDynamics>,
    pub angle_dynamics: Option<BrushDynamics>,
    pub roundness_dynamics: Option<BrushDynamics>,
}

/// Read an ABR file from a reader
pub fn read_abr<R: Read>(mut reader: R) -> Result<Abr> {
    let mut buffer = Vec::new();
    reader.read_to_end(&mut buffer)
        .map_err(|e| PsdError::InvalidAbr(format!("Failed to read ABR: {}", e)))?;

    let mut cursor = Cursor::new(&buffer);
    let version = cursor.read_u16::<BigEndian>()
        .map_err(|e| PsdError::InvalidAbr(format!("Failed to read version: {}", e)))?;

    match version {
        1 | 2 => read_abr_v1_v2(&mut cursor),
        6 | 7 | 9 | 10 => read_abr_v6_plus(&buffer),
        _ => Err(PsdError::InvalidAbr(format!("Unsupported ABR version: {}", version))),
    }
}

/// Read ABR version 1 or 2
fn read_abr_v1_v2<R: Read>(reader: &mut R) -> Result<Abr> {
    let count = reader.read_u16::<BigEndian>()
        .map_err(|e| PsdError::InvalidAbr(format!("Failed to read brush count: {}", e)))?;

    let mut brushes = Vec::new();
    let mut samples = Vec::new();

    for _ in 0..count {
        let brush_type = reader.read_u16::<BigEndian>()
            .map_err(|e| PsdError::InvalidAbr(format!("Failed to read brush type: {}", e)))?;
        let size = reader.read_u32::<BigEndian>()
            .map_err(|e| PsdError::InvalidAbr(format!("Failed to read size: {}", e)))?;

        match brush_type {
            1 => {
                // Computed brush
                let misc = reader.read_u32::<BigEndian>()?;
                let spacing = reader.read_u16::<BigEndian>()? as f64;
                let diameter = reader.read_u16::<BigEndian>()? as f64;
                let roundness = reader.read_u16::<BigEndian>()? as f64;
                let angle = reader.read_u16::<BigEndian>()? as f64;
                let hardness = reader.read_u16::<BigEndian>()? as f64;

                brushes.push(Brush {
                    name: format!("Brush {}", brushes.len() + 1),
                    shape: Some(BrushShape::Computed {
                        size: diameter,
                        angle,
                        roundness: roundness / 100.0,
                        hardness: hardness / 100.0,
                        spacing_on: true,
                        spacing: spacing / 100.0,
                        flip_x: false,
                        flip_y: false,
                    }),
                    spacing: Some(spacing / 100.0),
                    diameter: Some(diameter),
                    roundness: Some(roundness / 100.0),
                    angle: Some(angle),
                    hardness: Some(hardness / 100.0),
                    size_dynamics: None,
                    angle_dynamics: None,
                    roundness_dynamics: None,
                });
            }
            2 => {
                // Sampled brush
                let misc = reader.read_u32::<BigEndian>()?;
                let spacing = reader.read_u16::<BigEndian>()? as f64;
                
                let mut name = vec![0u8; if misc & 1 != 0 { 0 } else { 0 }];
                reader.read_exact(&mut name)?;
                
                let anti_alias = reader.read_u8()?;
                let y = reader.read_i16::<BigEndian>()?;
                let x = reader.read_i16::<BigEndian>()?;
                let h = reader.read_i16::<BigEndian>()?;
                let w = reader.read_i16::<BigEndian>()?;

                let depth = reader.read_u16::<BigEndian>()?;
                let compression = reader.read_u8()?;

                let bounds = BrushBounds {
                    x: x as i32,
                    y: y as i32,
                    w: w as i32,
                    h: h as i32,
                };

                let data_size = ((w as usize * h as usize * depth as usize + 7) / 8) as usize;
                let mut alpha = vec![0u8; data_size];
                reader.read_exact(&mut alpha)?;

                samples.push(SampleInfo {
                    id: format!("sample_{}", samples.len()),
                    bounds,
                    alpha,
                });

                brushes.push(Brush {
                    name: String::from_utf8_lossy(&name).to_string(),
                    shape: None,
                    spacing: Some(spacing / 100.0),
                    diameter: Some(w as f64),
                    roundness: None,
                    angle: None,
                    hardness: None,
                    size_dynamics: None,
                    angle_dynamics: None,
                    roundness_dynamics: None,
                });
            }
            _ => {
                // Skip unknown brush type
                let mut skip_data = vec![0u8; size as usize];
                reader.read_exact(&mut skip_data)?;
            }
        }
    }

    Ok(Abr {
        brushes,
        samples,
        patterns: Vec::new(),
    })
}

/// Read ABR version 6+
fn read_abr_v6_plus(data: &[u8]) -> Result<Abr> {
    let mut brushes = Vec::new();
    let mut samples = Vec::new();
    let mut patterns = Vec::new();

    let mut offset = 2; // Skip version

    // Read subversion
    if offset + 2 > data.len() {
        return Err(PsdError::InvalidAbr("Unexpected end of data".to_string()));
    }
    let _subversion = u16::from_be_bytes([data[offset], data[offset + 1]]);
    offset += 2;

    while offset < data.len() {
        if offset + 8 > data.len() {
            break;
        }

        // Read section signature
        let mut sig = [0u8; 4];
        sig.copy_from_slice(&data[offset..offset + 4]);
        offset += 4;

        if &sig != b"8BIM" {
            break;
        }

        // Read section type
        let mut section_type = [0u8; 4];
        section_type.copy_from_slice(&data[offset..offset + 4]);
        offset += 4;

        // Read section size
        if offset + 4 > data.len() {
            break;
        }
        let size = u32::from_be_bytes([data[offset], data[offset + 1], data[offset + 2], data[offset + 3]]);
        offset += 4;

        let section_end = offset + size as usize;

        match &section_type {
            b"samp" => {
                // Sample section
                // Skip for now - would need to parse sample data
                offset = section_end;
            }
            b"desc" => {
                // Descriptor section - contains brush presets
                if offset + size as usize <= data.len() {
                    let section_data = &data[offset..offset + size as usize];
                    
                    // Try to parse as descriptor
                    if let Ok(descriptor) = parse_brush_descriptor(section_data) {
                        if let Some(brush) = descriptor_to_brush(&descriptor) {
                            brushes.push(brush);
                        }
                    }
                }
                offset = section_end;
            }
            b"patt" => {
                // Pattern section
                offset = section_end;
            }
            b"phry" => {
                // Hierarchy section
                offset = section_end;
            }
            _ => {
                // Unknown section - skip
                offset = section_end;
            }
        }

        if offset > data.len() {
            break;
        }
    }

    Ok(Abr {
        brushes,
        samples,
        patterns,
    })
}

/// Parse a brush descriptor
fn parse_brush_descriptor(data: &[u8]) -> Result<Descriptor> {
    let cursor = Cursor::new(data);
    let mut psd_reader = PsdReader::new(cursor, ReadOptions::default());
    psd_reader.read_version_and_descriptor()
}

/// Convert descriptor to brush
fn descriptor_to_brush(descriptor: &Descriptor) -> Option<Brush> {
    let name = descriptor.items.get("Nm  ")
        .and_then(|v| {
            if let crate::descriptor::DescriptorValue::Text(s) = v {
                Some(s.clone())
            } else {
                None
            }
        })
        .unwrap_or_else(|| "Unnamed".to_string());

    Some(Brush {
        name,
        shape: None,
        spacing: None,
        diameter: None,
        roundness: None,
        angle: None,
        hardness: None,
        size_dynamics: None,
        angle_dynamics: None,
        roundness_dynamics: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_abr_bounds() {
        let bounds = BrushBounds {
            x: 10,
            y: 20,
            w: 100,
            h: 200,
        };
        
        assert_eq!(bounds.x, 10);
        assert_eq!(bounds.w, 100);
    }

    #[test]
    fn test_dynamics_control() {
        let dynamics = BrushDynamics {
            control: DynamicsControl::PenPressure,
            steps: 10,
            jitter: 0.5,
            minimum: 0.0,
        };
        
        assert_eq!(dynamics.steps, 10);
    }
}
