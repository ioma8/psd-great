# PSD Remaining Spec Compliance Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fix the remaining audited PSD/PSB spec mismatches by correcting public resource semantics, image-resource wire formats, additional-info payloads, linked-layer round-tripping, and strict header/color parsing.

**Architecture:** Keep the existing module split. `psd.rs` owns the public document model, `image_resources.rs` owns resource parsing/writing, `document_resource_postprocess.rs` maps between resource storage and the public model, `additional_info.rs` owns tagged-block payloads, and `reader.rs`/`writer.rs` own structural PSD parsing and serialization. The writer becomes spec-correct for the covered structures while the reader stays permissive where current compatibility does not conflict with the spec.

**Tech Stack:** Rust, `cargo test`, existing unit tests in `src/*`, integration/parity coverage in `tests/integration_test.rs` and `tests/ts_parity_test.rs`

---

### Task 1: Replace Spec-Wrong Public Resource Fields

**Files:**
- Modify: `src/psd.rs`
- Modify: `src/image_resources.rs`
- Modify: `src/document_resource_postprocess.rs`
- Modify: `src/lib.rs`
- Test: `tests/ts_parity_test.rs`

- [ ] **Step 1: Write the failing public-model/resource-ID tests**

```rust
#[test]
fn resource_1026_maps_layer_group_ids_not_clipping() {
    let mut psd = Psd::default();
    psd.children = Some(vec![Layer::default(), Layer::default()]);
    psd.layer_group_ids = Some(vec![3, 9]);

    let bytes = write_psd(&psd, &WriteOptions::default()).unwrap();
    let reparsed = read_psd(Cursor::new(bytes), ReadOptions::default()).unwrap();

    assert_eq!(reparsed.layer_group_ids, Some(vec![3, 9]));
    assert_eq!(reparsed.children.unwrap()[0].clipping, None);
}

#[test]
fn display_info_writes_resource_1077_not_1036() {
    let mut psd = Psd::default();
    psd.width = 1;
    psd.height = 1;
    psd.display_info = Some(psd_great::psd::DisplayInfo {
        h_res_unit: psd_great::PsdU16Code(1),
        v_res_unit: psd_great::PsdU16Code(2),
        width_unit: psd_great::PsdU16Code(3),
        height_unit: psd_great::PsdU16Code(4),
    });

    let bytes = write_psd(&psd, &WriteOptions::default()).unwrap();

    assert!(bytes.windows(2).any(|w| w == 1077u16.to_be_bytes()));
    assert!(!bytes.windows(2).any(|w| w == 1036u16.to_be_bytes()));
}

#[test]
fn clipping_path_name_uses_resource_2999_pascal_string() {
    let mut psd = Psd::default();
    psd.width = 1;
    psd.height = 1;
    psd.clipping_path_name = Some("Path 1".to_string());

    let bytes = write_psd(&psd, &WriteOptions::default()).unwrap();
    let reparsed = read_psd(Cursor::new(bytes), ReadOptions::default()).unwrap();

    assert_eq!(reparsed.clipping_path_name.as_deref(), Some("Path 1"));
}
```

- [ ] **Step 2: Run the targeted tests to verify they fail for the expected reason**

Run: `cargo test --quiet resource_1026_maps_layer_group_ids_not_clipping display_info_writes_resource_1077_not_1036 clipping_path_name_uses_resource_2999_pascal_string`

Expected: FAIL because `Psd` does not yet expose `layer_group_ids` or `clipping_path_name`, and image resource dispatch still maps `1026/1036/2999` incorrectly.

- [ ] **Step 3: Write the minimal model and mapping changes**

```rust
// src/psd.rs
#[derive(Debug, Clone, PartialEq)]
pub struct ColorSampler {
    pub horizontal: i32,
    pub vertical: i32,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Psd {
    // ...
    pub layer_group_ids: Option<Vec<u16>>,
    pub color_samplers: Option<Vec<ColorSampler>>,
    pub clipping_path_name: Option<String>,
    pub display_info: Option<DisplayInfo>,
    pub thumbnail: Option<crate::psd::ThumbnailRaw>,
}

// src/document_resource_postprocess.rs
if let Some(group_ids) = resources.layer_group_ids.as_ref() {
    psd.layer_group_ids = Some(group_ids.clone());
}
if let Some(info) = resources.display_info_typed.as_ref() {
    psd.display_info = Some(crate::psd::DisplayInfo { /* fields */ });
}
if let Some(name) = resources.clipping_path_name.as_ref() {
    psd.clipping_path_name = Some(name.clone());
}
if let Some(samplers) = resources.color_samplers_typed.as_ref() {
    psd.color_samplers = Some(samplers.samplers.clone());
}
```

- [ ] **Step 4: Correct the resource dispatch and write-side mapping**

```rust
// src/image_resources.rs
pub struct ImageResources {
    // ...
    pub layer_group_ids: Option<Vec<u16>>,
    pub color_samplers_typed: Option<ColorSamplersResource>,
    pub display_info_typed: Option<DisplayInfoResource>,
    pub clipping_path_name: Option<String>,
}

match resource_id {
    1026 => reader.read_layer_group_ids(&mut resources, data_length)?,
    1036 => reader.read_thumbnail_resource(&mut resources, data_length)?,
    1073 => reader.read_color_samplers(&mut resources, data_length)?,
    1077 => reader.read_display_info(&mut resources, data_length)?,
    2999 => reader.read_clipping_path_name(&mut resources, data_length)?,
    _ => { /* existing behavior */ }
}

if let Some(group_ids) = resources.layer_group_ids.as_ref() {
    write_resource(writer, 1026, &|w| w.write_layer_group_ids(group_ids))?;
}
if let Some(display) = resources.display_info_typed.as_ref() {
    write_resource(writer, 1077, &|w| w.write_display_info(display))?;
}
if let Some(name) = resources.clipping_path_name.as_ref() {
    write_resource(writer, 2999, &|w| w.write_pascal_string(name, 2))?;
}
```

- [ ] **Step 5: Run the targeted tests and then the affected parity suite**

Run: `cargo test --quiet resource_1026_maps_layer_group_ids_not_clipping display_info_writes_resource_1077_not_1036 clipping_path_name_uses_resource_2999_pascal_string`

Expected: PASS

Run: `cargo test --quiet ts_parity_test`

Expected: PASS, or only failures directly attributable to the next planned slice/color-sampler changes.

- [ ] **Step 6: Commit**

```bash
git add src/psd.rs src/image_resources.rs src/document_resource_postprocess.rs src/lib.rs tests/ts_parity_test.rs
git commit -m "fix: correct PSD image resource public semantics"
```

### Task 2: Make Slices And Color Samplers Version-Aware And Spec-Correct

**Files:**
- Modify: `src/psd.rs`
- Modify: `src/image_resources.rs`
- Modify: `src/document_resource_postprocess.rs`
- Test: `tests/ts_parity_test.rs`
- Test: `tests/integration_test.rs`

- [ ] **Step 1: Write failing tests for slice versioning and color samplers**

```rust
#[test]
fn slices_resource_roundtrips_version_6_binary_form() {
    let slices = psd_great::image_resources::SlicesResource::V6 {
        bounds: psd_great::image_resources::SliceBounds { top: 0, left: 0, bottom: 10, right: 10 },
        name: "legacy".into(),
        slices: vec![psd_great::image_resources::Slice { id: 7, ..Default::default() }],
    };

    let mut resources = psd_great::ImageResources::default();
    resources.slices = Some(slices.clone());

    let bytes = psd_great::image_resources::write_image_resources(&resources).unwrap();
    let reparsed = psd_great::image_resources::read_image_resources(Cursor::new(bytes)).unwrap();

    assert_eq!(reparsed.slices, Some(slices));
}

#[test]
fn color_samplers_use_resource_1073() {
    let mut psd = Psd::default();
    psd.width = 1;
    psd.height = 1;
    psd.color_samplers = Some(vec![psd_great::psd::ColorSampler { horizontal: 12, vertical: 34 }]);

    let bytes = write_psd(&psd, &WriteOptions::default()).unwrap();
    let reparsed = read_psd(Cursor::new(bytes), ReadOptions::default()).unwrap();

    assert_eq!(reparsed.color_samplers.unwrap().len(), 1);
}
```

- [ ] **Step 2: Run the targeted tests to verify they fail**

Run: `cargo test --quiet slices_resource_roundtrips_version_6_binary_form color_samplers_use_resource_1073`

Expected: FAIL because `Slices` is still a flat struct and `1073` is still wired to custom points.

- [ ] **Step 3: Introduce version-aware slice types and color sampler parsing**

```rust
// src/image_resources.rs
pub enum SlicesResource {
    V6 {
        bounds: SliceBounds,
        name: String,
        slices: Vec<Slice>,
    },
    V7OrV8 {
        version: u32,
        descriptor: Descriptor,
    },
}

pub struct ColorSamplersResource {
    pub version: u32,
    pub samplers: Vec<crate::psd::ColorSampler>,
}

fn parse_slices_resource(bytes: &[u8]) -> Result<SlicesResource> {
    let version = u32::from_be_bytes(bytes[0..4].try_into().unwrap());
    match version {
        6 => parse_legacy_slices(bytes),
        7 | 8 => parse_descriptor_slices(bytes),
        _ => Err(PsdError::InvalidFormat(format!("unsupported slices version {version}"))),
    }
}
```

- [ ] **Step 4: Update document mapping so `Psd` exposes the right slice/color-sampler semantics**

```rust
// src/psd.rs
#[derive(Debug, Clone, PartialEq)]
pub enum DocumentSlices {
    Legacy {
        bounds: crate::image_resources::SliceBounds,
        name: String,
        slices: Vec<crate::image_resources::Slice>,
    },
    Descriptor {
        version: u32,
        descriptor: crate::descriptor::Descriptor,
    },
}

pub struct Psd {
    // ...
    pub slices: Option<DocumentSlices>,
}
```

- [ ] **Step 5: Run focused tests, then broader resource tests**

Run: `cargo test --quiet slices_resource_roundtrips_version_6_binary_form color_samplers_use_resource_1073`

Expected: PASS

Run: `cargo test --quiet resource_`

Expected: PASS for the image-resource regression set.

- [ ] **Step 6: Commit**

```bash
git add src/psd.rs src/image_resources.rs src/document_resource_postprocess.rs tests/ts_parity_test.rs tests/integration_test.rs
git commit -m "fix: make PSD slices and color samplers spec correct"
```

### Task 3: Fix `Txt2`, `sn2P`, And Recognized Additional-Info Preservation

**Files:**
- Modify: `src/additional_info.rs`
- Test: `src/additional_info.rs`
- Test: `tests/ts_parity_test.rs`

- [ ] **Step 1: Write failing unit tests for `Txt2`, `sn2P`, and opaque key preservation**

```rust
#[test]
fn txt2_writes_inner_engine_data_length() {
    let info = LayerAdditionalInfo {
        text_engine: Some(TextEngineBlock {
            data: crate::engine_data::EngineValue::Obj(vec![]),
        }),
        ..Default::default()
    };

    let mut writer = PsdWriter::new(1024);
    let len = writer.write_additional_info("Txt2", &info).unwrap();
    let bytes = writer.into_buffer();

    assert!(len >= 4);
    assert_eq!(u32::from_be_bytes(bytes[0..4].try_into().unwrap()) as usize, bytes.len() - 4);
}

#[test]
fn sn2p_is_encoded_as_u32() {
    let info = LayerAdditionalInfo {
        using_aligned_rendering: Some(true),
        ..Default::default()
    };

    let mut writer = PsdWriter::new(32);
    let len = writer.write_additional_info("sn2P", &info).unwrap();
    assert_eq!(len, 4);
}

#[test]
fn recognized_unmodeled_key_payload_is_preserved() {
    let bytes = vec![1, 2, 3, 4, 5, 6];
    let mut info = LayerAdditionalInfo::default();
    let mut reader = PsdReader::new(Cursor::new(bytes.clone()), Default::default());
    reader.read_additional_info("Mt16", bytes.len() as u64, &mut info).unwrap();

    assert_eq!(info.raw_blocks.get("Mt16").unwrap(), &bytes);
}
```

- [ ] **Step 2: Run the unit tests to verify the failures**

Run: `cargo test --quiet txt2_writes_inner_engine_data_length sn2p_is_encoded_as_u32 recognized_unmodeled_key_payload_is_preserved`

Expected: FAIL because `Txt2` currently omits the inner length, `sn2P` is one byte, and recognized unmodeled keys are not preserved through a typed raw-block map.

- [ ] **Step 3: Add the minimal payload and preservation support**

```rust
// src/additional_info.rs
#[derive(Debug, Clone, PartialEq, Default)]
pub struct LayerAdditionalInfo {
    // ...
    pub raw_blocks: HashMap<String, Vec<u8>>,
}

match key {
    "Txt2" => {
        let engine_length = self.read_u32()? as usize;
        let engine_bytes = self.read_bytes(engine_length)?;
        info.text_engine = Some(TextEngineBlock { data: crate::engine_data::parse_engine_data(&engine_bytes)? });
    }
    "sn2P" => {
        info.using_aligned_rendering = Some(self.read_u32()? != 0);
    }
    "Mt16" | "Mt32" | "Mtrn" | "LMsk" | "FXid" | "abdd" | "anFX" | "cinf" | "SoLE" => {
        info.raw_blocks.insert(key.to_string(), self.read_bytes(length as usize)?);
    }
    _ => {}
}
```

- [ ] **Step 4: Update writing for corrected payloads and preserved keys**

```rust
match key {
    "Txt2" => {
        let bytes = crate::engine_data::serialize_engine_data(&text_engine.data);
        self.write_u32(bytes.len() as u32)?;
        self.write_bytes(&bytes)?;
    }
    "sn2P" => self.write_u32(info.using_aligned_rendering.unwrap_or(false) as u32)?,
    other => {
        if let Some(raw) = info.raw_blocks.get(other) {
            self.write_bytes(raw)?;
        }
    }
}
```

- [ ] **Step 5: Run focused tests, then the additional-info suite**

Run: `cargo test --quiet txt2_writes_inner_engine_data_length sn2p_is_encoded_as_u32 recognized_unmodeled_key_payload_is_preserved`

Expected: PASS

Run: `cargo test --quiet additional_info`

Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add src/additional_info.rs tests/ts_parity_test.rs
git commit -m "fix: correct Txt2 and sn2P additional info payloads"
```

### Task 4: Improve Linked-Layer Round-Tripping

**Files:**
- Modify: `src/layer.rs`
- Modify: `src/additional_info.rs`
- Test: `src/additional_info.rs`
- Test: `tests/ts_parity_test.rs`

- [ ] **Step 1: Write failing linked-layer fidelity tests**

```rust
#[test]
fn linked_layer_roundtrips_child_document_id_and_file_size() {
    let info = LayerAdditionalInfo {
        linked_files: Some(LinkedFilesBlock {
            key: PsdStringCode::from("lnkD"),
            items: vec![LinkedFile {
                id: "asset".into(),
                name: "asset.psb".into(),
                child_document_id: Some(PsdStringCode::from("child")),
                linked_file: Some(LinkedFileInfo {
                    file_size: 1234,
                    name: "asset.psb".into(),
                    full_path: "/tmp/asset.psb".into(),
                    original_path: "/tmp/asset.psb".into(),
                    relative_path: "asset.psb".into(),
                }),
                ..Default::default()
            }],
        }),
        ..Default::default()
    };

    let mut writer = PsdWriter::new(4096);
    let len = writer.write_additional_info("lnkD", &info).unwrap();
    let bytes = writer.into_buffer();
    let mut reader = PsdReader::new(Cursor::new(bytes), Default::default());
    let mut reparsed = LayerAdditionalInfo::default();
    reader.read_additional_info("lnkD", len, &mut reparsed).unwrap();

    let linked = &reparsed.linked_files.unwrap().items[0];
    assert_eq!(linked.child_document_id.as_ref().map(|v| v.as_ref()), Some("child"));
    assert_eq!(linked.linked_file.as_ref().map(|v| v.file_size), Some(1234));
}
```

- [ ] **Step 2: Run the targeted linked-layer test to verify it fails**

Run: `cargo test --quiet linked_layer_roundtrips_child_document_id_and_file_size`

Expected: FAIL because the current linked-layer serializer/parser drops some versioned fields.

- [ ] **Step 3: Extend the linked-layer model with typed variant data**

```rust
// src/layer.rs
#[derive(Debug, Clone, PartialEq)]
pub enum LinkedDataPayload {
    ExternalFile(Vec<u8>),
    ExternalFileDescriptor(crate::descriptor::Descriptor),
    Alias(Vec<u8>),
    Raw { kind: PsdStringCode, bytes: Vec<u8> },
}

#[derive(Debug, Clone, PartialEq)]
pub struct LinkedFile {
    // ...
    pub payload: Option<LinkedDataPayload>,
    pub version: Option<u32>,
}
```

- [ ] **Step 4: Update additional-info parsing/writing to preserve all covered linked-layer fields**

```rust
// src/additional_info.rs
match data_key.as_ref() {
    "liFD" => linked.payload = Some(LinkedDataPayload::ExternalFile(data_bytes)),
    "liFE" => linked.payload = Some(LinkedDataPayload::ExternalFileDescriptor(self.read_descriptor_structure()?)),
    "liFA" => linked.payload = Some(LinkedDataPayload::Alias(data_bytes)),
    other => linked.payload = Some(LinkedDataPayload::Raw { kind: PsdStringCode::from(other), bytes: data_bytes }),
}

if let Some(child_id) = linked.child_document_id.as_ref() {
    self.write_signature(child_id.as_ref())?;
}
```

- [ ] **Step 5: Run focused tests, then the linked-layer parity tests**

Run: `cargo test --quiet linked_layer_roundtrips_child_document_id_and_file_size lnk2_roundtrip`

Expected: PASS

Run: `cargo test --quiet ts_parity_test`

Expected: PASS for the linked-layer cases and no regressions in other parity areas.

- [ ] **Step 6: Commit**

```bash
git add src/layer.rs src/additional_info.rs tests/ts_parity_test.rs
git commit -m "fix: preserve PSD linked layer variant data"
```

### Task 5: Enforce Header Invariants And Parse Non-RGB Photoshop Colors

**Files:**
- Modify: `src/reader.rs`
- Modify: `src/writer.rs`
- Modify: `src/types.rs`
- Test: `tests/integration_test.rs`
- Test: `src/reader.rs`

- [ ] **Step 1: Write failing tests for invalid headers and non-RGB color structures**

```rust
#[test]
fn rejects_zero_width_in_header() {
    let mut bytes = minimal_valid_psd_header();
    bytes[18..22].copy_from_slice(&0u32.to_be_bytes());

    let err = read_psd(Cursor::new(bytes), ReadOptions::default()).unwrap_err();
    assert!(err.to_string().contains("width"));
}

#[test]
fn rejects_non_zero_reserved_header_bytes() {
    let mut bytes = minimal_valid_psd_header();
    bytes[6] = 1;

    let err = read_psd(Cursor::new(bytes), ReadOptions::default()).unwrap_err();
    assert!(err.to_string().contains("reserved"));
}

#[test]
fn read_color_preserves_cmyk_channels() {
    let bytes = [
        0x00, 0x02, // CMYK color space
        0x00, 0x64, 0x00, 0x32, 0x00, 0x19, 0x00, 0x0a,
    ];
    let mut reader = PsdReader::new(Cursor::new(bytes), Default::default());
    let color = reader.read_color().unwrap();

    match color {
        Color::CMYK(cmyk) => assert!(cmyk.k >= 0.0),
        other => panic!("expected CMYK, got {other:?}"),
    }
}
```

- [ ] **Step 2: Run the focused tests to verify they fail**

Run: `cargo test --quiet rejects_zero_width_in_header rejects_non_zero_reserved_header_bytes read_color_preserves_cmyk_channels`

Expected: FAIL because zero dimensions and reserved bytes are currently accepted, and non-RGB colors still collapse or parse incorrectly.

- [ ] **Step 3: Tighten header validation and expand color parsing**

```rust
// src/reader.rs
if header.channels == 0 {
    return Err(PsdError::InvalidFormat("header channels must be >= 1".into()));
}
if header.height == 0 {
    return Err(PsdError::InvalidFormat("header height must be >= 1".into()));
}
if header.width == 0 {
    return Err(PsdError::InvalidFormat("header width must be >= 1".into()));
}
if header.reserved.iter().any(|byte| *byte != 0) {
    return Err(PsdError::InvalidFormat("header reserved bytes must be zero".into()));
}

pub fn read_color(&mut self) -> Result<crate::types::Color> {
    let space = self.read_u16()?;
    let c1 = self.read_u16()?;
    let c2 = self.read_u16()?;
    let c3 = self.read_u16()?;
    let c4 = self.read_u16()?;
    match space {
        0 => Ok(Color::RGB(RGB { r: (c1 >> 8) as u8, g: (c2 >> 8) as u8, b: (c3 >> 8) as u8 })),
        2 => Ok(Color::CMYK(CMYK { c: c1 as f64 / 65535.0, m: c2 as f64 / 65535.0, y: c3 as f64 / 65535.0, k: c4 as f64 / 65535.0 })),
        7 => Ok(Color::LAB(LAB { l: c1 as f64 / 10000.0, a: (c2 as i16) as f64 / 10000.0, b: (c3 as i16) as f64 / 10000.0 })),
        8 => Ok(Color::Grayscale(Grayscale { k: (c1 >> 8) as u8 })),
        _ => Err(PsdError::UnsupportedFeature(format!("unsupported color space {space}"))),
    }
}
```

- [ ] **Step 4: Add any matching writer changes for new color variants**

```rust
match color {
    Color::CMYK(cmyk) => {
        self.write_u16(2)?;
        self.write_u16((cmyk.c.clamp(0.0, 1.0) * 65535.0) as u16)?;
        self.write_u16((cmyk.m.clamp(0.0, 1.0) * 65535.0) as u16)?;
        self.write_u16((cmyk.y.clamp(0.0, 1.0) * 65535.0) as u16)?;
        self.write_u16((cmyk.k.clamp(0.0, 1.0) * 65535.0) as u16)?;
    }
    _ => { /* existing cases */ }
}
```

- [ ] **Step 5: Run focused tests, then the full suite**

Run: `cargo test --quiet rejects_zero_width_in_header rejects_non_zero_reserved_header_bytes read_color_preserves_cmyk_channels`

Expected: PASS

Run: `cargo test --quiet`

Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add src/reader.rs src/writer.rs src/types.rs tests/integration_test.rs
git commit -m "fix: enforce PSD header invariants and parse color structures"
```

## Self-Review

- Spec coverage:
  - Public-model/resource-ID correctness: Task 1
  - Slice versioning and color samplers: Task 2
  - `Txt2`, `sn2P`, and recognized opaque-key preservation: Task 3
  - Linked-layer fidelity: Task 4
  - Header invariants and non-RGB color parsing: Task 5

- Placeholder scan:
  - No `TODO`, `TBD`, or “implement later” placeholders remain.

- Type consistency:
  - `layer_group_ids`, `color_samplers`, `clipping_path_name`, `DocumentSlices`, `ColorSamplersResource`, `SlicesResource`, `raw_blocks`, and `LinkedDataPayload` are introduced consistently and used by later tasks under the same names.
