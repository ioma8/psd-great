use std::fmt;
use std::ops::Deref;

macro_rules! string_code_type {
    ($name:ident) => {
        #[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
        pub struct $name(pub String);

        impl From<&str> for $name {
            fn from(value: &str) -> Self {
                Self(value.to_string())
            }
        }

        impl From<String> for $name {
            fn from(value: String) -> Self {
                Self(value)
            }
        }

        impl AsRef<str> for $name {
            fn as_ref(&self) -> &str {
                &self.0
            }
        }

        impl Deref for $name {
            type Target = str;

            fn deref(&self) -> &Self::Target {
                self.0.as_str()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.fmt(f)
            }
        }
    };
}

macro_rules! int_code_type {
    ($name:ident, $ty:ty) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
        pub struct $name(pub $ty);
    };
}

string_code_type!(PsdStringCode);
int_code_type!(PsdIntCode, i32);
int_code_type!(PsdU32Code, u32);
int_code_type!(PsdU16Code, u16);

/// Blend mode types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum ColorMode {
    Bitmap = 0,
    Grayscale = 1,
    Indexed = 2,
    RGB = 3,
    CMYK = 4,
    Multichannel = 7,
    Duotone = 8,
    Lab = 9,
}

impl ColorMode {
    pub fn from_u16(value: u16) -> crate::error::Result<Self> {
        match value {
            0 => Ok(ColorMode::Bitmap),
            1 => Ok(ColorMode::Grayscale),
            2 => Ok(ColorMode::Indexed),
            3 => Ok(ColorMode::RGB),
            4 => Ok(ColorMode::CMYK),
            7 => Ok(ColorMode::Multichannel),
            8 => Ok(ColorMode::Duotone),
            9 => Ok(ColorMode::Lab),
            _ => Err(crate::error::PsdError::InvalidColorMode(value as u8)),
        }
    }
}

/// Section divider types for layer groups
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum SectionDividerType {
    Other = 0,
    OpenFolder = 1,
    ClosedFolder = 2,
    BoundingSectionDivider = 3,
}

/// RGBA color (values from 0 to 255)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RGBA {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

/// RGB color (values from 0 to 255)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RGB {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

/// FRGB color (floating point RGB, values from 0 to 1, can be above 1, can be negative)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FRGB {
    pub fr: f64,
    pub fg: f64,
    pub fb: f64,
}

/// CMYK color (full Photoshop color-structure values)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CMYK {
    pub c: u16,
    pub m: u16,
    pub y: u16,
    pub k: u16,
}

/// Grayscale color (0..10000)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Grayscale {
    pub k: u16,
}

/// Generic color type that can represent any color format
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Color {
    RGBA(RGBA),
    RGB(RGB),
    FRGB(FRGB),
    Rgb48 {
        red: u16,
        green: u16,
        blue: u16,
    },
    Hsb {
        hue: u16,
        saturation: u16,
        brightness: u16,
    },
    CMYK(CMYK),
    Lab {
        lightness: u16,
        a: i16,
        b: i16,
    },
    Grayscale(Grayscale),
    OpaqueColorSpace {
        color_space: u16,
        components: [u16; 4],
    },
}

/// Units for measurements
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UnitsValue {
    pub units: Units,
    pub value: f64,
}

/// Text gridding type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextGridding {
    None,
    Round,
}

/// Orientation type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Orientation {
    Horizontal,
    Vertical,
}

/// Anti-aliasing mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AntiAlias {
    None,
    Sharp,
    Crisp,
    Strong,
    Smooth,
    Platform,
    PlatformLCD,
}

/// Warp style
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WarpStyle {
    None,
    Arc,
    ArcLower,
    ArcUpper,
    Arch,
    Bulge,
    ShellLower,
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BevelStyle {
    OuterBevel,
    InnerBevel,
    Emboss,
    PillowEmboss,
    StrokeEmboss,
}

/// Bevel technique
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BevelTechnique {
    Smooth,
    ChiselHard,
    ChiselSoft,
}

/// Bevel direction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BevelDirection {
    Up,
    Down,
}

/// Glow technique
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GlowTechnique {
    Softer,
    Precise,
}

/// Glow source
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GlowSource {
    Edge,
    Center,
}

/// Gradient style
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GradientStyle {
    Linear,
    Radial,
    Angle,
    Reflected,
    Diamond,
}

/// Text justification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineCapType {
    Butt,
    Round,
    Square,
}

/// Line join type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineJoinType {
    Miter,
    Round,
    Bevel,
}

/// Line alignment
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineAlignment {
    Inside,
    Center,
    Outside,
}

/// Interpolation method
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterpolationMethod {
    Classic,
    Perceptual,
    Linear,
    Smooth,
}

/// Boolean operation for vector paths
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BooleanOperation {
    Exclude,
    Combine,
    Subtract,
    Intersect,
}

/// Rendering intent
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderingIntent {
    Perceptual,
    Saturation,
    RelativeColorimetric,
    AbsoluteColorimetric,
}

/// Layer color label
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum LayerCompCapturedInfo {
    None = 0,
    Visibility = 1,
    Position = 2,
    Appearance = 4,
}

/// Placed layer type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlacedLayerType {
    Unknown,
    Vector,
    Raster,
    ImageStack,
}

/// Timeline key interpolation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimelineKeyInterpolation {
    Linear,
    Hold,
}

/// Timeline track type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimelineTrackType {
    Opacity,
    Style,
    SheetTransform,
    SheetPosition,
    GlobalLighting,
}

/// Point coordinate
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

/// Fraction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Fraction {
    pub numerator: i32,
    pub denominator: i32,
}

/// Pixel data container
#[derive(Debug, Clone, PartialEq)]
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
        assert_eq!(ColorMode::Multichannel as u16, 7);
        assert_eq!(ColorMode::Duotone as u16, 8);
        assert_eq!(ColorMode::Lab as u16, 9);
    }

    #[test]
    fn color_mode_round_trips_from_u16() {
        for v in [0u16, 1, 2, 3, 4, 7, 8, 9] {
            let mode = ColorMode::from_u16(v).expect("should parse");
            assert_eq!(mode as u16, v);
        }
    }
}
