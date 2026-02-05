//! Additional layer information handlers
//!
//! Handles layer-specific additional information sections like text layers,
//! vector masks, layer effects, smart objects, and other layer properties.

use crate::error::{PsdError, Result};
use crate::reader::PsdReader;
use crate::writer::PsdWriter;
use crate::descriptor::{Descriptor, DescriptorValue};
use crate::types::Color;
use std::collections::HashMap;
use std::io::{Read, Seek};

/// Layer additional information
#[derive(Debug, Clone, Default)]
pub struct LayerAdditionalInfo {
    /// Unicode layer name
    pub unicode_name: Option<String>,
    /// Layer ID
    pub layer_id: Option<u32>,
    /// Layer color
    pub layer_color: Option<LayerColor>,
    /// Section divider (layer group)
    pub section_divider: Option<SectionDivider>,
    /// Blend clipped elements
    pub blend_clipped: Option<bool>,
    /// Blend interior elements
    pub blend_interior: Option<bool>,
    /// Knockout mode
    pub knockout: Option<bool>,
    /// Protected flags
    pub protected: Option<ProtectedFlags>,
    /// Layer name source
    pub name_source: Option<String>,
    /// Text layer data
    pub text: Option<TextLayerData>,
    /// Vector fill data
    pub vector_fill: Option<VectorFill>,
    /// Vector stroke data
    pub vector_stroke: Option<VectorStroke>,
    /// Vector mask data
    pub vector_mask: Option<VectorMask>,
    /// Layer effects
    pub layer_effects: Option<LayerEffects>,
    /// Placed layer (smart object)
    pub placed_layer: Option<PlacedLayer>,
    /// Artboard data
    pub artboard: Option<ArtboardData>,
    /// Using aligned rendering
    pub using_aligned_rendering: Option<bool>,
    /// Metadata
    pub metadata: Option<Metadata>,
    /// Vector origination data
    pub vector_origination: Option<Vec<u8>>,
    /// Unknown sections (for preservation)
    pub unknown: HashMap<String, Vec<u8>>,
}

/// Layer color label
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayerColor {
    None = 0,
    Red = 1,
    Orange = 2,
    Yellow = 3,
    Green = 4,
    Blue = 5,
    Violet = 6,
    Gray = 7,
}

impl LayerColor {
    pub fn from_u16(value: u16) -> Result<Self> {
        match value {
            0 => Ok(LayerColor::None),
            1 => Ok(LayerColor::Red),
            2 => Ok(LayerColor::Orange),
            3 => Ok(LayerColor::Yellow),
            4 => Ok(LayerColor::Green),
            5 => Ok(LayerColor::Blue),
            6 => Ok(LayerColor::Violet),
            7 => Ok(LayerColor::Gray),
            _ => Err(PsdError::InvalidFormat(format!("Invalid layer color: {}", value))),
        }
    }
}

/// Section divider (layer group info)
#[derive(Debug, Clone, PartialEq)]
pub struct SectionDivider {
    pub divider_type: u32,
    pub blend_mode: Option<String>,
    pub sub_type: Option<u32>,
}

/// Protected flags
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ProtectedFlags {
    pub transparency: bool,
    pub composite: bool,
    pub position: bool,
    pub artboards: Option<bool>,
}

/// Text layer data
#[derive(Debug, Clone, PartialEq)]
pub struct TextLayerData {
    pub transform: Vec<f64>,
    pub text: String,
    pub text_version: u16,
    pub descriptor_version: u32,
    pub text_data: Option<Descriptor>,
    pub warp_version: u16,
    pub warp_data: Option<Descriptor>,
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}

/// Vector fill
#[derive(Debug, Clone, PartialEq)]
pub struct VectorFill {
    pub fill_type: String,
    pub data: Descriptor,
}

/// Vector stroke
#[derive(Debug, Clone, PartialEq)]
pub struct VectorStroke {
    pub version: u32,
    pub descriptor: Descriptor,
}

/// Vector mask
#[derive(Debug, Clone, PartialEq)]
pub struct VectorMask {
    pub version: u32,
    pub invert: bool,
    pub not_link: bool,
    pub disable: bool,
    pub paths: Vec<VectorPath>,
}

/// Vector path
#[derive(Debug, Clone, PartialEq)]
pub struct VectorPath {
    pub path_type: PathType,
    pub initial_fill_rule: Option<u16>,
    pub clipboard_bounds: Option<Bounds>,
    pub points: Vec<PathPoint>,
}

/// Path type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathType {
    Closed = 0,
    Open = 1,
}

/// Path point
#[derive(Debug, Clone, PartialEq)]
pub struct PathPoint {
    pub anchor: Point,
    pub forward: Point,
    pub backward: Point,
    pub linked: bool,
}

/// 2D Point
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

/// Bounds rectangle
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bounds {
    pub top: f64,
    pub left: f64,
    pub bottom: f64,
    pub right: f64,
}

/// Layer effects
#[derive(Debug, Clone, PartialEq)]
pub struct LayerEffects {
    pub version: u32,
    pub descriptor: Option<Descriptor>,
}

/// Placed layer (smart object)
#[derive(Debug, Clone, PartialEq)]
pub struct PlacedLayer {
    pub id: String,
    pub page: Option<i32>,
    pub total_pages: Option<i32>,
    pub anti_alias_policy: Option<i32>,
    pub placed_layer_type: Option<i32>,
    pub transform: Vec<f64>,
    pub warp: Option<Descriptor>,
    pub placed: Option<String>,
}

/// Artboard data
#[derive(Debug, Clone, PartialEq)]
pub struct ArtboardData {
    pub rect: Bounds,
    pub preset_name: Option<String>,
    pub color: Option<Color>,
    pub background_type: Option<i32>,
}

/// Metadata
#[derive(Debug, Clone, PartialEq)]
pub struct Metadata {
    pub descriptor: Descriptor,
}

impl<R: Read + Seek> PsdReader<R> {
    /// Read layer additional info section
    pub fn read_additional_info(&mut self, key: &str, length: usize, info: &mut LayerAdditionalInfo) -> Result<()> {
        let start_offset = self.offset;
        
        match key {
            "luni" => self.read_unicode_layer_name(info)?,
            "lyid" => self.read_layer_id(info)?,
            "lclr" => self.read_layer_color(info)?,
            "lsct" | "lsdk" => self.read_section_divider(info, length)?,
            "clbl" => self.read_blend_clipped(info)?,
            "infx" => self.read_blend_interior(info)?,
            "knko" => self.read_knockout(info)?,
            "lspf" => self.read_protected_flags(info, length)?,
            "lnsr" => self.read_name_source(info)?,
            "TySh" => self.read_text_layer(info, length)?,
            "SoCo" | "GdFl" | "PtFl" => self.read_vector_fill(info, key)?,
            "vscg" | "vstk" => self.read_vector_stroke(info, length)?,
            "vmsk" | "vsms" => self.read_vector_mask(info, length)?,
            "vogk" => self.read_vector_origination(info, length)?,
            "lrFX" | "lfx2" => self.read_layer_effects(info, key, length)?,
            "PlLd" | "SoLd" => self.read_placed_layer(info, key, length)?,
            "artb" | "artd" => self.read_artboard(info, key, length)?,
            "sn2P" => self.read_using_aligned_rendering(info)?,
            "shmd" | "cust" => self.read_metadata(info, length)?,
            _ => {
                // Store unknown sections
                let data = self.read_bytes(length)?;
                info.unknown.insert(key.to_string(), data);
            }
        }
        
        // Ensure we consumed exactly the right amount
        let consumed = (self.offset - start_offset) as usize;
        if consumed < length {
            self.skip_bytes(length - consumed)?;
        }
        
        Ok(())
    }

    /// Read unicode layer name (luni)
    fn read_unicode_layer_name(&mut self, info: &mut LayerAdditionalInfo) -> Result<()> {
        info.unicode_name = Some(self.read_unicode_string()?);
        Ok(())
    }

    /// Read layer ID (lyid)
    fn read_layer_id(&mut self, info: &mut LayerAdditionalInfo) -> Result<()> {
        info.layer_id = Some(self.read_u32()?);
        Ok(())
    }

    /// Read layer color (lclr)
    fn read_layer_color(&mut self, info: &mut LayerAdditionalInfo) -> Result<()> {
        let color_value = self.read_u16()?;
        self.skip_bytes(6)?; // Skip padding
        info.layer_color = Some(LayerColor::from_u16(color_value)?);
        Ok(())
    }

    /// Read section divider (lsct/lsdk)
    fn read_section_divider(&mut self, info: &mut LayerAdditionalInfo, length: usize) -> Result<()> {
        let divider_type = self.read_u32()?;
        
        let mut blend_mode = None;
        let mut sub_type = None;
        
        if length >= 12 {
            let sig = self.read_signature()?;
            if sig != "8BIM" {
                return Err(PsdError::InvalidFormat(format!("Invalid section divider signature: {}", sig)));
            }
            blend_mode = Some(self.read_signature()?);
        }
        
        if length >= 16 {
            sub_type = Some(self.read_u32()?);
        }
        
        info.section_divider = Some(SectionDivider {
            divider_type,
            blend_mode,
            sub_type,
        });
        
        Ok(())
    }

    /// Read blend clipped elements (clbl)
    fn read_blend_clipped(&mut self, info: &mut LayerAdditionalInfo) -> Result<()> {
        info.blend_clipped = Some(self.read_u8()? != 0);
        Ok(())
    }

    /// Read blend interior elements (infx)
    fn read_blend_interior(&mut self, info: &mut LayerAdditionalInfo) -> Result<()> {
        info.blend_interior = Some(self.read_u8()? != 0);
        Ok(())
    }

    /// Read knockout mode (knko)
    fn read_knockout(&mut self, info: &mut LayerAdditionalInfo) -> Result<()> {
        info.knockout = Some(self.read_u8()? != 0);
        Ok(())
    }

    /// Read protected flags (lspf)
    fn read_protected_flags(&mut self, info: &mut LayerAdditionalInfo, length: usize) -> Result<()> {
        let flags = self.read_u32()?;
        
        let protected = ProtectedFlags {
            transparency: (flags & 0x01) != 0,
            composite: (flags & 0x02) != 0,
            position: (flags & 0x04) != 0,
            artboards: if length >= 8 {
                Some((flags & 0x08) != 0)
            } else {
                None
            },
        };
        
        info.protected = Some(protected);
        Ok(())
    }

    /// Read layer name source (lnsr)
    fn read_name_source(&mut self, info: &mut LayerAdditionalInfo) -> Result<()> {
        info.name_source = Some(self.read_signature()?);
        Ok(())
    }

    /// Read text layer data (TySh)
    fn read_text_layer(&mut self, info: &mut LayerAdditionalInfo, _length: usize) -> Result<()> {
        let version = self.read_i16()?;
        if version != 1 {
            return Err(PsdError::InvalidFormat(format!("Invalid TySh version: {}", version)));
        }
        
        // Read transform matrix (6 doubles)
        let mut transform = Vec::with_capacity(6);
        for _ in 0..6 {
            transform.push(self.read_f64()?);
        }
        
        // Read text version
        let text_version = self.read_i16()? as u16;
        if text_version != 50 {
            return Err(PsdError::InvalidFormat(format!("Invalid text version: {}", text_version)));
        }
        
        // Read text descriptor
        let text_descriptor = self.read_version_and_descriptor()?;
        
        // Read warp version
        let warp_version = self.read_i16()? as u16;
        if warp_version != 1 {
            return Err(PsdError::InvalidFormat(format!("Invalid warp version: {}", warp_version)));
        }
        
        // Read warp descriptor
        let warp_descriptor = self.read_version_and_descriptor()?;
        
        // Read bounds
        let left = self.read_f32()?;
        let top = self.read_f32()?;
        let right = self.read_f32()?;
        let bottom = self.read_f32()?;
        
        // Extract text from descriptor
        let text = text_descriptor.items.get("Txt ")
            .and_then(|v| if let DescriptorValue::Text(s) = v { Some(s.clone()) } else { None })
            .unwrap_or_default();
        
        info.text = Some(TextLayerData {
            transform,
            text: text.replace('\r', "\n"),
            text_version,
            descriptor_version: 1,
            text_data: Some(text_descriptor),
            warp_version,
            warp_data: Some(warp_descriptor),
            left,
            top,
            right,
            bottom,
        });
        
        Ok(())
    }

    /// Read vector fill (SoCo/GdFl/PtFl)
    fn read_vector_fill(&mut self, info: &mut LayerAdditionalInfo, key: &str) -> Result<()> {
        let descriptor = self.read_version_and_descriptor()?;
        
        let fill_type = match key {
            "SoCo" => "color",
            "GdFl" => "gradient",
            "PtFl" => "pattern",
            _ => "unknown",
        };
        
        info.vector_fill = Some(VectorFill {
            fill_type: fill_type.to_string(),
            data: descriptor,
        });
        
        Ok(())
    }

    /// Read vector stroke (vscg/vstk)
    fn read_vector_stroke(&mut self, info: &mut LayerAdditionalInfo, _length: usize) -> Result<()> {
        let version = self.read_u32()?;
        let descriptor = self.read_version_and_descriptor()?;
        
        info.vector_stroke = Some(VectorStroke {
            version,
            descriptor,
        });
        
        Ok(())
    }

    /// Read vector mask (vmsk/vsms)
    fn read_vector_mask(&mut self, info: &mut LayerAdditionalInfo, length: usize) -> Result<()> {
        let start_offset = self.offset;
        let version = self.read_u32()?;
        let flags = self.read_u32()?;
        
        let invert = (flags & 1) != 0;
        let not_link = (flags & 2) != 0;
        let disable = (flags & 4) != 0;
        
        let mut paths = Vec::new();
        
        // Read path records
        while (self.offset - start_offset) < length as u64 {
            let selector = self.read_u16()?;
            
            match selector {
                0 | 3 => {
                    // Closed or open subpath length record
                    let num_points = self.read_u16()?;
                    self.skip_bytes(22)?; // Skip rest of record
                    
                    let path_type = if selector == 0 {
                        PathType::Closed
                    } else {
                        PathType::Open
                    };
                    
                    let mut points = Vec::new();
                    
                    // Read bezier knot records
                    for _ in 0..num_points {
                        let knot_selector = self.read_u16()?;
                        if knot_selector != 1 && knot_selector != 2 {
                            return Err(PsdError::InvalidFormat(format!("Invalid knot selector: {}", knot_selector)));
                        }
                        
                        let linked = knot_selector == 1;
                        
                        // Read points (vertical, horizontal order)
                        let vert_y = self.read_fixed_point_path_32()?;
                        let hor_y = self.read_fixed_point_path_32()?;
                        let vert_anchor = self.read_fixed_point_path_32()?;
                        let hor_anchor = self.read_fixed_point_path_32()?;
                        let vert_forward = self.read_fixed_point_path_32()?;
                        let hor_forward = self.read_fixed_point_path_32()?;
                        
                        points.push(PathPoint {
                            anchor: Point { x: hor_anchor, y: vert_anchor },
                            forward: Point { x: hor_forward, y: vert_forward },
                            backward: Point { x: hor_y, y: vert_y },
                            linked,
                        });
                    }
                    
                    paths.push(VectorPath {
                        path_type,
                        initial_fill_rule: None,
                        clipboard_bounds: None,
                        points,
                    });
                }
                6 => {
                    // Path fill rule
                    self.skip_bytes(24)?;
                }
                7 => {
                    // Clipboard record
                    self.skip_bytes(24)?;
                }
                8 => {
                    // Initial fill rule
                    self.skip_bytes(24)?;
                }
                _ => {
                    // Unknown selector, skip
                    self.skip_bytes(24)?;
                }
            }
        }
        
        info.vector_mask = Some(VectorMask {
            version,
            invert,
            not_link,
            disable,
            paths,
        });
        
        Ok(())
    }

    /// Read vector origination data (vogk)
    fn read_vector_origination(&mut self, info: &mut LayerAdditionalInfo, length: usize) -> Result<()> {
        info.vector_origination = Some(self.read_bytes(length)?);
        Ok(())
    }

    /// Read layer effects (lrFX/lfx2)
    fn read_layer_effects(&mut self, info: &mut LayerAdditionalInfo, key: &str, _length: usize) -> Result<()> {
        let version = self.read_u32()?;
        
        let descriptor = if key == "lfx2" {
            Some(self.read_version_and_descriptor()?)
        } else {
            None
        };
        
        info.layer_effects = Some(LayerEffects {
            version,
            descriptor,
        });
        
        Ok(())
    }

    /// Read placed layer (PlLd/SoLd)
    fn read_placed_layer(&mut self, info: &mut LayerAdditionalInfo, _key: &str, length: usize) -> Result<()> {
        let start_offset = self.offset;
        
        // Read type (may be 'plcL' or 'sold')
        let placed_type = self.read_signature()?;
        let version = self.read_u32()?;
        
        // Read UUID
        let id_length = 32;
        let id_bytes = self.read_bytes(id_length)?;
        let id = String::from_utf8_lossy(&id_bytes).to_string();
        
        // Read page number
        let page = self.read_i32()?;
        let total_pages = self.read_i32()?;
        let anti_alias = self.read_i32()?;
        let placed_type_val = self.read_i32()?;
        
        // Read transform (8 doubles)
        let mut transform = Vec::with_capacity(8);
        for _ in 0..8 {
            transform.push(self.read_f64()?);
        }
        
        // Read warp descriptor if there's more data
        let warp = if (self.offset - start_offset) < length as u64 {
            Some(self.read_version_and_descriptor()?)
        } else {
            None
        };
        
        info.placed_layer = Some(PlacedLayer {
            id,
            page: Some(page),
            total_pages: Some(total_pages),
            anti_alias_policy: Some(anti_alias),
            placed_layer_type: Some(placed_type_val),
            transform,
            warp,
            placed: None,
        });
        
        Ok(())
    }

    /// Read artboard data (artb/artd)
    fn read_artboard(&mut self, info: &mut LayerAdditionalInfo, _key: &str, _length: usize) -> Result<()> {
        let descriptor = self.read_version_and_descriptor()?;
        
        // Extract artboard rectangle
        let rect = if let Some(DescriptorValue::Descriptor(rect_desc)) = descriptor.items.get("artboardRect") {
            let top = rect_desc.items.get("Top ")
                .and_then(|v| if let DescriptorValue::Double(d) = v { Some(*d) } else { None })
                .unwrap_or(0.0);
            let left = rect_desc.items.get("Left")
                .and_then(|v| if let DescriptorValue::Double(d) = v { Some(*d) } else { None })
                .unwrap_or(0.0);
            let bottom = rect_desc.items.get("Btom")
                .and_then(|v| if let DescriptorValue::Double(d) = v { Some(*d) } else { None })
                .unwrap_or(0.0);
            let right = rect_desc.items.get("Rght")
                .and_then(|v| if let DescriptorValue::Double(d) = v { Some(*d) } else { None })
                .unwrap_or(0.0);
            
            Bounds { top, left, bottom, right }
        } else {
            Bounds { top: 0.0, left: 0.0, bottom: 0.0, right: 0.0 }
        };
        
        info.artboard = Some(ArtboardData {
            rect,
            preset_name: None,
            color: None,
            background_type: None,
        });
        
        Ok(())
    }

    /// Read using aligned rendering (sn2P)
    fn read_using_aligned_rendering(&mut self, info: &mut LayerAdditionalInfo) -> Result<()> {
        info.using_aligned_rendering = Some(self.read_u8()? != 0);
        Ok(())
    }

    /// Read metadata (shmd/cust)
    fn read_metadata(&mut self, info: &mut LayerAdditionalInfo, _length: usize) -> Result<()> {
        let _count = self.read_u32()?;
        let descriptor = self.read_version_and_descriptor()?;
        
        info.metadata = Some(Metadata { descriptor });
        Ok(())
    }
}

impl PsdWriter {
    /// Write layer additional info section
    pub fn write_additional_info(&mut self, key: &str, info: &LayerAdditionalInfo) -> Result<usize> {
        let mut temp_writer = PsdWriter::new(1024);
        
        match key {
            "luni" => {
                if let Some(ref name) = info.unicode_name {
                    temp_writer.write_unicode_string(name)?;
                }
            }
            "lyid" => {
                if let Some(id) = info.layer_id {
                    temp_writer.write_u32(id)?;
                }
            }
            "lclr" => {
                if let Some(color) = info.layer_color {
                    temp_writer.write_u16(color as u16)?;
                    temp_writer.write_zeros(6)?;
                }
            }
            "lsct" => {
                if let Some(ref divider) = info.section_divider {
                    temp_writer.write_u32(divider.divider_type)?;
                    if let Some(ref blend_mode) = divider.blend_mode {
                        temp_writer.write_signature("8BIM")?;
                        temp_writer.write_signature(blend_mode)?;
                    }
                    if let Some(sub_type) = divider.sub_type {
                        temp_writer.write_u32(sub_type)?;
                    }
                }
            }
            "clbl" => {
                if let Some(blend_clipped) = info.blend_clipped {
                    temp_writer.write_u8(if blend_clipped { 1 } else { 0 })?;
                }
            }
            "infx" => {
                if let Some(blend_interior) = info.blend_interior {
                    temp_writer.write_u8(if blend_interior { 1 } else { 0 })?;
                }
            }
            "knko" => {
                if let Some(knockout) = info.knockout {
                    temp_writer.write_u8(if knockout { 1 } else { 0 })?;
                }
            }
            "lspf" => {
                if let Some(ref protected) = info.protected {
                    let mut flags = 0u32;
                    if protected.transparency { flags |= 0x01; }
                    if protected.composite { flags |= 0x02; }
                    if protected.position { flags |= 0x04; }
                    if protected.artboards.unwrap_or(false) { flags |= 0x08; }
                    temp_writer.write_u32(flags)?;
                }
            }
            "lnsr" => {
                if let Some(ref source) = info.name_source {
                    temp_writer.write_signature(source)?;
                }
            }
            _ => {
                // Write unknown sections
                if let Some(data) = info.unknown.get(key) {
                    temp_writer.write_bytes(data)?;
                }
            }
        }
        
        let data = temp_writer.get_buffer();
        let length = data.len();
        
        if length > 0 {
            self.write_bytes(data)?;
        }
        
        Ok(length)
    }
}

/// Read all additional info sections for a layer
pub fn read_layer_additional_info<R: Read + Seek>(
    reader: &mut PsdReader<R>,
    length: usize,
) -> Result<LayerAdditionalInfo> {
    let mut info = LayerAdditionalInfo::default();
    let start_offset = reader.offset;
    
    while (reader.offset - start_offset) < length as u64 {
        let signature = reader.read_signature()?;
        if signature != "8BIM" && signature != "8B64" {
            return Err(PsdError::InvalidFormat(format!("Invalid additional info signature: {}", signature)));
        }
        
        let key = reader.read_signature()?;
        let data_length = if signature == "8B64" {
            // Read 64-bit length
            let high = reader.read_u32()?;
            if high != 0 {
                return Err(PsdError::InvalidFormat("Resource size above 4GB".to_string()));
            }
            reader.read_u32()? as usize
        } else {
            reader.read_u32()? as usize
        };
        
        reader.read_additional_info(&key, data_length, &mut info)?;
        
        // Some sections need padding to even boundary
        if data_length % 2 != 0 && key != "vmsk" && key != "vsms" {
            reader.skip_bytes(1)?;
        }
    }
    
    Ok(info)
}

/// Write all additional info sections for a layer
pub fn write_layer_additional_info(
    writer: &mut PsdWriter,
    info: &LayerAdditionalInfo,
) -> Result<()> {
    let sections = vec![
        "luni", "lyid", "lclr", "lsct", "clbl", "infx", "knko", "lspf", "lnsr",
    ];
    
    for key in sections {
        // Create temporary writer for section data
        let mut temp_writer = PsdWriter::new(1024);
        let length = temp_writer.write_additional_info(key, info)?;
        
        if length > 0 {
            // Write section header
            writer.write_signature("8BIM")?;
            writer.write_signature(key)?;
            writer.write_u32(length as u32)?;
            // Write data
            writer.write_bytes(temp_writer.get_buffer())?;
            
            // Pad to even boundary
            if length % 2 != 0 {
                writer.write_u8(0)?;
            }
        }
    }
    
    // Write unknown sections
    for (key, data) in &info.unknown {
        writer.write_signature("8BIM")?;
        writer.write_signature(key)?;
        writer.write_u32(data.len() as u32)?;
        writer.write_bytes(data)?;
        
        if data.len() % 2 != 0 {
            writer.write_u8(0)?;
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layer_id_roundtrip() {
        let mut info = LayerAdditionalInfo::default();
        info.layer_id = Some(12345);
        
        let mut writer = PsdWriter::new(128);
        let length = writer.write_additional_info("lyid", &info).unwrap();
        
        assert_eq!(length, 4);
        
        let buffer = writer.into_buffer();
        let cursor = std::io::Cursor::new(buffer);
        let mut reader = PsdReader::new(cursor, Default::default());
        
        let mut read_info = LayerAdditionalInfo::default();
        reader.read_additional_info("lyid", length, &mut read_info).unwrap();
        
        assert_eq!(read_info.layer_id, Some(12345));
    }

    #[test]
    fn test_layer_color_roundtrip() {
        let mut info = LayerAdditionalInfo::default();
        info.layer_color = Some(LayerColor::Blue);
        
        let mut writer = PsdWriter::new(128);
        let length = writer.write_additional_info("lclr", &info).unwrap();
        
        assert_eq!(length, 8);
        
        let buffer = writer.into_buffer();
        let cursor = std::io::Cursor::new(buffer);
        let mut reader = PsdReader::new(cursor, Default::default());
        
        let mut read_info = LayerAdditionalInfo::default();
        reader.read_additional_info("lclr", length, &mut read_info).unwrap();
        
        assert_eq!(read_info.layer_color, Some(LayerColor::Blue));
    }

    #[test]
    fn test_protected_flags_roundtrip() {
        let mut info = LayerAdditionalInfo::default();
        info.protected = Some(ProtectedFlags {
            transparency: true,
            composite: false,
            position: true,
            artboards: Some(false),
        });
        
        let mut writer = PsdWriter::new(128);
        let length = writer.write_additional_info("lspf", &info).unwrap();
        
        let buffer = writer.into_buffer();
        let cursor = std::io::Cursor::new(buffer);
        let mut reader = PsdReader::new(cursor, Default::default());
        
        let mut read_info = LayerAdditionalInfo::default();
        reader.read_additional_info("lspf", length, &mut read_info).unwrap();
        
        let protected = read_info.protected.unwrap();
        assert_eq!(protected.transparency, true);
        assert_eq!(protected.composite, false);
        assert_eq!(protected.position, true);
    }
}
