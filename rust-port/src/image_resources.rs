//! Image resource handlers for PSD files
//!
//! Image resources contain document-level information like resolution,
//! guides, grids, color profiles, and thumbnails.

use crate::binrw_support::{
    decode_be, encode_be, GridAndGuidesHeaderRecord, GuideRecord, ImageResourceHeaderRecord,
    ImageResourceLengthRecord, LayerStateRecord, PrintFlagsRecord, PrintScaleRecord,
    ResolutionInfoRecord, SignedI32Record, U16ListCountRecord, U32ValueRecord, U8BoolRecord,
};
use crate::descriptor::Descriptor;
use crate::error::{PsdError, Result};
use crate::reader::PsdReader;
use crate::types::{BlendMode, Color, LayerCompCapturedInfo, PsdIntCode, PsdU16Code, PsdU32Code};
use crate::writer::PsdWriter;
use std::collections::HashMap;
use std::io::{Read, Seek};

/// Image resources structure
#[derive(Debug, Clone, PartialEq, Default)]
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
    /// Layer clipping values (resource 1026)
    pub clipping: Option<Vec<u16>>,
    /// Resource visibility (resource 1072)
    pub resource_visibility_typed: Option<ResourceVisibility>,
    /// Custom points (resource 1073)
    pub custom_points_typed: Option<CustomPointsResource>,
    /// Display info (resource 1036)
    pub display_info_typed: Option<DisplayInfoResource>,
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
    /// Path resources (2000..=2998)
    pub path_resources: HashMap<u16, Vec<PathResourceRecord>>,
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
    /// Variables XML (resource 7000)
    pub variables: Option<String>,
    /// Data sets XML (resource 7001)
    pub data_sets: Option<String>,
    /// Generic descriptor resources (1065, 1074, 1075)
    pub descriptor_resources: HashMap<u16, Descriptor>,
}

/// Simplified path resource record (26-byte document path record)
#[derive(Debug, Clone, PartialEq)]
pub struct PathResourceRecord {
    pub record_type: u16,
    pub points: Vec<Point>,
    pub closed: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResourceVisibility {
    pub values: Vec<bool>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CustomPointsResource {
    pub version: u32,
    pub points: Vec<crate::psd::CustomPoint>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DisplayInfoResource {
    pub version: u16,
    pub h_res_unit: PsdU16Code,
    pub v_res_unit: PsdU16Code,
    pub width_unit: PsdU16Code,
    pub height_unit: PsdU16Code,
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

fn parse_display_info_resource(bytes: &[u8]) -> Option<DisplayInfoResource> {
    if bytes.len() < 18 {
        return None;
    }
    Some(DisplayInfoResource {
        version: u16::from_be_bytes([bytes[0], bytes[1]]),
        h_res_unit: PsdU16Code(u16::from_be_bytes([bytes[2], bytes[3]])),
        v_res_unit: PsdU16Code(u16::from_be_bytes([bytes[6], bytes[7]])),
        width_unit: PsdU16Code(u16::from_be_bytes([bytes[10], bytes[11]])),
        height_unit: PsdU16Code(u16::from_be_bytes([bytes[14], bytes[15]])),
    })
}

fn build_display_info_resource(info: &DisplayInfoResource) -> Vec<u8> {
    let mut bytes = vec![0u8; 28];
    bytes[0..2].copy_from_slice(&info.version.to_be_bytes());
    bytes[2..4].copy_from_slice(&info.h_res_unit.0.to_be_bytes());
    bytes[4..6].copy_from_slice(&1u16.to_be_bytes());
    bytes[6..8].copy_from_slice(&info.v_res_unit.0.to_be_bytes());
    bytes[8..10].copy_from_slice(&1u16.to_be_bytes());
    bytes[10..12].copy_from_slice(&info.width_unit.0.to_be_bytes());
    bytes[12..14].copy_from_slice(&1u16.to_be_bytes());
    bytes[14..16].copy_from_slice(&info.height_unit.0.to_be_bytes());
    bytes[16..18].copy_from_slice(&1u16.to_be_bytes());
    bytes
}

fn parse_custom_points_resource(bytes: &[u8]) -> CustomPointsResource {
    if bytes.len() < 8 {
        return CustomPointsResource {
            version: 0,
            points: Vec::new(),
        };
    }
    let version = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
    let count = u32::from_be_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]) as usize;
    let mut points = Vec::new();
    let mut offset = 8;
    for _ in 0..count {
        if offset + 14 > bytes.len() {
            break;
        }
        let y_raw = i32::from_be_bytes([
            bytes[offset + 2],
            bytes[offset + 3],
            bytes[offset + 4],
            bytes[offset + 5],
        ]);
        let x_raw = i32::from_be_bytes([
            bytes[offset + 6],
            bytes[offset + 7],
            bytes[offset + 8],
            bytes[offset + 9],
        ]);
        points.push(crate::psd::CustomPoint {
            x: x_raw as f64 / 65536.0,
            y: if version >= 2 {
                y_raw as f64 / 65536.0
            } else {
                i16::from_be_bytes([bytes[offset + 2], bytes[offset + 3]]) as f64
            },
        });
        offset += 14;
    }
    CustomPointsResource { version, points }
}

fn build_custom_points_resource(points: &CustomPointsResource) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(8 + points.points.len() * 14);
    bytes.extend_from_slice(&points.version.to_be_bytes());
    bytes.extend_from_slice(&(points.points.len() as u32).to_be_bytes());
    for point in &points.points {
        let y_fixed = (point.y * 65536.0) as i32;
        let x_fixed = (point.x * 65536.0) as i32;
        bytes.extend_from_slice(&14i16.to_be_bytes());
        bytes.extend_from_slice(&y_fixed.to_be_bytes());
        bytes.extend_from_slice(&x_fixed.to_be_bytes());
        bytes.extend_from_slice(&(-1i16).to_be_bytes());
        bytes.extend_from_slice(&8i16.to_be_bytes());
    }
    bytes
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
    Builtin {
        name: String,
    },
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
    pub slices: Vec<Slice>,
}

/// Slice
#[derive(Debug, Clone, PartialEq)]
pub struct Slice {
    pub id: u32,
    pub group_id: u32,
    pub origin: PsdU32Code,
    pub associated_layer_id: u32,
    pub name: String,
    pub slice_type: PsdU32Code,
    pub bounds: Bounds,
    pub url: String,
    pub target: String,
    pub message: String,
    pub alt_tag: String,
    pub cell_text: String,
    pub horizontal_align: PsdIntCode,
    pub vertical_align: PsdIntCode,
    pub alpha: u8,
    pub bg_color: [u8; 4],
    pub cell_is_html: bool,
    pub source_id: Option<u32>,
    pub source_type: Option<PsdU32Code>,
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
    pub captured_info: LayerCompCapturedInfo,
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
    pub blend_mode: BlendMode,
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
    fn read_slice_string(&mut self) -> Result<String> {
        let len = decode_be::<U32ValueRecord>(&self.read_bytes(4)?, "slice string length")?.value
            as usize;
        Ok(String::from_utf8_lossy(&self.read_bytes(len)?).to_string())
    }

    fn read_slices(&mut self, resources: &mut ImageResources, length: usize) -> Result<()> {
        let start = self.offset;
        if length < 8 {
            self.skip_bytes(length)?;
            return Ok(());
        }
        let version = decode_be::<U32ValueRecord>(&self.read_bytes(4)?, "slices version")?.value;
        let count =
            decode_be::<U32ValueRecord>(&self.read_bytes(4)?, "slices count")?.value as usize;
        let mut slices = Vec::new();
        for _ in 0..count {
            if (self.offset - start) as usize + 16 > length {
                break;
            }
            let id = self.read_i32()? as u32;
            let group_id = self.read_i32()? as u32;
            let origin = self.read_i32()? as u32;
            let associated_layer_id = self.read_i32()? as u32;
            let name = self.read_slice_string()?;
            if (self.offset - start) as usize + 20 > length {
                break;
            }
            let slice_type = self.read_i32()? as u32;
            let top = self.read_i32()?;
            let left = self.read_i32()?;
            let bottom = self.read_i32()?;
            let right = self.read_i32()?;
            let url = self.read_slice_string()?;
            let target = self.read_slice_string()?;
            let message = self.read_slice_string()?;
            let alt_tag = self.read_slice_string()?;
            let cell_text = self.read_slice_string()?;
            if (self.offset - start) as usize + 13 > length {
                break;
            }
            let horizontal_align = self.read_i32()?;
            let vertical_align = self.read_i32()?;
            let alpha = self.read_u8()?;
            if (self.offset - start) as usize + 8 > length {
                break;
            }
            let bg_color = [
                self.read_u8()?,
                self.read_u8()?,
                self.read_u8()?,
                self.read_u8()?,
            ];
            let cell_is_html = self.read_i32()?;
            let mut source_id = None;
            let mut source_type = None;
            if version >= 7 && (self.offset - start) as usize + 8 <= length {
                source_id = Some(self.read_u32()?);
                source_type = Some(self.read_u32()?);
            }
            slices.push(Slice {
                id,
                group_id,
                origin: PsdU32Code(origin),
                associated_layer_id,
                name,
                slice_type: PsdU32Code(slice_type),
                bounds: Bounds {
                    top,
                    left,
                    bottom,
                    right,
                },
                url,
                target,
                message,
                alt_tag,
                cell_text,
                horizontal_align: PsdIntCode(horizontal_align),
                vertical_align: PsdIntCode(vertical_align),
                alpha,
                bg_color,
                cell_is_html: cell_is_html != 0,
                source_id,
                source_type: source_type.map(PsdU32Code),
            });
        }
        resources.slices = Some(Slices { version, slices });
        Ok(())
    }

    fn read_path_resource_records(
        &mut self,
        resources: &mut ImageResources,
        resource_id: u16,
        length: usize,
    ) -> Result<()> {
        let bytes = self.read_bytes(length)?;
        let mut records = Vec::new();
        let mut offset = 0usize;
        while offset + 26 <= bytes.len() {
            let record_type = u16::from_be_bytes([bytes[offset], bytes[offset + 1]]);
            offset += 2;
            let mut points = Vec::with_capacity(4);
            for _ in 0..4 {
                let x = i32::from_be_bytes(bytes[offset..offset + 4].try_into().unwrap()) as f64
                    / 65536.0;
                offset += 4;
                let y = i32::from_be_bytes(bytes[offset..offset + 4].try_into().unwrap()) as f64
                    / 65536.0;
                offset += 4;
                points.push(Point { x, y });
            }
            offset += 2;
            records.push(PathResourceRecord {
                record_type,
                closed: matches!(record_type, 1 | 2 | 3),
                points,
            });
        }
        if !records.is_empty() {
            resources.path_resources.insert(resource_id, records);
        }
        Ok(())
    }

    /// Read resolution info (resource 1005)
    pub fn read_resolution_info(&mut self, resources: &mut ImageResources) -> Result<()> {
        let record: ResolutionInfoRecord = decode_be(&self.read_bytes(16)?, "resolution info")?;
        let horizontal_res_unit = match record.horizontal_res_unit {
            1 => ResolutionUnit::PixelsPerInch,
            2 => ResolutionUnit::PixelsPerCentimeter,
            v => {
                return Err(PsdError::InvalidFormat(format!(
                    "Invalid resolution unit: {}",
                    v
                )))
            }
        };
        let width_unit = match record.width_unit {
            1 => MeasurementUnit::Inches,
            2 => MeasurementUnit::Centimeters,
            3 => MeasurementUnit::Points,
            4 => MeasurementUnit::Picas,
            5 => MeasurementUnit::Columns,
            v => {
                return Err(PsdError::InvalidFormat(format!(
                    "Invalid measurement unit: {}",
                    v
                )))
            }
        };

        let vertical_res_unit = match record.vertical_res_unit {
            1 => ResolutionUnit::PixelsPerInch,
            2 => ResolutionUnit::PixelsPerCentimeter,
            v => {
                return Err(PsdError::InvalidFormat(format!(
                    "Invalid resolution unit: {}",
                    v
                )))
            }
        };
        let height_unit = match record.height_unit {
            1 => MeasurementUnit::Inches,
            2 => MeasurementUnit::Centimeters,
            3 => MeasurementUnit::Points,
            4 => MeasurementUnit::Picas,
            5 => MeasurementUnit::Columns,
            v => {
                return Err(PsdError::InvalidFormat(format!(
                    "Invalid measurement unit: {}",
                    v
                )))
            }
        };

        resources.resolution_info = Some(ResolutionInfo {
            horizontal_res: record.horizontal_res_fixed as f64 / 65536.0,
            horizontal_res_unit,
            width_unit,
            vertical_res: record.vertical_res_fixed as f64 / 65536.0,
            vertical_res_unit,
            height_unit,
        });

        Ok(())
    }

    /// Read XMP metadata (resource 1060)
    pub fn read_xmp_metadata(
        &mut self,
        resources: &mut ImageResources,
        length: usize,
    ) -> Result<()> {
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
        let record: PrintFlagsRecord = decode_be(&self.read_bytes(9)?, "print flags")?;
        resources.print_flags = Some(PrintFlags {
            labels: record.labels != 0,
            crop_marks: record.crop_marks != 0,
            color_bars: record.color_bars != 0,
            registration_marks: record.registration_marks != 0,
            negative: record.negative != 0,
            flip: record.flip != 0,
            interpolate: record.interpolate != 0,
            caption: record.caption != 0,
            print_flags: record.print_flags != 0,
        });
        Ok(())
    }

    /// Read copyright flag (resource 1034)
    pub fn read_copyright_flag(&mut self, resources: &mut ImageResources) -> Result<()> {
        let record: U8BoolRecord = decode_be(&self.read_bytes(1)?, "copyright flag")?;
        resources.copyrighted = Some(record.value != 0);
        Ok(())
    }

    /// Read URL (resource 1035)
    pub fn read_url(&mut self, resources: &mut ImageResources, length: usize) -> Result<()> {
        resources.url = Some(self.read_ascii_string(length)?);
        Ok(())
    }

    /// Read grid and guides (resource 1032)
    pub fn read_grid_and_guides(&mut self, resources: &mut ImageResources) -> Result<()> {
        let header: GridAndGuidesHeaderRecord =
            decode_be(&self.read_bytes(16)?, "grid and guides header")?;
        if header.version != 1 {
            return Err(PsdError::InvalidFormat(format!(
                "Invalid grid/guides version: {}",
                header.version
            )));
        }

        let mut guides = Vec::with_capacity(header.guide_count as usize);
        for _ in 0..header.guide_count {
            let guide: GuideRecord = decode_be(&self.read_bytes(5)?, "guide record")?;
            let direction = if guide.direction == 1 {
                GuideDirection::Horizontal
            } else {
                GuideDirection::Vertical
            };
            guides.push(Guide {
                location: guide.location_times_32 as f64 / 32.0,
                direction,
            });
        }

        resources.grid_and_guides = Some(GridAndGuides {
            grid: Grid {
                horizontal: header.grid_horizontal,
                vertical: header.grid_vertical,
            },
            guides,
        });

        Ok(())
    }

    /// Read global angle (resource 1037)
    pub fn read_global_angle(&mut self, resources: &mut ImageResources) -> Result<()> {
        let record: SignedI32Record = decode_be(&self.read_bytes(4)?, "global angle")?;
        resources.global_angle = Some(record.value);
        Ok(())
    }

    /// Read global altitude (resource 1049)
    pub fn read_global_altitude(&mut self, resources: &mut ImageResources) -> Result<()> {
        let record: SignedI32Record = decode_be(&self.read_bytes(4)?, "global altitude")?;
        resources.global_altitude = Some(record.value);
        Ok(())
    }

    /// Read layer state (resource 1024)
    pub fn read_layer_state(&mut self, resources: &mut ImageResources) -> Result<()> {
        let record: LayerStateRecord = decode_be(&self.read_bytes(2)?, "layer state")?;
        resources.layer_state = Some(record.state);
        Ok(())
    }

    /// Read layers group (resource 1026)
    pub fn read_clipping(&mut self, resources: &mut ImageResources, length: usize) -> Result<()> {
        let mut groups = Vec::new();
        let count = length / 2;
        for _ in 0..count {
            groups.push(
                decode_be::<LayerStateRecord>(&self.read_bytes(2)?, "layer group entry")?.state,
            );
        }
        resources.clipping = Some(groups);
        Ok(())
    }

    /// Read layer selection IDs (resource 1069)
    pub fn read_layer_selection_ids(&mut self, resources: &mut ImageResources) -> Result<()> {
        let count = decode_be::<U16ListCountRecord>(&self.read_bytes(2)?, "layer selection count")?
            .count as usize;
        let mut ids = Vec::new();
        for _ in 0..count {
            ids.push(
                decode_be::<U32ValueRecord>(&self.read_bytes(4)?, "layer selection id")?.value,
            );
        }
        resources.layer_selection_ids = Some(ids);
        Ok(())
    }

    /// Read alpha names (resource 1006)
    pub fn read_alpha_names(
        &mut self,
        resources: &mut ImageResources,
        length: usize,
    ) -> Result<()> {
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
    pub fn read_alpha_unicode_names(
        &mut self,
        resources: &mut ImageResources,
        length: usize,
    ) -> Result<()> {
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
    pub fn read_alpha_identifiers(
        &mut self,
        resources: &mut ImageResources,
        length: usize,
    ) -> Result<()> {
        let count = length / 4;
        let mut ids = Vec::new();
        for _ in 0..count {
            ids.push(decode_be::<U32ValueRecord>(&self.read_bytes(4)?, "alpha identifier")?.value);
        }
        resources.alpha_identifiers = Some(ids);
        Ok(())
    }

    /// Read ICC profile (resource 1039)
    pub fn read_icc_profile(
        &mut self,
        resources: &mut ImageResources,
        length: usize,
    ) -> Result<()> {
        resources.icc_profile = Some(self.read_bytes(length)?);
        Ok(())
    }

    /// Read print scale (resource 1062)
    pub fn read_print_scale(&mut self, resources: &mut ImageResources) -> Result<()> {
        let record: PrintScaleRecord = decode_be(&self.read_bytes(14)?, "print scale")?;
        let style = match record.style {
            0 => PrintScaleStyle::Centered,
            1 => PrintScaleStyle::SizeToFit,
            2 => PrintScaleStyle::UserDefined,
            v => {
                return Err(PsdError::InvalidFormat(format!(
                    "Invalid print scale style: {}",
                    v
                )))
            }
        };

        resources.print_scale = Some(PrintScale {
            style,
            x: record.x,
            y: record.y,
            scale: record.scale,
        });
        Ok(())
    }
}

impl PsdWriter {
    fn write_slice_string(&mut self, value: &str) -> Result<()> {
        self.write_bytes(&encode_be(
            &U32ValueRecord {
                value: value.len() as u32,
            },
            "slice string length",
        )?)?;
        self.write_bytes(value.as_bytes())
    }

    /// Write resolution info
    pub fn write_resolution_info(&mut self, info: &ResolutionInfo) -> Result<()> {
        let record = ResolutionInfoRecord {
            horizontal_res_fixed: (info.horizontal_res * 65536.0) as i32,
            horizontal_res_unit: info.horizontal_res_unit as u16,
            width_unit: info.width_unit as u16,
            vertical_res_fixed: (info.vertical_res * 65536.0) as i32,
            vertical_res_unit: info.vertical_res_unit as u16,
            height_unit: info.height_unit as u16,
        };
        self.write_bytes(&encode_be(&record, "resolution info")?)
    }

    /// Write XMP metadata
    pub fn write_xmp_metadata(&mut self, xmp: &str) -> Result<()> {
        self.write_bytes(xmp.as_bytes())?;
        Ok(())
    }

    /// Write caption digest
    pub fn write_caption_digest(&mut self, digest: &str) -> Result<()> {
        for i in 0..16 {
            let byte = u8::from_str_radix(&digest[i * 2..i * 2 + 2], 16)
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
        let record = PrintFlagsRecord {
            labels: u8::from(flags.labels),
            crop_marks: u8::from(flags.crop_marks),
            color_bars: u8::from(flags.color_bars),
            registration_marks: u8::from(flags.registration_marks),
            negative: u8::from(flags.negative),
            flip: u8::from(flags.flip),
            interpolate: u8::from(flags.interpolate),
            caption: u8::from(flags.caption),
            print_flags: u8::from(flags.print_flags),
        };
        self.write_bytes(&encode_be(&record, "print flags")?)
    }

    /// Write copyright flag
    pub fn write_copyright_flag(&mut self, copyrighted: bool) -> Result<()> {
        self.write_bytes(&encode_be(
            &U8BoolRecord {
                value: u8::from(copyrighted),
            },
            "copyright flag",
        )?)
    }

    /// Write URL
    pub fn write_url(&mut self, url: &str) -> Result<()> {
        self.write_ascii_string(url)?;
        Ok(())
    }

    /// Write grid and guides
    pub fn write_grid_and_guides(&mut self, grid_guides: &GridAndGuides) -> Result<()> {
        let header = GridAndGuidesHeaderRecord {
            version: 1,
            grid_horizontal: grid_guides.grid.horizontal,
            grid_vertical: grid_guides.grid.vertical,
            guide_count: grid_guides.guides.len() as u32,
        };
        self.write_bytes(&encode_be(&header, "grid and guides header")?)?;

        for guide in &grid_guides.guides {
            let record = GuideRecord {
                location_times_32: (guide.location * 32.0) as u32,
                direction: if guide.direction == GuideDirection::Horizontal {
                    1
                } else {
                    0
                },
            };
            self.write_bytes(&encode_be(&record, "guide record")?)?;
        }

        Ok(())
    }

    /// Write global angle
    pub fn write_global_angle(&mut self, angle: i32) -> Result<()> {
        self.write_bytes(&encode_be(
            &SignedI32Record { value: angle },
            "global angle",
        )?)
    }

    /// Write global altitude
    pub fn write_global_altitude(&mut self, altitude: i32) -> Result<()> {
        self.write_bytes(&encode_be(
            &SignedI32Record { value: altitude },
            "global altitude",
        )?)
    }

    /// Write layer state
    pub fn write_layer_state(&mut self, state: u16) -> Result<()> {
        self.write_bytes(&encode_be(&LayerStateRecord { state }, "layer state")?)
    }

    /// Write layers group
    pub fn write_clipping(&mut self, clipping: &[u16]) -> Result<()> {
        for value in clipping {
            self.write_bytes(&encode_be(
                &LayerStateRecord { state: *value },
                "layer group entry",
            )?)?;
        }
        Ok(())
    }

    /// Write layer selection IDs
    pub fn write_layer_selection_ids(&mut self, ids: &[u32]) -> Result<()> {
        self.write_bytes(&encode_be(
            &U16ListCountRecord {
                count: ids.len() as u16,
            },
            "layer selection count",
        )?)?;
        for id in ids {
            self.write_bytes(&encode_be(
                &U32ValueRecord { value: *id },
                "layer selection id",
            )?)?;
        }
        Ok(())
    }

    /// Write print scale
    pub fn write_print_scale(&mut self, scale: &PrintScale) -> Result<()> {
        self.write_bytes(&encode_be(
            &PrintScaleRecord {
                style: scale.style as i16,
                x: scale.x,
                y: scale.y,
                scale: scale.scale,
            },
            "print scale",
        )?)
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
        let header: ImageResourceHeaderRecord =
            decode_be(&reader.read_bytes(7)?, "image resource header")?;
        if &header.signature != b"8BIM" && &header.signature != b"MeSa" {
            return Err(PsdError::InvalidFormat(format!(
                "Invalid resource signature: {}",
                String::from_utf8_lossy(&header.signature),
            )));
        }
        let resource_id = header.resource_id;

        // Read pascal string (name)
        let name_length = header.name_length as usize;
        reader.skip_bytes(name_length)?;
        if (name_length + 1) % 2 != 0 {
            reader.skip_bytes(1)?; // Padding
        }

        let data_length = decode_be::<ImageResourceLengthRecord>(
            &reader.read_bytes(4)?,
            "image resource length",
        )?
        .data_length as usize;
        let resource_start = reader.offset;

        // Dispatch to appropriate handler
        match resource_id {
            1005 => reader.read_resolution_info(&mut resources)?,
            1010 => reader.read_background_color(&mut resources)?,
            1011 => reader.read_print_flags(&mut resources)?,
            1024 => reader.read_layer_state(&mut resources)?,
            1026 => reader.read_clipping(&mut resources, data_length)?,
            1032 => reader.read_grid_and_guides(&mut resources)?,
            1036 => {
                let bytes = reader.read_bytes(data_length)?;
                resources.display_info_typed = parse_display_info_resource(&bytes);
            }
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
            1050 => reader.read_slices(&mut resources, data_length)?,
            1072 => {
                let bytes = reader.read_bytes(data_length)?;
                resources.resource_visibility_typed = Some(ResourceVisibility {
                    values: bytes.into_iter().map(|b| b == 1).collect(),
                });
            }
            1073 => {
                let bytes = reader.read_bytes(data_length)?;
                resources.custom_points_typed = Some(parse_custom_points_resource(&bytes));
            }
            1065 | 1074 | 1075 | 2999 | 3000 => {
                let _version = decode_be::<U32ValueRecord>(
                    &reader.read_bytes(4)?,
                    "descriptor resource version",
                )?
                .value;
                let desc = reader.read_descriptor_structure()?;
                resources.descriptor_resources.insert(resource_id, desc);
            }
            2000..=2998 => {
                reader.read_path_resource_records(&mut resources, resource_id, data_length)?
            }
            7000 => {
                let bytes = reader.read_bytes(data_length)?;
                resources.variables = Some(String::from_utf8_lossy(&bytes).to_string());
            }
            7001 => {
                let bytes = reader.read_bytes(data_length)?;
                resources.data_sets = Some(String::from_utf8_lossy(&bytes).to_string());
            }
            _ => reader.skip_bytes(data_length)?,
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
pub fn write_image_resources(writer: &mut PsdWriter, resources: &ImageResources) -> Result<()> {
    // Helper to write a resource block
    let write_resource = |writer: &mut PsdWriter,
                          id: u16,
                          write_fn: &dyn Fn(&mut PsdWriter) -> Result<()>|
     -> Result<()> {
        writer.write_bytes(&encode_be(
            &ImageResourceHeaderRecord {
                signature: *b"8BIM",
                resource_id: id,
                name_length: 0,
            },
            "image resource header",
        )?)?;
        writer.write_u8(0)?; // Name padding

        // Write to temp buffer to get length
        let mut temp_writer = PsdWriter::new(1024);
        write_fn(&mut temp_writer)?;
        let data = temp_writer.get_buffer();

        writer.write_bytes(&encode_be(
            &ImageResourceLengthRecord {
                data_length: data.len() as u32,
            },
            "image resource length",
        )?)?;
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

    if let Some(ref clipping) = resources.clipping {
        write_resource(writer, 1026, &|w| w.write_clipping(clipping))?;
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

    if let Some(ref info) = resources.display_info_typed {
        write_resource(writer, 1036, &|w| {
            w.write_bytes(&build_display_info_resource(info))
        })?;
    }

    if let Some(ref visibility) = resources.resource_visibility_typed {
        write_resource(writer, 1072, &|w| {
            w.write_bytes(
                &visibility
                    .values
                    .iter()
                    .map(|v| if *v { 1 } else { 0 })
                    .collect::<Vec<u8>>(),
            )
        })?;
    }

    if let Some(ref points) = resources.custom_points_typed {
        write_resource(writer, 1073, &|w| {
            w.write_bytes(&build_custom_points_resource(points))
        })?;
    }

    if let Some(ref slices) = resources.slices {
        write_resource(writer, 1050, &|w| {
            w.write_bytes(&encode_be(
                &U32ValueRecord {
                    value: slices.version,
                },
                "slices version",
            )?)?;
            w.write_bytes(&encode_be(
                &U32ValueRecord {
                    value: slices.slices.len() as u32,
                },
                "slices count",
            )?)?;
            for slice in &slices.slices {
                w.write_i32(slice.id as i32)?;
                w.write_i32(slice.group_id as i32)?;
                w.write_i32(slice.origin.0 as i32)?;
                w.write_i32(slice.associated_layer_id as i32)?;
                w.write_slice_string(&slice.name)?;
                w.write_i32(slice.slice_type.0 as i32)?;
                w.write_i32(slice.bounds.top)?;
                w.write_i32(slice.bounds.left)?;
                w.write_i32(slice.bounds.bottom)?;
                w.write_i32(slice.bounds.right)?;
                w.write_slice_string(&slice.url)?;
                w.write_slice_string(&slice.target)?;
                w.write_slice_string(&slice.message)?;
                w.write_slice_string(&slice.alt_tag)?;
                w.write_slice_string(&slice.cell_text)?;
                w.write_i32(slice.horizontal_align.0)?;
                w.write_i32(slice.vertical_align.0)?;
                w.write_u8(slice.alpha)?;
                w.write_bytes(&slice.bg_color)?;
                w.write_i32(i32::from(slice.cell_is_html))?;
                if slices.version >= 7 {
                    w.write_u32(slice.source_id.unwrap_or(0))?;
                    w.write_u32(slice.source_type.unwrap_or(PsdU32Code(0)).0)?;
                }
            }
            Ok(())
        })?;
    }

    // Write alpha names (1006)
    if let Some(ref names) = resources.alpha_names {
        write_resource(writer, 1006, &|w| {
            for name in names {
                w.write_u8(name.len() as u8)?;
                w.write_bytes(name.as_bytes())?;
            }
            Ok(())
        })?;
    }

    // Write alpha unicode names (1045)
    if let Some(ref names) = resources.alpha_unicode_names {
        write_resource(writer, 1045, &|w| {
            for name in names {
                w.write_unicode_string(name)?;
            }
            Ok(())
        })?;
    }

    // Write alpha identifiers (1053)
    if let Some(ref ids) = resources.alpha_identifiers {
        write_resource(writer, 1053, &|w| {
            for &id in ids {
                w.write_u32(id)?;
            }
            Ok(())
        })?;
    }

    // Write ICC profile (1039)
    if let Some(ref profile) = resources.icc_profile {
        write_resource(writer, 1039, &|w| w.write_bytes(profile))?;
    }

    // Write descriptor resources (1065, 1074, 1075)
    for (&id, desc) in &resources.descriptor_resources {
        write_resource(writer, id, &|w| {
            w.write_bytes(&encode_be(
                &U32ValueRecord { value: 16 },
                "descriptor resource version",
            )?)?;
            w.write_descriptor_structure(desc)
        })?;
    }

    for (&id, records) in &resources.path_resources {
        write_resource(writer, id, &|w| {
            for record in records {
                w.write_u16(record.record_type)?;
                for point in record.points.iter().take(4) {
                    w.write_i32((point.x * 65536.0).round() as i32)?;
                    w.write_i32((point.y * 65536.0).round() as i32)?;
                }
                for _ in record.points.len()..4 {
                    w.write_i32(0)?;
                    w.write_i32(0)?;
                }
                w.write_u16(0)?;
            }
            Ok(())
        })?;
    }

    if let Some(ref xml) = resources.variables {
        write_resource(writer, 7000, &|w| w.write_bytes(xml.as_bytes()))?;
    }
    if let Some(ref xml) = resources.data_sets {
        write_resource(writer, 7001, &|w| w.write_bytes(xml.as_bytes()))?;
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
            grid: Grid {
                horizontal: 576,
                vertical: 576,
            },
            guides: vec![
                Guide {
                    location: 100.0,
                    direction: GuideDirection::Vertical,
                },
                Guide {
                    location: 200.0,
                    direction: GuideDirection::Horizontal,
                },
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
        assert_eq!(read_grid_guides, grid_guides);
    }

    #[test]
    fn test_print_flags_roundtrip() {
        let flags = PrintFlags {
            labels: true,
            crop_marks: false,
            color_bars: true,
            registration_marks: false,
            negative: true,
            flip: false,
            interpolate: true,
            caption: false,
            print_flags: true,
        };

        let mut writer = PsdWriter::new(64);
        writer.write_print_flags(&flags).unwrap();

        let buffer = writer.into_buffer();
        let cursor = std::io::Cursor::new(buffer);
        let mut reader = PsdReader::new(cursor, Default::default());

        let mut resources = ImageResources::default();
        reader.read_print_flags(&mut resources).unwrap();

        assert_eq!(resources.print_flags, Some(flags));
    }

    #[test]
    fn test_print_scale_roundtrip() {
        let scale = PrintScale {
            style: PrintScaleStyle::UserDefined,
            x: 12.5,
            y: 42.0,
            scale: 66.0,
        };

        let mut writer = PsdWriter::new(64);
        writer.write_print_scale(&scale).unwrap();

        let buffer = writer.into_buffer();
        let cursor = std::io::Cursor::new(buffer);
        let mut reader = PsdReader::new(cursor, Default::default());

        let mut resources = ImageResources::default();
        reader.read_print_scale(&mut resources).unwrap();

        assert_eq!(resources.print_scale, Some(scale));
    }

    #[test]
    fn test_layer_selection_ids_roundtrip() {
        let ids = vec![11, 22, 33];

        let mut writer = PsdWriter::new(64);
        writer.write_layer_selection_ids(&ids).unwrap();

        let buffer = writer.into_buffer();
        let cursor = std::io::Cursor::new(buffer);
        let mut reader = PsdReader::new(cursor, Default::default());

        let mut resources = ImageResources::default();
        reader.read_layer_selection_ids(&mut resources).unwrap();

        assert_eq!(resources.layer_selection_ids, Some(ids));
    }

    #[test]
    fn path_selection_descriptor_roundtrip() {
        let mut resources = ImageResources::default();
        let mut desc = Descriptor {
            name: String::new(),
            class_id: "null".to_string(),
            items: HashMap::new(),
        };
        desc.items.insert(
            "path".to_string(),
            crate::descriptor::DescriptorValue::Text("selection".to_string()),
        );
        resources.descriptor_resources.insert(3000, desc.clone());

        let mut w = PsdWriter::new(256);
        write_image_resources(&mut w, &resources).unwrap();
        let buf = w.into_buffer();
        let len = buf.len();
        let mut reader = PsdReader::new(std::io::Cursor::new(buf), Default::default());
        let reparsed = read_image_resources(&mut reader, len).unwrap();

        assert_eq!(reparsed.descriptor_resources.get(&3000), Some(&desc));
    }

    #[test]
    fn path_resource_roundtrip() {
        let mut resources = ImageResources::default();
        resources.path_resources.insert(
            2000,
            vec![PathResourceRecord {
                record_type: 1,
                closed: true,
                points: vec![
                    Point { x: 1.0, y: 2.0 },
                    Point { x: 3.0, y: 4.0 },
                    Point { x: 5.0, y: 6.0 },
                    Point { x: 7.0, y: 8.0 },
                ],
            }],
        );

        let mut w = PsdWriter::new(256);
        write_image_resources(&mut w, &resources).unwrap();
        let buf = w.into_buffer();
        let len = buf.len();
        let mut reader = PsdReader::new(std::io::Cursor::new(buf), Default::default());
        let reparsed = read_image_resources(&mut reader, len).unwrap();

        assert_eq!(
            reparsed.path_resources.get(&2000),
            resources.path_resources.get(&2000)
        );
    }

    #[test]
    fn slices_roundtrip() {
        let mut resources = ImageResources::default();
        resources.slices = Some(Slices {
            version: 7,
            slices: vec![Slice {
                id: 1,
                group_id: 2,
                origin: PsdU32Code(1),
                associated_layer_id: 3,
                name: "slice".to_string(),
                slice_type: PsdU32Code(1),
                bounds: Bounds {
                    top: 10,
                    left: 20,
                    bottom: 30,
                    right: 40,
                },
                url: "https://example.com".to_string(),
                target: "_blank".to_string(),
                message: "msg".to_string(),
                alt_tag: "alt".to_string(),
                cell_text: "cell".to_string(),
                horizontal_align: PsdIntCode(3),
                vertical_align: PsdIntCode(5),
                alpha: 255,
                bg_color: [1, 2, 3, 4],
                cell_is_html: true,
                source_id: Some(9),
                source_type: Some(PsdU32Code(10)),
            }],
        });

        let mut w = PsdWriter::new(512);
        write_image_resources(&mut w, &resources).unwrap();
        let buf = w.into_buffer();
        let len = buf.len();
        let mut reader = PsdReader::new(std::io::Cursor::new(buf), Default::default());
        let reparsed = read_image_resources(&mut reader, len).unwrap();

        assert_eq!(reparsed.slices, resources.slices);
    }

    #[test]
    fn icc_profile_roundtrip() {
        let mut res = ImageResources::default();
        res.icc_profile = Some(vec![0x01, 0x02, 0x03, 0x04]);
        let mut w = PsdWriter::new(256);
        write_image_resources(&mut w, &res).unwrap();
        let buf = w.into_buffer();
        let buf_len = buf.len();
        let cursor = std::io::Cursor::new(buf);
        let mut reader = PsdReader::new(cursor, Default::default());
        let read_res = read_image_resources(&mut reader, buf_len).unwrap();
        assert_eq!(read_res.icc_profile, Some(vec![0x01, 0x02, 0x03, 0x04]));
    }

    #[test]
    fn variables_xml_roundtrip() {
        let mut res = ImageResources::default();
        res.variables = Some("<variables/>".to_string());
        let mut w = PsdWriter::new(256);
        write_image_resources(&mut w, &res).unwrap();
        let buf = w.into_buffer();
        let buf_len = buf.len();
        let cursor = std::io::Cursor::new(buf);
        let mut reader = PsdReader::new(cursor, Default::default());
        let read_res = read_image_resources(&mut reader, buf_len).unwrap();
        assert_eq!(read_res.variables.as_deref(), Some("<variables/>"));
    }

    #[test]
    fn accepts_mesa_resource_signature() {
        let mut writer = PsdWriter::new(64);
        writer.write_bytes(b"MeSa").unwrap();
        writer.write_u16(1039).unwrap();
        writer.write_u8(0).unwrap();
        writer.write_u8(0).unwrap();
        writer.write_u32(4).unwrap();
        writer.write_bytes(&[1, 2, 3, 4]).unwrap();
        let buf = writer.into_buffer();
        let len = buf.len();
        let mut reader = PsdReader::new(std::io::Cursor::new(buf), Default::default());
        let resources = read_image_resources(&mut reader, len).unwrap();
        assert_eq!(resources.icc_profile, Some(vec![1, 2, 3, 4]));
    }
}
