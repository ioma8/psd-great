//! PSD file reader implementation
//!
//! Provides functionality to read PSD files and parse their structure.

use crate::error::{PsdError, Result};
use crate::types::{ChannelID, ColorMode, Compression, PixelData, SectionDividerType};
use crate::psd::{Psd, ReadOptions, GlobalLayerMaskInfo};
use crate::layer::Layer;
use crate::helpers::to_blend_mode;
use crate::compression;
use byteorder::{BigEndian, ReadBytesExt};
use std::io::{Read, Seek, SeekFrom};

/// PSD reader for binary data
pub struct PsdReader<R: Read + Seek> {
    reader: R,
    pub offset: u64,
    pub large: bool,
    pub global_alpha: bool,
    pub options: ReadOptions,
}

impl<R: Read + Seek> PsdReader<R> {
    /// Create a new PSD reader
    pub fn new(reader: R, options: ReadOptions) -> Self {
        Self {
            reader,
            offset: 0,
            large: false,
            global_alpha: false,
            options,
        }
    }

    /// Read an unsigned 8-bit integer
    pub fn read_u8(&mut self) -> Result<u8> {
        let val = self.reader.read_u8()?;
        self.offset += 1;
        Ok(val)
    }

    /// Peek at an unsigned 8-bit integer without advancing
    pub fn peek_u8(&mut self) -> Result<u8> {
        let pos = self.reader.stream_position()?;
        let val = self.reader.read_u8()?;
        self.reader.seek(SeekFrom::Start(pos))?;
        Ok(val)
    }

    /// Read a signed 16-bit integer (big-endian)
    pub fn read_i16(&mut self) -> Result<i16> {
        let val = self.reader.read_i16::<BigEndian>()?;
        self.offset += 2;
        Ok(val)
    }

    /// Read an unsigned 16-bit integer (big-endian)
    pub fn read_u16(&mut self) -> Result<u16> {
        let val = self.reader.read_u16::<BigEndian>()?;
        self.offset += 2;
        Ok(val)
    }

    /// Read a signed 32-bit integer (big-endian)
    pub fn read_i32(&mut self) -> Result<i32> {
        let val = self.reader.read_i32::<BigEndian>()?;
        self.offset += 4;
        Ok(val)
    }

    /// Read an unsigned 32-bit integer (big-endian)
    pub fn read_u32(&mut self) -> Result<u32> {
        let val = self.reader.read_u32::<BigEndian>()?;
        self.offset += 4;
        Ok(val)
    }

    /// Read a 32-bit float (big-endian)
    pub fn read_f32(&mut self) -> Result<f32> {
        let val = self.reader.read_f32::<BigEndian>()?;
        self.offset += 4;
        Ok(val)
    }

    /// Read a 64-bit float (big-endian)
    pub fn read_f64(&mut self) -> Result<f64> {
        let val = self.reader.read_f64::<BigEndian>()?;
        self.offset += 8;
        Ok(val)
    }

    /// Read raw bytes
    pub fn read_bytes(&mut self, length: usize) -> Result<Vec<u8>> {
        let mut buffer = vec![0u8; length];
        self.reader.read_exact(&mut buffer)?;
        self.offset += length as u64;
        Ok(buffer)
    }

    /// Skip bytes
    pub fn skip_bytes(&mut self, count: usize) -> Result<()> {
        self.reader.seek(SeekFrom::Current(count as i64))?;
        self.offset += count as u64;
        Ok(())
    }

    /// Read a 4-character signature
    pub fn read_signature(&mut self) -> Result<String> {
        let bytes = self.read_bytes(4)?;
        Ok(String::from_utf8_lossy(&bytes).to_string())
    }

    /// Check signature matches expected value
    pub fn check_signature(&mut self, expected: &str) -> Result<()> {
        let sig = self.read_signature()?;
        if sig != expected {
            return Err(PsdError::InvalidFormat(format!(
                "Invalid signature: expected '{}', got '{}'",
                expected, sig
            )));
        }
        Ok(())
    }

    /// Read a Pascal string (length-prefixed, padded)
    pub fn read_pascal_string(&mut self, pad_to: usize) -> Result<String> {
        let mut length = self.read_u8()? as usize;
        let text = if length > 0 {
            let bytes = self.read_bytes(length)?;
            String::from_utf8_lossy(&bytes).to_string()
        } else {
            String::new()
        };

        length += 1; // Include the length byte
        while length % pad_to != 0 {
            self.skip_bytes(1)?;
            length += 1;
        }

        Ok(text)
    }

    /// Read a Unicode string (UTF-16 BE)
    pub fn read_unicode_string(&mut self) -> Result<String> {
        let length = self.read_u32()? as usize;
        self.read_unicode_string_with_length(length)
    }

    /// Read a Unicode string with known length
    pub fn read_unicode_string_with_length(&mut self, length: usize) -> Result<String> {
        let mut chars = Vec::with_capacity(length);
        
        for _ in 0..length {
            let value = self.read_u16()?;
            if value != 0 || chars.is_empty() {
                if let Some(c) = char::from_u32(value as u32) {
                    chars.push(c);
                }
            }
        }

        Ok(chars.into_iter().collect())
    }

    /// Read an ASCII string
    pub fn read_ascii_string(&mut self, length: usize) -> Result<String> {
        let bytes = self.read_bytes(length)?;
        Ok(String::from_utf8_lossy(&bytes).to_string())
    }

    /// Read a section with length prefix
    pub fn read_section<F, T>(&mut self, _round: usize, func: F) -> Result<T>
    where
        F: FnOnce(&mut Self, u64) -> Result<T>,
    {
        let length = if self.large {
            let high = self.read_u32()? as usize;
            if high != 0 {
                return Err(PsdError::UnsupportedFeature(
                    "Sizes larger than 4GB are not supported".to_string(),
                ));
            }
            self.read_u32()? as usize
        } else {
            self.read_u32()? as usize
        };

        let start_offset = self.offset;
        let end_offset = start_offset + length as u64;

        let result = func(self, end_offset)?;

        // Skip to end of section
        if self.offset < end_offset {
            let remaining = (end_offset - self.offset) as usize;
            self.skip_bytes(remaining)?;
        }

        Ok(result)
    }
    
    /// Get bytes left in current section
    pub fn bytes_left(&self, end_offset: u64) -> usize {
        if self.offset >= end_offset {
            0
        } else {
            (end_offset - self.offset) as usize
        }
    }

    /// Read a fixed-point number (16.16)
    pub fn read_fixed_point_32(&mut self) -> Result<f64> {
        let val = self.read_i32()?;
        Ok(val as f64 / 65536.0)
    }

    /// Read a fixed-point path number (8.24)
    pub fn read_fixed_point_path_32(&mut self) -> Result<f64> {
        let val = self.read_i32()?;
        Ok(val as f64 / 16777216.0)
    }

    /// Read a color value
    pub fn read_color(&mut self) -> Result<crate::types::Color> {
        use crate::types::{Color, RGBA};
        let color_space = self.read_u16()?;
        
        match color_space {
            0 => {
                // RGB
                let r = (self.read_u16()? >> 8) as u8;
                let g = (self.read_u16()? >> 8) as u8;
                let b = (self.read_u16()? >> 8) as u8;
                self.read_u16()?; // Skip padding
                Ok(Color::RGBA(RGBA { r, g, b, a: 255 }))
            }
            _ => {
                // Skip other color spaces for now
                self.skip_bytes(6)?;
                Ok(Color::RGBA(RGBA { r: 0, g: 0, b: 0, a: 255 }))
            }
        }
    }
}

/// Read a PSD file from a reader
pub fn read_psd<R: Read + Seek>(
    mut reader: R,
    options: ReadOptions,
) -> Result<Psd> {
    let mut psd_reader = PsdReader::new(&mut reader, options);
    
    // Read header
    psd_reader.check_signature("8BPS")?;
    
    let version = psd_reader.read_u16()?;
    if version != 1 && version != 2 {
        return Err(PsdError::InvalidFormat(format!(
            "Invalid PSD file version: {}",
            version
        )));
    }

    psd_reader.large = version == 2;
    
    // Skip reserved bytes
    psd_reader.skip_bytes(6)?;
    
    let channels = psd_reader.read_u16()?;
    let height = psd_reader.read_u32()?;
    let width = psd_reader.read_u32()?;
    let bits_per_channel = psd_reader.read_u16()?;
    let color_mode = psd_reader.read_u16()?;

    // Validate dimensions
    let max_size = if version == 1 { 30000 } else { 300000 };
    if width > max_size || height > max_size {
        return Err(PsdError::InvalidFormat(format!(
            "Invalid size: {}x{}",
            width, height
        )));
    }

    if channels > 56 {
        return Err(PsdError::InvalidFormat(format!(
            "Invalid channel count: {}",
            channels
        )));
    }

    if ![1, 8, 16, 32].contains(&bits_per_channel) {
        return Err(PsdError::InvalidFormat(format!(
            "Invalid bits per channel: {}",
            bits_per_channel
        )));
    }

    let color_mode = ColorMode::from_u16(color_mode)?;

    let mut psd = Psd {
        width,
        height,
        channels: Some(channels),
        bits_per_channel: Some(bits_per_channel as u8),
        color_mode: Some(color_mode),
        palette: None,
        image_data: None,
        children: None,
        image_resources: None,
        linked_files: None,
        artboards: None,
        global_layer_mask_info: None,
        annotations: None,
        additional_info: Default::default(),
    };

    // Read color mode data section
    read_color_mode_data(&mut psd_reader, &mut psd)?;

    // Read image resources section
    read_image_resources(&mut psd_reader, &mut psd)?;

    // Read layer and mask information section
    read_layer_and_mask_info(&mut psd_reader, &mut psd)?;

    // Read image data section
    if !psd_reader.options.skip_composite_image_data.unwrap_or(false) {
        read_image_data(&mut psd_reader, &mut psd)?;
    }

    Ok(psd)
}

/// Read color mode data section
fn read_color_mode_data<R: Read + Seek>(
    reader: &mut PsdReader<R>,
    psd: &mut Psd,
) -> Result<()> {
    reader.read_section(1, |reader, end_offset| {
        if reader.bytes_left(end_offset) == 0 {
            return Ok(());
        }

        if psd.color_mode == Some(ColorMode::Indexed) {
            if reader.bytes_left(end_offset) != 768 {
                return Err(PsdError::InvalidFormat(
                    "Invalid color palette size".to_string(),
                ));
            }

            let mut palette = Vec::with_capacity(256);
            
            // Read red values
            for _ in 0..256 {
                let r = reader.read_u8()?;
                palette.push(crate::types::RGB { r, g: 0, b: 0 });
            }
            
            // Read green values
            for i in 0..256 {
                palette[i].g = reader.read_u8()?;
            }
            
            // Read blue values
            for i in 0..256 {
                palette[i].b = reader.read_u8()?;
            }

            // Store palette in image resources
            // psd.palette = Some(palette);
        } else {
            // Skip unknown color mode data
            reader.skip_bytes(reader.bytes_left(end_offset))?;
        }

        Ok(())
    })
}

/// Read image resources section
fn read_image_resources<R: Read + Seek>(
    reader: &mut PsdReader<R>,
    psd: &mut Psd,
) -> Result<()> {
    reader.read_section(1, |reader, end_offset| {
        while reader.bytes_left(end_offset) > 0 {
            // Read signature
            let sig = reader.read_signature()?;
            if sig != "8BIM" && sig != "MeSa" && sig != "AgHg" && sig != "PHUT" && sig != "DCSR" {
                return Err(PsdError::InvalidFormat(format!(
                    "Invalid image resource signature: {}",
                    sig
                )));
            }

            let _id = reader.read_u16()?;
            let _name = reader.read_pascal_string(2)?;

            // Read resource data
            reader.read_section(2, |reader, end_offset| {
                // Skip resource data for now
                reader.skip_bytes(reader.bytes_left(end_offset))?;
                Ok(())
            })?;
        }

        Ok(())
    })
}

/// Read layer and mask information section
fn read_layer_and_mask_info<R: Read + Seek>(
    reader: &mut PsdReader<R>,
    psd: &mut Psd,
) -> Result<()> {
    reader.read_section(1, |reader, end_offset| {
        // Read layer info
        if reader.bytes_left(end_offset) > 0 {
            reader.read_section(2, |reader, end_offset| {
                read_layer_info(reader, psd)?;
                reader.skip_bytes(reader.bytes_left(end_offset))?;
                Ok(())
            })?;
        }

        // Read global layer mask info
        if reader.bytes_left(end_offset) > 0 {
            let global_mask = read_global_layer_mask_info(reader)?;
            if let Some(mask) = global_mask {
                psd.global_layer_mask_info = Some(mask);
            }
        }

        // Skip additional layer info
        reader.skip_bytes(reader.bytes_left(end_offset))?;

        Ok(())
    })
}

/// Read layer info
fn read_layer_info<R: Read + Seek>(
    reader: &mut PsdReader<R>,
    psd: &mut Psd,
) -> Result<()> {
    let mut layer_count = reader.read_i16()? as i32;

    if layer_count < 0 {
        reader.global_alpha = true;
        layer_count = -layer_count;
    }

    let mut layers = Vec::new();
    let mut layer_channels = Vec::new();

    // Read layer records
    for _ in 0..layer_count {
        let (layer, channels) = read_layer_record(reader)?;
        layers.push(layer);
        layer_channels.push(channels);
    }

    // Read layer channel image data
    for (i, channels) in layer_channels.iter().enumerate() {
        read_layer_channel_image_data(reader, &mut layers[i], channels)?;
    }

    // Build layer hierarchy
    build_layer_hierarchy(psd, layers)?;

    Ok(())
}

/// Read a single layer record
fn read_layer_record<R: Read + Seek>(
    reader: &mut PsdReader<R>,
) -> Result<(Layer, Vec<ChannelInfo>)> {
    let mut layer = Layer::default();

    layer.top = Some(reader.read_i32()?);
    layer.left = Some(reader.read_i32()?);
    layer.bottom = Some(reader.read_i32()?);
    layer.right = Some(reader.read_i32()?);

    let channel_count = reader.read_u16()? as usize;
    let mut channels = Vec::with_capacity(channel_count);

    for _ in 0..channel_count {
        let id = reader.read_i16()?;
        let mut length = reader.read_u32()? as u64;

        if reader.large {
            if length != 0 {
                return Err(PsdError::UnsupportedFeature(
                    "Sizes larger than 4GB are not supported".to_string(),
                ));
            }
            length = reader.read_u32()? as u64;
        }

        channels.push(ChannelInfo {
            id: ChannelID::from_i16(id),
            length,
        });
    }

    // Read blend mode signature
    reader.check_signature("8BIM")?;
    
    let blend_sig = reader.read_signature()?;
    layer.blend_mode = Some(to_blend_mode(&blend_sig)?);

    layer.opacity = Some(reader.read_u8()? as f64 / 255.0);
    layer.clipping = Some(reader.read_u8()? == 1);

    let flags = reader.read_u8()?;
    layer.transparency_protected = Some((flags & 0x01) != 0);
    layer.hidden = Some((flags & 0x02) != 0);

    reader.skip_bytes(1)?; // Filler

    // Read extra data
    reader.read_section(1, |reader, end_offset| {
        // Read layer mask data
        read_layer_mask_data(reader, &mut layer)?;

        // Skip blending ranges
        reader.read_section(1, |reader, end_offset| {
            reader.skip_bytes(reader.bytes_left(end_offset))?;
            Ok(())
        })?;

        // Read layer name
        layer.additional_info.name = Some(reader.read_pascal_string(4)?);

        // Skip additional layer info
        reader.skip_bytes(reader.bytes_left(end_offset))?;

        Ok(())
    })?;

    Ok((layer, channels))
}

/// Channel information
#[derive(Debug, Clone)]
struct ChannelInfo {
    id: ChannelID,
    length: u64,
}

/// Read layer mask data
fn read_layer_mask_data<R: Read + Seek>(
    reader: &mut PsdReader<R>,
    layer: &mut Layer,
) -> Result<()> {
    reader.read_section(1, |reader, end_offset| {
        if reader.bytes_left(end_offset) == 0 {
            return Ok(());
        }

        let _top = reader.read_i32()?;
        let _left = reader.read_i32()?;
        let _bottom = reader.read_i32()?;
        let _right = reader.read_i32()?;
        let _default_color = reader.read_u8()?;
        let _flags = reader.read_u8()?;

        // Skip remaining mask data
        reader.skip_bytes(reader.bytes_left(end_offset))?;

        Ok(())
    })
}

/// Read layer channel image data
fn read_layer_channel_image_data<R: Read + Seek>(
    reader: &mut PsdReader<R>,
    layer: &mut Layer,
    channels: &[ChannelInfo],
) -> Result<()> {
    if reader.options.skip_layer_image_data.unwrap_or(false) {
        for channel in channels {
            reader.skip_bytes(channel.length as usize)?;
        }
        return Ok(());
    }

    let width = (layer.right.unwrap_or(0) - layer.left.unwrap_or(0)).max(0) as usize;
    let height = (layer.bottom.unwrap_or(0) - layer.top.unwrap_or(0)).max(0) as usize;
    if width == 0 || height == 0 {
        for channel in channels {
            reader.skip_bytes(channel.length as usize)?;
        }
        return Ok(());
    }

    let expected_len = width * height;
    let mut red: Option<Vec<u8>> = None;
    let mut green: Option<Vec<u8>> = None;
    let mut blue: Option<Vec<u8>> = None;
    let mut alpha: Option<Vec<u8>> = None;

    for channel in channels {
        let compression = reader.read_u16()?;
        let compression = Compression::from_u16(compression)?;
        let data_length = channel
            .length
            .checked_sub(2)
            .ok_or_else(|| PsdError::InvalidFormat("Invalid channel length".to_string()))?
            as usize;

        let decoded = match compression {
            Compression::RawData => {
                let data = reader.read_bytes(data_length)?;
                normalize_channel_data(data, expected_len)
            }
            Compression::RleCompressed => {
                let row_count = height;
                let byte_counts_len = row_count * 2;
                if data_length < byte_counts_len {
                    return Err(PsdError::InvalidFormat(
                        "Invalid RLE channel data length".to_string(),
                    ));
                }
                let mut byte_counts = Vec::with_capacity(row_count);
                for _ in 0..row_count {
                    byte_counts.push(reader.read_u16()?);
                }
                let compressed_len = data_length - byte_counts_len;
                let compressed = reader.read_bytes(compressed_len)?;
                let mut out = vec![0u8; expected_len];
                compression::decompress_rle(
                    &compressed,
                    &mut out,
                    width,
                    height,
                    &byte_counts,
                )?;
                out
            }
            Compression::ZipWithoutPrediction => {
                let compressed = reader.read_bytes(data_length)?;
                let out = compression::decompress_zip(&compressed, expected_len)?;
                normalize_channel_data(out, expected_len)
            }
            Compression::ZipWithPrediction => {
                let compressed = reader.read_bytes(data_length)?;
                let out = compression::decompress_zip_with_prediction(&compressed, width, height, 1)?;
                normalize_channel_data(out, expected_len)
            }
        };
        match channel.id {
            ChannelID::Color0 => red = Some(decoded),
            ChannelID::Color1 => green = Some(decoded),
            ChannelID::Color2 => blue = Some(decoded),
            ChannelID::Transparency => alpha = Some(decoded),
            _ => {}
        }
    }

    let mut rgba = vec![0u8; expected_len * 4];
    for i in 0..expected_len {
        rgba[i * 4] = red.as_ref().and_then(|d| d.get(i)).copied().unwrap_or(0);
        rgba[i * 4 + 1] = green.as_ref().and_then(|d| d.get(i)).copied().unwrap_or(0);
        rgba[i * 4 + 2] = blue.as_ref().and_then(|d| d.get(i)).copied().unwrap_or(0);
        rgba[i * 4 + 3] = alpha.as_ref().and_then(|d| d.get(i)).copied().unwrap_or(255);
    }

    layer.image_data = Some(PixelData { data: rgba, width, height });
    Ok(())
}

/// Build layer hierarchy from flat layer list
fn build_layer_hierarchy(psd: &mut Psd, layers: Vec<Layer>) -> Result<()> {
    psd.children = Some(Vec::new());
    
    let mut stack: Vec<&mut Vec<Layer>> = vec![psd.children.as_mut().unwrap()];

    for layer in layers.into_iter().rev() {
        let section_type = layer
            .additional_info
            .section_divider
            .as_ref()
            .map(|sd| sd.divider_type)
            .unwrap_or(SectionDividerType::Other);

        match section_type {
            SectionDividerType::OpenFolder | SectionDividerType::ClosedFolder => {
                let current = stack.last_mut().unwrap();
                current.insert(0, layer);
                
                // Push new group onto stack
                let last_idx = current.len() - 1;
                let last_layer = &mut current[last_idx];
                if last_layer.children.is_none() {
                    last_layer.children = Some(Vec::new());
                }
            }
            SectionDividerType::BoundingSectionDivider => {
                stack.pop();
            }
            SectionDividerType::Other => {
                let current = stack.last_mut().unwrap();
                current.insert(0, layer);
            }
        }
    }

    Ok(())
}

/// Read global layer mask info
fn read_global_layer_mask_info<R: Read + Seek>(
    reader: &mut PsdReader<R>,
) -> Result<Option<GlobalLayerMaskInfo>> {
    reader.read_section(1, |reader, end_offset| {
        if reader.bytes_left(end_offset) == 0 {
            return Ok(None);
        }

        let overlay_color_space = reader.read_u16()?;
        let color_space_1 = reader.read_u16()?;
        let color_space_2 = reader.read_u16()?;
        let color_space_3 = reader.read_u16()?;
        let color_space_4 = reader.read_u16()?;
        let opacity = reader.read_u16()?;
        let kind = reader.read_u8()?;
        
        reader.skip_bytes(reader.bytes_left(end_offset))?;

        Ok(Some(GlobalLayerMaskInfo {
            overlay_color_space,
            color_space1: color_space_1,
            color_space2: color_space_2,
            color_space3: color_space_3,
            color_space4: color_space_4,
            opacity,
            kind,
        }))
    })
}

/// Read image data section
fn read_image_data<R: Read + Seek>(
    reader: &mut PsdReader<R>,
    psd: &mut Psd,
) -> Result<()> {
    let compression = reader.read_u16()?;
    let compression = Compression::from_u16(compression)?;
    let width = psd.width as usize;
    let height = psd.height as usize;
    if width == 0 || height == 0 {
        return Ok(());
    }

    let channels = psd.channels.unwrap_or(3) as usize;
    if channels == 0 {
        return Ok(());
    }
    let channel_len = width * height;

    let mut planes: Vec<Vec<u8>> = Vec::with_capacity(channels);
    match compression {
        Compression::RawData => {
            for _ in 0..channels {
                let plane = reader.read_bytes(channel_len)?;
                planes.push(normalize_channel_data(plane, channel_len));
            }
        }
        Compression::RleCompressed => {
            let row_count = channels * height;
            let mut byte_counts = Vec::with_capacity(row_count);
            for _ in 0..row_count {
                byte_counts.push(reader.read_u16()?);
            }
            for channel_index in 0..channels {
                let start = channel_index * height;
                let end = start + height;
                let channel_counts = &byte_counts[start..end];
                let compressed_len = channel_counts.iter().map(|v| *v as usize).sum();
                let compressed = reader.read_bytes(compressed_len)?;
                let mut out = vec![0u8; channel_len];
                compression::decompress_rle(
                    &compressed,
                    &mut out,
                    width,
                    height,
                    channel_counts,
                )?;
                planes.push(out);
            }
        }
        Compression::ZipWithoutPrediction | Compression::ZipWithPrediction => {
            return Err(PsdError::UnsupportedFeature(
                "ZIP composite image data is not supported yet".to_string(),
            ));
        }
    }

    let mut rgba = vec![0u8; channel_len * 4];
    for i in 0..channel_len {
        rgba[i * 4] = planes
            .get(0)
            .and_then(|d| d.get(i))
            .copied()
            .unwrap_or(0);
        rgba[i * 4 + 1] = planes
            .get(1)
            .and_then(|d| d.get(i))
            .copied()
            .unwrap_or(0);
        rgba[i * 4 + 2] = planes
            .get(2)
            .and_then(|d| d.get(i))
            .copied()
            .unwrap_or(0);
        rgba[i * 4 + 3] = planes
            .get(3)
            .and_then(|d| d.get(i))
            .copied()
            .unwrap_or(255);
    }

    psd.image_data = Some(PixelData { data: rgba, width, height });

    Ok(())
}

fn normalize_channel_data(mut data: Vec<u8>, expected_len: usize) -> Vec<u8> {
    if data.len() < expected_len {
        data.resize(expected_len, 0);
        return data;
    }
    if data.len() > expected_len {
        data.truncate(expected_len);
    }
    data
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use crate::{write_psd, Layer, Psd, PixelData, WriteOptions};

    #[test]
    fn test_read_signature() {
        let data = b"8BPS";
        let mut reader = PsdReader::new(Cursor::new(data), ReadOptions::default());
        let sig = reader.read_signature().unwrap();
        assert_eq!(sig, "8BPS");
    }

    #[test]
    fn test_read_pascal_string() {
        let data = vec![5, b'H', b'e', b'l', b'l', b'o', 0, 0]; // "Hello" padded to 8 bytes
        let mut reader = PsdReader::new(Cursor::new(data), ReadOptions::default());
        let s = reader.read_pascal_string(4).unwrap();
        assert_eq!(s, "Hello");
    }

    #[test]
    fn test_read_integers() {
        let data = vec![
            0x00, 0x01, // u16: 1
            0x00, 0x00, 0x00, 0x02, // u32: 2
        ];
        let mut reader = PsdReader::new(Cursor::new(data), ReadOptions::default());
        assert_eq!(reader.read_u16().unwrap(), 1);
        assert_eq!(reader.read_u32().unwrap(), 2);
    }

    #[test]
    fn test_roundtrip_reads_layer_and_composite_pixels() {
        let layer = Layer {
            top: Some(0),
            left: Some(0),
            bottom: Some(1),
            right: Some(1),
            image_data: Some(PixelData {
                data: vec![255, 0, 0, 255],
                width: 1,
                height: 1,
            }),
            ..Default::default()
        };
        let psd = Psd {
            width: 1,
            height: 1,
            children: Some(vec![layer]),
            image_data: Some(PixelData {
                data: vec![0, 255, 0, 255],
                width: 1,
                height: 1,
            }),
            ..Default::default()
        };
        let bytes = write_psd(&psd, &WriteOptions { compress: Some(false), ..Default::default() })
            .expect("write psd");
        let loaded = read_psd(
            Cursor::new(bytes),
            ReadOptions {
                skip_layer_image_data: Some(false),
                skip_composite_image_data: Some(false),
                ..Default::default()
            },
        )
        .expect("read psd");

        let top = loaded
            .children
            .as_ref()
            .and_then(|c| c.first())
            .and_then(|l| l.image_data.as_ref())
            .expect("layer image");
        assert_eq!(top.data, vec![255, 0, 0, 255]);

        let composite = loaded.image_data.expect("composite image");
        assert_eq!(composite.data, vec![0, 255, 0, 255]);
    }
}
