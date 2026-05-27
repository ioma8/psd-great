use serde::{Deserialize, Serialize};

/// Blend mode types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BlendMode {
    PassThrough,
    Normal,
    Dissolve,
    Darken,
    Multiply,
    ColorBurn,
    LinearBurn,
    DarkerColor,
    Lighten,
    Screen,
    ColorDodge,
    LinearDodge,
    LighterColor,
    Overlay,
    SoftLight,
    HardLight,
    VividLight,
    LinearLight,
    PinLight,
    HardMix,
    Difference,
    Exclusion,
    Subtract,
    Divide,
    Hue,
    Saturation,
    Color,
    Luminosity,
}

/// Color mode types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u16)]
pub enum ColorMode {
    Bitmap = 0,
    Grayscale = 1,
    Indexed = 2,
    RGB = 3,
    CMYK = 4,
    Multichannel = 5,
    Duotone = 6,
    Lab = 7,
}

impl ColorMode {
    pub fn from_u16(value: u16) -> crate::error::Result<Self> {
        match value {
            0 => Ok(ColorMode::Bitmap),
            1 => Ok(ColorMode::Grayscale),
            2 => Ok(ColorMode::Indexed),
            3 => Ok(ColorMode::RGB),
            4 => Ok(ColorMode::CMYK),
            5 => Ok(ColorMode::Multichannel),
            6 => Ok(ColorMode::Duotone),
            7 => Ok(ColorMode::Lab),
            _ => Err(crate::error::PsdError::InvalidColorMode(value as u8)),
        }
    }
}

/// Section divider types for layer groups
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u32)]
pub enum SectionDividerType {
    Other = 0,
    OpenFolder = 1,
    ClosedFolder = 2,
    BoundingSectionDivider = 3,
}

/// RGBA color (values from 0 to 255)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct RGBA {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

/// RGB color (values from 0 to 255)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct RGB {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

/// FRGB color (floating point RGB, values from 0 to 1, can be above 1, can be negative)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct FRGB {
    pub fr: f64,
    pub fg: f64,
    pub fb: f64,
}

/// HSB color (values from 0 to 1)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct HSB {
    pub h: f64,
    pub s: f64,
    pub b: f64,
}

/// CMYK color (values from 0 to 255)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CMYK {
    pub c: u8,
    pub m: u8,
    pub y: u8,
    pub k: u8,
}

/// LAB color (l from 0 to 1; a and b from -1 to 1)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct LAB {
    pub l: f64,
    pub a: f64,
    pub b: f64,
}

/// Grayscale color (values from 0 to 255)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Grayscale {
    pub k: u8,
}

/// Generic color type that can represent any color format
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Color {
    RGBA(RGBA),
    RGB(RGB),
    FRGB(FRGB),
    HSB(HSB),
    CMYK(CMYK),
    LAB(LAB),
    Grayscale(Grayscale),
}

/// Units for measurements
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Units {
    Pixels,
    Points,
    Picas,
    Millimeters,
    Centimeters,
    Inches,
    None,
    Density,
}

/// Value with units
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct UnitsValue {
    pub units: Units,
    pub value: f64,
}

/// Text gridding type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TextGridding {
    None,
    Round,
}

/// Orientation type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Orientation {
    Horizontal,
    Vertical,
}

/// Anti-aliasing mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AntiAlias {
    None,
    Sharp,
    Crisp,
    Strong,
    Smooth,
    Platform,
    #[serde(rename = "platformLCD")]
    PlatformLCD,
}

/// Warp style
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WarpStyle {
    None,
    Arc,
    #[serde(rename = "arcLower")]
    ArcLower,
    #[serde(rename = "arcUpper")]
    ArcUpper,
    Arch,
    Bulge,
    #[serde(rename = "shellLower")]
    ShellLower,
    #[serde(rename = "shellUpper")]
    ShellUpper,
    Flag,
    Wave,
    Fish,
    Rise,
    Fisheye,
    Inflate,
    Squeeze,
    Twist,
    Custom,
    Cylinder,
}

/// Bevel style
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BevelStyle {
    OuterBevel,
    InnerBevel,
    Emboss,
    PillowEmboss,
    StrokeEmboss,
}

/// Bevel technique
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BevelTechnique {
    Smooth,
    ChiselHard,
    ChiselSoft,
}

/// Bevel direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BevelDirection {
    Up,
    Down,
}

/// Glow technique
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GlowTechnique {
    Softer,
    Precise,
}

/// Glow source
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GlowSource {
    Edge,
    Center,
}

/// Gradient style
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GradientStyle {
    Linear,
    Radial,
    Angle,
    Reflected,
    Diamond,
}

/// Text justification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Justification {
    Left,
    Right,
    Center,
    JustifyLeft,
    JustifyRight,
    JustifyCenter,
    JustifyAll,
}

/// Line cap type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LineCapType {
    Butt,
    Round,
    Square,
}

/// Line join type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LineJoinType {
    Miter,
    Round,
    Bevel,
}

/// Line alignment
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LineAlignment {
    Inside,
    Center,
    Outside,
}

/// Interpolation method
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum InterpolationMethod {
    Classic,
    Perceptual,
    Linear,
    Smooth,
}

/// Boolean operation for vector paths
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BooleanOperation {
    Exclude,
    Combine,
    Subtract,
    Intersect,
}

/// Rendering intent
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RenderingIntent {
    Perceptual,
    Saturation,
    RelativeColorimetric,
    AbsoluteColorimetric,
}

/// Layer color label
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LayerColor {
    None,
    Red,
    Orange,
    Yellow,
    Green,
    Blue,
    Violet,
    Gray,
}

/// Channel ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(i16)]
pub enum ChannelID {
    Color0 = 0,
    Color1 = 1,
    Color2 = 2,
    Color3 = 3,
    Transparency = -1,
    UserMask = -2,
    RealUserMask = -3,
}

impl ChannelID {
    pub fn from_i16(value: i16) -> Self {
        match value {
            0 => ChannelID::Color0,
            1 => ChannelID::Color1,
            2 => ChannelID::Color2,
            3 => ChannelID::Color3,
            -1 => ChannelID::Transparency,
            -2 => ChannelID::UserMask,
            -3 => ChannelID::RealUserMask,
            _ => ChannelID::Color0, // Default fallback
        }
    }
}

/// Compression type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u16)]
pub enum Compression {
    RawData = 0,
    RleCompressed = 1,
    ZipWithoutPrediction = 2,
    ZipWithPrediction = 3,
}

impl Compression {
    pub fn from_u16(value: u16) -> crate::error::Result<Self> {
        match value {
            0 => Ok(Compression::RawData),
            1 => Ok(Compression::RleCompressed),
            2 => Ok(Compression::ZipWithoutPrediction),
            3 => Ok(Compression::ZipWithPrediction),
            _ => Err(crate::error::PsdError::Compression(format!(
                "Invalid compression type: {}",
                value
            ))),
        }
    }
}

/// Layer composition captured info flags
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u32)]
pub enum LayerCompCapturedInfo {
    None = 0,
    Visibility = 1,
    Position = 2,
    Appearance = 4,
}

/// Placed layer type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PlacedLayerType {
    Unknown,
    Vector,
    Raster,
    ImageStack,
}

/// Timeline key interpolation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TimelineKeyInterpolation {
    Linear,
    Hold,
}

/// Timeline track type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TimelineTrackType {
    Opacity,
    Style,
    SheetTransform,
    SheetPosition,
    GlobalLighting,
}

/// Point coordinate
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

/// Fraction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Fraction {
    pub numerator: i32,
    pub denominator: i32,
}

/// Pixel data container
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PixelData {
    pub data: Vec<u8>,
    pub width: usize,
    pub height: usize,
}

#[cfg(test)]
mod color_mode_tests {
    use super::*;

    #[test]
    fn color_mode_values_match_psd_spec() {
        assert_eq!(ColorMode::Bitmap as u16, 0);
        assert_eq!(ColorMode::Grayscale as u16, 1);
        assert_eq!(ColorMode::Indexed as u16, 2);
        assert_eq!(ColorMode::RGB as u16, 3);
        assert_eq!(ColorMode::CMYK as u16, 4);
        assert_eq!(ColorMode::Multichannel as u16, 5);
        assert_eq!(ColorMode::Duotone as u16, 6);
        assert_eq!(ColorMode::Lab as u16, 7);
    }

    #[test]
    fn color_mode_round_trips_from_u16() {
        for v in [0u16, 1, 2, 3, 4, 5, 6, 7] {
            let mode = ColorMode::from_u16(v).expect("should parse");
            assert_eq!(mode as u16, v);
        }
    }
}
