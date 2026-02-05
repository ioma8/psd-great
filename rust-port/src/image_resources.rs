//! Image resource handlers for PSD files
//!
//! Image resources contain document-level information like resolution,
//! guides, grids, color profiles, and thumbnails.

use crate::error::{PsdError, Result};
use crate::reader::PsdReader;
use crate::writer::PsdWriter;
use crate::descriptor::{Descriptor, DescriptorValue};
use crate::types::Color;
use std::collections::HashMap;
use std::io::{Read, Seek};

/// Image resources structure
#[derive(Debug, Clone, Default)]
pub struct ImageResources {
    /// Resolution information (DPI)
    pub resolution_info: Option<ResolutionInfo>,
    /// XMP metadata
    pub xmp_metadata: Option<String>,
    /// Caption digest (MD5 hash)
    pub caption_digest: Option<String>,
    /// Print information
    pub print_information: Option<PrintInformation>,
    /// Print flags
    pub print_flags: Option<PrintFlags>,
    /// Background color
    pub background_color: Option<Color>,
    /// Copyright flag
    pub copyrighted: Option<bool>,
    /// URL
    pub url: Option<String>,
    /// Thumbnail
    pub thumbnail: Option<Thumbnail>,
    /// Grid and guides
    pub grid_and_guides: Option<GridAndGuides>,
    /// Global lighting angle
    pub global_angle: Option<i32>,
    /// Global altitude
    pub global_altitude: Option<i32>,
    /// Layer state
    pub layer_state: Option<u16>,
    /// Layers group indices
    pub layers_group: Option<Vec<u16>>,
    /// Layer selection IDs
    pub layer_selection_ids: Option<Vec<u32>>,
    /// Alpha channel names
    pub alpha_names: Option<Vec<String>>,
    /// Alpha channel unicode names
    pub alpha_unicode_names: Option<Vec<String>>,
    /// Alpha identifiers
    pub alpha_identifiers: Option<Vec<u32>>,
    /// ICC profile data
    pub icc_profile: Option<Vec<u8>>,
    /// Print scale
    pub print_scale: Option<PrintScale>,
    /// Slices
    pub slices: Option<Slices>,
    /// Layer comps
    pub layer_comps: Option<LayerComps>,
    /// Timeline information
    pub timeline: Option<Timeline>,
    /// Sheet disclosure
    pub sheet_disclosure: Option<Vec<u8>>,
    /// Onion skins
    pub onion_skins: Option<OnionSkins>,
    /// Count information
    pub count_information: Option<Vec<CountGroup>>,
    /// URL list
    pub url_list: Option<Vec<UrlEntry>>,
    /// Unknown resources (for preservation)
    pub unknown: HashMap<u16, Vec<u8>>,
}

/// Resolution information
#[derive(Debug, Clone, PartialEq)]
pub struct ResolutionInfo {
    pub horizontal_res: f64,
    pub horizontal_res_unit: ResolutionUnit,
    pub width_unit: MeasurementUnit,
    pub vertical_res: f64,
    pub vertical_res_unit: ResolutionUnit,
    pub height_unit: MeasurementUnit,
}

/// Resolution units
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolutionUnit {
    PixelsPerInch = 1,
    PixelsPerCentimeter = 2,
}

/// Measurement units
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MeasurementUnit {
    Inches = 1,
    Centimeters = 2,
    Points = 3,
    Picas = 4,
    Columns = 5,
}

/// Print information
#[derive(Debug, Clone, PartialEq)]
pub struct PrintInformation {
    pub printer_name: String,
    pub rendering_intent: RenderingIntent,
    pub black_point_compensation: Option<bool>,
    pub printer_manages_colors: Option<bool>,
    pub printer_profile: Option<String>,
    pub print_sixteen_bit: Option<bool>,
    pub hard_proof: Option<bool>,
    pub proof_setup: Option<ProofSetup>,
}

/// Rendering intent
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderingIntent {
    Perceptual,
    Saturation,
    RelativeColorimetric,
    AbsoluteColorimetric,
}

/// Proof setup
#[derive(Debug, Clone, PartialEq)]
pub enum ProofSetup {
    Builtin { name: String },
    Custom {
        profile: String,
        rendering_intent: RenderingIntent,
        black_point_compensation: bool,
        paper_white: bool,
    },
}

/// Print flags
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PrintFlags {
    pub labels: bool,
    pub crop_marks: bool,
    pub color_bars: bool,
    pub registration_marks: bool,
    pub negative: bool,
    pub flip: bool,
    pub interpolate: bool,
    pub caption: bool,
    pub print_flags: bool,
}

/// Thumbnail
#[derive(Debug, Clone, PartialEq)]
pub struct Thumbnail {
    pub width: u32,
    pub height: u32,
    pub format: ThumbnailFormat,
    pub data: Vec<u8>,
}

/// Thumbnail format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThumbnailFormat {
    Rgb = 1,
    Jpeg = 0,
}

/// Grid and guides
#[derive(Debug, Clone, PartialEq)]
pub struct GridAndGuides {
    pub grid: Grid,
    pub guides: Vec<Guide>,
}

/// Grid spacing
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Grid {
    pub horizontal: u32,
    pub vertical: u32,
}

/// Guide line
#[derive(Debug, Clone, PartialEq)]
pub struct Guide {
    pub location: f64,
    pub direction: GuideDirection,
}

/// Guide direction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuideDirection {
    Vertical,
    Horizontal,
}

/// Print scale
#[derive(Debug, Clone, PartialEq)]
pub struct PrintScale {
    pub style: PrintScaleStyle,
    pub x: f32,
    pub y: f32,
    pub scale: f32,
}

/// Print scale style
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrintScaleStyle {
    Centered = 0,
    SizeToFit = 1,
    UserDefined = 2,
}

/// Slices
#[derive(Debug, Clone, PartialEq)]
pub struct Slices {
    pub version: u32,
    pub bounds: Bounds,
    pub name: String,
    pub slices: Vec<Slice>,
}

/// Slice
#[derive(Debug, Clone, PartialEq)]
pub struct Slice {
    pub id: u32,
    pub group_id: u32,
    pub origin: SliceOrigin,
    pub name: String,
    pub slice_type: SliceType,
    pub bounds: Bounds,
    pub url: Option<String>,
    pub target: Option<String>,
    pub message: Option<String>,
    pub alt_tag: Option<String>,
    pub cell_is_html: Option<bool>,
    pub cell_text: Option<String>,
    pub horizontal_align: Option<HorizontalAlign>,
    pub vertical_align: Option<VerticalAlign>,
    pub bg_color: Option<Color>,
}

/// Slice origin
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SliceOrigin {
    UserGenerated = 1,
    LayerBased = 2,
}

/// Slice type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SliceType {
    Image = 1,
    NoImage = 2,
}

/// Horizontal alignment
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HorizontalAlign {
    Default = 0,
    Left = 1,
    Center = 3,
    Right = 5,
}

/// Vertical alignment
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerticalAlign {
    Default = 0,
    Top = 1,
    Center = 3,
    Bottom = 5,
}

/// Bounds rectangle
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bounds {
    pub top: i32,
    pub left: i32,
    pub bottom: i32,
    pub right: i32,
}

/// Layer comps
#[derive(Debug, Clone, PartialEq)]
pub struct LayerComps {
    pub list: Vec<LayerComp>,
    pub last_applied: Option<i32>,
}

/// Layer comp
#[derive(Debug, Clone, PartialEq)]
pub struct LayerComp {
    pub id: i32,
    pub name: String,
    pub comment: Option<String>,
    pub captured_info: i32,
}

/// Timeline
#[derive(Debug, Clone, PartialEq)]
pub struct Timeline {
    pub enabled: bool,
    pub frame_step: Fraction,
    pub frame_rate: f64,
    pub time: Fraction,
    pub duration: Fraction,
    pub work_in_time: Fraction,
    pub work_out_time: Fraction,
    pub layer_count: i32,
    pub has_motion: bool,
}

/// Fraction
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Fraction {
    pub numerator: i32,
    pub denominator: i32,
}

/// Onion skins
#[derive(Debug, Clone, PartialEq)]
pub struct OnionSkins {
    pub enabled: bool,
    pub frames_before: i32,
    pub frames_after: i32,
    pub frame_spacing: i32,
    pub min_opacity: f64,
    pub max_opacity: f64,
    pub blend_mode: String,
}

/// Count information group
#[derive(Debug, Clone, PartialEq)]
pub struct CountGroup {
    pub color: Color,
    pub name: String,
    pub size: i32,
    pub font_size: i32,
    pub visible: bool,
    pub points: Vec<Point>,
}

/// Point
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

/// URL entry
#[derive(Debug, Clone, PartialEq)]
pub struct UrlEntry {
    pub id: u32,
    pub url: String,
}

/// Resource handler function type
// Note: Function pointers can't use impl Trait, so handlers are called directly

impl<R: Read + Seek> PsdReader<R> {
    /// Read resolution info (resource 1005)
    pub fn read_resolution_info(&mut self, resources: &mut ImageResources) -> Result<()> {
        let horizontal_res = self.read_fixed_point_32()?;
        let horizontal_res_unit = match self.read_u16()? {
            1 => ResolutionUnit::PixelsPerInch,
            2 => ResolutionUnit::PixelsPerCentimeter,
            v => return Err(PsdError::InvalidFormat(format!("Invalid resolution unit: {}", v))),
        };
        let width_unit = match self.read_u16()? {
            1 => MeasurementUnit::Inches,
            2 => MeasurementUnit::Centimeters,
            3 => MeasurementUnit::Points,
            4 => MeasurementUnit::Picas,
            5 => MeasurementUnit::Columns,
            v => return Err(PsdError::InvalidFormat(format!("Invalid measurement unit: {}", v))),
        };
        
        let vertical_res = self.read_fixed_point_32()?;
        let vertical_res_unit = match self.read_u16()? {
            1 => ResolutionUnit::PixelsPerInch,
            2 => ResolutionUnit::PixelsPerCentimeter,
            v => return Err(PsdError::InvalidFormat(format!("Invalid resolution unit: {}", v))),
        };
        let height_unit = match self.read_u16()? {
            1 => MeasurementUnit::Inches,
            2 => MeasurementUnit::Centimeters,
            3 => MeasurementUnit::Points,
            4 => MeasurementUnit::Picas,
            5 => MeasurementUnit::Columns,
            v => return Err(PsdError::InvalidFormat(format!("Invalid measurement unit: {}", v))),
        };
        
        resources.resolution_info = Some(ResolutionInfo {
            horizontal_res,
            horizontal_res_unit,
            width_unit,
            vertical_res,
            vertical_res_unit,
            height_unit,
        });
        
        Ok(())
    }

    /// Read XMP metadata (resource 1060)
    pub fn read_xmp_metadata(&mut self, resources: &mut ImageResources, length: usize) -> Result<()> {
        let data = self.read_bytes(length)?;
        resources.xmp_metadata = Some(String::from_utf8_lossy(&data).to_string());
        Ok(())
    }

    /// Read caption digest (resource 1061)
    pub fn read_caption_digest(&mut self, resources: &mut ImageResources) -> Result<()> {
        let mut hex = String::with_capacity(32);
        for _ in 0..16 {
            hex.push_str(&format!("{:02x}", self.read_u8()?));
        }
        resources.caption_digest = Some(hex);
        Ok(())
    }

    /// Read background color (resource 1010)
    pub fn read_background_color(&mut self, resources: &mut ImageResources) -> Result<()> {
        resources.background_color = Some(self.read_color()?);
        Ok(())
    }

    /// Read print flags (resource 1011)
    pub fn read_print_flags(&mut self, resources: &mut ImageResources) -> Result<()> {
        resources.print_flags = Some(PrintFlags {
            labels: self.read_u8()? != 0,
            crop_marks: self.read_u8()? != 0,
            color_bars: self.read_u8()? != 0,
            registration_marks: self.read_u8()? != 0,
            negative: self.read_u8()? != 0,
            flip: self.read_u8()? != 0,
            interpolate: self.read_u8()? != 0,
            caption: self.read_u8()? != 0,
            print_flags: self.read_u8()? != 0,
        });
        Ok(())
    }

    /// Read copyright flag (resource 1034)
    pub fn read_copyright_flag(&mut self, resources: &mut ImageResources) -> Result<()> {
        resources.copyrighted = Some(self.read_u8()? != 0);
        Ok(())
    }

    /// Read URL (resource 1035)
    pub fn read_url(&mut self, resources: &mut ImageResources, length: usize) -> Result<()> {
        resources.url = Some(self.read_ascii_string(length)?);
        Ok(())
    }

    /// Read grid and guides (resource 1032)
    pub fn read_grid_and_guides(&mut self, resources: &mut ImageResources) -> Result<()> {
        let version = self.read_u32()?;
        if version != 1 {
            return Err(PsdError::InvalidFormat(format!("Invalid grid/guides version: {}", version)));
        }
        
        let grid_h = self.read_u32()?;
        let grid_v = self.read_u32()?;
        let guide_count = self.read_u32()?;
        
        let mut guides = Vec::new();
        for _ in 0..guide_count {
            let location = self.read_u32()? as f64 / 32.0;
            let direction = if self.read_u8()? == 1 {
                GuideDirection::Horizontal
            } else {
                GuideDirection::Vertical
            };
            guides.push(Guide { location, direction });
        }
        
        resources.grid_and_guides = Some(GridAndGuides {
            grid: Grid { horizontal: grid_h, vertical: grid_v },
            guides,
        });
        
        Ok(())
    }

    /// Read global angle (resource 1037)
    pub fn read_global_angle(&mut self, resources: &mut ImageResources) -> Result<()> {
        resources.global_angle = Some(self.read_i32()?);
        Ok(())
    }

    /// Read global altitude (resource 1049)
    pub fn read_global_altitude(&mut self, resources: &mut ImageResources) -> Result<()> {
        resources.global_altitude = Some(self.read_i32()?);
        Ok(())
    }

    /// Read layer state (resource 1024)
    pub fn read_layer_state(&mut self, resources: &mut ImageResources) -> Result<()> {
        resources.layer_state = Some(self.read_u16()?);
        Ok(())
    }

    /// Read layers group (resource 1026)
    pub fn read_layers_group(&mut self, resources: &mut ImageResources, length: usize) -> Result<()> {
        let mut groups = Vec::new();
        let count = length / 2;
        for _ in 0..count {
            groups.push(self.read_u16()?);
        }
        resources.layers_group = Some(groups);
        Ok(())
    }

    /// Read layer selection IDs (resource 1069)
    pub fn read_layer_selection_ids(&mut self, resources: &mut ImageResources) -> Result<()> {
        let count = self.read_u16()? as usize;
        let mut ids = Vec::new();
        for _ in 0..count {
            ids.push(self.read_u32()?);
        }
        resources.layer_selection_ids = Some(ids);
        Ok(())
    }

    /// Read alpha names (resource 1006)
    pub fn read_alpha_names(&mut self, resources: &mut ImageResources, length: usize) -> Result<()> {
        let mut names = Vec::new();
        let mut remaining = length;
        
        while remaining > 0 {
            let name_len = self.read_u8()? as usize;
            remaining -= 1;
            let name = self.read_bytes(name_len)?;
            remaining -= name_len;
            names.push(String::from_utf8_lossy(&name).to_string());
        }
        
        resources.alpha_names = Some(names);
        Ok(())
    }

    /// Read alpha unicode names (resource 1045)
    pub fn read_alpha_unicode_names(&mut self, resources: &mut ImageResources, length: usize) -> Result<()> {
        let mut names = Vec::new();
        let mut remaining = length;
        
        while remaining > 0 {
            let name = self.read_unicode_string()?;
            remaining -= name.len() * 2 + 4;
            names.push(name);
        }
        
        resources.alpha_unicode_names = Some(names);
        Ok(())
    }

    /// Read alpha identifiers (resource 1053)
    pub fn read_alpha_identifiers(&mut self, resources: &mut ImageResources, length: usize) -> Result<()> {
        let count = length / 4;
        let mut ids = Vec::new();
        for _ in 0..count {
            ids.push(self.read_u32()?);
        }
        resources.alpha_identifiers = Some(ids);
        Ok(())
    }

    /// Read ICC profile (resource 1039)
    pub fn read_icc_profile(&mut self, resources: &mut ImageResources, length: usize) -> Result<()> {
        resources.icc_profile = Some(self.read_bytes(length)?);
        Ok(())
    }

    /// Read print scale (resource 1062)
    pub fn read_print_scale(&mut self, resources: &mut ImageResources) -> Result<()> {
        let style = match self.read_i16()? {
            0 => PrintScaleStyle::Centered,
            1 => PrintScaleStyle::SizeToFit,
            2 => PrintScaleStyle::UserDefined,
            v => return Err(PsdError::InvalidFormat(format!("Invalid print scale style: {}", v))),
        };
        
        let x = self.read_f32()?;
        let y = self.read_f32()?;
        let scale = self.read_f32()?;
        
        resources.print_scale = Some(PrintScale { style, x, y, scale });
        Ok(())
    }
}

impl PsdWriter {
    /// Write resolution info
    pub fn write_resolution_info(&mut self, info: &ResolutionInfo) -> Result<()> {
        self.write_fixed_point_32(info.horizontal_res)?;
        self.write_u16(info.horizontal_res_unit as u16)?;
        self.write_u16(info.width_unit as u16)?;
        self.write_fixed_point_32(info.vertical_res)?;
        self.write_u16(info.vertical_res_unit as u16)?;
        self.write_u16(info.height_unit as u16)?;
        Ok(())
    }

    /// Write XMP metadata
    pub fn write_xmp_metadata(&mut self, xmp: &str) -> Result<()> {
        self.write_bytes(xmp.as_bytes())?;
        Ok(())
    }

    /// Write caption digest
    pub fn write_caption_digest(&mut self, digest: &str) -> Result<()> {
        for i in 0..16 {
            let byte = u8::from_str_radix(&digest[i*2..i*2+2], 16)
                .map_err(|_| PsdError::InvalidFormat("Invalid hex digest".to_string()))?;
            self.write_u8(byte)?;
        }
        Ok(())
    }

    /// Write background color
    pub fn write_background_color(&mut self, color: &Color) -> Result<()> {
        self.write_color(Some(color))?;
        Ok(())
    }

    /// Write print flags
    pub fn write_print_flags(&mut self, flags: &PrintFlags) -> Result<()> {
        self.write_u8(if flags.labels { 1 } else { 0 })?;
        self.write_u8(if flags.crop_marks { 1 } else { 0 })?;
        self.write_u8(if flags.color_bars { 1 } else { 0 })?;
        self.write_u8(if flags.registration_marks { 1 } else { 0 })?;
        self.write_u8(if flags.negative { 1 } else { 0 })?;
        self.write_u8(if flags.flip { 1 } else { 0 })?;
        self.write_u8(if flags.interpolate { 1 } else { 0 })?;
        self.write_u8(if flags.caption { 1 } else { 0 })?;
        self.write_u8(if flags.print_flags { 1 } else { 0 })?;
        Ok(())
    }

    /// Write copyright flag
    pub fn write_copyright_flag(&mut self, copyrighted: bool) -> Result<()> {
        self.write_u8(if copyrighted { 1 } else { 0 })?;
        Ok(())
    }

    /// Write URL
    pub fn write_url(&mut self, url: &str) -> Result<()> {
        self.write_ascii_string(url)?;
        Ok(())
    }

    /// Write grid and guides
    pub fn write_grid_and_guides(&mut self, grid_guides: &GridAndGuides) -> Result<()> {
        self.write_u32(1)?; // version
        self.write_u32(grid_guides.grid.horizontal)?;
        self.write_u32(grid_guides.grid.vertical)?;
        self.write_u32(grid_guides.guides.len() as u32)?;
        
        for guide in &grid_guides.guides {
            self.write_u32((guide.location * 32.0) as u32)?;
            self.write_u8(if guide.direction == GuideDirection::Horizontal { 1 } else { 0 })?;
        }
        
        Ok(())
    }

    /// Write global angle
    pub fn write_global_angle(&mut self, angle: i32) -> Result<()> {
        self.write_i32(angle)?;
        Ok(())
    }

    /// Write global altitude
    pub fn write_global_altitude(&mut self, altitude: i32) -> Result<()> {
        self.write_i32(altitude)?;
        Ok(())
    }

    /// Write layer state
    pub fn write_layer_state(&mut self, state: u16) -> Result<()> {
        self.write_u16(state)?;
        Ok(())
    }

    /// Write layers group
    pub fn write_layers_group(&mut self, groups: &[u16]) -> Result<()> {
        for group in groups {
            self.write_u16(*group)?;
        }
        Ok(())
    }

    /// Write layer selection IDs
    pub fn write_layer_selection_ids(&mut self, ids: &[u32]) -> Result<()> {
        self.write_u16(ids.len() as u16)?;
        for id in ids {
            self.write_u32(*id)?;
        }
        Ok(())
    }

    /// Write print scale
    pub fn write_print_scale(&mut self, scale: &PrintScale) -> Result<()> {
        self.write_i16(scale.style as i16)?;
        self.write_f32(scale.x)?;
        self.write_f32(scale.y)?;
        self.write_f32(scale.scale)?;
        Ok(())
    }
}

/// Process all image resources from reader
pub fn read_image_resources<R: Read + Seek>(
    reader: &mut PsdReader<R>,
    length: usize,
) -> Result<ImageResources> {
    let mut resources = ImageResources::default();
    let start_offset = reader.offset;
    
    while (reader.offset - start_offset) < length as u64 {
        let signature = reader.read_signature()?;
        if signature != "8BIM" {
            return Err(PsdError::InvalidFormat(format!("Invalid resource signature: {}", signature)));
        }
        
        let resource_id = reader.read_u16()?;
        
        // Read pascal string (name)
        let name_length = reader.read_u8()? as usize;
        reader.skip_bytes(name_length)?;
        if (name_length + 1) % 2 != 0 {
            reader.skip_bytes(1)?; // Padding
        }
        
        let data_length = reader.read_u32()? as usize;
        let resource_start = reader.offset;
        
        // Dispatch to appropriate handler
        match resource_id {
            1005 => reader.read_resolution_info(&mut resources)?,
            1010 => reader.read_background_color(&mut resources)?,
            1011 => reader.read_print_flags(&mut resources)?,
            1024 => reader.read_layer_state(&mut resources)?,
            1026 => reader.read_layers_group(&mut resources, data_length)?,
            1032 => reader.read_grid_and_guides(&mut resources)?,
            1034 => reader.read_copyright_flag(&mut resources)?,
            1035 => reader.read_url(&mut resources, data_length)?,
            1037 => reader.read_global_angle(&mut resources)?,
            1049 => reader.read_global_altitude(&mut resources)?,
            1060 => reader.read_xmp_metadata(&mut resources, data_length)?,
            1061 => reader.read_caption_digest(&mut resources)?,
            1062 => reader.read_print_scale(&mut resources)?,
            1069 => reader.read_layer_selection_ids(&mut resources)?,
            1006 => reader.read_alpha_names(&mut resources, data_length)?,
            1045 => reader.read_alpha_unicode_names(&mut resources, data_length)?,
            1053 => reader.read_alpha_identifiers(&mut resources, data_length)?,
            1039 => reader.read_icc_profile(&mut resources, data_length)?,
            _ => {
                // Store unknown resources
                let data = reader.read_bytes(data_length)?;
                resources.unknown.insert(resource_id, data);
            }
        }
        
        // Ensure we consumed exactly the right amount
        let consumed = (reader.offset - resource_start) as usize;
        if consumed < data_length {
            reader.skip_bytes(data_length - consumed)?;
        }
        
        // Align to even boundary
        if data_length % 2 != 0 {
            reader.skip_bytes(1)?;
        }
    }
    
    Ok(resources)
}

/// Write all image resources
pub fn write_image_resources(
    writer: &mut PsdWriter,
    resources: &ImageResources,
) -> Result<()> {
    // Helper to write a resource block
    let write_resource = |writer: &mut PsdWriter, id: u16, write_fn: &dyn Fn(&mut PsdWriter) -> Result<()>| -> Result<()> {
        writer.write_signature("8BIM")?;
        writer.write_u16(id)?;
        writer.write_u8(0)?; // Empty name
        writer.write_u8(0)?; // Name padding
        
        // Write to temp buffer to get length
        let mut temp_writer = PsdWriter::new(1024);
        write_fn(&mut temp_writer)?;
        let data = temp_writer.get_buffer();
        
        writer.write_u32(data.len() as u32)?;
        writer.write_bytes(data)?;
        
        // Pad to even boundary
        if data.len() % 2 != 0 {
            writer.write_u8(0)?;
        }
        
        Ok(())
    };
    
    // Write each resource type
    if let Some(ref info) = resources.resolution_info {
        write_resource(writer, 1005, &|w| w.write_resolution_info(info))?;
    }
    
    if let Some(ref color) = resources.background_color {
        write_resource(writer, 1010, &|w| w.write_background_color(color))?;
    }
    
    if let Some(ref flags) = resources.print_flags {
        write_resource(writer, 1011, &|w| w.write_print_flags(flags))?;
    }
    
    if let Some(state) = resources.layer_state {
        write_resource(writer, 1024, &|w| w.write_layer_state(state))?;
    }
    
    if let Some(ref groups) = resources.layers_group {
        write_resource(writer, 1026, &|w| w.write_layers_group(groups))?;
    }
    
    if let Some(ref grid_guides) = resources.grid_and_guides {
        write_resource(writer, 1032, &|w| w.write_grid_and_guides(grid_guides))?;
    }
    
    if let Some(copyrighted) = resources.copyrighted {
        write_resource(writer, 1034, &|w| w.write_copyright_flag(copyrighted))?;
    }
    
    if let Some(ref url) = resources.url {
        write_resource(writer, 1035, &|w| w.write_url(url))?;
    }
    
    if let Some(angle) = resources.global_angle {
        write_resource(writer, 1037, &|w| w.write_global_angle(angle))?;
    }
    
    if let Some(altitude) = resources.global_altitude {
        write_resource(writer, 1049, &|w| w.write_global_altitude(altitude))?;
    }
    
    if let Some(ref xmp) = resources.xmp_metadata {
        write_resource(writer, 1060, &|w| w.write_xmp_metadata(xmp))?;
    }
    
    if let Some(ref digest) = resources.caption_digest {
        write_resource(writer, 1061, &|w| w.write_caption_digest(digest))?;
    }
    
    if let Some(ref scale) = resources.print_scale {
        write_resource(writer, 1062, &|w| w.write_print_scale(scale))?;
    }
    
    if let Some(ref ids) = resources.layer_selection_ids {
        write_resource(writer, 1069, &|w| w.write_layer_selection_ids(ids))?;
    }
    
    // Write unknown resources
    for (id, data) in &resources.unknown {
        writer.write_signature("8BIM")?;
        writer.write_u16(*id)?;
        writer.write_u8(0)?;
        writer.write_u8(0)?;
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
    fn test_resolution_info_roundtrip() {
        let info = ResolutionInfo {
            horizontal_res: 72.0,
            horizontal_res_unit: ResolutionUnit::PixelsPerInch,
            width_unit: MeasurementUnit::Inches,
            vertical_res: 72.0,
            vertical_res_unit: ResolutionUnit::PixelsPerInch,
            height_unit: MeasurementUnit::Inches,
        };
        
        let mut writer = PsdWriter::new(128);
        writer.write_resolution_info(&info).unwrap();
        
        let buffer = writer.into_buffer();
        let cursor = std::io::Cursor::new(buffer);
        let mut reader = PsdReader::new(cursor, Default::default());
        
        let mut resources = ImageResources::default();
        reader.read_resolution_info(&mut resources).unwrap();
        
        let read_info = resources.resolution_info.unwrap();
        assert_eq!(read_info.horizontal_res, info.horizontal_res);
        assert_eq!(read_info.vertical_res, info.vertical_res);
    }

    #[test]
    fn test_grid_and_guides_roundtrip() {
        let grid_guides = GridAndGuides {
            grid: Grid { horizontal: 576, vertical: 576 },
            guides: vec![
                Guide { location: 100.0, direction: GuideDirection::Vertical },
                Guide { location: 200.0, direction: GuideDirection::Horizontal },
            ],
        };
        
        let mut writer = PsdWriter::new(256);
        writer.write_grid_and_guides(&grid_guides).unwrap();
        
        let buffer = writer.into_buffer();
        let cursor = std::io::Cursor::new(buffer);
        let mut reader = PsdReader::new(cursor, Default::default());
        
        let mut resources = ImageResources::default();
        reader.read_grid_and_guides(&mut resources).unwrap();
        
        let read_grid_guides = resources.grid_and_guides.unwrap();
        assert_eq!(read_grid_guides.grid.horizontal, grid_guides.grid.horizontal);
        assert_eq!(read_grid_guides.guides.len(), grid_guides.guides.len());
    }
}
