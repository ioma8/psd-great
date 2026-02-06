use serde::{Deserialize, Serialize};
use crate::types::*;

/// Effect contour definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EffectContour {
    pub name: String,
    pub curve: Vec<Point>,
}

/// Effect pattern
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EffectPattern {
    pub name: String,
    pub id: String,
}

/// Layer effect shadow (drop shadow and inner shadow)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LayerEffectShadow {
    pub present: Option<bool>,
    #[serde(rename = "showInDialog")]
    pub show_in_dialog: Option<bool>,
    pub enabled: Option<bool>,
    pub size: Option<UnitsValue>,
    pub angle: Option<f64>,
    pub distance: Option<UnitsValue>,
    pub color: Option<Color>,
    #[serde(rename = "blendMode")]
    pub blend_mode: Option<BlendMode>,
    pub opacity: Option<f64>,
    #[serde(rename = "useGlobalLight")]
    pub use_global_light: Option<bool>,
    pub antialiased: Option<bool>,
    pub contour: Option<EffectContour>,
    pub choke: Option<UnitsValue>,
    #[serde(rename = "layerConceals")]
    pub layer_conceals: Option<bool>,
}

/// Layer effect outer glow
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LayerEffectsOuterGlow {
    pub present: Option<bool>,
    #[serde(rename = "showInDialog")]
    pub show_in_dialog: Option<bool>,
    pub enabled: Option<bool>,
    pub size: Option<UnitsValue>,
    pub color: Option<Color>,
    #[serde(rename = "blendMode")]
    pub blend_mode: Option<BlendMode>,
    pub opacity: Option<f64>,
    pub source: Option<GlowSource>,
    pub antialiased: Option<bool>,
    pub noise: Option<f64>,
    pub range: Option<f64>,
    pub choke: Option<UnitsValue>,
    pub jitter: Option<f64>,
    pub contour: Option<EffectContour>,
}

/// Layer effect inner glow
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LayerEffectInnerGlow {
    pub present: Option<bool>,
    #[serde(rename = "showInDialog")]
    pub show_in_dialog: Option<bool>,
    pub enabled: Option<bool>,
    pub size: Option<UnitsValue>,
    pub color: Option<Color>,
    #[serde(rename = "blendMode")]
    pub blend_mode: Option<BlendMode>,
    pub opacity: Option<f64>,
    pub source: Option<GlowSource>,
    pub technique: Option<GlowTechnique>,
    pub antialiased: Option<bool>,
    pub noise: Option<f64>,
    pub range: Option<f64>,
    pub choke: Option<UnitsValue>,
    pub jitter: Option<f64>,
    pub contour: Option<EffectContour>,
}

/// Layer effect bevel
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LayerEffectBevel {
    pub present: Option<bool>,
    #[serde(rename = "showInDialog")]
    pub show_in_dialog: Option<bool>,
    pub enabled: Option<bool>,
    pub size: Option<UnitsValue>,
    pub angle: Option<f64>,
    pub strength: Option<f64>,
    #[serde(rename = "highlightBlendMode")]
    pub highlight_blend_mode: Option<BlendMode>,
    #[serde(rename = "shadowBlendMode")]
    pub shadow_blend_mode: Option<BlendMode>,
    #[serde(rename = "highlightColor")]
    pub highlight_color: Option<Color>,
    #[serde(rename = "shadowColor")]
    pub shadow_color: Option<Color>,
    pub style: Option<BevelStyle>,
    #[serde(rename = "highlightOpacity")]
    pub highlight_opacity: Option<f64>,
    #[serde(rename = "shadowOpacity")]
    pub shadow_opacity: Option<f64>,
    pub soften: Option<UnitsValue>,
    #[serde(rename = "useGlobalLight")]
    pub use_global_light: Option<bool>,
    pub altitude: Option<f64>,
    pub technique: Option<BevelTechnique>,
    pub direction: Option<BevelDirection>,
    #[serde(rename = "useTexture")]
    pub use_texture: Option<bool>,
    #[serde(rename = "useShape")]
    pub use_shape: Option<bool>,
    #[serde(rename = "antialiasGloss")]
    pub antialias_gloss: Option<bool>,
    pub contour: Option<EffectContour>,
}

/// Layer effect solid fill
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LayerEffectSolidFill {
    pub present: Option<bool>,
    #[serde(rename = "showInDialog")]
    pub show_in_dialog: Option<bool>,
    pub enabled: Option<bool>,
    #[serde(rename = "blendMode")]
    pub blend_mode: Option<BlendMode>,
    pub color: Option<Color>,
    pub opacity: Option<f64>,
}

/// Color stop in gradient
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ColorStop {
    pub color: Color,
    pub location: f64,
    pub midpoint: f64,
}

/// Opacity stop in gradient
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OpacityStop {
    pub opacity: f64,
    pub location: f64,
    pub midpoint: f64,
}

/// Solid gradient definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EffectSolidGradient {
    pub name: String,
    #[serde(rename = "type")]
    pub gradient_type: String,
    pub smoothness: Option<f64>,
    #[serde(rename = "colorStops")]
    pub color_stops: Vec<ColorStop>,
    #[serde(rename = "opacityStops")]
    pub opacity_stops: Vec<OpacityStop>,
}

/// Noise gradient definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EffectNoiseGradient {
    pub name: String,
    #[serde(rename = "type")]
    pub gradient_type: String,
    pub roughness: Option<f64>,
    #[serde(rename = "colorModel")]
    pub color_model: Option<String>,
    #[serde(rename = "randomSeed")]
    pub random_seed: Option<i32>,
    #[serde(rename = "restrictColors")]
    pub restrict_colors: Option<bool>,
    #[serde(rename = "addTransparency")]
    pub add_transparency: Option<bool>,
    pub min: Vec<f64>,
    pub max: Vec<f64>,
}

/// Extra gradient info
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExtraGradientInfo {
    pub style: Option<GradientStyle>,
    pub scale: Option<f64>,
    pub angle: Option<f64>,
    pub dither: Option<bool>,
    #[serde(rename = "interpolationMethod")]
    pub interpolation_method: Option<InterpolationMethod>,
    pub reverse: Option<bool>,
    pub align: Option<bool>,
    pub offset: Option<Point>,
}

/// Gradient type for effects
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum EffectGradient {
    Solid(EffectSolidGradient),
    Noise(EffectNoiseGradient),
}

/// Layer effect stroke
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LayerEffectStroke {
    pub present: Option<bool>,
    #[serde(rename = "showInDialog")]
    pub show_in_dialog: Option<bool>,
    pub enabled: Option<bool>,
    pub overprint: Option<bool>,
    pub size: Option<UnitsValue>,
    pub position: Option<String>,
    #[serde(rename = "fillType")]
    pub fill_type: Option<String>,
    #[serde(rename = "blendMode")]
    pub blend_mode: Option<BlendMode>,
    pub opacity: Option<f64>,
    pub color: Option<Color>,
    pub gradient: Option<EffectGradient>,
    pub pattern: Option<EffectPattern>,
}

/// Layer effect satin
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LayerEffectSatin {
    pub present: Option<bool>,
    #[serde(rename = "showInDialog")]
    pub show_in_dialog: Option<bool>,
    pub enabled: Option<bool>,
    pub size: Option<UnitsValue>,
    #[serde(rename = "blendMode")]
    pub blend_mode: Option<BlendMode>,
    pub color: Option<Color>,
    pub antialiased: Option<bool>,
    pub opacity: Option<f64>,
    pub distance: Option<UnitsValue>,
    pub invert: Option<bool>,
    pub angle: Option<f64>,
    pub contour: Option<EffectContour>,
}

/// Layer effect pattern overlay
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LayerEffectPatternOverlay {
    pub present: Option<bool>,
    #[serde(rename = "showInDialog")]
    pub show_in_dialog: Option<bool>,
    pub enabled: Option<bool>,
    #[serde(rename = "blendMode")]
    pub blend_mode: Option<BlendMode>,
    pub opacity: Option<f64>,
    pub scale: Option<f64>,
    pub pattern: Option<EffectPattern>,
    pub phase: Option<Point>,
    pub align: Option<bool>,
}

/// Layer effect gradient overlay
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LayerEffectGradientOverlay {
    pub present: Option<bool>,
    #[serde(rename = "showInDialog")]
    pub show_in_dialog: Option<bool>,
    pub enabled: Option<bool>,
    #[serde(rename = "blendMode")]
    pub blend_mode: Option<String>,
    pub opacity: Option<f64>,
    pub align: Option<bool>,
    pub scale: Option<f64>,
    pub dither: Option<bool>,
    pub reverse: Option<bool>,
    #[serde(rename = "type")]
    pub gradient_type: Option<GradientStyle>,
    pub offset: Option<Point>,
    pub gradient: Option<EffectGradient>,
    #[serde(rename = "interpolationMethod")]
    pub interpolation_method: Option<InterpolationMethod>,
    pub angle: Option<f64>,
}

/// Layer effects info container
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LayerEffectsInfo {
    pub disabled: Option<bool>,
    pub scale: Option<f64>,
    #[serde(rename = "dropShadow")]
    pub drop_shadow: Option<Vec<LayerEffectShadow>>,
    #[serde(rename = "innerShadow")]
    pub inner_shadow: Option<Vec<LayerEffectShadow>>,
    #[serde(rename = "outerGlow")]
    pub outer_glow: Option<LayerEffectsOuterGlow>,
    #[serde(rename = "innerGlow")]
    pub inner_glow: Option<LayerEffectInnerGlow>,
    pub bevel: Option<LayerEffectBevel>,
    #[serde(rename = "solidFill")]
    pub solid_fill: Option<Vec<LayerEffectSolidFill>>,
    pub satin: Option<LayerEffectSatin>,
    pub stroke: Option<Vec<LayerEffectStroke>>,
    #[serde(rename = "gradientOverlay")]
    pub gradient_overlay: Option<Vec<LayerEffectGradientOverlay>>,
    #[serde(rename = "patternOverlay")]
    pub pattern_overlay: Option<LayerEffectPatternOverlay>,
}
