//! # psd-great
//!
//! Rust library for reading and writing Adobe Photoshop PSD files.
//!
//! This codebase began as a Rust port of the TypeScript ag-psd library and has
//! since grown into a broader PSD/PSB implementation with expanded feature
//! coverage and typed data structures for working with Photoshop files.
//!
//! ## Modules
//!
//! - `types` - Core types including blend modes, color modes, and basic data structures
//! - `effects` - Layer effects structures (shadows, glows, strokes, overlays, etc.)
//! - `text` - Text layer related structures
//! - `layer` - Layer structure and related types including adjustments
//! - `psd` - Main Psd structure and image resources
//! - `error` - Error types using thiserror
//! - `reader` - PSD file reading functionality
//! - `writer` - PSD file writing functionality
//! - `helpers` - Helper utilities for PSD operations
//! - `compression` - Compression and decompression algorithms
//! - `descriptor` - Descriptor structure parsing and writing
//! - `image_resources` - Image resource handlers
//! - `additional_info` - Layer additional information handlers

pub mod additional_info;
pub mod adjustments;
pub mod compression;
pub mod descriptor;
pub mod document_resource_postprocess;
pub mod effects;
pub mod error;
pub mod helpers;
pub mod image_resources;
pub mod layer;
pub mod psd;
pub mod reader;
pub mod text;
pub mod types;
pub mod writer;

// Additional format support modules
pub mod abr;
pub mod ase;
mod binrw_support;
pub mod csh;
pub mod effects_helpers;
pub mod engine_data;
pub mod jpeg;
pub mod utf8;

// Re-export commonly used types at the crate root
pub use compression::{compress_rle, compress_zip, decompress_rle, decompress_zip};
pub use effects::{
    ColorStop, EffectContour, EffectPattern, LayerEffectBevel, LayerEffectGradientOverlay,
    LayerEffectInnerGlow, LayerEffectPatternOverlay, LayerEffectSatin, LayerEffectShadow,
    LayerEffectSolidFill, LayerEffectStroke, LayerEffectsInfo, LayerEffectsOuterGlow, OpacityStop,
};
pub use error::{PsdError, Result};
pub use helpers::{from_blend_mode, has_alpha, to_blend_mode};
pub use image_resources::ImageResources;
pub use layer::{
    AdjustmentLayer, BezierKnot, BezierPath, Layer, LayerAdditionalInfo, LayerMaskData,
    LayerVectorMask, LinkedFile, PatternInfo, PlacedLayer, VectorContent,
};
pub use psd::{Annotation, GlobalLayerMaskInfo, Psd, ReadOptions, WriteOptions};
pub use reader::{read_psd, PsdReader};
pub use text::{
    Font, LayerTextData, ParagraphStyle, ParagraphStyleRun, TextStyle, TextStyleRun, Warp,
};
pub use types::{
    AntiAlias, BlendMode, BooleanOperation, ChannelID, Color, ColorMode, Compression, Fraction,
    GradientStyle, Grayscale, InterpolationMethod, Justification, LayerColor,
    LayerCompCapturedInfo, Orientation, PixelData, Point, PsdIntCode, PsdStringCode, PsdU16Code,
    PsdU32Code, SectionDividerType, Units, UnitsValue, WarpStyle, CMYK, FRGB, HSB, LAB, RGB,
    RGBA,
};
pub use writer::{write_psd, PsdWriter};

// Re-export additional format types
pub use abr::{read_abr, Abr, Brush, BrushDynamics, BrushShape, SampleInfo};
pub use ase::{read_ase, write_ase, Ase, AseColor, AseColorOrGroup, AseColorType, AseGroup};
pub use csh::{read_csh, Csh, CustomShape};
pub use effects_helpers::{read_effects, write_effects};
pub use engine_data::{parse_engine_data, serialize_engine_data, EngineValue};
pub use jpeg::{decode_jpeg, decode_jpeg_raw};
pub use utf8::{decode_string, encode_string, string_length_in_bytes};
