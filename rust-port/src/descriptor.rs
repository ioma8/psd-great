//! Descriptor parsing and writing implementation
//!
//! Handles Adobe Photoshop descriptor structures used throughout PSD files.
//! Descriptors are key-value data structures with typed values.

use crate::error::{PsdError, Result};
use crate::reader::PsdReader;
use crate::writer::PsdWriter;
use std::collections::HashMap;
use std::io::{Read, Seek};

/// Descriptor value types
#[derive(Debug, Clone, PartialEq)]
pub enum DescriptorValue {
    Double(f64),
    Float(f32),
    Integer(i32),
    LargeInteger { high: u32, low: u32 },
    Boolean(bool),
    Text(String),
    Enum { enum_type: String, value: String },
    Class { name: String, class_id: String },
    Reference(Vec<ReferenceItem>),
    Descriptor(Descriptor),
    List(Vec<DescriptorValue>),
    RawData(Vec<u8>),
    UnitFloat { units: String, value: f64 },
    UnitDouble { units: String, value: f64 },
    Property(String),
    Alias(Vec<u8>),
    Path(Vec<PathPoint>),
}

/// Descriptor structure
#[derive(Debug, Clone, PartialEq)]
pub struct Descriptor {
    pub name: String,
    pub class_id: String,
    pub items: HashMap<String, DescriptorValue>,
}

/// Reference item types
#[derive(Debug, Clone, PartialEq)]
pub enum ReferenceItem {
    Property { name: String, class_id: String, key_id: String },
    Class { name: String, class_id: String },
    EnumeratedReference { name: String, class_id: String, type_id: String, enum_value: String },
    Offset { name: String, class_id: String, offset: i32 },
    Identifier { name: String, class_id: String, id: i32 },
    Index { name: String, class_id: String, index: i32 },
    Name { name: String, class_id: String, value: String },
}

/// Path point for bezier paths
#[derive(Debug, Clone, PartialEq)]
pub struct PathPoint {
    pub horizontal: f64,
    pub vertical: f64,
}

/// Units map for unit types
pub fn units_map(code: &str) -> Option<&'static str> {
    match code {
        "#Ang" => Some("Angle"),
        "#Rsl" => Some("Density"),
        "#Rlt" => Some("Distance"),
        "#Nne" => Some("None"),
        "#Prc" => Some("Percent"),
        "#Pxl" => Some("Pixels"),
        "#Mlm" => Some("Millimeters"),
        "#Pnt" => Some("Points"),
        "RrPi" => Some("Picas"),
        "RrIn" => Some("Inches"),
        "RrCm" => Some("Centimeters"),
        _ => None,
    }
}

/// Reverse units map
pub fn units_code(units: &str) -> Option<&'static str> {
    match units {
        "Angle" => Some("#Ang"),
        "Density" => Some("#Rsl"),
        "Distance" => Some("#Rlt"),
        "None" => Some("#Nne"),
        "Percent" => Some("#Prc"),
        "Pixels" => Some("#Pxl"),
        "Millimeters" => Some("#Mlm"),
        "Points" => Some("#Pnt"),
        "Picas" => Some("RrPi"),
        "Inches" => Some("RrIn"),
        "Centimeters" => Some("RrCm"),
        _ => None,
    }
}

impl<R: Read + Seek> PsdReader<R> {
    /// Read ASCII string or class ID
    pub fn read_ascii_string_or_class_id(&mut self) -> Result<String> {
        let length = self.read_i32()?;
        if length == 0 {
            self.read_signature()
        } else {
            self.read_ascii_string(length as usize)
        }
    }

    /// Read class structure (unicode name + class ID)
    pub fn read_class_structure(&mut self) -> Result<(String, String)> {
        let name = self.read_unicode_string()?;
        let class_id = self.read_ascii_string_or_class_id()?;
        Ok((name, class_id))
    }

    /// Read descriptor structure
    pub fn read_descriptor_structure(&mut self) -> Result<Descriptor> {
        let (name, class_id) = self.read_class_structure()?;
        let item_count = self.read_u32()?;
        
        let mut items = HashMap::new();
        for _ in 0..item_count {
            let key = self.read_ascii_string_or_class_id()?;
            let ostype = self.read_signature()?;
            let value = self.read_ostype(&ostype)?;
            items.insert(key, value);
        }

        Ok(Descriptor {
            name,
            class_id,
            items,
        })
    }

    /// Read OSType value based on type signature
    pub fn read_ostype(&mut self, ostype: &str) -> Result<DescriptorValue> {
        match ostype {
            "obj " => {
                // Reference
                let items = self.read_reference_structure()?;
                Ok(DescriptorValue::Reference(items))
            }
            "Objc" | "GlbO" => {
                // Descriptor or GlobalObject
                let desc = self.read_descriptor_structure()?;
                Ok(DescriptorValue::Descriptor(desc))
            }
            "VlLs" => {
                // List
                let count = self.read_i32()?;
                let mut items = Vec::new();
                for _ in 0..count {
                    let item_type = self.read_signature()?;
                    items.push(self.read_ostype(&item_type)?);
                }
                Ok(DescriptorValue::List(items))
            }
            "doub" => {
                // Double
                Ok(DescriptorValue::Double(self.read_f64()?))
            }
            "DBL " => {
                // Float (32-bit)
                Ok(DescriptorValue::Float(self.read_f32()?))
            }
            "UntF" => {
                // Unit float
                let units = self.read_signature()?;
                let value = self.read_f64()?;
                Ok(DescriptorValue::UnitDouble {
                    units: units_map(&units).unwrap_or(&units).to_string(),
                    value,
                })
            }
            "UnFl" => {
                // Unit float (32-bit)
                let units = self.read_signature()?;
                let value = self.read_f32()? as f64;
                Ok(DescriptorValue::UnitFloat {
                    units: units_map(&units).unwrap_or(&units).to_string(),
                    value,
                })
            }
            "long" => {
                // Integer
                Ok(DescriptorValue::Integer(self.read_i32()?))
            }
            "comp" => {
                // Large integer (64-bit split into high/low)
                let low = self.read_u32()?;
                let high = self.read_u32()?;
                Ok(DescriptorValue::LargeInteger { high, low })
            }
            "bool" => {
                // Boolean
                Ok(DescriptorValue::Boolean(self.read_u8()? != 0))
            }
            "TEXT" => {
                // Unicode string
                Ok(DescriptorValue::Text(self.read_unicode_string()?))
            }
            "enum" => {
                // Enumerated value
                let enum_type = self.read_ascii_string_or_class_id()?;
                let value = self.read_ascii_string_or_class_id()?;
                Ok(DescriptorValue::Enum { enum_type, value })
            }
            "tdta" => {
                // Raw data
                let length = self.read_i32()? as usize;
                Ok(DescriptorValue::RawData(self.read_bytes(length)?))
            }
            "ObAr" => {
                // Object array
                let _count = self.read_i32()?;
                let (_name, _class_id) = self.read_class_structure()?;
                let item_count = self.read_i32()?;
                
                let mut items = Vec::new();
                for _ in 0..item_count {
                    let desc = self.read_descriptor_structure()?;
                    items.push(DescriptorValue::Descriptor(desc));
                }
                Ok(DescriptorValue::List(items))
            }
            "type" | "GlbC" => {
                // Class reference
                let (name, class_id) = self.read_class_structure()?;
                Ok(DescriptorValue::Class { name, class_id })
            }
            "alis" => {
                // Alias (file path)
                let length = self.read_i32()? as usize;
                Ok(DescriptorValue::Alias(self.read_bytes(length)?))
            }
            "Pth " => {
                // Path
                let _path_size = self.read_i32()?;
                let _path_length = self.read_i32()?;
                let path_type = self.read_signature()?;
                
                if path_type != "PtLs" {
                    return Err(PsdError::InvalidFormat(format!("Invalid path type: {}", path_type)));
                }
                
                let point_count = self.read_i32()?;
                let mut points = Vec::new();
                
                for _ in 0..point_count {
                    let (_name, _class_id) = self.read_class_structure()?;
                    let _item_count = self.read_u32()?;
                    
                    // Read Hrzn
                    let _hrzn_key = self.read_ascii_string_or_class_id()?;
                    let _hrzn_type = self.read_signature()?;
                    let horizontal = self.read_f64()?;
                    
                    // Read Vrtc
                    let _vrtc_key = self.read_ascii_string_or_class_id()?;
                    let _vrtc_type = self.read_signature()?;
                    let vertical = self.read_f64()?;
                    
                    points.push(PathPoint { horizontal, vertical });
                }
                
                Ok(DescriptorValue::Path(points))
            }
            _ => {
                Err(PsdError::UnsupportedFeature(format!("Unknown OSType: {}", ostype)))
            }
        }
    }

    /// Read reference structure
    pub fn read_reference_structure(&mut self) -> Result<Vec<ReferenceItem>> {
        let _count = self.read_i32()?;
        let item_count = self.read_i32()?;
        
        let mut items = Vec::new();
        for _ in 0..item_count {
            let ostype = self.read_signature()?;
            
            let item = match ostype.as_str() {
                "prop" => {
                    let (name, class_id) = self.read_class_structure()?;
                    let key_id = self.read_ascii_string_or_class_id()?;
                    ReferenceItem::Property { name, class_id, key_id }
                }
                "Clss" => {
                    let (name, class_id) = self.read_class_structure()?;
                    ReferenceItem::Class { name, class_id }
                }
                "Enmr" => {
                    let (name, class_id) = self.read_class_structure()?;
                    let type_id = self.read_ascii_string_or_class_id()?;
                    let enum_value = self.read_ascii_string_or_class_id()?;
                    ReferenceItem::EnumeratedReference { name, class_id, type_id, enum_value }
                }
                "rele" => {
                    let (name, class_id) = self.read_class_structure()?;
                    let offset = self.read_i32()?;
                    ReferenceItem::Offset { name, class_id, offset }
                }
                "Idnt" => {
                    let (name, class_id) = self.read_class_structure()?;
                    let id = self.read_i32()?;
                    ReferenceItem::Identifier { name, class_id, id }
                }
                "indx" => {
                    let (name, class_id) = self.read_class_structure()?;
                    let index = self.read_i32()?;
                    ReferenceItem::Index { name, class_id, index }
                }
                "name" => {
                    let (name, class_id) = self.read_class_structure()?;
                    let value = self.read_unicode_string()?;
                    ReferenceItem::Name { name, class_id, value }
                }
                _ => return Err(PsdError::UnsupportedFeature(format!("Unknown reference type: {}", ostype))),
            };
            
            items.push(item);
        }
        
        Ok(items)
    }

    /// Read version and descriptor (common pattern in PSD)
    pub fn read_version_and_descriptor(&mut self) -> Result<Descriptor> {
        let _version = self.read_u32()?;
        self.read_descriptor_structure()
    }
}

impl PsdWriter {
    /// Write ASCII string or class ID
    pub fn write_ascii_string_or_class_id(&mut self, value: &str) -> Result<()> {
        // Special cases that should always be written as strings
        let always_string = ["warp", "time", "hold", "list"];
        
        if value.len() == 4 && !always_string.contains(&value) {
            // Write as class ID
            self.write_i32(0)?;
            self.write_signature(value)?;
        } else {
            // Write as ASCII string
            self.write_i32(value.len() as i32)?;
            for ch in value.chars() {
                self.write_u8(ch as u8)?;
            }
        }
        Ok(())
    }

    /// Write class structure
    pub fn write_class_structure(&mut self, name: &str, class_id: &str) -> Result<()> {
        self.write_unicode_string_with_padding(name)?;
        self.write_ascii_string_or_class_id(class_id)?;
        Ok(())
    }

    /// Write descriptor structure
    pub fn write_descriptor_structure(&mut self, desc: &Descriptor) -> Result<()> {
        self.write_class_structure(&desc.name, &desc.class_id)?;
        self.write_u32(desc.items.len() as u32)?;
        
        // Sort keys for consistent output
        let mut keys: Vec<_> = desc.items.keys().collect();
        keys.sort();
        
        for key in keys {
            let value = &desc.items[key];
            self.write_ascii_string_or_class_id(key)?;
            
            // Determine type signature
            let type_sig = match value {
                DescriptorValue::Double(_) => "doub",
                DescriptorValue::Float(_) => "DBL ",
                DescriptorValue::Integer(_) => "long",
                DescriptorValue::LargeInteger { .. } => "comp",
                DescriptorValue::Boolean(_) => "bool",
                DescriptorValue::Text(_) => "TEXT",
                DescriptorValue::Enum { .. } => "enum",
                DescriptorValue::Class { .. } => "type",
                DescriptorValue::Reference(_) => "obj ",
                DescriptorValue::Descriptor(_) => "Objc",
                DescriptorValue::List(_) => "VlLs",
                DescriptorValue::RawData(_) => "tdta",
                DescriptorValue::UnitFloat { .. } => "UnFl",
                DescriptorValue::UnitDouble { .. } => "UntF",
                DescriptorValue::Property(_) => "prop",
                DescriptorValue::Alias(_) => "alis",
                DescriptorValue::Path(_) => "Pth ",
            };
            
            self.write_signature(type_sig)?;
            self.write_ostype(value)?;
        }
        
        Ok(())
    }

    /// Write OSType value
    pub fn write_ostype(&mut self, value: &DescriptorValue) -> Result<()> {
        match value {
            DescriptorValue::Double(v) => self.write_f64(*v)?,
            DescriptorValue::Float(v) => self.write_f32(*v)?,
            DescriptorValue::Integer(v) => self.write_i32(*v)?,
            DescriptorValue::LargeInteger { high, low } => {
                self.write_u32(*low)?;
                self.write_u32(*high)?;
            }
            DescriptorValue::Boolean(v) => self.write_u8(if *v { 1 } else { 0 })?,
            DescriptorValue::Text(s) => self.write_unicode_string(s)?,
            DescriptorValue::Enum { enum_type, value } => {
                self.write_ascii_string_or_class_id(enum_type)?;
                self.write_ascii_string_or_class_id(value)?;
            }
            DescriptorValue::Class { name, class_id } => {
                self.write_class_structure(name, class_id)?;
            }
            DescriptorValue::Reference(items) => {
                self.write_reference_structure(items)?;
            }
            DescriptorValue::Descriptor(desc) => {
                self.write_descriptor_structure(desc)?;
            }
            DescriptorValue::List(items) => {
                self.write_i32(items.len() as i32)?;
                for item in items {
                    let type_sig = match item {
                        DescriptorValue::Double(_) => "doub",
                        DescriptorValue::Float(_) => "DBL ",
                        DescriptorValue::Integer(_) => "long",
                        DescriptorValue::Boolean(_) => "bool",
                        DescriptorValue::Text(_) => "TEXT",
                        DescriptorValue::Enum { .. } => "enum",
                        DescriptorValue::Descriptor(_) => "Objc",
                        DescriptorValue::UnitDouble { .. } => "UntF",
                        _ => "Objc",
                    };
                    self.write_signature(type_sig)?;
                    self.write_ostype(item)?;
                }
            }
            DescriptorValue::RawData(data) => {
                self.write_i32(data.len() as i32)?;
                self.write_bytes(data)?;
            }
            DescriptorValue::UnitFloat { units, value } => {
                let code = units_code(units).unwrap_or("#Pxl");
                self.write_signature(code)?;
                self.write_f32(*value as f32)?;
            }
            DescriptorValue::UnitDouble { units, value } => {
                let code = units_code(units).unwrap_or("#Pxl");
                self.write_signature(code)?;
                self.write_f64(*value)?;
            }
            DescriptorValue::Alias(data) => {
                self.write_i32(data.len() as i32)?;
                self.write_bytes(data)?;
            }
            DescriptorValue::Path(points) => {
                // Calculate path size
                let path_size = 4 + 4 + (points.len() as i32) * 60; // Approximate
                self.write_i32(path_size)?;
                self.write_i32(points.len() as i32)?;
                self.write_signature("PtLs")?;
                self.write_i32(points.len() as i32)?;
                
                for point in points {
                    self.write_class_structure("", "Pnt ")?;
                    self.write_u32(2)?; // 2 items: Hrzn and Vrtc
                    
                    self.write_ascii_string_or_class_id("Hrzn")?;
                    self.write_signature("doub")?;
                    self.write_f64(point.horizontal)?;
                    
                    self.write_ascii_string_or_class_id("Vrtc")?;
                    self.write_signature("doub")?;
                    self.write_f64(point.vertical)?;
                }
            }
            _ => return Err(PsdError::UnsupportedFeature("Unsupported descriptor value type".to_string())),
        }
        Ok(())
    }

    /// Write reference structure
    pub fn write_reference_structure(&mut self, items: &[ReferenceItem]) -> Result<()> {
        // Calculate total size (approximate)
        let size = items.len() * 20;
        self.write_i32(size as i32)?;
        self.write_i32(items.len() as i32)?;
        
        for item in items {
            match item {
                ReferenceItem::Property { name, class_id, key_id } => {
                    self.write_signature("prop")?;
                    self.write_class_structure(name, class_id)?;
                    self.write_ascii_string_or_class_id(key_id)?;
                }
                ReferenceItem::Class { name, class_id } => {
                    self.write_signature("Clss")?;
                    self.write_class_structure(name, class_id)?;
                }
                ReferenceItem::EnumeratedReference { name, class_id, type_id, enum_value } => {
                    self.write_signature("Enmr")?;
                    self.write_class_structure(name, class_id)?;
                    self.write_ascii_string_or_class_id(type_id)?;
                    self.write_ascii_string_or_class_id(enum_value)?;
                }
                ReferenceItem::Offset { name, class_id, offset } => {
                    self.write_signature("rele")?;
                    self.write_class_structure(name, class_id)?;
                    self.write_i32(*offset)?;
                }
                ReferenceItem::Identifier { name, class_id, id } => {
                    self.write_signature("Idnt")?;
                    self.write_class_structure(name, class_id)?;
                    self.write_i32(*id)?;
                }
                ReferenceItem::Index { name, class_id, index } => {
                    self.write_signature("indx")?;
                    self.write_class_structure(name, class_id)?;
                    self.write_i32(*index)?;
                }
                ReferenceItem::Name { name, class_id, value } => {
                    self.write_signature("name")?;
                    self.write_class_structure(name, class_id)?;
                    self.write_unicode_string(value)?;
                }
            }
        }
        
        Ok(())
    }

    /// Write version and descriptor
    pub fn write_version_and_descriptor(&mut self, version: u32, desc: &Descriptor) -> Result<()> {
        self.write_u32(version)?;
        self.write_descriptor_structure(desc)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_descriptor_roundtrip() {
        let mut desc = Descriptor {
            name: "Test".to_string(),
            class_id: "TEST".to_string(),
            items: HashMap::new(),
        };
        
        desc.items.insert("long".to_string(), DescriptorValue::Integer(42));
        desc.items.insert("doub".to_string(), DescriptorValue::Double(3.14));
        desc.items.insert("bool".to_string(), DescriptorValue::Boolean(true));
        desc.items.insert("TEXT".to_string(), DescriptorValue::Text("Hello".to_string()));
        
        // Write
        let mut writer = PsdWriter::new(1024);
        writer.write_descriptor_structure(&desc).unwrap();
        let buffer = writer.into_buffer();
        
        // Read
        let cursor = Cursor::new(buffer);
        let mut reader = PsdReader::new(cursor, Default::default());
        let read_desc = reader.read_descriptor_structure().unwrap();
        
        assert_eq!(read_desc.name, desc.name);
        assert_eq!(read_desc.class_id, desc.class_id);
        assert_eq!(read_desc.items.len(), desc.items.len());
    }

    #[test]
    fn test_unit_float() {
        let mut writer = PsdWriter::new(128);
        let value = DescriptorValue::UnitDouble {
            units: "Pixels".to_string(),
            value: 100.0,
        };
        
        writer.write_signature("UntF").unwrap();
        writer.write_ostype(&value).unwrap();
        
        let buffer = writer.into_buffer();
        let cursor = Cursor::new(buffer);
        let mut reader = PsdReader::new(cursor, Default::default());
        
        let _sig = reader.read_signature().unwrap();
        if let DescriptorValue::UnitDouble { units, value } = reader.read_ostype("UntF").unwrap() {
            assert_eq!(units, "Pixels");
            assert_eq!(value, 100.0);
        } else {
            panic!("Expected UnitDouble");
        }
    }
}
