use crate::effects::LayerEffectsInfo;
use crate::text::LayerTextData;
use crate::types::*;
use serde::{Deserialize, Serialize};

/// Layer mask data
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct LayerMaskData {
    pub top: Option<i32>,
    pub left: Option<i32>,
    pub bottom: Option<i32>,
    pub right: Option<i32>,
    #[serde(rename = "defaultColor")]
    pub default_color: Option<u8>,
    pub disabled: Option<bool>,
    #[serde(rename = "positionRelativeToLayer")]
    pub position_relative_to_layer: Option<bool>,
    #[serde(rename = "fromVectorData")]
    pub from_vector_data: Option<bool>,
    #[serde(rename = "userMaskDensity")]
    pub user_mask_density: Option<f64>,
    #[serde(rename = "userMaskFeather")]
    pub user_mask_feather: Option<f64>,
    #[serde(rename = "vectorMaskDensity")]
    pub vector_mask_density: Option<f64>,
    #[serde(rename = "vectorMaskFeather")]
    pub vector_mask_feather: Option<f64>,
    #[serde(rename = "imageData")]
    pub image_data: Option<PixelData>,
}

/// Pattern info
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PatternInfo {
    pub name: String,
    pub id: String,
    pub x: f64,
    pub y: f64,
    pub bounds: PatternBounds,
    pub data: Vec<u8>,
}

/// Pattern bounds
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PatternBounds {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

/// Bezier knot
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BezierKnot {
    pub linked: bool,
    pub points: Vec<f64>,
}

/// Bezier path
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BezierPath {
    pub open: bool,
    pub operation: Option<BooleanOperation>,
    pub knots: Vec<BezierKnot>,
    #[serde(rename = "fillRule")]
    pub fill_rule: String,
}

/// Extra pattern info
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExtraPatternInfo {
    pub linked: Option<bool>,
    pub phase: Option<Point>,
}

/// Vector content types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum VectorContent {
    Color {
        #[serde(rename = "type")]
        content_type: String,
        color: Color,
    },
    SolidGradient {
        name: String,
        #[serde(rename = "type")]
        gradient_type: String,
        smoothness: Option<f64>,
        style: Option<GradientStyle>,
        scale: Option<f64>,
        angle: Option<f64>,
        dither: Option<bool>,
        reverse: Option<bool>,
        align: Option<bool>,
        offset: Option<Point>,
    },
    Pattern {
        name: String,
        id: String,
        #[serde(rename = "type")]
        content_type: String,
        linked: Option<bool>,
        phase: Option<Point>,
    },
}

/// Brightness adjustment
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BrightnessAdjustment {
    #[serde(rename = "type")]
    pub adjustment_type: String,
    pub brightness: Option<f64>,
    pub contrast: Option<f64>,
    #[serde(rename = "meanValue")]
    pub mean_value: Option<f64>,
    #[serde(rename = "useLegacy")]
    pub use_legacy: Option<bool>,
    #[serde(rename = "labColorOnly")]
    pub lab_color_only: Option<bool>,
    pub auto: Option<bool>,
}

/// Levels adjustment channel
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LevelsAdjustmentChannel {
    #[serde(rename = "shadowInput")]
    pub shadow_input: f64,
    #[serde(rename = "highlightInput")]
    pub highlight_input: f64,
    #[serde(rename = "shadowOutput")]
    pub shadow_output: f64,
    #[serde(rename = "highlightOutput")]
    pub highlight_output: f64,
    #[serde(rename = "midtoneInput")]
    pub midtone_input: f64,
}

/// Preset info
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PresetInfo {
    #[serde(rename = "presetKind")]
    pub preset_kind: Option<i32>,
    #[serde(rename = "presetFileName")]
    pub preset_file_name: Option<String>,
}

/// Levels adjustment
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LevelsAdjustment {
    #[serde(rename = "type")]
    pub adjustment_type: String,
    pub rgb: Option<LevelsAdjustmentChannel>,
    pub red: Option<LevelsAdjustmentChannel>,
    pub green: Option<LevelsAdjustmentChannel>,
    pub blue: Option<LevelsAdjustmentChannel>,
    #[serde(flatten)]
    pub preset: Option<PresetInfo>,
}

/// Curves adjustment channel
pub type CurvesAdjustmentChannel = Vec<CurvePoint>;

/// Curve point
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CurvePoint {
    pub input: f64,
    pub output: f64,
}

/// Curves adjustment
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CurvesAdjustment {
    #[serde(rename = "type")]
    pub adjustment_type: String,
    pub rgb: Option<CurvesAdjustmentChannel>,
    pub red: Option<CurvesAdjustmentChannel>,
    pub green: Option<CurvesAdjustmentChannel>,
    pub blue: Option<CurvesAdjustmentChannel>,
    #[serde(flatten)]
    pub preset: Option<PresetInfo>,
}

/// Exposure adjustment
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExposureAdjustment {
    #[serde(rename = "type")]
    pub adjustment_type: String,
    pub exposure: Option<f64>,
    pub offset: Option<f64>,
    pub gamma: Option<f64>,
    #[serde(flatten)]
    pub preset: Option<PresetInfo>,
}

/// Vibrance adjustment
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VibranceAdjustment {
    #[serde(rename = "type")]
    pub adjustment_type: String,
    pub vibrance: Option<f64>,
    pub saturation: Option<f64>,
}

/// Hue saturation adjustment channel
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HueSaturationAdjustmentChannel {
    pub a: f64,
    pub b: f64,
    pub c: f64,
    pub d: f64,
    pub hue: f64,
    pub saturation: f64,
    pub lightness: f64,
}

/// Hue saturation adjustment
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HueSaturationAdjustment {
    #[serde(rename = "type")]
    pub adjustment_type: String,
    pub master: Option<HueSaturationAdjustmentChannel>,
    pub reds: Option<HueSaturationAdjustmentChannel>,
    pub yellows: Option<HueSaturationAdjustmentChannel>,
    pub greens: Option<HueSaturationAdjustmentChannel>,
    pub cyans: Option<HueSaturationAdjustmentChannel>,
    pub blues: Option<HueSaturationAdjustmentChannel>,
    pub magentas: Option<HueSaturationAdjustmentChannel>,
    #[serde(flatten)]
    pub preset: Option<PresetInfo>,
}

/// Color balance values
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ColorBalanceValues {
    #[serde(rename = "cyanRed")]
    pub cyan_red: f64,
    #[serde(rename = "magentaGreen")]
    pub magenta_green: f64,
    #[serde(rename = "yellowBlue")]
    pub yellow_blue: f64,
}

/// Color balance adjustment
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ColorBalanceAdjustment {
    #[serde(rename = "type")]
    pub adjustment_type: String,
    pub shadows: Option<ColorBalanceValues>,
    pub midtones: Option<ColorBalanceValues>,
    pub highlights: Option<ColorBalanceValues>,
    #[serde(rename = "preserveLuminosity")]
    pub preserve_luminosity: Option<bool>,
}

/// Black and white adjustment
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BlackAndWhiteAdjustment {
    #[serde(rename = "type")]
    pub adjustment_type: String,
    pub reds: Option<f64>,
    pub yellows: Option<f64>,
    pub greens: Option<f64>,
    pub cyans: Option<f64>,
    pub blues: Option<f64>,
    pub magentas: Option<f64>,
    #[serde(rename = "useTint")]
    pub use_tint: Option<bool>,
    #[serde(rename = "tintColor")]
    pub tint_color: Option<Color>,
    #[serde(flatten)]
    pub preset: Option<PresetInfo>,
}

/// Photo filter adjustment
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PhotoFilterAdjustment {
    #[serde(rename = "type")]
    pub adjustment_type: String,
    pub color: Option<Color>,
    pub density: Option<f64>,
    #[serde(rename = "preserveLuminosity")]
    pub preserve_luminosity: Option<bool>,
}

/// Channel mixer channel
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChannelMixerChannel {
    pub red: f64,
    pub green: f64,
    pub blue: f64,
    pub constant: f64,
}

/// Channel mixer adjustment
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChannelMixerAdjustment {
    #[serde(rename = "type")]
    pub adjustment_type: String,
    pub monochrome: Option<bool>,
    pub red: Option<ChannelMixerChannel>,
    pub green: Option<ChannelMixerChannel>,
    pub blue: Option<ChannelMixerChannel>,
    pub gray: Option<ChannelMixerChannel>,
    #[serde(flatten)]
    pub preset: Option<PresetInfo>,
}

/// Color lookup adjustment
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ColorLookupAdjustment {
    #[serde(rename = "type")]
    pub adjustment_type: String,
    #[serde(rename = "lookupType")]
    pub lookup_type: Option<String>,
    pub name: Option<String>,
    pub dither: Option<bool>,
    pub profile: Option<Vec<u8>>,
    #[serde(rename = "lutFormat")]
    pub lut_format: Option<String>,
    #[serde(rename = "dataOrder")]
    pub data_order: Option<String>,
    #[serde(rename = "tableOrder")]
    pub table_order: Option<String>,
    #[serde(rename = "lut3DFileData")]
    pub lut3d_file_data: Option<Vec<u8>>,
    #[serde(rename = "lut3DFileName")]
    pub lut3d_file_name: Option<String>,
}

/// Invert adjustment
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InvertAdjustment {
    #[serde(rename = "type")]
    pub adjustment_type: String,
}

/// Posterize adjustment
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PosterizeAdjustment {
    #[serde(rename = "type")]
    pub adjustment_type: String,
    pub levels: Option<i32>,
}

/// Threshold adjustment
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ThresholdAdjustment {
    #[serde(rename = "type")]
    pub adjustment_type: String,
    pub level: Option<f64>,
}

/// Gradient map adjustment
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GradientMapAdjustment {
    #[serde(rename = "type")]
    pub adjustment_type: String,
    pub name: Option<String>,
    #[serde(rename = "gradientType")]
    pub gradient_type: String,
    pub dither: Option<bool>,
    pub reverse: Option<bool>,
    pub method: Option<InterpolationMethod>,
    pub smoothness: Option<f64>,
    #[serde(rename = "colorStops")]
    pub color_stops: Option<Vec<crate::effects::ColorStop>>,
    #[serde(rename = "opacityStops")]
    pub opacity_stops: Option<Vec<crate::effects::OpacityStop>>,
    pub roughness: Option<f64>,
    #[serde(rename = "colorModel")]
    pub color_model: Option<String>,
    #[serde(rename = "randomSeed")]
    pub random_seed: Option<i32>,
    #[serde(rename = "restrictColors")]
    pub restrict_colors: Option<bool>,
    #[serde(rename = "addTransparency")]
    pub add_transparency: Option<bool>,
    pub min: Option<Vec<f64>>,
    pub max: Option<Vec<f64>>,
}

/// Selective color adjustment
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SelectiveColorAdjustment {
    #[serde(rename = "type")]
    pub adjustment_type: String,
    pub mode: Option<String>,
    pub reds: Option<CMYK>,
    pub yellows: Option<CMYK>,
    pub greens: Option<CMYK>,
    pub cyans: Option<CMYK>,
    pub blues: Option<CMYK>,
    pub magentas: Option<CMYK>,
    pub whites: Option<CMYK>,
    pub neutrals: Option<CMYK>,
    pub blacks: Option<CMYK>,
}

/// Adjustment layer types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AdjustmentLayer {
    Brightness(BrightnessAdjustment),
    Levels(LevelsAdjustment),
    Curves(CurvesAdjustment),
    Exposure(ExposureAdjustment),
    Vibrance(VibranceAdjustment),
    HueSaturation(HueSaturationAdjustment),
    ColorBalance(ColorBalanceAdjustment),
    BlackAndWhite(BlackAndWhiteAdjustment),
    PhotoFilter(PhotoFilterAdjustment),
    ChannelMixer(ChannelMixerAdjustment),
    ColorLookup(ColorLookupAdjustment),
    Invert(InvertAdjustment),
    Posterize(PosterizeAdjustment),
    Threshold(ThresholdAdjustment),
    GradientMap(GradientMapAdjustment),
    SelectiveColor(SelectiveColorAdjustment),
}

/// Linked file
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LinkedFile {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub file_type: Option<String>,
    pub creator: Option<String>,
    pub data: Option<Vec<u8>>,
    pub time: Option<String>,
    pub descriptor: Option<LinkedFileDescriptor>,
    #[serde(rename = "childDocumentID")]
    pub child_document_id: Option<String>,
    #[serde(rename = "assetModTime")]
    pub asset_mod_time: Option<f64>,
    #[serde(rename = "assetLockedState")]
    pub asset_locked_state: Option<i32>,
    #[serde(rename = "linkedFile")]
    pub linked_file: Option<LinkedFileInfo>,
}

/// Linked file descriptor
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LinkedFileDescriptor {
    #[serde(rename = "compInfo")]
    pub comp_info: CompInfo,
}

/// Comp info
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompInfo {
    #[serde(rename = "compID")]
    pub comp_id: i32,
    #[serde(rename = "originalCompID")]
    pub original_comp_id: i32,
}

/// Linked file info
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LinkedFileInfo {
    #[serde(rename = "fileSize")]
    pub file_size: u64,
    pub name: String,
    #[serde(rename = "fullPath")]
    pub full_path: String,
    #[serde(rename = "originalPath")]
    pub original_path: String,
    #[serde(rename = "relativePath")]
    pub relative_path: String,
}

/// Placed layer filter
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlacedLayerFilter {
    pub enabled: bool,
    #[serde(rename = "validAtPosition")]
    pub valid_at_position: bool,
    #[serde(rename = "maskEnabled")]
    pub mask_enabled: bool,
    #[serde(rename = "maskLinked")]
    pub mask_linked: bool,
    #[serde(rename = "maskExtendWithWhite")]
    pub mask_extend_with_white: bool,
    pub list: Vec<Filter>,
}

/// Filter
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Filter {
    pub name: String,
    pub opacity: f64,
    #[serde(rename = "blendMode")]
    pub blend_mode: BlendMode,
    pub enabled: bool,
    #[serde(rename = "hasOptions")]
    pub has_options: bool,
    #[serde(rename = "foregroundColor")]
    pub foreground_color: Color,
    #[serde(rename = "backgroundColor")]
    pub background_color: Color,
}

/// Placed layer
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlacedLayer {
    pub id: String,
    pub placed: Option<String>,
    #[serde(rename = "type")]
    pub layer_type: PlacedLayerType,
    #[serde(rename = "pageNumber")]
    pub page_number: Option<i32>,
    #[serde(rename = "totalPages")]
    pub total_pages: Option<i32>,
    #[serde(rename = "frameStep")]
    pub frame_step: Option<Fraction>,
    pub duration: Option<Fraction>,
    #[serde(rename = "frameCount")]
    pub frame_count: Option<i32>,
    pub transform: Vec<f64>,
    #[serde(rename = "nonAffineTransform")]
    pub non_affine_transform: Option<Vec<f64>>,
    pub width: Option<f64>,
    pub height: Option<f64>,
    pub resolution: Option<UnitsValue>,
    pub warp: Option<crate::text::Warp>,
    pub crop: Option<f64>,
    pub comp: Option<i32>,
    #[serde(rename = "compInfo")]
    pub comp_info: Option<CompInfo>,
    pub filter: Option<PlacedLayerFilter>,
}

/// Key descriptor item
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KeyDescriptorItem {
    #[serde(rename = "keyShapeInvalidated")]
    pub key_shape_invalidated: Option<bool>,
    #[serde(rename = "keyOriginType")]
    pub key_origin_type: Option<i32>,
    #[serde(rename = "keyOriginResolution")]
    pub key_origin_resolution: Option<f64>,
    #[serde(rename = "keyOriginRRectRadii")]
    pub key_origin_rrect_radii: Option<RRectRadii>,
    #[serde(rename = "keyOriginShapeBoundingBox")]
    pub key_origin_shape_bounding_box: Option<crate::text::UnitsBounds>,
    #[serde(rename = "keyOriginBoxCorners")]
    pub key_origin_box_corners: Option<Vec<Point>>,
    pub transform: Option<Vec<f64>>,
}

/// Rounded rectangle radii
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RRectRadii {
    #[serde(rename = "topRight")]
    pub top_right: UnitsValue,
    #[serde(rename = "topLeft")]
    pub top_left: UnitsValue,
    #[serde(rename = "bottomLeft")]
    pub bottom_left: UnitsValue,
    #[serde(rename = "bottomRight")]
    pub bottom_right: UnitsValue,
}

/// Layer vector mask
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LayerVectorMask {
    pub invert: Option<bool>,
    #[serde(rename = "notLink")]
    pub not_link: Option<bool>,
    pub disable: Option<bool>,
    #[serde(rename = "fillStartsWithAllPixels")]
    pub fill_starts_with_all_pixels: Option<bool>,
    pub clipboard: Option<ClipboardData>,
    pub paths: Vec<BezierPath>,
}

/// Clipboard data
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClipboardData {
    pub top: f64,
    pub left: f64,
    pub bottom: f64,
    pub right: f64,
    pub resolution: f64,
}

/// Animation frame
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnimationFrame {
    pub frames: Vec<i32>,
    pub enable: Option<bool>,
    pub offset: Option<Point>,
    #[serde(rename = "referencePoint")]
    pub reference_point: Option<Point>,
    pub opacity: Option<f64>,
    pub effects: Option<LayerEffectsInfo>,
}

/// Timeline key types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum TimelineKey {
    #[serde(rename = "opacity")]
    Opacity {
        interpolation: TimelineKeyInterpolation,
        time: Fraction,
        selected: Option<bool>,
        value: f64,
    },
    #[serde(rename = "position")]
    Position {
        interpolation: TimelineKeyInterpolation,
        time: Fraction,
        selected: Option<bool>,
        x: f64,
        y: f64,
    },
    #[serde(rename = "transform")]
    Transform {
        interpolation: TimelineKeyInterpolation,
        time: Fraction,
        selected: Option<bool>,
        scale: Point,
        skew: Point,
        rotation: f64,
        translation: Point,
    },
    #[serde(rename = "style")]
    Style {
        interpolation: TimelineKeyInterpolation,
        time: Fraction,
        selected: Option<bool>,
        style: Option<LayerEffectsInfo>,
    },
    #[serde(rename = "globalLighting")]
    GlobalLighting {
        interpolation: TimelineKeyInterpolation,
        time: Fraction,
        selected: Option<bool>,
        #[serde(rename = "globalAngle")]
        global_angle: f64,
        #[serde(rename = "globalAltitude")]
        global_altitude: f64,
    },
}

/// Timeline track
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimelineTrack {
    #[serde(rename = "type")]
    pub track_type: TimelineTrackType,
    pub enabled: Option<bool>,
    #[serde(rename = "effectParams")]
    pub effect_params: Option<EffectParams>,
    pub keys: Vec<TimelineKey>,
}

/// Effect params
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EffectParams {
    pub keys: Vec<TimelineKey>,
    #[serde(rename = "fillCanvas")]
    pub fill_canvas: bool,
    #[serde(rename = "zoomOrigin")]
    pub zoom_origin: f64,
}

/// Timeline
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Timeline {
    pub start: Fraction,
    pub duration: Fraction,
    #[serde(rename = "inTime")]
    pub in_time: Fraction,
    #[serde(rename = "outTime")]
    pub out_time: Fraction,
    #[serde(rename = "autoScope")]
    pub auto_scope: bool,
    #[serde(rename = "audioLevel")]
    pub audio_level: f64,
    pub tracks: Option<Vec<TimelineTrack>>,
}

/// Vector stroke content
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VectorStroke {
    #[serde(rename = "strokeEnabled")]
    pub stroke_enabled: Option<bool>,
    #[serde(rename = "fillEnabled")]
    pub fill_enabled: Option<bool>,
    #[serde(rename = "lineWidth")]
    pub line_width: Option<UnitsValue>,
    #[serde(rename = "lineDashOffset")]
    pub line_dash_offset: Option<UnitsValue>,
    #[serde(rename = "miterLimit")]
    pub miter_limit: Option<f64>,
    #[serde(rename = "lineCapType")]
    pub line_cap_type: Option<LineCapType>,
    #[serde(rename = "lineJoinType")]
    pub line_join_type: Option<LineJoinType>,
    #[serde(rename = "lineAlignment")]
    pub line_alignment: Option<LineAlignment>,
    #[serde(rename = "scaleLock")]
    pub scale_lock: Option<bool>,
    #[serde(rename = "strokeAdjust")]
    pub stroke_adjust: Option<bool>,
    #[serde(rename = "lineDashSet")]
    pub line_dash_set: Option<Vec<UnitsValue>>,
    #[serde(rename = "blendMode")]
    pub blend_mode: Option<BlendMode>,
    pub opacity: Option<f64>,
    pub content: Option<VectorContent>,
    pub resolution: Option<f64>,
}

/// Protected flags
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Protected {
    pub transparency: Option<bool>,
    pub composite: Option<bool>,
    pub position: Option<bool>,
    pub artboards: Option<bool>,
}

/// Section divider
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SectionDivider {
    #[serde(rename = "type")]
    pub divider_type: SectionDividerType,
    pub key: Option<String>,
    #[serde(rename = "subType")]
    pub sub_type: Option<i32>,
}

/// Filter mask
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FilterMask {
    #[serde(rename = "colorSpace")]
    pub color_space: Color,
    pub opacity: f64,
}

/// Compositor used info
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompositorUsed {
    pub version: Option<Version>,
    #[serde(rename = "photoshopVersion")]
    pub photoshop_version: Option<Version>,
    pub description: String,
    pub reason: String,
    pub engine: String,
    #[serde(rename = "enableCompCore")]
    pub enable_comp_core: Option<String>,
    #[serde(rename = "enableCompCoreGPU")]
    pub enable_comp_core_gpu: Option<String>,
    #[serde(rename = "enableCompCoreThreads")]
    pub enable_comp_core_threads: Option<String>,
    #[serde(rename = "compCoreSupport")]
    pub comp_core_support: Option<String>,
    #[serde(rename = "compCoreGPUSupport")]
    pub comp_core_gpu_support: Option<String>,
}

/// Version
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Version {
    pub major: i32,
    pub minor: i32,
    pub fix: i32,
}

/// Artboard info
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Artboard {
    pub rect: Rect,
    #[serde(rename = "guideIndices")]
    pub guide_indices: Option<Vec<serde_json::Value>>,
    #[serde(rename = "presetName")]
    pub preset_name: Option<String>,
    pub color: Option<Color>,
    #[serde(rename = "backgroundType")]
    pub background_type: Option<i32>,
}

/// Rectangle
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Rect {
    pub top: f64,
    pub left: f64,
    pub bottom: f64,
    pub right: f64,
}

/// Animation frame flags
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnimationFrameFlags {
    #[serde(rename = "propagateFrameOne")]
    pub propagate_frame_one: Option<bool>,
    #[serde(rename = "unifyLayerPosition")]
    pub unify_layer_position: Option<bool>,
    #[serde(rename = "unifyLayerStyle")]
    pub unify_layer_style: Option<bool>,
    #[serde(rename = "unifyLayerVisibility")]
    pub unify_layer_visibility: Option<bool>,
}

/// Filter effects mask channel
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FilterEffectsMaskChannel {
    #[serde(rename = "compressionMode")]
    pub compression_mode: i32,
    pub data: Vec<u8>,
}

/// Filter effects mask extra
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FilterEffectsMaskExtra {
    pub top: f64,
    pub left: f64,
    pub bottom: f64,
    pub right: f64,
    #[serde(rename = "compressionMode")]
    pub compression_mode: i32,
    pub data: Vec<u8>,
}

/// Filter effects mask
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FilterEffectsMask {
    pub id: String,
    pub top: f64,
    pub left: f64,
    pub bottom: f64,
    pub right: f64,
    pub depth: i32,
    pub channels: Vec<Option<FilterEffectsMaskChannel>>,
    pub extra: Option<FilterEffectsMaskExtra>,
}

/// Comps settings
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompsSettings {
    pub enabled: Option<bool>,
    #[serde(rename = "compList")]
    pub comp_list: Vec<i32>,
    pub offset: Option<Point>,
    #[serde(rename = "effectsReferencePoint")]
    pub effects_reference_point: Option<Point>,
}

/// Comps
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Comps {
    #[serde(rename = "originalEffectsReferencePoint")]
    pub original_effects_reference_point: Option<Point>,
    pub settings: Vec<CompsSettings>,
}

/// User mask
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserMask {
    #[serde(rename = "colorSpace")]
    pub color_space: Color,
    pub opacity: f64,
}

/// Blending ranges
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BlendingRanges {
    #[serde(rename = "compositeGrayBlendSource")]
    pub composite_gray_blend_source: Vec<f64>,
    #[serde(rename = "compositeGraphBlendDestinationRange")]
    pub composite_graph_blend_destination_range: Vec<f64>,
    pub ranges: Vec<BlendRange>,
}

/// Blend range
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BlendRange {
    #[serde(rename = "sourceRange")]
    pub source_range: Vec<f64>,
    #[serde(rename = "destRange")]
    pub dest_range: Vec<f64>,
}

/// Pixel source
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PixelSource {
    #[serde(rename = "type")]
    pub source_type: String,
    pub origin: Point,
    pub interpretation: Interpretation,
    #[serde(rename = "frameReader")]
    pub frame_reader: FrameReader,
    #[serde(rename = "showAlteredVideo")]
    pub show_altered_video: bool,
}

/// Interpretation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Interpretation {
    #[serde(rename = "interpretAlpha")]
    pub interpret_alpha: String,
    pub profile: Vec<u8>,
}

/// Frame reader
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FrameReader {
    #[serde(rename = "type")]
    pub reader_type: String,
    pub link: FrameReaderLink,
    #[serde(rename = "mediaDescriptor")]
    pub media_descriptor: String,
}

/// Frame reader link
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FrameReaderLink {
    pub name: String,
    #[serde(rename = "fullPath")]
    pub full_path: String,
    #[serde(rename = "originalPath")]
    pub original_path: String,
    #[serde(rename = "relativePath")]
    pub relative_path: String,
    pub alias: String,
}

/// Layer raw data channel
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LayerRawDataChannel {
    pub id: ChannelID,
    pub compression: Compression,
    pub data: Option<Vec<u8>>,
}

/// Layer raw data
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LayerRawData {
    #[serde(rename = "colorMode")]
    pub color_mode: ColorMode,
    #[serde(rename = "bitsPerChannel")]
    pub bits_per_channel: u8,
    pub channels: Vec<LayerRawDataChannel>,
    pub large: bool,
}

/// Layer additional info
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct LayerAdditionalInfo {
    pub name: Option<String>,
    #[serde(rename = "nameSource")]
    pub name_source: Option<String>,
    pub id: Option<i32>,
    pub version: Option<i32>,
    pub mask: Option<LayerMaskData>,
    #[serde(rename = "realMask")]
    pub real_mask: Option<LayerMaskData>,
    #[serde(rename = "blendClippedElements")]
    pub blend_clipped_elements: Option<bool>,
    #[serde(rename = "blendInteriorElements")]
    pub blend_interior_elements: Option<bool>,
    pub knockout: Option<bool>,
    #[serde(rename = "layerMaskAsGlobalMask")]
    pub layer_mask_as_global_mask: Option<bool>,
    pub protected: Option<Protected>,
    #[serde(rename = "layerColor")]
    pub layer_color: Option<LayerColor>,
    #[serde(rename = "referencePoint")]
    pub reference_point: Option<Point>,
    #[serde(rename = "sectionDivider")]
    pub section_divider: Option<SectionDivider>,
    #[serde(rename = "filterMask")]
    pub filter_mask: Option<FilterMask>,
    pub effects: Option<LayerEffectsInfo>,
    pub text: Option<LayerTextData>,
    pub patterns: Option<Vec<PatternInfo>>,
    #[serde(rename = "vectorFill")]
    pub vector_fill: Option<VectorContent>,
    #[serde(rename = "vectorStroke")]
    pub vector_stroke: Option<VectorStroke>,
    #[serde(rename = "vectorMask")]
    pub vector_mask: Option<LayerVectorMask>,
    #[serde(rename = "usingAlignedRendering")]
    pub using_aligned_rendering: Option<bool>,
    pub timestamp: Option<f64>,
    #[serde(rename = "pathList")]
    pub path_list: Option<Vec<serde_json::Value>>,
    pub adjustment: Option<AdjustmentLayer>,
    #[serde(rename = "placedLayer")]
    pub placed_layer: Option<PlacedLayer>,
    #[serde(rename = "vectorOrigination")]
    pub vector_origination: Option<VectorOrigination>,
    #[serde(rename = "compositorUsed")]
    pub compositor_used: Option<CompositorUsed>,
    pub artboard: Option<Artboard>,
    #[serde(rename = "fillOpacity")]
    pub fill_opacity: Option<f64>,
    #[serde(rename = "transparencyShapesLayer")]
    pub transparency_shapes_layer: Option<bool>,
    #[serde(rename = "channelBlendingRestrictions")]
    pub channel_blending_restrictions: Option<Vec<i32>>,
    #[serde(rename = "animationFrames")]
    pub animation_frames: Option<Vec<AnimationFrame>>,
    #[serde(rename = "animationFrameFlags")]
    pub animation_frame_flags: Option<AnimationFrameFlags>,
    pub timeline: Option<Timeline>,
    #[serde(rename = "filterEffectsMasks")]
    pub filter_effects_masks: Option<Vec<FilterEffectsMask>>,
    pub comps: Option<Comps>,
    #[serde(rename = "userMask")]
    pub user_mask: Option<UserMask>,
    #[serde(rename = "blendingRanges")]
    pub blending_ranges: Option<BlendingRanges>,
    pub vowv: Option<i32>,
    #[serde(rename = "pixelSource")]
    pub pixel_source: Option<PixelSource>,
    #[serde(rename = "engineData")]
    pub engine_data: Option<String>,
}

/// Vector origination
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VectorOrigination {
    #[serde(rename = "keyDescriptorList")]
    pub key_descriptor_list: Vec<KeyDescriptorItem>,
}

/// Layer structure
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Layer {
    pub top: Option<i32>,
    pub left: Option<i32>,
    pub bottom: Option<i32>,
    pub right: Option<i32>,
    #[serde(rename = "blendMode")]
    pub blend_mode: Option<BlendMode>,
    pub opacity: Option<f64>,
    #[serde(rename = "transparencyProtected")]
    pub transparency_protected: Option<bool>,
    #[serde(rename = "effectsOpen")]
    pub effects_open: Option<bool>,
    pub hidden: Option<bool>,
    pub clipping: Option<u16>,
    #[serde(rename = "resourceVisible")]
    pub resource_visible: Option<bool>,
    #[serde(rename = "imageData")]
    pub image_data: Option<PixelData>,
    #[serde(rename = "rawData")]
    pub raw_data: Option<LayerRawData>,
    pub children: Option<Vec<Layer>>,
    pub opened: Option<bool>,
    #[serde(rename = "linkGroup")]
    pub link_group: Option<i32>,
    #[serde(rename = "linkGroupEnabled")]
    pub link_group_enabled: Option<bool>,

    #[serde(flatten)]
    pub additional_info: LayerAdditionalInfo,

    /// Parsed tagged blocks from the PSD extra data section.
    /// Used internally for round-trip fidelity of all layer metadata.
    #[serde(skip)]
    pub tagged_blocks: crate::additional_info::LayerAdditionalInfo,

    /// Raw blending ranges data for round-trip preservation.
    #[serde(skip)]
    pub blending_ranges_raw: Option<Vec<u8>>,
}
