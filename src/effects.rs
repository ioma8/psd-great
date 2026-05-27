use crate::types::*;

/// Effect contour definition
#[derive(Debug, Clone, PartialEq)]
pub struct EffectContour {
    pub name: String,
    pub curve: Vec<Point>,
}

/// Effect pattern
#[derive(Debug, Clone, PartialEq)]
pub struct EffectPattern {
    pub name: String,
    pub id: String,
}

/// Layer effect shadow (drop shadow and inner shadow)
#[derive(Debug, Clone, PartialEq)]
pub struct LayerEffectShadow {
    pub present: Option<bool>,
    pub show_in_dialog: Option<bool>,
    pub enabled: Option<bool>,
    pub size: Option<UnitsValue>,
    pub angle: Option<f64>,
    pub distance: Option<UnitsValue>,
    pub color: Option<Color>,
    pub blend_mode: Option<BlendMode>,
    pub opacity: Option<f64>,
    pub use_global_light: Option<bool>,
    pub antialiased: Option<bool>,
    pub contour: Option<EffectContour>,
    pub choke: Option<UnitsValue>,
    pub layer_conceals: Option<bool>,
}

/// Layer effect outer glow
#[derive(Debug, Clone, PartialEq)]
pub struct LayerEffectsOuterGlow {
    pub present: Option<bool>,
    pub show_in_dialog: Option<bool>,
    pub enabled: Option<bool>,
    pub size: Option<UnitsValue>,
    pub color: Option<Color>,
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
#[derive(Debug, Clone, PartialEq)]
pub struct LayerEffectInnerGlow {
    pub present: Option<bool>,
    pub show_in_dialog: Option<bool>,
    pub enabled: Option<bool>,
    pub size: Option<UnitsValue>,
    pub color: Option<Color>,
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
#[derive(Debug, Clone, PartialEq)]
pub struct LayerEffectBevel {
    pub present: Option<bool>,
    pub show_in_dialog: Option<bool>,
    pub enabled: Option<bool>,
    pub size: Option<UnitsValue>,
    pub angle: Option<f64>,
    pub strength: Option<f64>,
    pub highlight_blend_mode: Option<BlendMode>,
    pub shadow_blend_mode: Option<BlendMode>,
    pub highlight_color: Option<Color>,
    pub shadow_color: Option<Color>,
    pub style: Option<BevelStyle>,
    pub highlight_opacity: Option<f64>,
    pub shadow_opacity: Option<f64>,
    pub soften: Option<UnitsValue>,
    pub use_global_light: Option<bool>,
    pub altitude: Option<f64>,
    pub technique: Option<BevelTechnique>,
    pub direction: Option<BevelDirection>,
    pub use_texture: Option<bool>,
    pub use_shape: Option<bool>,
    pub antialias_gloss: Option<bool>,
    pub contour: Option<EffectContour>,
}

/// Layer effect solid fill
#[derive(Debug, Clone, PartialEq)]
pub struct LayerEffectSolidFill {
    pub present: Option<bool>,
    pub show_in_dialog: Option<bool>,
    pub enabled: Option<bool>,
    pub blend_mode: Option<BlendMode>,
    pub color: Option<Color>,
    pub opacity: Option<f64>,
}

/// Color stop in gradient
#[derive(Debug, Clone, PartialEq)]
pub struct ColorStop {
    pub color: Color,
    pub location: f64,
    pub midpoint: f64,
}

/// Opacity stop in gradient
#[derive(Debug, Clone, PartialEq)]
pub struct OpacityStop {
    pub opacity: f64,
    pub location: f64,
    pub midpoint: f64,
}

/// Solid gradient definition
#[derive(Debug, Clone, PartialEq)]
pub struct EffectSolidGradient {
    pub name: String,
    pub gradient_type: String,
    pub smoothness: Option<f64>,
    pub color_stops: Vec<ColorStop>,
    pub opacity_stops: Vec<OpacityStop>,
}

/// Noise gradient definition
#[derive(Debug, Clone, PartialEq)]
pub struct EffectNoiseGradient {
    pub name: String,
    pub gradient_type: String,
    pub roughness: Option<f64>,
    pub color_model: Option<String>,
    pub random_seed: Option<i32>,
    pub restrict_colors: Option<bool>,
    pub add_transparency: Option<bool>,
    pub min: Vec<f64>,
    pub max: Vec<f64>,
}

/// Extra gradient info
#[derive(Debug, Clone, PartialEq)]
pub struct ExtraGradientInfo {
    pub style: Option<GradientStyle>,
    pub scale: Option<f64>,
    pub angle: Option<f64>,
    pub dither: Option<bool>,
    pub interpolation_method: Option<InterpolationMethod>,
    pub reverse: Option<bool>,
    pub align: Option<bool>,
    pub offset: Option<Point>,
}

/// Gradient type for effects
#[derive(Debug, Clone, PartialEq)]
pub enum EffectGradient {
    Solid(EffectSolidGradient),
    Noise(EffectNoiseGradient),
}

/// Layer effect stroke
#[derive(Debug, Clone, PartialEq)]
pub struct LayerEffectStroke {
    pub present: Option<bool>,
    pub show_in_dialog: Option<bool>,
    pub enabled: Option<bool>,
    pub overprint: Option<bool>,
    pub size: Option<UnitsValue>,
    pub position: Option<String>,
    pub fill_type: Option<String>,
    pub blend_mode: Option<BlendMode>,
    pub opacity: Option<f64>,
    pub color: Option<Color>,
    pub gradient: Option<EffectGradient>,
    pub pattern: Option<EffectPattern>,
}

/// Layer effect satin
#[derive(Debug, Clone, PartialEq)]
pub struct LayerEffectSatin {
    pub present: Option<bool>,
    pub show_in_dialog: Option<bool>,
    pub enabled: Option<bool>,
    pub size: Option<UnitsValue>,
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
#[derive(Debug, Clone, PartialEq)]
pub struct LayerEffectPatternOverlay {
    pub present: Option<bool>,
    pub show_in_dialog: Option<bool>,
    pub enabled: Option<bool>,
    pub blend_mode: Option<BlendMode>,
    pub opacity: Option<f64>,
    pub scale: Option<f64>,
    pub pattern: Option<EffectPattern>,
    pub phase: Option<Point>,
    pub align: Option<bool>,
}

/// Layer effect gradient overlay
#[derive(Debug, Clone, PartialEq)]
pub struct LayerEffectGradientOverlay {
    pub present: Option<bool>,
    pub show_in_dialog: Option<bool>,
    pub enabled: Option<bool>,
    pub blend_mode: Option<String>,
    pub opacity: Option<f64>,
    pub align: Option<bool>,
    pub scale: Option<f64>,
    pub dither: Option<bool>,
    pub reverse: Option<bool>,
    pub gradient_type: Option<GradientStyle>,
    pub offset: Option<Point>,
    pub gradient: Option<EffectGradient>,
    pub interpolation_method: Option<InterpolationMethod>,
    pub angle: Option<f64>,
}

/// Layer effects info container
#[derive(Debug, Clone, PartialEq)]
pub struct LayerEffectsInfo {
    pub disabled: Option<bool>,
    pub scale: Option<f64>,
    pub drop_shadow: Option<Vec<LayerEffectShadow>>,
    pub inner_shadow: Option<Vec<LayerEffectShadow>>,
    pub outer_glow: Option<LayerEffectsOuterGlow>,
    pub inner_glow: Option<LayerEffectInnerGlow>,
    pub bevel: Option<LayerEffectBevel>,
    pub solid_fill: Option<Vec<LayerEffectSolidFill>>,
    pub satin: Option<LayerEffectSatin>,
    pub stroke: Option<Vec<LayerEffectStroke>>,
    pub gradient_overlay: Option<Vec<LayerEffectGradientOverlay>>,
    pub pattern_overlay: Option<LayerEffectPatternOverlay>,
}
