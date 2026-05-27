use std::io::Cursor;

use binrw::{BinRead, BinReaderExt, BinWrite, BinWriterExt};

use crate::error::{PsdError, Result};

#[derive(binrw::BinRead, binrw::BinWrite, Debug, Clone, PartialEq, Eq)]
#[brw(big)]
pub(crate) struct PsdHeaderRecord {
    pub signature: [u8; 4],
    pub version: u16,
    pub reserved: [u8; 6],
    pub channels: u16,
    pub height: u32,
    pub width: u32,
    pub depth: u16,
    pub color_mode: u16,
}

#[derive(binrw::BinRead, binrw::BinWrite, Debug, Clone, PartialEq, Eq)]
#[brw(big)]
pub(crate) struct GlobalLayerMaskRecord {
    pub overlay_color_space: u16,
    pub color_space1: u16,
    pub color_space2: u16,
    pub color_space3: u16,
    pub color_space4: u16,
    pub opacity: u16,
    pub kind: u8,
    pub reserved: [u8; 3],
}

#[derive(binrw::BinRead, binrw::BinWrite, Debug, Clone, PartialEq, Eq)]
#[brw(big)]
pub(crate) struct ResolutionInfoRecord {
    pub horizontal_res_fixed: i32,
    pub horizontal_res_unit: u16,
    pub width_unit: u16,
    pub vertical_res_fixed: i32,
    pub vertical_res_unit: u16,
    pub height_unit: u16,
}

#[derive(binrw::BinRead, binrw::BinWrite, Debug, Clone, PartialEq, Eq, Default)]
#[brw(big)]
pub(crate) struct PrintFlagsRecord {
    pub labels: u8,
    pub crop_marks: u8,
    pub color_bars: u8,
    pub registration_marks: u8,
    pub negative: u8,
    pub flip: u8,
    pub interpolate: u8,
    pub caption: u8,
    pub print_flags: u8,
}

#[derive(binrw::BinRead, binrw::BinWrite, Debug, Clone, PartialEq, Eq)]
#[brw(big)]
pub(crate) struct GridAndGuidesHeaderRecord {
    pub version: u32,
    pub grid_horizontal: u32,
    pub grid_vertical: u32,
    pub guide_count: u32,
}

#[derive(binrw::BinRead, binrw::BinWrite, Debug, Clone, PartialEq, Eq)]
#[brw(big)]
pub(crate) struct GuideRecord {
    pub location_times_32: u32,
    pub direction: u8,
}

#[derive(binrw::BinRead, binrw::BinWrite, Debug, Clone, PartialEq, Eq)]
#[brw(big)]
pub(crate) struct LayerStateRecord {
    pub state: u16,
}

#[derive(binrw::BinRead, binrw::BinWrite, Debug, Clone, PartialEq)]
#[brw(big)]
pub(crate) struct PrintScaleRecord {
    pub style: i16,
    pub x: f32,
    pub y: f32,
    pub scale: f32,
}

#[derive(binrw::BinRead, binrw::BinWrite, Debug, Clone, PartialEq, Eq)]
#[brw(big)]
pub(crate) struct SignedI32Record {
    pub value: i32,
}

#[derive(binrw::BinRead, binrw::BinWrite, Debug, Clone, PartialEq, Eq)]
#[brw(big)]
pub(crate) struct U8BoolRecord {
    pub value: u8,
}

#[derive(binrw::BinRead, binrw::BinWrite, Debug, Clone, PartialEq, Eq)]
#[brw(big)]
pub(crate) struct U16ListCountRecord {
    pub count: u16,
}

#[derive(binrw::BinRead, binrw::BinWrite, Debug, Clone, PartialEq, Eq)]
#[brw(big)]
pub(crate) struct U32ValueRecord {
    pub value: u32,
}

#[derive(binrw::BinRead, binrw::BinWrite, Debug, Clone, PartialEq, Eq)]
#[brw(big)]
pub(crate) struct ImageResourceHeaderRecord {
    pub signature: [u8; 4],
    pub resource_id: u16,
    pub name_length: u8,
}

#[derive(binrw::BinRead, binrw::BinWrite, Debug, Clone, PartialEq, Eq)]
#[brw(big)]
pub(crate) struct ImageResourceLengthRecord {
    pub data_length: u32,
}

#[derive(binrw::BinRead, binrw::BinWrite, Debug, Clone, PartialEq, Eq)]
#[brw(big)]
pub(crate) struct LayerRecordBounds {
    pub top: i32,
    pub left: i32,
    pub bottom: i32,
    pub right: i32,
    pub channel_count: u16,
}

#[derive(binrw::BinRead, binrw::BinWrite, Debug, Clone, PartialEq, Eq)]
#[brw(big)]
pub(crate) struct ChannelInfoRecord {
    pub id: i16,
    pub length: u32,
}

#[derive(binrw::BinRead, binrw::BinWrite, Debug, Clone, PartialEq, Eq)]
#[brw(big)]
pub(crate) struct PsbChannelInfoRecord {
    pub id: i16,
    pub high_length: u32,
    pub low_length: u32,
}

#[derive(binrw::BinRead, binrw::BinWrite, Debug, Clone, PartialEq, Eq)]
#[brw(big)]
pub(crate) struct LayerBlendRecord {
    pub signature: [u8; 4],
    pub blend_mode: [u8; 4],
    pub opacity: u8,
    pub clipping: u8,
    pub flags: u8,
    pub filler: u8,
}

#[derive(binrw::BinRead, binrw::BinWrite, Debug, Clone, PartialEq, Eq)]
#[brw(big)]
pub(crate) struct LayerMaskPrefixRecord {
    pub top: i32,
    pub left: i32,
    pub bottom: i32,
    pub right: i32,
    pub default_color: u8,
    pub flags: u8,
}

#[derive(binrw::BinRead, binrw::BinWrite, Debug, Clone, PartialEq, Eq)]
#[brw(big)]
pub(crate) struct LayerColorRecord {
    pub color_value: u16,
    pub padding: [u8; 6],
}

#[derive(binrw::BinRead, binrw::BinWrite, Debug, Clone, PartialEq, Eq)]
#[brw(big)]
pub(crate) struct SectionDividerBaseRecord {
    pub divider_type: u32,
}

#[derive(binrw::BinRead, binrw::BinWrite, Debug, Clone, PartialEq, Eq)]
#[brw(big)]
pub(crate) struct SectionDividerExtendedRecord {
    pub signature: [u8; 4],
    pub blend_mode: [u8; 4],
}

#[derive(binrw::BinRead, binrw::BinWrite, Debug, Clone, PartialEq, Eq)]
#[brw(big)]
pub(crate) struct ProtectedFlagsRecord {
    pub flags: u32,
}

#[derive(binrw::BinRead, binrw::BinWrite, Debug, Clone, PartialEq, Eq)]
#[brw(big)]
pub(crate) struct NameSourceRecord {
    pub signature: [u8; 4],
}

#[derive(binrw::BinRead, binrw::BinWrite, Debug, Clone, PartialEq, Eq)]
#[brw(big)]
pub(crate) struct EffectsHeaderRecord {
    pub version: u16,
    pub effects_count: u16,
}

#[derive(binrw::BinRead, binrw::BinWrite, Debug, Clone, PartialEq, Eq)]
#[brw(big)]
pub(crate) struct EffectsCommonStateRecord {
    pub size: u32,
    pub version: u32,
    pub visible: u8,
    pub padding: [u8; 2],
}

#[derive(binrw::BinRead, binrw::BinWrite, Debug, Clone, PartialEq, Eq)]
#[brw(big)]
pub(crate) struct EffectBlockHeaderRecord {
    pub block_size: u32,
    pub version: u32,
}

pub(crate) fn decode_be<T>(bytes: &[u8], context: &str) -> Result<T>
where
    for<'a> T: BinRead<Args<'a> = ()>,
{
    let mut cursor = Cursor::new(bytes);
    cursor
        .read_be()
        .map_err(|err| PsdError::InvalidFormat(format!("Failed to parse {}: {}", context, err)))
}

pub(crate) fn encode_be<T>(value: &T, context: &str) -> Result<Vec<u8>>
where
    for<'a> T: BinWrite<Args<'a> = ()>,
{
    let mut cursor = Cursor::new(Vec::new());
    cursor
        .write_be(value)
        .map_err(|err| PsdError::InvalidFormat(format!("Failed to write {}: {}", context, err)))?;
    Ok(cursor.into_inner())
}
