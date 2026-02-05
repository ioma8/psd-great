//! # ag-psd
//!
//! Rust library for reading and writing Adobe Photoshop PSD files.
//!
//! This is a Rust port of the TypeScript ag-psd library, providing complete
//! data structure definitions for working with PSD files.
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

pub mod error;
pub mod types;
pub mod effects;
pub mod text;
pub mod layer;
pub mod psd;
pub mod reader;
pub mod writer;
pub mod helpers;
pub mod compression;

// Re-export commonly used types at the crate root
pub use error::{PsdError, Result};
pub use types::{
    BlendMode, ColorMode, SectionDividerType,
    RGBA, RGB, FRGB, HSB, CMYK, LAB, Grayscale, Color,
    Units, UnitsValue, Point, Fraction, PixelData,
    AntiAlias, Orientation, WarpStyle, GradientStyle,
    Justification, InterpolationMethod, BooleanOperation,
    LayerColor, ChannelID, Compression, LayerCompCapturedInfo,
};
pub use effects::{
    LayerEffectsInfo, LayerEffectShadow, LayerEffectsOuterGlow,
    LayerEffectInnerGlow, LayerEffectBevel, LayerEffectSolidFill,
    LayerEffectStroke, LayerEffectSatin, LayerEffectPatternOverlay,
    LayerEffectGradientOverlay, EffectContour, EffectPattern,
    ColorStop, OpacityStop,
};
pub use text::{
    LayerTextData, Font, TextStyle, TextStyleRun,
    ParagraphStyle, ParagraphStyleRun, Warp,
};
pub use layer::{
    Layer, LayerAdditionalInfo, LayerMaskData, PatternInfo,
    BezierPath, BezierKnot, VectorContent, AdjustmentLayer,
    LinkedFile, PlacedLayer, LayerVectorMask,
};
pub use psd::{
    Psd, ImageResources, ReadOptions, WriteOptions,
    GlobalLayerMaskInfo, Annotation,
};
pub use reader::{PsdReader, read_psd};
pub use writer::{PsdWriter, write_psd};
pub use helpers::{to_blend_mode, from_blend_mode, has_alpha};
pub use compression::{compress_rle, decompress_rle, compress_zip, decompress_zip};
