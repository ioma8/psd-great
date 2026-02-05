use serde::{Deserialize, Serialize};
use crate::types::*;
use crate::layer::{Layer, LayerAdditionalInfo, LinkedFile};

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
    pub direction: String,
}

/// Grid and guides information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GridAndGuidesInformation {
    pub grid: Option<GridInfo>,
    pub guides: Option<Vec<GuideInfo>>,
}

/// Resolution info
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResolutionInfo {
    #[serde(rename = "horizontalResolution")]
    pub horizontal_resolution: f64,
    #[serde(rename = "horizontalResolutionUnit")]
    pub horizontal_resolution_unit: String,
    #[serde(rename = "widthUnit")]
    pub width_unit: String,
    #[serde(rename = "verticalResolution")]
    pub vertical_resolution: f64,
    #[serde(rename = "verticalResolutionUnit")]
    pub vertical_resolution_unit: String,
    #[serde(rename = "heightUnit")]
    pub height_unit: String,
}

/// Thumbnail raw data
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ThumbnailRaw {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

/// Print scale
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PrintScale {
    pub style: Option<String>,
    pub x: Option<f64>,
    pub y: Option<f64>,
    pub scale: Option<f64>,
}

/// Proof setup builtin
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProofSetupBuiltin {
    pub builtin: String,
}

/// Proof setup profile
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProofSetupProfile {
    pub profile: String,
    #[serde(rename = "renderingIntent")]
    pub rendering_intent: Option<RenderingIntent>,
    #[serde(rename = "blackPointCompensation")]
    pub black_point_compensation: Option<bool>,
    #[serde(rename = "paperWhite")]
    pub paper_white: Option<bool>,
}

/// Proof setup
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ProofSetup {
    Builtin(ProofSetupBuiltin),
    Profile(ProofSetupProfile),
}

/// Print information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PrintInformation {
    #[serde(rename = "printerManagesColors")]
    pub printer_manages_colors: Option<bool>,
    #[serde(rename = "printerName")]
    pub printer_name: Option<String>,
    #[serde(rename = "printerProfile")]
    pub printer_profile: Option<String>,
    #[serde(rename = "printSixteenBit")]
    pub print_sixteen_bit: Option<bool>,
    #[serde(rename = "renderingIntent")]
    pub rendering_intent: Option<RenderingIntent>,
    #[serde(rename = "hardProof")]
    pub hard_proof: Option<bool>,
    #[serde(rename = "blackPointCompensation")]
    pub black_point_compensation: Option<bool>,
    #[serde(rename = "proofSetup")]
    pub proof_setup: Option<ProofSetup>,
}

/// Print flags
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PrintFlags {
    pub labels: Option<bool>,
    #[serde(rename = "cropMarks")]
    pub crop_marks: Option<bool>,
    #[serde(rename = "colorBars")]
    pub color_bars: Option<bool>,
    #[serde(rename = "registrationMarks")]
    pub registration_marks: Option<bool>,
    pub negative: Option<bool>,
    pub flip: Option<bool>,
    pub interpolate: Option<bool>,
    pub caption: Option<bool>,
    #[serde(rename = "printFlags")]
    pub print_flags: Option<bool>,
}

/// Onion skins
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OnionSkins {
    pub enabled: bool,
    #[serde(rename = "framesBefore")]
    pub frames_before: i32,
    #[serde(rename = "framesAfter")]
    pub frames_after: i32,
    #[serde(rename = "frameSpacing")]
    pub frame_spacing: i32,
    #[serde(rename = "minOpacity")]
    pub min_opacity: f64,
    #[serde(rename = "maxOpacity")]
    pub max_opacity: f64,
    #[serde(rename = "blendMode")]
    pub blend_mode: BlendMode,
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

/// Timeline information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimelineInformation {
    pub enabled: bool,
    #[serde(rename = "frameStep")]
    pub frame_step: Fraction,
    #[serde(rename = "frameRate")]
    pub frame_rate: f64,
    pub time: Fraction,
    pub duration: Fraction,
    #[serde(rename = "workInTime")]
    pub work_in_time: Fraction,
    #[serde(rename = "workOutTime")]
    pub work_out_time: Fraction,
    pub repeats: i32,
    #[serde(rename = "hasMotion")]
    pub has_motion: bool,
    #[serde(rename = "globalTracks")]
    pub global_tracks: Vec<crate::layer::TimelineTrack>,
    #[serde(rename = "audioClipGroups")]
    pub audio_clip_groups: Option<Vec<AudioClipGroup>>,
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

/// Slice bounds
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SliceBounds {
    pub left: f64,
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
}

/// Slice
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Slice {
    pub id: i32,
    #[serde(rename = "groupId")]
    pub group_id: i32,
    pub origin: String,
    #[serde(rename = "associatedLayerId")]
    pub associated_layer_id: i32,
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub slice_type: String,
    pub bounds: SliceBounds,
    pub url: String,
    pub target: String,
    pub message: String,
    #[serde(rename = "altTag")]
    pub alt_tag: String,
    #[serde(rename = "cellTextIsHTML")]
    pub cell_text_is_html: bool,
    #[serde(rename = "cellText")]
    pub cell_text: String,
    #[serde(rename = "horizontalAlignment")]
    pub horizontal_alignment: String,
    #[serde(rename = "verticalAlignment")]
    pub vertical_alignment: String,
    #[serde(rename = "backgroundColorType")]
    pub background_color_type: String,
    #[serde(rename = "backgroundColor")]
    pub background_color: RGBA,
    #[serde(rename = "topOutset")]
    pub top_outset: Option<f64>,
    #[serde(rename = "leftOutset")]
    pub left_outset: Option<f64>,
    #[serde(rename = "bottomOutset")]
    pub bottom_outset: Option<f64>,
    #[serde(rename = "rightOutset")]
    pub right_outset: Option<f64>,
}

/// Slices info
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SlicesInfo {
    pub bounds: SliceBounds,
    #[serde(rename = "groupName")]
    pub group_name: String,
    pub slices: Vec<Slice>,
}

/// Layer comp info
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LayerCompInfo {
    pub id: i32,
    pub name: String,
    pub comment: Option<String>,
    #[serde(rename = "capturedInfo")]
    pub captured_info: LayerCompCapturedInfo,
}

/// Layer comps
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LayerComps {
    pub list: Vec<LayerCompInfo>,
    #[serde(rename = "lastApplied")]
    pub last_applied: Option<i32>,
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

/// Image resources
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImageResources {
    #[serde(rename = "layerState")]
    pub layer_state: Option<u16>,
    #[serde(rename = "layerSelectionIds")]
    pub layer_selection_ids: Option<Vec<i32>>,
    #[serde(rename = "versionInfo")]
    pub version_info: Option<VersionInfo>,
    #[serde(rename = "alphaIdentifiers")]
    pub alpha_identifiers: Option<Vec<i32>>,
    #[serde(rename = "alphaChannelNames")]
    pub alpha_channel_names: Option<Vec<String>>,
    #[serde(rename = "globalAngle")]
    pub global_angle: Option<f64>,
    #[serde(rename = "globalAltitude")]
    pub global_altitude: Option<f64>,
    #[serde(rename = "pixelAspectRatio")]
    pub pixel_aspect_ratio: Option<PixelAspectRatio>,
    #[serde(rename = "urlsList")]
    pub urls_list: Option<Vec<UrlsListItem>>,
    #[serde(rename = "gridAndGuidesInformation")]
    pub grid_and_guides_information: Option<GridAndGuidesInformation>,
    #[serde(rename = "resolutionInfo")]
    pub resolution_info: Option<ResolutionInfo>,
    #[serde(rename = "thumbnailRaw")]
    pub thumbnail_raw: Option<ThumbnailRaw>,
    #[serde(rename = "captionDigest")]
    pub caption_digest: Option<String>,
    #[serde(rename = "xmpMetadata")]
    pub xmp_metadata: Option<String>,
    #[serde(rename = "printScale")]
    pub print_scale: Option<PrintScale>,
    #[serde(rename = "printInformation")]
    pub print_information: Option<PrintInformation>,
    #[serde(rename = "backgroundColor")]
    pub background_color: Option<Color>,
    #[serde(rename = "idsSeedNumber")]
    pub ids_seed_number: Option<u32>,
    #[serde(rename = "printFlags")]
    pub print_flags: Option<PrintFlags>,
    #[serde(rename = "iccUntaggedProfile")]
    pub icc_untagged_profile: Option<bool>,
    #[serde(rename = "pathSelectionState")]
    pub path_selection_state: Option<Vec<String>>,
    #[serde(rename = "imageReadyVariables")]
    pub image_ready_variables: Option<String>,
    #[serde(rename = "imageReadyDataSets")]
    pub image_ready_data_sets: Option<String>,
    pub animations: Option<Animations>,
    #[serde(rename = "onionSkins")]
    pub onion_skins: Option<OnionSkins>,
    #[serde(rename = "timelineInformation")]
    pub timeline_information: Option<TimelineInformation>,
    #[serde(rename = "sheetDisclosure")]
    pub sheet_disclosure: Option<SheetDisclosure>,
    #[serde(rename = "countInformation")]
    pub count_information: Option<Vec<CountInformation>>,
    pub slices: Option<Vec<SlicesInfo>>,
    #[serde(rename = "layerComps")]
    pub layer_comps: Option<LayerComps>,
    pub copyrighted: Option<bool>,
    pub url: Option<String>,
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

/// Main PSD document structure
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Psd {
    pub width: u32,
    pub height: u32,
    pub channels: Option<u16>,
    #[serde(rename = "bitsPerChannel")]
    pub bits_per_channel: Option<u8>,
    #[serde(rename = "colorMode")]
    pub color_mode: Option<ColorMode>,
    pub palette: Option<Vec<RGB>>,
    pub children: Option<Vec<Layer>>,
    #[serde(rename = "imageData")]
    pub image_data: Option<PixelData>,
    #[serde(rename = "imageResources")]
    pub image_resources: Option<ImageResources>,
    #[serde(rename = "linkedFiles")]
    pub linked_files: Option<Vec<LinkedFile>>,
    pub artboards: Option<ArtboardsInfo>,
    #[serde(rename = "globalLayerMaskInfo")]
    pub global_layer_mask_info: Option<GlobalLayerMaskInfo>,
    pub annotations: Option<Vec<Annotation>>,
    
    #[serde(flatten)]
    pub additional_info: LayerAdditionalInfo,
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
