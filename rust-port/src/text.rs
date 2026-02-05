use serde::{Deserialize, Serialize};
use crate::types::*;

/// Font information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Font {
    pub name: String,
    pub script: Option<i32>,
    #[serde(rename = "type")]
    pub font_type: Option<i32>,
    pub synthetic: Option<i32>,
}

/// Paragraph style
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParagraphStyle {
    pub justification: Option<Justification>,
    #[serde(rename = "firstLineIndent")]
    pub first_line_indent: Option<f64>,
    #[serde(rename = "startIndent")]
    pub start_indent: Option<f64>,
    #[serde(rename = "endIndent")]
    pub end_indent: Option<f64>,
    #[serde(rename = "spaceBefore")]
    pub space_before: Option<f64>,
    #[serde(rename = "spaceAfter")]
    pub space_after: Option<f64>,
    #[serde(rename = "autoHyphenate")]
    pub auto_hyphenate: Option<bool>,
    #[serde(rename = "hyphenatedWordSize")]
    pub hyphenated_word_size: Option<i32>,
    #[serde(rename = "preHyphen")]
    pub pre_hyphen: Option<i32>,
    #[serde(rename = "postHyphen")]
    pub post_hyphen: Option<i32>,
    #[serde(rename = "consecutiveHyphens")]
    pub consecutive_hyphens: Option<i32>,
    pub zone: Option<f64>,
    #[serde(rename = "wordSpacing")]
    pub word_spacing: Option<Vec<f64>>,
    #[serde(rename = "letterSpacing")]
    pub letter_spacing: Option<Vec<f64>>,
    #[serde(rename = "glyphSpacing")]
    pub glyph_spacing: Option<Vec<f64>>,
    #[serde(rename = "autoLeading")]
    pub auto_leading: Option<f64>,
    #[serde(rename = "leadingType")]
    pub leading_type: Option<i32>,
    pub hanging: Option<bool>,
    pub burasagari: Option<bool>,
    #[serde(rename = "kinsokuOrder")]
    pub kinsoku_order: Option<i32>,
    #[serde(rename = "everyLineComposer")]
    pub every_line_composer: Option<bool>,
}

/// Paragraph style run
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParagraphStyleRun {
    pub length: usize,
    pub style: ParagraphStyle,
}

/// Text style
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextStyle {
    pub font: Option<Font>,
    #[serde(rename = "fontSize")]
    pub font_size: Option<f64>,
    #[serde(rename = "fauxBold")]
    pub faux_bold: Option<bool>,
    #[serde(rename = "fauxItalic")]
    pub faux_italic: Option<bool>,
    #[serde(rename = "autoLeading")]
    pub auto_leading: Option<bool>,
    pub leading: Option<f64>,
    #[serde(rename = "horizontalScale")]
    pub horizontal_scale: Option<f64>,
    #[serde(rename = "verticalScale")]
    pub vertical_scale: Option<f64>,
    pub tracking: Option<f64>,
    #[serde(rename = "autoKerning")]
    pub auto_kerning: Option<bool>,
    pub kerning: Option<f64>,
    #[serde(rename = "baselineShift")]
    pub baseline_shift: Option<f64>,
    #[serde(rename = "fontCaps")]
    pub font_caps: Option<i32>,
    #[serde(rename = "fontBaseline")]
    pub font_baseline: Option<i32>,
    pub underline: Option<bool>,
    pub strikethrough: Option<bool>,
    pub ligatures: Option<bool>,
    #[serde(rename = "dLigatures")]
    pub d_ligatures: Option<bool>,
    #[serde(rename = "baselineDirection")]
    pub baseline_direction: Option<i32>,
    pub tsume: Option<f64>,
    #[serde(rename = "styleRunAlignment")]
    pub style_run_alignment: Option<i32>,
    pub language: Option<i32>,
    #[serde(rename = "noBreak")]
    pub no_break: Option<bool>,
    #[serde(rename = "fillColor")]
    pub fill_color: Option<Color>,
    #[serde(rename = "strokeColor")]
    pub stroke_color: Option<Color>,
    #[serde(rename = "fillFlag")]
    pub fill_flag: Option<bool>,
    #[serde(rename = "strokeFlag")]
    pub stroke_flag: Option<bool>,
    #[serde(rename = "fillFirst")]
    pub fill_first: Option<bool>,
    #[serde(rename = "yUnderline")]
    pub y_underline: Option<f64>,
    #[serde(rename = "outlineWidth")]
    pub outline_width: Option<f64>,
    #[serde(rename = "characterDirection")]
    pub character_direction: Option<i32>,
    #[serde(rename = "hindiNumbers")]
    pub hindi_numbers: Option<bool>,
    pub kashida: Option<f64>,
    #[serde(rename = "diacriticPos")]
    pub diacritic_pos: Option<i32>,
}

/// Text style run
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextStyleRun {
    pub length: usize,
    pub style: TextStyle,
}

/// Text grid info
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextGridInfo {
    #[serde(rename = "isOn")]
    pub is_on: Option<bool>,
    pub show: Option<bool>,
    pub size: Option<f64>,
    pub leading: Option<f64>,
    pub color: Option<Color>,
    #[serde(rename = "leadingFillColor")]
    pub leading_fill_color: Option<Color>,
    #[serde(rename = "alignLineHeightToGridFlags")]
    pub align_line_height_to_grid_flags: Option<bool>,
}

/// Units bounds
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnitsBounds {
    pub top: UnitsValue,
    pub left: UnitsValue,
    pub right: UnitsValue,
    pub bottom: UnitsValue,
}

/// Text path
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextPath {
    pub name: Option<Vec<u8>>,
    #[serde(rename = "bezierCurve")]
    pub bezier_curve: Option<TextPathBezierCurve>,
    pub data: TextPathData,
    pub uuid: Option<String>,
}

/// Bezier curve in text path
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextPathBezierCurve {
    #[serde(rename = "controlPoints")]
    pub control_points: Vec<f64>,
}

/// Text path data
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextPathData {
    #[serde(rename = "type")]
    pub path_type: Option<i32>,
    pub orientation: Option<i32>,
    #[serde(rename = "frameMatrix")]
    pub frame_matrix: Vec<f64>,
    #[serde(rename = "textRange")]
    pub text_range: Vec<f64>,
    #[serde(rename = "rowGutter")]
    pub row_gutter: Option<f64>,
    #[serde(rename = "columnGutter")]
    pub column_gutter: Option<f64>,
    #[serde(rename = "BaselineAlignment")]
    pub baseline_alignment: Option<BaselineAlignment>,
    #[serde(rename = "pathData")]
    pub path_data: PathData,
}

/// Baseline alignment
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BaselineAlignment {
    pub flag: Option<i32>,
    pub min: Option<f64>,
}

/// Path data
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PathData {
    pub reversed: Option<bool>,
    pub spacing: Option<f64>,
}

/// Warp definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Warp {
    pub style: Option<WarpStyle>,
    pub value: Option<f64>,
    pub values: Option<Vec<f64>>,
    pub perspective: Option<f64>,
    #[serde(rename = "perspectiveOther")]
    pub perspective_other: Option<f64>,
    pub rotate: Option<Orientation>,
    pub bounds: Option<UnitsBounds>,
    #[serde(rename = "uOrder")]
    pub u_order: Option<i32>,
    #[serde(rename = "vOrder")]
    pub v_order: Option<i32>,
    #[serde(rename = "deformNumRows")]
    pub deform_num_rows: Option<i32>,
    #[serde(rename = "deformNumCols")]
    pub deform_num_cols: Option<i32>,
    #[serde(rename = "customEnvelopeWarp")]
    pub custom_envelope_warp: Option<CustomEnvelopeWarp>,
}

/// Custom envelope warp
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CustomEnvelopeWarp {
    #[serde(rename = "quiltSliceX")]
    pub quilt_slice_x: Option<Vec<f64>>,
    #[serde(rename = "quiltSliceY")]
    pub quilt_slice_y: Option<Vec<f64>>,
    #[serde(rename = "meshPoints")]
    pub mesh_points: Vec<Point>,
}

/// Layer text data
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LayerTextData {
    pub text: String,
    pub transform: Option<Vec<f64>>,
    #[serde(rename = "antiAlias")]
    pub anti_alias: Option<AntiAlias>,
    pub gridding: Option<TextGridding>,
    pub orientation: Option<Orientation>,
    pub index: Option<usize>,
    pub warp: Option<Warp>,
    pub top: Option<f64>,
    pub left: Option<f64>,
    pub bottom: Option<f64>,
    pub right: Option<f64>,
    #[serde(rename = "gridInfo")]
    pub grid_info: Option<TextGridInfo>,
    #[serde(rename = "useFractionalGlyphWidths")]
    pub use_fractional_glyph_widths: Option<bool>,
    pub style: Option<TextStyle>,
    #[serde(rename = "styleRuns")]
    pub style_runs: Option<Vec<TextStyleRun>>,
    #[serde(rename = "paragraphStyle")]
    pub paragraph_style: Option<ParagraphStyle>,
    #[serde(rename = "paragraphStyleRuns")]
    pub paragraph_style_runs: Option<Vec<ParagraphStyleRun>>,
    #[serde(rename = "superscriptSize")]
    pub superscript_size: Option<f64>,
    #[serde(rename = "superscriptPosition")]
    pub superscript_position: Option<f64>,
    #[serde(rename = "subscriptSize")]
    pub subscript_size: Option<f64>,
    #[serde(rename = "subscriptPosition")]
    pub subscript_position: Option<f64>,
    #[serde(rename = "smallCapSize")]
    pub small_cap_size: Option<f64>,
    #[serde(rename = "shapeType")]
    pub shape_type: Option<String>,
    #[serde(rename = "pointBase")]
    pub point_base: Option<Vec<f64>>,
    #[serde(rename = "boxBounds")]
    pub box_bounds: Option<Vec<f64>>,
    pub bounds: Option<UnitsBounds>,
    #[serde(rename = "boundingBox")]
    pub bounding_box: Option<UnitsBounds>,
    #[serde(rename = "textPath")]
    pub text_path: Option<TextPath>,
}
