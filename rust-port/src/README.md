# PSD Rust Data Structures

This directory contains a complete Rust port of the TypeScript PSD data structures from `src/psd.ts`.

## Structure

The implementation is organized into the following modules:

### `error.rs`
Error types using `thiserror`:
- `PsdError` - Main error type with variants for IO, invalid format, compression, etc.
- `Result<T>` - Type alias for `std::result::Result<T, PsdError>`

### `types.rs`
Core types and enums:
- **Enums**: `BlendMode`, `ColorMode`, `SectionDividerType`, `AntiAlias`, `Orientation`, `WarpStyle`, `BevelStyle`, `BevelTechnique`, `BevelDirection`, `GlowTechnique`, `GlowSource`, `GradientStyle`, `Justification`, `LineCapType`, `LineJoinType`, `LineAlignment`, `InterpolationMethod`, `BooleanOperation`, `RenderingIntent`, `LayerColor`, `ChannelID`, `Compression`, etc.
- **Color Types**: `RGBA`, `RGB`, `FRGB`, `HSB`, `CMYK`, `LAB`, `Grayscale`, `Color` (enum)
- **Basic Structs**: `UnitsValue`, `Point`, `Fraction`, `PixelData`

### `effects.rs`
Layer effects structures:
- `LayerEffectsInfo` - Container for all layer effects
- `LayerEffectShadow` - Drop shadow and inner shadow
- `LayerEffectsOuterGlow` - Outer glow effect
- `LayerEffectInnerGlow` - Inner glow effect
- `LayerEffectBevel` - Bevel and emboss effect
- `LayerEffectSolidFill` - Solid color fill
- `LayerEffectStroke` - Stroke effect
- `LayerEffectSatin` - Satin effect
- `LayerEffectPatternOverlay` - Pattern overlay
- `LayerEffectGradientOverlay` - Gradient overlay
- `EffectContour`, `EffectPattern` - Supporting structures
- `ColorStop`, `OpacityStop` - Gradient stops
- `EffectSolidGradient`, `EffectNoiseGradient` - Gradient definitions

### `text.rs`
Text layer related structures:
- `LayerTextData` - Main text layer data
- `Font` - Font information
- `TextStyle` - Text styling properties
- `TextStyleRun` - Text style spans
- `ParagraphStyle` - Paragraph styling
- `ParagraphStyleRun` - Paragraph style spans
- `TextGridInfo` - Text grid information
- `Warp` - Text warp settings
- `TextPath` - Text on path data
- `UnitsBounds` - Bounds with units

### `layer.rs`
Layer structure and related types:
- `Layer` - Main layer structure
- `LayerAdditionalInfo` - Extended layer properties
- `LayerMaskData` - Layer mask information
- `LayerVectorMask` - Vector mask data
- `BezierPath`, `BezierKnot` - Vector path data
- `VectorContent` - Vector fill/stroke content
- **Adjustment Layers**: `BrightnessAdjustment`, `LevelsAdjustment`, `CurvesAdjustment`, `ExposureAdjustment`, `VibranceAdjustment`, `HueSaturationAdjustment`, `ColorBalanceAdjustment`, `BlackAndWhiteAdjustment`, `PhotoFilterAdjustment`, `ChannelMixerAdjustment`, `ColorLookupAdjustment`, `InvertAdjustment`, `PosterizeAdjustment`, `ThresholdAdjustment`, `GradientMapAdjustment`, `SelectiveColorAdjustment`
- `AdjustmentLayer` - Enum of all adjustment types
- `LinkedFile` - Smart object linked file data
- `PlacedLayer` - Placed/smart object layer
- `AnimationFrame` - Animation frame data
- `Timeline`, `TimelineTrack`, `TimelineKey` - Timeline animation data
- `VectorStroke` - Vector stroke properties
- `LayerRawData`, `LayerRawDataChannel` - Raw layer data
- Supporting structures for filters, compositions, pixel sources, etc.

### `psd.rs`
Main PSD document structure and image resources:
- `Psd` - Main PSD document structure
- `ImageResources` - Document-level resources
- `Animations` - Frame-by-frame animations
- `VersionInfo` - File version information
- `GridAndGuidesInformation` - Guides and grid
- `ResolutionInfo` - Document resolution
- `PrintInformation`, `PrintScale`, `PrintFlags` - Print settings
- `TimelineInformation` - Video timeline data
- `OnionSkins` - Onion skinning settings
- `SlicesInfo`, `Slice` - Image slices
- `LayerComps` - Layer compositions
- `GlobalLayerMaskInfo` - Global layer mask
- `Annotation` - Document annotations
- `ReadOptions` - Options for reading PSD files
- `WriteOptions` - Options for writing PSD files

## Design Decisions

### Type Mappings

TypeScript → Rust:
- `number` → `f64` (or `f32` where appropriate)
- `string` → `String`
- `boolean` → `bool`
- `Array<T>` → `Vec<T>`
- `T | undefined` → `Option<T>`
- `T1 | T2` → `enum` with variants (for discriminated unions)
- `interface` → `struct`
- `type` (union types) → `enum` with `#[serde(untagged)]`

### Rust Idioms

1. **Derive Macros**: All structs use `#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]`
2. **Naming**: Snake_case for struct fields (with `#[serde(rename)]` to match JSON keys)
3. **Enums**: Used for fixed sets of values (e.g., `BlendMode`, `ColorMode`)
4. **Option<T>**: Used for all optional fields
5. **Documentation**: Doc comments (`///`) on public types

### Serialization

All types support serde serialization/deserialization with:
- `#[serde(rename = "camelCase")]` for field names matching TypeScript
- `#[serde(untagged)]` for union types that don't have explicit discriminators
- `#[serde(tag = "type")]` for discriminated unions
- `#[serde(flatten)]` for embedding additional info

## Usage Example

```rust
use ag_psd::*;

// Create a new PSD document
let psd = Psd {
    width: 1920,
    height: 1080,
    channels: Some(3),
    bits_per_channel: Some(8),
    color_mode: Some(ColorMode::RGB),
    palette: None,
    children: None,
    image_data: None,
    image_resources: None,
    linked_files: None,
    artboards: None,
    global_layer_mask_info: None,
    annotations: None,
    additional_info: LayerAdditionalInfo {
        name: Some("My Document".to_string()),
        ..Default::default()
    },
};

// Create a layer with effects
let layer = Layer {
    top: Some(100),
    left: Some(100),
    bottom: Some(200),
    right: Some(300),
    blend_mode: Some(BlendMode::Normal),
    opacity: Some(100.0),
    additional_info: LayerAdditionalInfo {
        name: Some("My Layer".to_string()),
        effects: Some(LayerEffectsInfo {
            drop_shadow: Some(vec![LayerEffectShadow {
                enabled: Some(true),
                color: Some(Color::RGBA(RGBA { r: 0, g: 0, b: 0, a: 255 })),
                opacity: Some(0.75),
                angle: Some(120.0),
                distance: Some(UnitsValue {
                    units: Units::Pixels,
                    value: 10.0,
                }),
                ..Default::default()
            }]),
            ..Default::default()
        }),
        ..Default::default()
    },
    ..Default::default()
};

// Serialize to JSON
let json = serde_json::to_string_pretty(&psd).unwrap();
println!("{}", json);
```

## Testing

Run tests with:
```bash
cargo test
```

The test suite includes:
- Basic type creation and validation
- Layer structure creation
- PSD document creation
- Serialization/deserialization
- Effect structures

## Completeness

This port includes **ALL** types from the original TypeScript `psd.ts` file (1858 lines):

✅ All blend modes  
✅ All color types (RGBA, RGB, FRGB, HSB, CMYK, LAB, Grayscale)  
✅ All layer effects (shadows, glows, bevels, strokes, overlays, etc.)  
✅ All text-related types  
✅ All adjustment layer types  
✅ All filter types (as documented but not fully implemented in original)  
✅ Layer structures with all properties  
✅ PSD document structure with all image resources  
✅ Timeline and animation structures  
✅ Vector and smart object structures  
✅ All enums and constants  

## Future Work

While this module provides complete data structure definitions, future work could include:
- Binary PSD file reading/writing implementation
- Image data encoding/decoding
- Compression algorithms (RLE, ZIP)
- Advanced filter implementations
- Validation logic for PSD constraints
