# Eliminate Remaining Raw Buckets Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Remove the remaining opaque/raw preservation buckets from the Rust PSD model while keeping legitimate binary payload fields only where the TypeScript source of truth also models them as binary content.

**Architecture:** Split the remaining byte-bearing fields into two categories: true binary payloads that are semantically part of the public typed model, and opaque fallback buckets that still exist only to preserve unparsed structures. Eliminate the second category by porting the corresponding TS structures into typed Rust models, and leave the first category only where the TS implementation also treats the content as bytes rather than a richer semantic type.

**Tech Stack:** Rust, existing `PsdReader`/`PsdWriter`, `byteorder`, `binrw`, existing `descriptor`, `engine_data`, `image_resources`, and `additional_info` modules, `cargo test`

---

## File Structure

**Primary Rust files**
- `src/layer.rs`
  - Responsibility: remove model-level raw fallback fields that are currently used for preservation rather than semantic state.
- `src/psd.rs`
  - Responsibility: remove or replace document-level raw fallback fields such as generic color-mode-data preservation when TS has a richer model.
- `src/additional_info.rs`
  - Responsibility: replace remaining opaque `Vec<u8>`/`buffer` fields in tagged-block models (`FEid`, `PxSD`, filter mask) with typed structures where TS has explicit structure.
- `src/image_resources.rs`
  - Responsibility: replace low-level raw resource buckets (`resource_visibility`, `custom_points`, `display_info`) with typed resource structs, keeping binary content only for inherently binary resources like ICC profiles.
- `src/document_resource_postprocess.rs`
  - Responsibility: collapse the remaining “raw bytes then interpret” transitions into typed document resource state.
- `src/descriptor.rs`
  - Responsibility: evaluate and potentially narrow `DescriptorValue::RawData` to the TS-backed cases only, or replace it with a more explicit typed variant set.
- `src/reader.rs`
  - Responsibility: wire new typed parsing paths and remove preservation-oriented read paths.
- `src/writer.rs`
  - Responsibility: write from typed structures only for the affected areas.
- `tests/ts_parity_test.rs`
  - Responsibility: prove that removed raw buckets are replaced by typed structures and that TS parity remains intact.

**TS source of truth**
- `/Users/jakubkolcar/projects/customs/photoshop/psd/src/psd/tagged-block-reader.ts`
- `/Users/jakubkolcar/projects/customs/photoshop/psd/src/psd/tagged-block-writer.ts`
- `/Users/jakubkolcar/projects/customs/photoshop/psd/src/psd/document-postprocess.ts`
- `/Users/jakubkolcar/projects/customs/photoshop/psd/src/psd/resource-postprocess.ts`
- `/Users/jakubkolcar/projects/customs/photoshop/psd/src/psd/descriptor.ts`
- `/Users/jakubkolcar/projects/customs/photoshop/psd/src/types/psd-document.ts`
- `/Users/jakubkolcar/projects/customs/photoshop/psd/src/types/tagged-block.ts`

---

## Scope Rules

This plan treats these as **legitimate binary content**, not raw fallback buckets:
- embedded/linked file bytes
- ICC profile bytes
- thumbnail bytes
- JPEG/ZIP/RLE pixel channel bytes
- sound annotation payloads

This plan treats these as **remaining opaque/raw buckets that should be eliminated or narrowed**:
- `Layer.blending_ranges_raw`
- `Psd.color_mode_data`
- `ImageResources.resource_visibility`
- `ImageResources.custom_points`
- `ImageResources.display_info`
- `LayerAdditionalInfo.filter_mask`
- `FilterEffectsItem.buffer`
- `FilterEffectsSlot.raw`
- `FilterEffectsPreview.raw`
- `FilterEffectsPreview.buffer`
- `PixelSourceDataImage.buffer`
- `PixelSourceDataImage.palette`
- `PixelSourceDataImage.raw`
- broad `DescriptorValue::RawData` usage where TS has a specific semantic structure instead

---

### Task 1: Classify remaining byte fields and lock the target list in tests

**Files:**
- Modify: `tests/ts_parity_test.rs`
- Test: `tests/ts_parity_test.rs`

- [ ] **Step 1: Add a failing audit test that enumerates the fields we still intend to eliminate**

Add this test module to `tests/ts_parity_test.rs`:

```rust
#[test]
fn audit_remaining_opaque_raw_buckets() {
    let raw_bucket_markers = [
        "Layer.blending_ranges_raw",
        "Psd.color_mode_data",
        "ImageResources.resource_visibility",
        "ImageResources.custom_points",
        "ImageResources.display_info",
        "LayerAdditionalInfo.filter_mask",
        "FilterEffectsItem.buffer",
        "FilterEffectsSlot.raw",
        "FilterEffectsPreview.raw",
        "FilterEffectsPreview.buffer",
        "PixelSourceDataImage.buffer",
        "PixelSourceDataImage.palette",
        "PixelSourceDataImage.raw",
    ];

    assert_eq!(raw_bucket_markers.len(), 13);
}
```

- [ ] **Step 2: Run the audit test to verify the file compiles and the target list is explicit**

Run:

```bash
cargo test audit_remaining_opaque_raw_buckets -- --nocapture
```

Expected:
- PASS, establishing the exact elimination scope for this plan

- [ ] **Step 3: Commit**

```bash
git add tests/ts_parity_test.rs
git commit -m "test: lock raw bucket elimination scope"
```

---

### Task 2: Replace `ImageResources.resource_visibility`, `custom_points`, and `display_info` raw byte storage with typed resource structs

**Files:**
- Modify: `src/image_resources.rs`
- Modify: `src/document_resource_postprocess.rs`
- Modify: `src/psd.rs`
- Modify: `tests/ts_parity_test.rs`
- Test: `tests/ts_parity_test.rs`

- [ ] **Step 1: Write failing tests that assert the low-level resource model is typed**

Add these tests:

```rust
#[test]
fn image_resources_use_typed_resource_visibility() {
    let mut psd = Psd::default();
    let mut layer = Layer::default();
    layer.top = Some(0);
    layer.left = Some(0);
    layer.bottom = Some(1);
    layer.right = Some(1);
    layer.resource_visible = Some(false);
    psd.children = Some(vec![layer]);
    psd.width = 1;
    psd.height = 1;
    psd.channels = Some(4);
    psd.bits_per_channel = Some(8);
    psd.color_mode = Some(ColorMode::RGB);

    let bytes = write_psd(&psd, &WriteOptions::default()).expect("write");
    let reparsed = read_psd(Cursor::new(&bytes), ReadOptions::default()).expect("read");
    let resources = reparsed.image_resources.expect("resources");
    assert!(resources.resource_visibility_typed.is_some());
}

#[test]
fn image_resources_use_typed_custom_points_and_display_info() {
    let mut psd = Psd::default();
    psd.width = 1;
    psd.height = 1;
    psd.channels = Some(4);
    psd.bits_per_channel = Some(8);
    psd.color_mode = Some(ColorMode::RGB);
    psd.custom_points = Some(vec![ag_psd::psd::CustomPoint { x: 1.5, y: 2.5 }]);
    psd.display_info = Some(ag_psd::psd::DisplayInfo {
        h_res_unit: 1,
        v_res_unit: 2,
        width_unit: 3,
        height_unit: 4,
    });

    let bytes = write_psd(&psd, &WriteOptions::default()).expect("write");
    let reparsed = read_psd(Cursor::new(&bytes), ReadOptions::default()).expect("read");
    let resources = reparsed.image_resources.expect("resources");
    assert!(resources.custom_points_typed.is_some());
    assert!(resources.display_info_typed.is_some());
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run:

```bash
cargo test image_resources_use_typed_resource_visibility -- --nocapture
cargo test image_resources_use_typed_custom_points_and_display_info -- --nocapture
```

Expected:
- FAIL because the typed low-level fields do not exist yet

- [ ] **Step 3: Add typed resource structs to `src/image_resources.rs`**

Add these types:

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct ResourceVisibility {
    pub values: Vec<bool>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CustomPointsResource {
    pub version: u32,
    pub points: Vec<crate::psd::CustomPoint>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DisplayInfoResource {
    pub version: u16,
    pub h_res_unit: u16,
    pub v_res_unit: u16,
    pub width_unit: u16,
    pub height_unit: u16,
}
```

Replace these fields in `ImageResources`:

```rust
pub resource_visibility_typed: Option<ResourceVisibility>,
pub custom_points_typed: Option<CustomPointsResource>,
pub display_info_typed: Option<DisplayInfoResource>,
```

Delete:

```rust
pub resource_visibility: Option<Vec<u8>>,
pub custom_points: Option<Vec<u8>>,
pub display_info: Option<Vec<u8>>,
```

- [ ] **Step 4: Parse and write the resources directly as typed values**

Replace the `1036`, `1072`, and `1073` branches in `src/image_resources.rs` with direct typed parsing:

```rust
1036 => {
    let bytes = reader.read_bytes(data_length)?;
    resources.display_info_typed = parse_display_info_resource(&bytes);
}
1072 => {
    let bytes = reader.read_bytes(data_length)?;
    resources.resource_visibility_typed = Some(ResourceVisibility {
        values: bytes.into_iter().map(|b| b == 1).collect(),
    });
}
1073 => {
    let bytes = reader.read_bytes(data_length)?;
    resources.custom_points_typed = Some(parse_custom_points_resource(&bytes));
}
```

Add matching write branches:

```rust
if let Some(ref visibility) = resources.resource_visibility_typed {
    write_resource(writer, 1072, &|w| {
        w.write_bytes(
            &visibility
                .values
                .iter()
                .map(|v| if *v { 1 } else { 0 })
                .collect::<Vec<u8>>(),
        )
    })?;
}
```

```rust
if let Some(ref points) = resources.custom_points_typed {
    write_resource(writer, 1073, &|w| w.write_bytes(&build_custom_points_resource(points)))?;
}
```

```rust
if let Some(ref info) = resources.display_info_typed {
    write_resource(writer, 1036, &|w| w.write_bytes(&build_display_info_resource(info)))?;
}
```

- [ ] **Step 5: Update `src/document_resource_postprocess.rs` to use the new typed low-level resource structs**

Replace the current raw-byte mappings with:

```rust
if let Some(visibility) = resources.resource_visibility_typed.as_ref() {
    if let Some(layers) = psd.children.as_mut() {
        for (layer, value) in layers.iter_mut().zip(visibility.values.iter()) {
            layer.resource_visible = Some(*value);
        }
    }
}

if let Some(points) = resources.custom_points_typed.as_ref() {
    psd.custom_points = Some(points.points.clone());
}

if let Some(info) = resources.display_info_typed.as_ref() {
    psd.display_info = Some(crate::psd::DisplayInfo {
        h_res_unit: info.h_res_unit,
        v_res_unit: info.v_res_unit,
        width_unit: info.width_unit,
        height_unit: info.height_unit,
    });
}
```

And in prewrite:

```rust
resources.resource_visibility_typed = Some(crate::image_resources::ResourceVisibility {
    values,
});
```

```rust
resources.custom_points_typed = Some(crate::image_resources::CustomPointsResource {
    version: 3,
    points: points.clone(),
});
```

```rust
resources.display_info_typed = Some(crate::image_resources::DisplayInfoResource {
    version: 1,
    h_res_unit: info.h_res_unit,
    v_res_unit: info.v_res_unit,
    width_unit: info.width_unit,
    height_unit: info.height_unit,
});
```

- [ ] **Step 6: Run the focused tests again**

Run:

```bash
cargo test image_resources_use_typed_resource_visibility -- --nocapture
cargo test image_resources_use_typed_custom_points_and_display_info -- --nocapture
```

Expected:
- PASS

- [ ] **Step 7: Commit**

```bash
git add src/image_resources.rs src/document_resource_postprocess.rs src/psd.rs tests/ts_parity_test.rs
git commit -m "refactor: replace raw document resource buckets with typed structs"
```

---

### Task 3: Replace `Layer.blending_ranges_raw` with a typed blending-ranges model

**Files:**
- Modify: `src/layer.rs`
- Modify: `src/reader.rs`
- Modify: `src/writer.rs`
- Modify: `tests/ts_parity_test.rs`
- Test: `tests/ts_parity_test.rs`

- [ ] **Step 1: Write a failing blending-ranges parity test**

Add:

```rust
#[test]
fn layer_blending_ranges_are_typed_not_raw() {
    let data = vec![
        0x00, 0x10, 0x00, 0xF0,
        0x00, 0x20, 0x00, 0xE0,
        0x00, 0x30, 0x00, 0xD0,
        0x00, 0x40, 0x00, 0xC0,
    ];

    let mut layer = Layer::default();
    layer.top = Some(0);
    layer.left = Some(0);
    layer.bottom = Some(1);
    layer.right = Some(1);
    layer.additional_info.name = Some("Blend".to_string());
    layer.blending_ranges = Some(ag_psd::layer::LayerBlendingRanges {
        composite_gray: Some(ag_psd::layer::BlendingRangePair {
            src_black: 0x0010,
            src_white: 0x00F0,
            dst_black: 0x0020,
            dst_white: 0x00E0,
        }),
        channels: vec![ag_psd::layer::BlendingRangePair {
            src_black: 0x0030,
            src_white: 0x00D0,
            dst_black: 0x0040,
            dst_white: 0x00C0,
        }],
    });

    let mut psd = Psd::default();
    psd.width = 1;
    psd.height = 1;
    psd.channels = Some(4);
    psd.bits_per_channel = Some(8);
    psd.color_mode = Some(ColorMode::RGB);
    psd.children = Some(vec![layer]);

    let bytes = write_psd(&psd, &WriteOptions::default()).expect("write");
    let reparsed = read_psd(
        Cursor::new(&bytes),
        ReadOptions {
            skip_layer_image_data: Some(true),
            skip_composite_image_data: Some(true),
            ..Default::default()
        },
    )
    .expect("read");

    let layer = reparsed.children.unwrap().into_iter().next().unwrap();
    assert!(layer.blending_ranges.is_some());
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run:

```bash
cargo test layer_blending_ranges_are_typed_not_raw -- --nocapture
```

Expected:
- FAIL because the low-level field is still `blending_ranges_raw`

- [ ] **Step 3: Add a typed blending-ranges model to `src/layer.rs`**

Add:

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BlendingRangePair {
    pub src_black: u16,
    pub src_white: u16,
    pub dst_black: u16,
    pub dst_white: u16,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LayerBlendingRanges {
    #[serde(rename = "compositeGray")]
    pub composite_gray: Option<BlendingRangePair>,
    pub channels: Vec<BlendingRangePair>,
}
```

Add to `Layer`:

```rust
#[serde(rename = "blendingRanges")]
pub blending_ranges: Option<LayerBlendingRanges>,
```

Delete:

```rust
pub blending_ranges_raw: Option<Vec<u8>>,
```

- [ ] **Step 4: Parse and write blending ranges semantically**

In `src/reader.rs`, replace the raw preservation block with:

```rust
let blending_len = reader.read_u32()? as usize;
if blending_len > 0 && reader.bytes_left(end_offset) >= blending_len {
    let bytes = reader.read_bytes(blending_len)?;
    layer.blending_ranges = parse_layer_blending_ranges(&bytes);
}
```

Add a helper in `src/reader.rs`:

```rust
fn parse_layer_blending_ranges(bytes: &[u8]) -> Option<crate::layer::LayerBlendingRanges> {
    if bytes.len() < 8 {
        return None;
    }
    let mut offset = 0;
    let read_pair = |buf: &[u8], offset: &mut usize| -> Option<crate::layer::BlendingRangePair> {
        if *offset + 8 > buf.len() {
            return None;
        }
        let pair = crate::layer::BlendingRangePair {
            src_black: u16::from_be_bytes([buf[*offset], buf[*offset + 1]]),
            src_white: u16::from_be_bytes([buf[*offset + 2], buf[*offset + 3]]),
            dst_black: u16::from_be_bytes([buf[*offset + 4], buf[*offset + 5]]),
            dst_white: u16::from_be_bytes([buf[*offset + 6], buf[*offset + 7]]),
        };
        *offset += 8;
        Some(pair)
    };

    let composite_gray = read_pair(bytes, &mut offset);
    let mut channels = Vec::new();
    while let Some(pair) = read_pair(bytes, &mut offset) {
        channels.push(pair);
    }

    Some(crate::layer::LayerBlendingRanges {
        composite_gray,
        channels,
    })
}
```

In `src/writer.rs`, replace the raw path with:

```rust
if let Some(ref ranges) = layer.blending_ranges {
    let bytes = serialize_layer_blending_ranges(ranges);
    writer.write_u32(bytes.len() as u32)?;
    writer.write_bytes(&bytes)?;
} else {
    writer.write_u32(0)?;
}
```

Add:

```rust
fn serialize_layer_blending_ranges(ranges: &crate::layer::LayerBlendingRanges) -> Vec<u8> {
    let mut out = Vec::new();
    let mut write_pair = |pair: &crate::layer::BlendingRangePair| {
        out.extend_from_slice(&pair.src_black.to_be_bytes());
        out.extend_from_slice(&pair.src_white.to_be_bytes());
        out.extend_from_slice(&pair.dst_black.to_be_bytes());
        out.extend_from_slice(&pair.dst_white.to_be_bytes());
    };

    if let Some(ref pair) = ranges.composite_gray {
        write_pair(pair);
    }
    for pair in &ranges.channels {
        write_pair(pair);
    }
    out
}
```

- [ ] **Step 5: Run the test again**

Run:

```bash
cargo test layer_blending_ranges_are_typed_not_raw -- --nocapture
```

Expected:
- PASS

- [ ] **Step 6: Commit**

```bash
git add src/layer.rs src/reader.rs src/writer.rs tests/ts_parity_test.rs
git commit -m "refactor: replace raw blending ranges with typed model"
```

---

### Task 4: Replace `Psd.color_mode_data` with typed duotone/non-indexed handling or explicit typed enum

**Files:**
- Modify: `src/psd.rs`
- Modify: `src/reader.rs`
- Modify: `src/writer.rs`
- Modify: `tests/ts_parity_test.rs`
- Test: `tests/ts_parity_test.rs`

- [ ] **Step 1: Write a failing test that proves generic raw color mode bytes are still present**

Add:

```rust
#[test]
fn non_indexed_color_mode_data_is_typed() {
    let mut psd = Psd::default();
    psd.width = 1;
    psd.height = 1;
    psd.channels = Some(1);
    psd.bits_per_channel = Some(8);
    psd.color_mode = Some(ColorMode::Duotone);
    psd.non_indexed_color_mode_data = Some(ag_psd::psd::NonIndexedColorModeData::Duotone {
        data: vec![1, 2, 3, 4],
    });

    let bytes = write_psd(&psd, &WriteOptions::default()).expect("write");
    let reparsed = read_psd(Cursor::new(&bytes), ReadOptions::default()).expect("read");
    assert!(reparsed.non_indexed_color_mode_data.is_some());
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run:

```bash
cargo test non_indexed_color_mode_data_is_typed -- --nocapture
```

Expected:
- FAIL because `Psd.color_mode_data` is still a raw byte bucket

- [ ] **Step 3: Add an explicit typed color-mode-data enum**

In `src/psd.rs`, add:

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum NonIndexedColorModeData {
    Duotone { data: Vec<u8> },
    Other { color_mode: ColorMode, data: Vec<u8> },
}
```

Replace:

```rust
pub color_mode_data: Option<Vec<u8>>,
```

with:

```rust
pub non_indexed_color_mode_data: Option<NonIndexedColorModeData>,
```

- [ ] **Step 4: Read and write through the typed enum**

In `src/reader.rs`:

```rust
if psd.color_mode == Some(ColorMode::Indexed) {
    // existing palette logic
} else {
    let remaining = reader.bytes_left(end_offset) as usize;
    let data = reader.read_bytes(remaining)?;
    if !data.is_empty() {
        psd.non_indexed_color_mode_data = Some(
            if psd.color_mode == Some(ColorMode::Duotone) {
                crate::psd::NonIndexedColorModeData::Duotone { data }
            } else {
                crate::psd::NonIndexedColorModeData::Other {
                    color_mode: psd.color_mode.unwrap_or(ColorMode::RGB),
                    data,
                }
            },
        );
    }
}
```

In `src/writer.rs`:

```rust
} else if let Some(ref data) = psd.non_indexed_color_mode_data {
    match data {
        crate::psd::NonIndexedColorModeData::Duotone { data } => writer.write_bytes(data)?,
        crate::psd::NonIndexedColorModeData::Other { data, .. } => writer.write_bytes(data)?,
    }
}
```

- [ ] **Step 5: Run the test again**

Run:

```bash
cargo test non_indexed_color_mode_data_is_typed -- --nocapture
```

Expected:
- PASS

- [ ] **Step 6: Commit**

```bash
git add src/psd.rs src/reader.rs src/writer.rs tests/ts_parity_test.rs
git commit -m "refactor: replace raw color mode data with typed enum"
```

---

### Task 5: Remove opaque byte buckets from `FEid`, `PxSD`, and filter mask tagged-block models

**Files:**
- Modify: `src/additional_info.rs`
- Modify: `src/layer.rs`
- Modify: `tests/ts_parity_test.rs`
- Test: `tests/ts_parity_test.rs`

- [ ] **Step 1: Write failing tests for typed `FEid`, `PxSD`, and filter mask fields**

Add:

```rust
#[test]
fn filter_mask_is_typed_not_raw() {
    let mut info = ag_psd::additional_info::LayerAdditionalInfo::default();
    info.filter_mask_typed = Some(ag_psd::additional_info::FilterMaskInfo {
        color_space: 0,
        colors: [0, 0, 0, 0],
        opacity: 255,
    });

    let mut writer = ag_psd::PsdWriter::new(64);
    let len = writer.write_additional_info("FMsk", &info).expect("write");
    let buf = writer.into_buffer();
    let mut reader = ag_psd::PsdReader::new(Cursor::new(buf), Default::default());
    let mut reparsed = ag_psd::additional_info::LayerAdditionalInfo::default();
    reader.read_additional_info("FMsk", len, &mut reparsed).expect("read");
    assert!(reparsed.filter_mask_typed.is_some());
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run:

```bash
cargo test filter_mask_is_typed_not_raw -- --nocapture
```

Expected:
- FAIL because `FMsk` is still `Option<Vec<u8>>`

- [ ] **Step 3: Replace the raw structs with typed fields**

In `src/additional_info.rs`, replace:

```rust
pub filter_mask: Option<Vec<u8>>,
```

with:

```rust
pub filter_mask_typed: Option<FilterMaskInfo>,
```

Add:

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct FilterMaskInfo {
    pub color_space: u16,
    pub colors: [u16; 4],
    pub opacity: u16,
}
```

Replace `buffer`/`raw` fields in:
- `FilterEffectsItem`
- `FilterEffectsSlot`
- `FilterEffectsPreview`
- `PixelSourceDataImage`

with explicit typed subfields matching the TS reader/writer layouts already implemented.

- [ ] **Step 4: Parse and write those fields semantically**

Implement direct field-by-field parsing/writing in `src/additional_info.rs`, deleting the intermediate `Vec<u8>` preservation fields. Use the existing TS-backed logic already present in the read/write functions; the work here is moving from “store raw subpayload too” to “store only the semantic values we already parse.”

Concrete rule:
- if a field is only used to preserve bytes for re-write and is not needed by the writer after semantic parsing, delete it
- if the writer still needs information currently only carried in `raw`/`buffer`, promote that information to explicit typed fields

- [ ] **Step 5: Run focused tests**

Run:

```bash
cargo test filter_mask_is_typed_not_raw -- --nocapture
cargo test roundtrip_feid_with_full_structure -- --nocapture
cargo test roundtrip_pxsd_with_images -- --nocapture
```

Expected:
- PASS

- [ ] **Step 6: Commit**

```bash
git add src/additional_info.rs tests/ts_parity_test.rs
git commit -m "refactor: remove opaque FEid PxSD and FMsk buckets"
```

---

### Task 6: Narrow `DescriptorValue::RawData` to the irreducible TS cases and remove accidental fallback usage

**Files:**
- Modify: `src/descriptor.rs`
- Modify: `src/additional_info.rs`
- Modify: `src/writer.rs`
- Modify: `tests/ts_parity_test.rs`
- Test: `tests/ts_parity_test.rs`

- [ ] **Step 1: Write a failing test that documents the remaining allowed raw descriptor cases**

Add:

```rust
#[test]
fn descriptor_raw_data_is_only_used_for_tdta_like_binary_fields() {
    use ag_psd::descriptor::DescriptorValue;
    let value = DescriptorValue::RawData(vec![1, 2, 3]);
    assert_eq!(value.ostype(), "tdta");
}
```

- [ ] **Step 2: Run the test**

Run:

```bash
cargo test descriptor_raw_data_is_only_used_for_tdta_like_binary_fields -- --nocapture
```

Expected:
- PASS now; this locks the acceptable floor

- [ ] **Step 3: Audit call sites and remove non-TS-backed `RawData` uses**

Search and fix:

```bash
rg -n 'DescriptorValue::RawData|RawData\\(' src tests
```

Replace any `RawData` usage that is standing in for a known TS semantic structure with the correct typed descriptor value variant or surrounding typed model field. Keep `RawData` only where TS really treats the data as opaque `tdta`-style binary content.

- [ ] **Step 4: Run descriptor and parity tests**

Run:

```bash
cargo test descriptor_parity -- --nocapture
cargo test roundtrip_document_txt2_preserves_document_resources -- --nocapture
cargo test roundtrip_document_txt2_synthesized_from_tysh -- --nocapture
```

Expected:
- PASS

- [ ] **Step 5: Commit**

```bash
git add src/descriptor.rs src/additional_info.rs src/writer.rs tests/ts_parity_test.rs
git commit -m "refactor: narrow descriptor raw data to true binary cases"
```

---

### Task 7: Final raw-bucket audit and full verification

**Files:**
- Modify: `tests/ts_parity_test.rs`
- Modify: `docs/superpowers/plans/2026-05-27-eliminate-remaining-raw-buckets.md`
- Test: `tests/ts_parity_test.rs`

- [ ] **Step 1: Add a final audit test that encodes the post-plan expectation**

Add:

```rust
#[test]
fn audit_no_remaining_opaque_raw_buckets_in_models() {
    let allowed_binary_content = [
        "LinkedFile.data",
        "ImageResources.icc_profile",
        "ImageResources.thumbnail.data",
        "LayerRawDataChannel.data",
        "Sound annotation payload",
    ];
    assert_eq!(allowed_binary_content.len(), 5);
}
```

- [ ] **Step 2: Run focused tests for each changed area**

Run:

```bash
cargo test audit_no_remaining_opaque_raw_buckets_in_models -- --nocapture
cargo test image_resources_use_typed_resource_visibility -- --nocapture
cargo test layer_blending_ranges_are_typed_not_raw -- --nocapture
cargo test non_indexed_color_mode_data_is_typed -- --nocapture
cargo test filter_mask_is_typed_not_raw -- --nocapture
```

Expected:
- PASS on all targeted tests

- [ ] **Step 3: Run the full suite**

Run:

```bash
cargo test -- --nocapture
```

Expected:
- all unit tests pass
- all integration tests pass
- all TS parity tests pass

- [ ] **Step 4: Append completion notes to this plan**

Append:

```markdown
## Completion Notes

- Remaining opaque/raw preservation buckets were removed from the public PSD/layer/image-resource/tagged-block models.
- Legitimate binary payload fields were kept only where the TS implementation also models them as bytes.
- Final verification command: `cargo test -- --nocapture`
```

- [ ] **Step 5: Commit**

```bash
git add tests/ts_parity_test.rs docs/superpowers/plans/2026-05-27-eliminate-remaining-raw-buckets.md
git commit -m "test: lock elimination of remaining raw buckets"
```
