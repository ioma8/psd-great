use crate::types::*;

/// Font information
#[derive(Debug, Clone, PartialEq)]
pub struct Font {
    pub name: String,
    pub script: Option<i32>,
    pub font_type: Option<i32>,
    pub synthetic: Option<i32>,
}

/// Paragraph style
#[derive(Debug, Clone, PartialEq)]
pub struct ParagraphStyle {
    pub justification: Option<Justification>,
    pub first_line_indent: Option<f64>,
    pub start_indent: Option<f64>,
    pub end_indent: Option<f64>,
    pub space_before: Option<f64>,
    pub space_after: Option<f64>,
    pub auto_hyphenate: Option<bool>,
    pub hyphenated_word_size: Option<i32>,
    pub pre_hyphen: Option<i32>,
    pub post_hyphen: Option<i32>,
    pub consecutive_hyphens: Option<i32>,
    pub zone: Option<f64>,
    pub word_spacing: Option<Vec<f64>>,
    pub letter_spacing: Option<Vec<f64>>,
    pub glyph_spacing: Option<Vec<f64>>,
    pub auto_leading: Option<f64>,
    pub leading_type: Option<i32>,
    pub hanging: Option<bool>,
    pub burasagari: Option<bool>,
    pub kinsoku_order: Option<i32>,
    pub every_line_composer: Option<bool>,
}

/// Paragraph style run
#[derive(Debug, Clone, PartialEq)]
pub struct ParagraphStyleRun {
    pub length: usize,
    pub style: ParagraphStyle,
}

/// Text style
#[derive(Debug, Clone, PartialEq)]
pub struct TextStyle {
    pub font: Option<Font>,
    pub font_size: Option<f64>,
    pub faux_bold: Option<bool>,
    pub faux_italic: Option<bool>,
    pub auto_leading: Option<bool>,
    pub leading: Option<f64>,
    pub horizontal_scale: Option<f64>,
    pub vertical_scale: Option<f64>,
    pub tracking: Option<f64>,
    pub auto_kerning: Option<bool>,
    pub kerning: Option<f64>,
    pub baseline_shift: Option<f64>,
    pub font_caps: Option<i32>,
    pub font_baseline: Option<i32>,
    pub underline: Option<bool>,
    pub strikethrough: Option<bool>,
    pub ligatures: Option<bool>,
    pub d_ligatures: Option<bool>,
    pub baseline_direction: Option<i32>,
    pub tsume: Option<f64>,
    pub style_run_alignment: Option<i32>,
    pub language: Option<i32>,
    pub no_break: Option<bool>,
    pub fill_color: Option<Color>,
    pub stroke_color: Option<Color>,
    pub fill_flag: Option<bool>,
    pub stroke_flag: Option<bool>,
    pub fill_first: Option<bool>,
    pub y_underline: Option<f64>,
    pub outline_width: Option<f64>,
    pub character_direction: Option<i32>,
    pub hindi_numbers: Option<bool>,
    pub kashida: Option<f64>,
    pub diacritic_pos: Option<i32>,
}

/// Text style run
#[derive(Debug, Clone, PartialEq)]
pub struct TextStyleRun {
    pub length: usize,
    pub style: TextStyle,
}

/// Text grid info
#[derive(Debug, Clone, PartialEq)]
pub struct TextGridInfo {
    pub is_on: Option<bool>,
    pub show: Option<bool>,
    pub size: Option<f64>,
    pub leading: Option<f64>,
    pub color: Option<Color>,
    pub leading_fill_color: Option<Color>,
    pub align_line_height_to_grid_flags: Option<bool>,
}

/// Units bounds
#[derive(Debug, Clone, PartialEq)]
pub struct UnitsBounds {
    pub top: UnitsValue,
    pub left: UnitsValue,
    pub right: UnitsValue,
    pub bottom: UnitsValue,
}

/// Text path
#[derive(Debug, Clone, PartialEq)]
pub struct TextPath {
    pub name: Option<Vec<u8>>,
    pub bezier_curve: Option<TextPathBezierCurve>,
    pub data: TextPathData,
    pub uuid: Option<String>,
}

/// Bezier curve in text path
#[derive(Debug, Clone, PartialEq)]
pub struct TextPathBezierCurve {
    pub control_points: Vec<f64>,
}

/// Text path data
#[derive(Debug, Clone, PartialEq)]
pub struct TextPathData {
    pub path_type: Option<i32>,
    pub orientation: Option<i32>,
    pub frame_matrix: Vec<f64>,
    pub text_range: Vec<f64>,
    pub row_gutter: Option<f64>,
    pub column_gutter: Option<f64>,
    pub baseline_alignment: Option<BaselineAlignment>,
    pub path_data: PathData,
}

/// Baseline alignment
#[derive(Debug, Clone, PartialEq)]
pub struct BaselineAlignment {
    pub flag: Option<i32>,
    pub min: Option<f64>,
}

/// Path data
#[derive(Debug, Clone, PartialEq)]
pub struct PathData {
    pub reversed: Option<bool>,
    pub spacing: Option<f64>,
}

/// Warp definition
#[derive(Debug, Clone, PartialEq)]
pub struct Warp {
    pub style: Option<WarpStyle>,
    pub value: Option<f64>,
    pub values: Option<Vec<f64>>,
    pub perspective: Option<f64>,
    pub perspective_other: Option<f64>,
    pub rotate: Option<Orientation>,
    pub bounds: Option<UnitsBounds>,
    pub u_order: Option<i32>,
    pub v_order: Option<i32>,
    pub deform_num_rows: Option<i32>,
    pub deform_num_cols: Option<i32>,
    pub custom_envelope_warp: Option<CustomEnvelopeWarp>,
}

/// Custom envelope warp
#[derive(Debug, Clone, PartialEq)]
pub struct CustomEnvelopeWarp {
    pub quilt_slice_x: Option<Vec<f64>>,
    pub quilt_slice_y: Option<Vec<f64>>,
    pub mesh_points: Vec<Point>,
}

/// Layer text data
#[derive(Debug, Clone, PartialEq)]
pub struct LayerTextData {
    pub text: String,
    pub transform: Option<Vec<f64>>,
    pub anti_alias: Option<AntiAlias>,
    pub gridding: Option<TextGridding>,
    pub orientation: Option<Orientation>,
    pub index: Option<usize>,
    pub warp: Option<Warp>,
    pub top: Option<f64>,
    pub left: Option<f64>,
    pub bottom: Option<f64>,
    pub right: Option<f64>,
    pub grid_info: Option<TextGridInfo>,
    pub use_fractional_glyph_widths: Option<bool>,
    pub style: Option<TextStyle>,
    pub style_runs: Option<Vec<TextStyleRun>>,
    pub paragraph_style: Option<ParagraphStyle>,
    pub paragraph_style_runs: Option<Vec<ParagraphStyleRun>>,
    pub superscript_size: Option<f64>,
    pub superscript_position: Option<f64>,
    pub subscript_size: Option<f64>,
    pub subscript_position: Option<f64>,
    pub small_cap_size: Option<f64>,
    pub shape_type: Option<String>,
    pub point_base: Option<Vec<f64>>,
    pub box_bounds: Option<Vec<f64>>,
    pub bounds: Option<UnitsBounds>,
    pub bounding_box: Option<UnitsBounds>,
    pub text_path: Option<TextPath>,
}
