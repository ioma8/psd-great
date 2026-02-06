//! PSD file writer implementation
//!
//! Provides functionality to write PSD files.

use crate::error::{PsdError, Result};
use crate::types::{BlendMode, ChannelID, ColorMode, Compression, Color};
use crate::psd::{Psd, WriteOptions, GlobalLayerMaskInfo};
use crate::layer::Layer;
use crate::helpers::{from_blend_mode, clamp, has_alpha};
use crate::compression;
use byteorder::{BigEndian, WriteBytesExt};
use std::io::Cursor;

/// PSD writer for binary data
pub struct PsdWriter {
    buffer: Vec<u8>,
    pub offset: usize,
}

impl PsdWriter {
    /// Create a new PSD writer with initial capacity
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: Vec::with_capacity(capacity),
            offset: 0,
        }
    }

    /// Create a new writer with default capacity (4KB)
    pub fn with_default_capacity() -> Self {
        Self::new(4096)
    }

    /// Get the written buffer
    pub fn get_buffer(&self) -> &[u8] {
        &self.buffer[..self.offset]
    }

    /// Get the written buffer as a Vec
    pub fn into_buffer(mut self) -> Vec<u8> {
        self.buffer.truncate(self.offset);
        self.buffer
    }

    /// Ensure buffer has enough capacity
    fn ensure_capacity(&mut self, additional: usize) {
        let required = self.offset + additional;
        if self.buffer.len() < required {
            let mut new_capacity = self.buffer.capacity();
            while new_capacity < required {
                new_capacity *= 2;
            }
            self.buffer.resize(new_capacity, 0);
        }
    }

    /// Write an unsigned 8-bit integer
    pub fn write_u8(&mut self, value: u8) -> Result<()> {
        self.ensure_capacity(1);
        self.buffer[self.offset] = value;
        self.offset += 1;
        Ok(())
    }

    /// Write a signed 16-bit integer (big-endian)
    pub fn write_i16(&mut self, value: i16) -> Result<()> {
        self.ensure_capacity(2);
        let mut cursor = Cursor::new(&mut self.buffer[self.offset..]);
        cursor.write_i16::<BigEndian>(value)?;
        self.offset += 2;
        Ok(())
    }

    /// Write an unsigned 16-bit integer (big-endian)
    pub fn write_u16(&mut self, value: u16) -> Result<()> {
        self.ensure_capacity(2);
        let mut cursor = Cursor::new(&mut self.buffer[self.offset..]);
        cursor.write_u16::<BigEndian>(value)?;
        self.offset += 2;
        Ok(())
    }

    /// Write a signed 32-bit integer (big-endian)
    pub fn write_i32(&mut self, value: i32) -> Result<()> {
        self.ensure_capacity(4);
        let mut cursor = Cursor::new(&mut self.buffer[self.offset..]);
        cursor.write_i32::<BigEndian>(value)?;
        self.offset += 4;
        Ok(())
    }

    /// Write an unsigned 32-bit integer (big-endian)
    pub fn write_u32(&mut self, value: u32) -> Result<()> {
        self.ensure_capacity(4);
        let mut cursor = Cursor::new(&mut self.buffer[self.offset..]);
        cursor.write_u32::<BigEndian>(value)?;
        self.offset += 4;
        Ok(())
    }

    /// Write a 32-bit float (big-endian)
    pub fn write_f32(&mut self, value: f32) -> Result<()> {
        self.ensure_capacity(4);
        let mut cursor = Cursor::new(&mut self.buffer[self.offset..]);
        cursor.write_f32::<BigEndian>(value)?;
        self.offset += 4;
        Ok(())
    }

    /// Write a 64-bit float (big-endian)
    pub fn write_f64(&mut self, value: f64) -> Result<()> {
        self.ensure_capacity(8);
        let mut cursor = Cursor::new(&mut self.buffer[self.offset..]);
        cursor.write_f64::<BigEndian>(value)?;
        self.offset += 8;
        Ok(())
    }

    /// Write raw bytes
    pub fn write_bytes(&mut self, data: &[u8]) -> Result<()> {
        self.ensure_capacity(data.len());
        self.buffer[self.offset..self.offset + data.len()].copy_from_slice(data);
        self.offset += data.len();
        Ok(())
    }

    /// Write zeros
    pub fn write_zeros(&mut self, count: usize) -> Result<()> {
        self.ensure_capacity(count);
        for _ in 0..count {
            self.buffer[self.offset] = 0;
            self.offset += 1;
        }
        Ok(())
    }

    /// Write a 4-character signature
    pub fn write_signature(&mut self, sig: &str) -> Result<()> {
        if sig.len() != 4 {
            return Err(PsdError::InvalidFormat(format!(
                "Invalid signature length: {}",
                sig
            )));
        }
        self.write_bytes(sig.as_bytes())
    }

    /// Write an ASCII string
    pub fn write_ascii_string(&mut self, text: &str) -> Result<()> {
        self.write_bytes(text.as_bytes())
    }

    /// Write a Pascal string (length-prefixed, padded)
    pub fn write_pascal_string(&mut self, text: &str, pad_to: usize) -> Result<()> {
        let mut length = text.len();
        if length > 255 {
            return Err(PsdError::InvalidFormat(
                "Pascal string too long".to_string(),
            ));
        }

        self.write_u8(length as u8)?;

        for byte in text.bytes() {
            let byte = if byte < 128 { byte } else { b'?' };
            self.write_u8(byte)?;
        }

        length += 1; // Include length byte
        while length % pad_to != 0 {
            self.write_u8(0)?;
            length += 1;
        }

        Ok(())
    }

    /// Write a Unicode string (UTF-16 BE)
    pub fn write_unicode_string(&mut self, text: &str) -> Result<()> {
        self.write_u32(text.len() as u32)?;
        
        for ch in text.chars() {
            self.write_u16(ch as u16)?;
        }

        Ok(())
    }

    /// Write a Unicode string with padding
    pub fn write_unicode_string_with_padding(&mut self, text: &str) -> Result<()> {
        self.write_u32((text.len() + 1) as u32)?;
        
        for ch in text.chars() {
            self.write_u16(ch as u16)?;
        }
        
        self.write_u16(0)?;

        Ok(())
    }

    /// Write a section with length prefix
    pub fn write_section<F>(&mut self, round: usize, large: bool, func: F) -> Result<()>
    where
        F: FnOnce(&mut Self) -> Result<()>,
    {
        if large {
            self.write_u32(0)?; // High 32 bits
        }
        
        let length_offset = self.offset;
        self.write_u32(0)?; // Placeholder for length

        let start_offset = self.offset;
        func(self)?;

        // Pad to alignment
        while (self.offset - start_offset) % round != 0 {
            self.write_u8(0)?;
        }

        // Write actual length
        let actual_length = (self.offset - start_offset) as u32;
        let mut cursor = Cursor::new(&mut self.buffer[length_offset..]);
        cursor.write_u32::<BigEndian>(actual_length)?;

        Ok(())
    }

    /// Write a fixed-point number (16.16)
    pub fn write_fixed_point_32(&mut self, value: f64) -> Result<()> {
        let fixed = (value * 65536.0) as i32;
        self.write_i32(fixed)
    }

    /// Write a fixed-point path number (8.24)
    pub fn write_fixed_point_path_32(&mut self, value: f64) -> Result<()> {
        let fixed = (value * 16777216.0) as i32;
        self.write_i32(fixed)
    }

    /// Write a color value
    pub fn write_color(&mut self, color: Option<&Color>) -> Result<()> {
        match color {
            None => {
                self.write_u16(0)?; // RGB color space
                self.write_zeros(8)?;
            }
            Some(Color::RGBA(c)) => {
                self.write_u16(0)?; // RGB
                self.write_u16((c.r as f64 * 257.0).round() as u16)?;
                self.write_u16((c.g as f64 * 257.0).round() as u16)?;
                self.write_u16((c.b as f64 * 257.0).round() as u16)?;
                self.write_u16(0)?;
            }
            Some(Color::RGB(c)) => {
                self.write_u16(0)?; // RGB
                self.write_u16((c.r as f64 * 257.0).round() as u16)?;
                self.write_u16((c.g as f64 * 257.0).round() as u16)?;
                self.write_u16((c.b as f64 * 257.0).round() as u16)?;
                self.write_u16(0)?;
            }
            Some(Color::HSB(c)) => {
                self.write_u16(1)?; // HSB
                self.write_u16((c.h * 0xffff as f64).round() as u16)?;
                self.write_u16((c.s * 0xffff as f64).round() as u16)?;
                self.write_u16((c.b * 0xffff as f64).round() as u16)?;
                self.write_u16(0)?;
            }
            Some(Color::CMYK(c)) => {
                self.write_u16(2)?; // CMYK
                self.write_u16((c.c as f64 * 257.0).round() as u16)?;
                self.write_u16((c.m as f64 * 257.0).round() as u16)?;
                self.write_u16((c.y as f64 * 257.0).round() as u16)?;
                self.write_u16((c.k as f64 * 257.0).round() as u16)?;
            }
            Some(Color::LAB(c)) => {
                self.write_u16(7)?; // Lab
                self.write_i16((c.l * 10000.0).round() as i16)?;
                let a_val = if c.a < 0.0 {
                    (c.a * 12800.0).round() as i16
                } else {
                    (c.a * 12700.0).round() as i16
                };
                self.write_i16(a_val)?;
                let b_val = if c.b < 0.0 {
                    (c.b * 12800.0).round() as i16
                } else {
                    (c.b * 12700.0).round() as i16
                };
                self.write_i16(b_val)?;
                self.write_u16(0)?;
            }
            Some(Color::Grayscale(c)) => {
                self.write_u16(8)?; // Grayscale
                self.write_u16((c.k as f64 * 10000.0 / 255.0).round() as u16)?;
                self.write_zeros(6)?;
            }
            _ => {
                self.write_u16(0)?;
                self.write_zeros(8)?;
            }
        }

        Ok(())
    }
}

/// Write a PSD file
pub fn write_psd(psd: &Psd, options: &WriteOptions) -> Result<Vec<u8>> {
    if psd.width == 0 || psd.height == 0 {
        return Err(PsdError::InvalidFormat("Invalid document size".to_string()));
    }

    let max_size = if options.psb.unwrap_or(false) { 300000 } else { 30000 };
    if psd.width > max_size || psd.height > max_size {
        return Err(PsdError::InvalidFormat(format!(
            "Document size too large: {}x{} (max is {}x{})",
            psd.width, psd.height, max_size, max_size
        )));
    }

    let bits_per_channel = psd.bits_per_channel.unwrap_or(8);
    if bits_per_channel != 8 {
        return Err(PsdError::UnsupportedFeature(
            "Only 8 bits per channel is supported for writing".to_string(),
        ));
    }

    let mut writer = PsdWriter::new(1024 * 1024); // 1MB initial capacity

    // Write header
    writer.write_signature("8BPS")?;
    writer.write_u16(if options.psb.unwrap_or(false) { 2 } else { 1 })?; // Version
    writer.write_zeros(6)?; // Reserved

    let global_alpha = if let Some(ref image_data) = psd.image_data {
        has_alpha(image_data)
    } else {
        false
    };

    writer.write_u16(if global_alpha { 4 } else { 3 })?; // Channels
    writer.write_u32(psd.height)?;
    writer.write_u32(psd.width)?;
    writer.write_u16(bits_per_channel as u16)?;
    writer.write_u16(ColorMode::RGB as u16)?; // Only RGB supported for now

    // Write color mode data section
    write_color_mode_data(&mut writer, psd)?;

    // Write image resources section
    write_image_resources(&mut writer, psd, options)?;

    // Write layer and mask information section
    write_layer_and_mask_info(&mut writer, psd, options)?;

    // Write image data section
    write_image_data(&mut writer, psd, options, global_alpha)?;

    Ok(writer.into_buffer())
}

/// Write color mode data section
fn write_color_mode_data(writer: &mut PsdWriter, psd: &Psd) -> Result<()> {
    writer.write_section(1, false, |writer| {
        // Empty for RGB mode
        Ok(())
    })
}

/// Write image resources section
fn write_image_resources(writer: &mut PsdWriter, psd: &Psd, options: &WriteOptions) -> Result<()> {
    writer.write_section(1, false, |writer| {
        // Write minimal image resources for now
        Ok(())
    })
}

/// Write layer and mask information section
fn write_layer_and_mask_info(
    writer: &mut PsdWriter,
    psd: &Psd,
    options: &WriteOptions,
) -> Result<()> {
    let psb = options.psb.unwrap_or(false);
    writer.write_section(1, psb, |writer| {
        // Write layer info
        write_layer_info(writer, psd, options)?;
        
        // Write global layer mask info
        write_global_layer_mask_info(writer, psd.global_layer_mask_info.as_ref())?;

        Ok(())
    })
}

/// Write layer info section
fn write_layer_info(writer: &mut PsdWriter, psd: &Psd, options: &WriteOptions) -> Result<()> {
    let psb = options.psb.unwrap_or(false);
    writer.write_section(2, psb, |writer| {
        let layers = flatten_layers(psd.children.as_ref());
        
        let layer_count = layers.len() as i16;
        writer.write_i16(layer_count)?;

        // Write layer records
        for layer in &layers {
            write_layer_record(writer, layer, options)?;
        }

        // Write layer channel image data
        for layer in &layers {
            write_layer_channel_data(writer, layer, options)?;
        }

        Ok(())
    })
}

/// Flatten layer hierarchy to a list
fn flatten_layers(children: Option<&Vec<Layer>>) -> Vec<Layer> {
    let mut result = Vec::new();
    
    if let Some(children) = children {
        for child in children {
            if let Some(ref child_children) = child.children {
                // Add opening folder marker
                let mut folder = child.clone();
                folder.children = None;
                result.push(folder);
                
                // Add children
                result.extend(flatten_layers(Some(child_children)));
                
                // Add closing folder marker
                let mut closing = Layer::default();
                closing.additional_info.name = Some("</Layer group>".to_string());
                result.push(closing);
            } else {
                result.push(child.clone());
            }
        }
    }
    
    result
}

/// Write a single layer record
fn write_layer_record(writer: &mut PsdWriter, layer: &Layer, options: &WriteOptions) -> Result<()> {
    let psb = options.psb.unwrap_or(false);
    
    // Write bounds
    writer.write_i32(layer.top.unwrap_or(0))?;
    writer.write_i32(layer.left.unwrap_or(0))?;
    writer.write_i32(layer.bottom.unwrap_or(0))?;
    writer.write_i32(layer.right.unwrap_or(0))?;

    // Write channel count (A, R, G, B)
    let channel_ids = [
        ChannelID::Transparency,
        ChannelID::Color0,
        ChannelID::Color1,
        ChannelID::Color2,
    ];
    let channel_count = channel_ids.len() as u16;
    writer.write_u16(channel_count)?;

    for channel_id in channel_ids {
        let channel_payload_len = layer_channel_payload(layer, channel_id, options)?.len() as u32;
        writer.write_i16(channel_id as i16)?;
        if psb {
            writer.write_u32(0)?;
        }
        writer.write_u32(channel_payload_len + 2)?; // Compression field + payload
    }

    // Write blend mode signature
    writer.write_signature("8BIM")?;
    
    let blend_mode = layer.blend_mode.unwrap_or(BlendMode::Normal);
    writer.write_signature(from_blend_mode(blend_mode))?;

    // Write opacity
    let opacity = layer.opacity.unwrap_or(1.0);
    writer.write_u8((clamp(opacity, 0.0, 1.0) * 255.0).round() as u8)?;

    // Write clipping
    writer.write_u8(if layer.clipping.unwrap_or(false) { 1 } else { 0 })?;

    // Write flags
    let mut flags = 0x08u8; // Photoshop 5.0+ bit
    if layer.transparency_protected.unwrap_or(false) {
        flags |= 0x01;
    }
    if layer.hidden.unwrap_or(false) {
        flags |= 0x02;
    }
    writer.write_u8(flags)?;
    writer.write_u8(0)?; // Filler

    // Write extra data
    writer.write_section(1, false, |writer| {
        // Write empty mask data
        writer.write_section(1, false, |_writer| Ok(()))?;

        // Write empty blending ranges
        writer.write_section(1, false, |_writer| Ok(()))?;

        // Write layer name
        let name = layer.additional_info.name.as_deref().unwrap_or("");
        writer.write_pascal_string(name, 4)?;

        Ok(())
    })?;

    Ok(())
}

/// Write layer channel image data
fn write_layer_channel_data(
    writer: &mut PsdWriter,
    layer: &Layer,
    options: &WriteOptions,
) -> Result<()> {
    let channel_ids = [
        ChannelID::Transparency,
        ChannelID::Color0,
        ChannelID::Color1,
        ChannelID::Color2,
    ];
    let compression = if options.compress.unwrap_or(false) {
        Compression::RleCompressed
    } else {
        Compression::RawData
    };

    for channel_id in channel_ids {
        let payload = layer_channel_payload(layer, channel_id, options)?;
        writer.write_u16(compression as u16)?;
        writer.write_bytes(&payload)?;
    }

    Ok(())
}

/// Write global layer mask info
fn write_global_layer_mask_info(
    writer: &mut PsdWriter,
    info: Option<&GlobalLayerMaskInfo>,
) -> Result<()> {
    writer.write_section(1, false, |writer| {
        if let Some(info) = info {
            writer.write_u16(info.overlay_color_space)?;
            writer.write_u16(info.color_space1)?;
            writer.write_u16(info.color_space2)?;
            writer.write_u16(info.color_space3)?;
            writer.write_u16(info.color_space4)?;
            writer.write_u16(info.opacity)?;
            writer.write_u8(info.kind)?;
            writer.write_zeros(3)?;
        }
        Ok(())
    })
}

/// Write image data section
fn write_image_data(
    writer: &mut PsdWriter,
    psd: &Psd,
    options: &WriteOptions,
    global_alpha: bool,
) -> Result<()> {
    let compression = if options.compress.unwrap_or(false) {
        Compression::RleCompressed
    } else {
        Compression::RawData
    };
    writer.write_u16(compression as u16)?;

    let fallback_width = psd.width as usize;
    let fallback_height = psd.height as usize;
    let fallback_rgba = vec![0u8; fallback_width * fallback_height * 4];
    let (image_data, width, height) = if let Some(ref image_data) = psd.image_data {
        (image_data.data.as_slice(), image_data.width, image_data.height)
    } else {
        (fallback_rgba.as_slice(), fallback_width, fallback_height)
    };

    let offsets: &[usize] = if global_alpha { &[0, 1, 2, 3] } else { &[0, 1, 2] };
    match compression {
        Compression::RawData => {
            for &offset in offsets {
                writer.write_bytes(&extract_channel_data_from_rgba(image_data, width, height, offset))?;
            }
        }
        Compression::RleCompressed => {
            let mut compressed_channels = Vec::with_capacity(offsets.len());
            for &offset in offsets {
                let raw = extract_channel_data_from_rgba(image_data, width, height, offset);
                let compressed = compression::compress_rle(&raw, width, height)?;
                compressed_channels.push(compressed);
            }

            // PSD composite RLE stores all row byte-counts first, then compressed row data.
            for channel in &compressed_channels {
                let table_len = height * 2;
                writer.write_bytes(&channel[..table_len])?;
            }
            for channel in &compressed_channels {
                let table_len = height * 2;
                writer.write_bytes(&channel[table_len..])?;
            }
        }
        Compression::ZipWithoutPrediction | Compression::ZipWithPrediction => {
            return Err(PsdError::UnsupportedFeature(
                "ZIP image compression is not supported for writing".to_string(),
            ));
        }
    }

    Ok(())
}

fn layer_channel_payload(layer: &Layer, channel_id: ChannelID, options: &WriteOptions) -> Result<Vec<u8>> {
    let width = (layer.right.unwrap_or(0) - layer.left.unwrap_or(0)).max(0) as usize;
    let height = (layer.bottom.unwrap_or(0) - layer.top.unwrap_or(0)).max(0) as usize;
    let image_data = layer.image_data.as_ref();
    let offset = match channel_id {
        ChannelID::Color0 => 0,
        ChannelID::Color1 => 1,
        ChannelID::Color2 => 2,
        ChannelID::Transparency => 3,
        _ => 0,
    };
    let raw = extract_layer_channel_data(image_data, width, height, offset);

    if options.compress.unwrap_or(false) {
        compression::compress_rle(&raw, width, height)
    } else {
        Ok(raw)
    }
}

fn extract_layer_channel_data(image_data: Option<&crate::types::PixelData>, width: usize, height: usize, offset: usize) -> Vec<u8> {
    let mut out = vec![0u8; width * height];
    if let Some(image_data) = image_data {
        for i in 0..(width * height) {
            let src = i * 4 + offset;
            if src < image_data.data.len() {
                out[i] = image_data.data[src];
            } else if offset == 3 {
                out[i] = 255;
            }
        }
    } else if offset == 3 {
        out.fill(255);
    }
    out
}

fn extract_channel_data_from_rgba(image_data: &[u8], width: usize, height: usize, offset: usize) -> Vec<u8> {
    let mut out = vec![0u8; width * height];
    for i in 0..(width * height) {
        let src = i * 4 + offset;
        if src < image_data.len() {
            out[i] = image_data[src];
        } else if offset == 3 {
            out[i] = 255;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_signature() {
        let mut writer = PsdWriter::with_default_capacity();
        writer.write_signature("8BPS").unwrap();
        assert_eq!(writer.get_buffer(), b"8BPS");
    }

    #[test]
    fn test_write_pascal_string() {
        let mut writer = PsdWriter::with_default_capacity();
        writer.write_pascal_string("Hi", 4).unwrap();
        assert_eq!(writer.get_buffer(), &[2, b'H', b'i', 0]);
    }

    #[test]
    fn test_write_integers() {
        let mut writer = PsdWriter::with_default_capacity();
        writer.write_u16(256).unwrap();
        writer.write_u32(512).unwrap();
        assert_eq!(writer.get_buffer(), &[0x01, 0x00, 0x00, 0x00, 0x02, 0x00]);
    }
}
