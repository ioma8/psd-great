use crate::additional_info::LayerAdditionalInfo;
use crate::layer::{Layer, LinkedFile};
use crate::types::*;
use serde::{Deserialize, Serialize};

/// Animations definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Animations {
    pub frames: Vec<AnimationFrameInfo>,
    pub animations: Vec<AnimationInfo>,
}

/// Animation frame info
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnimationFrameInfo {
    pub id: i32,
    pub delay: f64,
    pub dispose: Option<String>,
}

/// Animation info
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnimationInfo {
    pub id: i32,
    pub frames: Vec<i32>,
    pub repeats: Option<i32>,
    #[serde(rename = "activeFrame")]
    pub active_frame: Option<i32>,
}

/// Version info
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VersionInfo {
    #[serde(rename = "hasRealMergedData")]
    pub has_real_merged_data: bool,
    #[serde(rename = "writerName")]
    pub writer_name: String,
    #[serde(rename = "readerName")]
    pub reader_name: String,
    #[serde(rename = "fileVersion")]
    pub file_version: u32,
}

/// Pixel aspect ratio
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PixelAspectRatio {
    pub aspect: f64,
}

/// URL list item
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UrlsListItem {
    pub id: i32,
    #[serde(rename = "ref")]
    pub reference: String,
    pub url: String,
}

/// Grid info
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GridInfo {
    pub horizontal: f64,
    pub vertical: f64,
}

/// Guide info
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GuideInfo {
    pub location: f64,
    pub direction: PsdStringCode,
}

/// Grid and guides information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GridAndGuidesInformation {
    pub grid: Option<GridInfo>,
    pub guides: Option<Vec<GuideInfo>>,
}

/// Thumbnail raw data
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ThumbnailRaw {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}



/// Audio clip frame reader
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AudioClipFrameReader {
    #[serde(rename = "type")]
    pub reader_type: i32,
    #[serde(rename = "mediaDescriptor")]
    pub media_descriptor: String,
    pub link: AudioClipLink,
}

/// Audio clip link
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AudioClipLink {
    pub name: String,
    #[serde(rename = "fullPath")]
    pub full_path: String,
    #[serde(rename = "relativePath")]
    pub relative_path: String,
}

/// Audio clip
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AudioClip {
    pub id: String,
    pub start: Fraction,
    pub duration: Fraction,
    #[serde(rename = "inTime")]
    pub in_time: Fraction,
    #[serde(rename = "outTime")]
    pub out_time: Fraction,
    pub muted: bool,
    #[serde(rename = "audioLevel")]
    pub audio_level: f64,
    #[serde(rename = "frameReader")]
    pub frame_reader: AudioClipFrameReader,
}

/// Audio clip group
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AudioClipGroup {
    pub id: String,
    pub muted: bool,
    #[serde(rename = "audioClips")]
    pub audio_clips: Vec<AudioClip>,
}

/// Sheet timeline options
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SheetTimelineOptions {
    #[serde(rename = "sheetID")]
    pub sheet_id: i32,
    #[serde(rename = "sheetDisclosed")]
    pub sheet_disclosed: bool,
    #[serde(rename = "lightsDisclosed")]
    pub lights_disclosed: bool,
    #[serde(rename = "meshesDisclosed")]
    pub meshes_disclosed: bool,
    #[serde(rename = "materialsDisclosed")]
    pub materials_disclosed: bool,
}

/// Sheet disclosure
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SheetDisclosure {
    #[serde(rename = "sheetTimelineOptions")]
    pub sheet_timeline_options: Option<Vec<SheetTimelineOptions>>,
}

/// Count information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CountInformation {
    pub color: RGB,
    pub name: String,
    pub size: f64,
    #[serde(rename = "fontSize")]
    pub font_size: f64,
    pub visible: bool,
    pub points: Vec<Point>,
}



/// Global layer mask info
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GlobalLayerMaskInfo {
    #[serde(rename = "overlayColorSpace")]
    pub overlay_color_space: u16,
    #[serde(rename = "colorSpace1")]
    pub color_space1: u16,
    #[serde(rename = "colorSpace2")]
    pub color_space2: u16,
    #[serde(rename = "colorSpace3")]
    pub color_space3: u16,
    #[serde(rename = "colorSpace4")]
    pub color_space4: u16,
    pub opacity: u16,
    pub kind: u8,
}

/// Annotation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Annotation {
    #[serde(rename = "type")]
    pub annotation_type: String,
    pub open: bool,
    #[serde(rename = "iconLocation")]
    pub icon_location: AnnotationLocation,
    #[serde(rename = "popupLocation")]
    pub popup_location: AnnotationLocation,
    pub color: Color,
    pub author: String,
    pub name: String,
    pub date: String,
    pub data: AnnotationData,
}

/// Annotation location
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnnotationLocation {
    pub left: f64,
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
}

/// Annotation data
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AnnotationData {
    Text(String),
    Sound(Vec<u8>),
}

/// Artboards info
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArtboardsInfo {
    pub count: i32,
    #[serde(rename = "autoExpandOffset")]
    pub auto_expand_offset: Option<ArtboardOffset>,
    pub origin: Option<ArtboardOffset>,
    #[serde(rename = "autoExpandEnabled")]
    pub auto_expand_enabled: Option<bool>,
    #[serde(rename = "autoNestEnabled")]
    pub auto_nest_enabled: Option<bool>,
    #[serde(rename = "autoPositionEnabled")]
    pub auto_position_enabled: Option<bool>,
    #[serde(rename = "shrinkwrapOnSaveEnabled")]
    pub shrinkwrap_on_save_enabled: Option<bool>,
    #[serde(rename = "docDefaultNewArtboardBackgroundColor")]
    pub doc_default_new_artboard_background_color: Option<Color>,
    #[serde(rename = "docDefaultNewArtboardBackgroundType")]
    pub doc_default_new_artboard_background_type: Option<i32>,
}

/// Artboard offset
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArtboardOffset {
    pub horizontal: f64,
    pub vertical: f64,
}

/// Variable set (resource 7000)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VariableSet {
    #[serde(rename = "varName")]
    pub var_name: Option<String>,
    #[serde(rename = "trait")]
    pub trait_name: Option<String>,
    #[serde(rename = "docRef")]
    pub doc_ref: Option<String>,
    #[serde(rename = "placementMethod")]
    pub placement_method: Option<String>,
    pub align: Option<String>,
    pub valign: Option<String>,
    pub clip: Option<String>,
}

/// Custom point (resource 1073)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CustomPoint {
    pub x: f64,
    pub y: f64,
}

/// Display info (resource 1036)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DisplayInfo {
    #[serde(rename = "hResUnit")]
    pub h_res_unit: PsdU16Code,
    #[serde(rename = "vResUnit")]
    pub v_res_unit: PsdU16Code,
    #[serde(rename = "widthUnit")]
    pub width_unit: PsdU16Code,
    #[serde(rename = "heightUnit")]
    pub height_unit: PsdU16Code,
}

/// Generic color-mode section payload for non-indexed modes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(transparent)]
pub struct ColorModeSectionData {
    pub bytes: Vec<u8>,
}

/// Main PSD document structure
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Psd {
    pub width: u32,
    pub height: u32,
    pub channels: Option<u16>,
    pub bits_per_channel: Option<u8>,
    pub color_mode: Option<ColorMode>,
    pub palette: Option<Vec<RGB>>,
    pub children: Option<Vec<Layer>>,
    pub image_data: Option<PixelData>,
    pub image_resources: Option<crate::image_resources::ImageResources>,
    pub linked_files: Option<Vec<LinkedFile>>,
    pub artboards: Option<ArtboardsInfo>,
    pub global_layer_mask_info: Option<GlobalLayerMaskInfo>,
    pub annotations: Option<Vec<Annotation>>,
    pub additional_info: LayerAdditionalInfo,
    pub color_mode_data: Option<ColorModeSectionData>,
    /// Document path selection descriptor (resource 3000)
    pub path_selection_descriptor: Option<crate::descriptor::Descriptor>,
    pub variable_sets: Option<Vec<VariableSet>>,
    pub data_sets: Option<Vec<Vec<String>>>,
    pub descriptor_1065: Option<crate::descriptor::Descriptor>,
    pub descriptor_1074: Option<crate::descriptor::Descriptor>,
    pub descriptor_1075: Option<crate::descriptor::Descriptor>,
    pub custom_points: Option<Vec<CustomPoint>>,
    pub display_info: Option<DisplayInfo>,
}

/// Read options for PSD parsing
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ReadOptions {
    #[serde(rename = "skipLayerImageData")]
    pub skip_layer_image_data: Option<bool>,
    #[serde(rename = "skipCompositeImageData")]
    pub skip_composite_image_data: Option<bool>,
    #[serde(rename = "skipThumbnail")]
    pub skip_thumbnail: Option<bool>,
    #[serde(rename = "skipLinkedFilesData")]
    pub skip_linked_files_data: Option<bool>,
    #[serde(rename = "throwForMissingFeatures")]
    pub throw_for_missing_features: Option<bool>,
    #[serde(rename = "logMissingFeatures")]
    pub log_missing_features: Option<bool>,
    #[serde(rename = "useImageData")]
    pub use_image_data: Option<bool>,
    #[serde(rename = "useRawData")]
    pub use_raw_data: Option<bool>,
    #[serde(rename = "useRawThumbnail")]
    pub use_raw_thumbnail: Option<bool>,
    #[serde(rename = "logDevFeatures")]
    pub log_dev_features: Option<bool>,
    pub strict: Option<bool>,
    pub debug: Option<bool>,
}

/// Write options for PSD generation
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct WriteOptions {
    #[serde(rename = "generateThumbnail")]
    pub generate_thumbnail: Option<bool>,
    #[serde(rename = "trimImageData")]
    pub trim_image_data: Option<bool>,
    #[serde(rename = "invalidateTextLayers")]
    pub invalidate_text_layers: Option<bool>,
    #[serde(rename = "logMissingFeatures")]
    pub log_missing_features: Option<bool>,
    #[serde(rename = "noBackground")]
    pub no_background: Option<bool>,
    pub psb: Option<bool>,
    pub compress: Option<bool>,
}
