# PSD Final Spec Compliance Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Close the final PSD/PSB spec-compliance gaps by correcting thumbnail format semantics, fully modeling color samplers and slices, improving linked-layer fidelity, and making Photoshop color-structure handling spec-correct.

**Architecture:** Expand the public model only where the remaining mismatches require it, then update the low-level parsers and writers to use those richer types. `psd.rs`, `types.rs`, and `layer.rs` define the new public surface; `image_resources.rs` and `additional_info.rs` perform the format-correct parsing and writing; `document_resource_postprocess.rs` bridges the richer low-level resources back into `Psd`; `reader.rs` and `writer.rs` complete the color-structure fixes.

**Tech Stack:** Rust, `cargo test`, existing unit tests in `src/*`, parity tests in `tests/ts_parity_test.rs`, integration coverage in `tests/integration_test.rs`

---

### Task 1: Expand The Public Model For Final Resource Fidelity

**Files:**
- Modify: `src/psd.rs`
- Modify: `src/types.rs`
- Modify: `src/layer.rs`
- Modify: `src/lib.rs`
- Test: `tests/ts_parity_test.rs`

- [ ] **Step 1: Write the failing public-type tests**

```rust
#[test]
fn thumbnail_format_matches_spec_values() {
    assert_eq!(psd_great::image_resources::ThumbnailFormat::JpegRgb as u32, 1);
    assert_eq!(psd_great::image_resources::ThumbnailFormat::RawRgb as u32, 0);
}

#[test]
fn color_sampler_carries_spec_fields() {
    let sampler = psd_great::psd::ColorSampler {
        version: 3,
        horizontal: 10,
        vertical: 20,
        color_space: 8,
        depth: None,
    };

    assert_eq!(sampler.version, 3);
    assert_eq!(sampler.color_space, 8);
}

#[test]
fn slices_document_model_is_version_aware() {
    let slices = psd_great::psd::DocumentSlices::Descriptor {
        version: 7,
        descriptor: psd_great::descriptor::Descriptor {
            name: String::new(),
            class_id: "null".to_string(),
            items: std::collections::HashMap::new(),
        },
    };

    match slices {
        psd_great::psd::DocumentSlices::Descriptor { version, .. } => assert_eq!(version, 7),
        _ => panic!("wrong slices variant"),
    }
}
```

- [ ] **Step 2: Run the focused tests to verify they fail**

Run: `cargo test --quiet thumbnail_format_matches_spec_values`

Expected: FAIL because the current thumbnail enum names and values are inverted, `ColorSampler` is still too small, and `Psd.slices` is still `Option<Vec<Slice>>`.

- [ ] **Step 3: Add the minimal public model changes**

```rust
// src/psd.rs
#[derive(Debug, Clone, PartialEq)]
pub struct ColorSampler {
    pub version: u32,
    pub horizontal: i32,
    pub vertical: i32,
    pub color_space: i16,
    pub depth: Option<u16>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DocumentSlices {
    Legacy(crate::image_resources::Slices),
    Descriptor {
        version: u32,
        descriptor: crate::descriptor::Descriptor,
    },
}

pub struct Psd {
    // ...
    pub slices: Option<DocumentSlices>,
}

// src/layer.rs
pub struct LinkedFile {
    pub item_version: Option<u32>,
    pub data_kind: Option<PsdStringCode>,
    pub open_descriptor: Option<crate::descriptor::Descriptor>,
    // existing fields remain
}
```

- [ ] **Step 4: Update exports and color value types**

```rust
// src/types.rs
pub struct CMYK {
    pub c: u16,
    pub m: u16,
    pub y: u16,
    pub k: u16,
}

pub struct Grayscale {
    pub k: u16,
}

pub enum Color {
    RGBA(RGBA),
    RGB(RGB),
    FRGB(FRGB),
    HSB(HSB),
    CMYK(CMYK),
    LAB(LAB),
    Grayscale(Grayscale),
    OpaqueColorSpace {
        color_space: u16,
        components: [u16; 4],
    },
}
```

- [ ] **Step 5: Run the focused type tests**

Run: `cargo test --quiet thumbnail_format_matches_spec_values`

Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add src/psd.rs src/types.rs src/layer.rs src/lib.rs tests/ts_parity_test.rs
git commit -m "fix: expand PSD model for final spec compliance"
```

### Task 2: Fix Thumbnail `1036` And Fully Model Color Samplers `1073`

**Files:**
- Modify: `src/image_resources.rs`
- Modify: `src/document_resource_postprocess.rs`
- Test: `src/image_resources.rs`
- Test: `tests/ts_parity_test.rs`

- [ ] **Step 1: Write failing resource tests**

```rust
#[test]
fn thumbnail_resource_uses_spec_format_codes() {
    let thumbnail = psd_great::image_resources::Thumbnail {
        width: 1,
        height: 1,
        format: psd_great::image_resources::ThumbnailFormat::JpegRgb,
        data: vec![0xFF, 0xD8, 0xFF],
    };

    let bytes = super::build_thumbnail_resource(&thumbnail);
    assert_eq!(u32::from_be_bytes(bytes[0..4].try_into().unwrap()), 1);
}

#[test]
fn color_sampler_roundtrips_color_space_and_depth() {
    let resource = psd_great::image_resources::ColorSamplersResource {
        version: 2,
        samplers: vec![psd_great::psd::ColorSampler {
            version: 2,
            horizontal: 100,
            vertical: 200,
            color_space: 8,
            depth: Some(16),
        }],
    };

    let bytes = super::build_color_samplers_resource(&resource);
    let reparsed = super::parse_color_samplers_resource(&bytes);
    assert_eq!(reparsed, resource);
}
```

- [ ] **Step 2: Run the focused resource tests to verify they fail**

Run: `cargo test --quiet thumbnail_resource_uses_spec_format_codes`

Expected: FAIL because the current format mapping is reversed and color samplers discard fields.

- [ ] **Step 3: Correct thumbnail mapping and expand color sampler parse/write**

```rust
// src/image_resources.rs
pub enum ThumbnailFormat {
    RawRgb = 0,
    JpegRgb = 1,
}

fn parse_thumbnail_resource(bytes: &[u8]) -> Option<Thumbnail> {
    let format = u32::from_be_bytes(bytes[0..4].try_into().ok()?);
    let format = match format {
        0 => ThumbnailFormat::RawRgb,
        1 => ThumbnailFormat::JpegRgb,
        _ => return None,
    };
    // ...
}

fn build_color_samplers_resource(samplers: &ColorSamplersResource) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&samplers.version.to_be_bytes());
    bytes.extend_from_slice(&(samplers.samplers.len() as u32).to_be_bytes());
    for sampler in &samplers.samplers {
        bytes.extend_from_slice(&sampler.horizontal.to_be_bytes());
        bytes.extend_from_slice(&sampler.vertical.to_be_bytes());
        bytes.extend_from_slice(&(sampler.color_space as u16).to_be_bytes());
        if let Some(depth) = sampler.depth {
            bytes.extend_from_slice(&depth.to_be_bytes());
        }
    }
    bytes
}
```

- [ ] **Step 4: Update document postprocess to use richer samplers**

```rust
// src/document_resource_postprocess.rs
if let Some(samplers) = resources.color_samplers_typed.as_ref() {
    psd.color_samplers = Some(samplers.samplers.clone());
}
```

- [ ] **Step 5: Run focused tests and resource suite**

Run: `cargo test --quiet thumbnail_resource_uses_spec_format_codes`

Expected: PASS

Run: `cargo test --quiet color_sampler_roundtrips_color_space_and_depth`

Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add src/image_resources.rs src/document_resource_postprocess.rs tests/ts_parity_test.rs
git commit -m "fix: make thumbnail and color samplers spec correct"
```

### Task 3: Make Slices `1050` Fully Version-Aware Through The Public Model

**Files:**
- Modify: `src/psd.rs`
- Modify: `src/image_resources.rs`
- Modify: `src/document_resource_postprocess.rs`
- Test: `src/image_resources.rs`
- Test: `tests/integration_test.rs`

- [ ] **Step 1: Write failing slices tests**

```rust
#[test]
fn slices_v7_roundtrip_descriptor_variant() {
    let descriptor = psd_great::descriptor::Descriptor {
        name: String::new(),
        class_id: "null".to_string(),
        items: std::collections::HashMap::new(),
    };
    let slices = psd_great::psd::DocumentSlices::Descriptor {
        version: 7,
        descriptor: descriptor.clone(),
    };

    let mut psd = Psd::default();
    psd.width = 1;
    psd.height = 1;
    psd.channels = Some(3);
    psd.bits_per_channel = Some(8);
    psd.color_mode = Some(ColorMode::RGB);
    psd.slices = Some(slices);

    let bytes = write_psd(&psd, &WriteOptions::default()).unwrap();
    let reparsed = read_psd(Cursor::new(bytes), ReadOptions::default()).unwrap();

    match reparsed.slices.unwrap() {
        psd_great::psd::DocumentSlices::Descriptor { version, descriptor: d } => {
            assert_eq!(version, 7);
            assert_eq!(d, descriptor);
        }
        other => panic!("unexpected slices variant: {other:?}"),
    }
}
```

- [ ] **Step 2: Run the targeted slices test to verify it fails**

Run: `cargo test --quiet slices_v7_roundtrip_descriptor_variant`

Expected: FAIL because `Psd.slices` still cannot represent descriptor-backed slices.

- [ ] **Step 3: Update mapping and low-level serialization**

```rust
// src/document_resource_postprocess.rs
if let Some(ref slices) = resources.slices {
    psd.slices = Some(match &slices.descriptor {
        Some(descriptor) => DocumentSlices::Descriptor {
            version: slices.version,
            descriptor: descriptor.clone(),
        },
        None => DocumentSlices::Legacy(slices.clone()),
    });
}

if let Some(ref slices) = psd.slices {
    resources.slices = Some(match slices {
        DocumentSlices::Legacy(s) => s.clone(),
        DocumentSlices::Descriptor { version, descriptor } => crate::image_resources::Slices {
            version: *version,
            bounds: None,
            group_name: None,
            slices: Vec::new(),
            descriptor: Some(descriptor.clone()),
        },
    });
}
```

- [ ] **Step 4: Add v6 per-slice descriptor handling**

```rust
// src/image_resources.rs
pub struct Slice {
    // ...
    pub descriptor: Option<Descriptor>,
}

if remaining_allows_descriptor {
    let descriptor_version = self.read_u32()?;
    if descriptor_version == 16 {
        slice.descriptor = Some(self.read_descriptor_structure()?);
    }
}
```

- [ ] **Step 5: Run focused tests, then image-resource tests**

Run: `cargo test --quiet slices_v7_roundtrip_descriptor_variant`

Expected: PASS

Run: `cargo test --quiet slices_roundtrip`

Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add src/psd.rs src/image_resources.rs src/document_resource_postprocess.rs tests/integration_test.rs
git commit -m "fix: preserve version-aware PSD slices"
```

### Task 4: Preserve Fuller Linked-Layer Metadata

**Files:**
- Modify: `src/layer.rs`
- Modify: `src/additional_info.rs`
- Test: `src/additional_info.rs`
- Test: `tests/ts_parity_test.rs`

- [ ] **Step 1: Write failing linked-layer tests**

```rust
#[test]
fn linked_layer_roundtrips_item_version_and_open_descriptor() {
    let desc = psd_great::descriptor::Descriptor {
        name: String::new(),
        class_id: "null".to_string(),
        items: std::collections::HashMap::new(),
    };
    let linked = psd_great::LinkedFile {
        id: "id".to_string(),
        name: "name".to_string(),
        item_version: Some(9),
        data_kind: Some(psd_great::PsdStringCode::from("liFE")),
        open_descriptor: Some(desc.clone()),
        data: Some(vec![1, 2, 3]),
        file_type: Some(psd_great::PsdStringCode::from("JPEG")),
        creator: Some(psd_great::PsdStringCode::from("8BIM")),
        time: None,
        descriptor: None,
        child_document_id: Some(psd_great::PsdStringCode::from("chid")),
        asset_mod_time: Some(12.5),
        asset_locked_state: Some(psd_great::PsdIntCode(1)),
        linked_file: None,
    };

    let mut info = LayerAdditionalInfo::default();
    info.linked_files = Some(psd_great::additional_info::LinkedFilesBlock {
        key: psd_great::PsdStringCode::from("lnkD"),
        items: vec![linked.clone()],
    });

    let mut writer = PsdWriter::new(2048);
    let len = writer.write_additional_info("lnkD", &info).unwrap();
    let bytes = writer.into_buffer();
    let mut reader = PsdReader::new(Cursor::new(bytes), Default::default());
    let mut reparsed = LayerAdditionalInfo::default();
    reader.read_additional_info("lnkD", len, &mut reparsed).unwrap();

    assert_eq!(reparsed.linked_files.unwrap().items[0], linked);
}
```

- [ ] **Step 2: Run the linked-layer test to verify it fails**

Run: `cargo test --quiet linked_layer_roundtrips_item_version_and_open_descriptor`

Expected: FAIL because item version, open descriptor, and metadata are still discarded or hardcoded.

- [ ] **Step 3: Parse and store the missing linked-layer fields**

```rust
// src/additional_info.rs
let item_version = self.read_u32()?;
let open = self.read_u8()?;
let open_descriptor = if open != 0 {
    let descriptor_version = self.read_u32()?;
    Some(self.read_descriptor_structure()?)
} else {
    None
};

items.push(LinkedFile {
    item_version: Some(item_version),
    data_kind: Some(PsdStringCode(kind)),
    open_descriptor,
    // parse child_document_id / asset_mod_time / asset_locked_state when present
    // keep payload and file metadata
});
```

- [ ] **Step 4: Write the missing linked-layer fields back out**

```rust
// src/additional_info.rs
item_writer.write_u32(item.item_version.unwrap_or(7))?;
if let Some(desc) = item.open_descriptor.as_ref() {
    item_writer.write_u8(1)?;
    item_writer.write_u32(16)?;
    item_writer.write_descriptor_structure(desc)?;
} else {
    item_writer.write_u8(0)?;
}
```

- [ ] **Step 5: Run focused tests and linked-layer parity tests**

Run: `cargo test --quiet linked_layer_roundtrips_item_version_and_open_descriptor`

Expected: PASS

Run: `cargo test --quiet lnk2_roundtrip`

Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add src/layer.rs src/additional_info.rs tests/ts_parity_test.rs
git commit -m "fix: preserve linked layer metadata"
```

### Task 5: Make Photoshop Color Structures Spec-Correct

**Files:**
- Modify: `src/types.rs`
- Modify: `src/reader.rs`
- Modify: `src/writer.rs`
- Test: `src/reader.rs`
- Test: `tests/integration_test.rs`

- [ ] **Step 1: Write failing color-structure tests**

```rust
#[test]
fn read_color_preserves_grayscale_0_to_10000() {
    let bytes = [
        0x00, 0x08, // grayscale
        0x27, 0x10, // 10000
        0x00, 0x00,
        0x00, 0x00,
        0x00, 0x00,
    ];
    let mut reader = PsdReader::new(Cursor::new(bytes), ReadOptions::default());
    let color = reader.read_color().unwrap();

    assert_eq!(
        color,
        crate::types::Color::Grayscale(crate::types::Grayscale { k: 10000 })
    );
}

#[test]
fn read_color_preserves_opaque_custom_space() {
    let bytes = [
        0x00, 0x03, // custom space
        0x00, 0x01,
        0x00, 0x02,
        0x00, 0x03,
        0x00, 0x04,
    ];
    let mut reader = PsdReader::new(Cursor::new(bytes), ReadOptions::default());
    let color = reader.read_color().unwrap();

    assert_eq!(
        color,
        crate::types::Color::OpaqueColorSpace {
            color_space: 3,
            components: [1, 2, 3, 4],
        }
    );
}
```

- [ ] **Step 2: Run the focused color tests to verify they fail**

Run: `cargo test --quiet read_color_preserves_grayscale_0_to_10000`

Expected: FAIL because grayscale is still compressed to byte scale and unsupported spaces collapse to black.

- [ ] **Step 3: Implement spec-correct color parsing**

```rust
// src/reader.rs
match color_space {
    0 => Ok(Color::RGB(RGB { /* 16-bit RGB -> u8 public RGB */ })),
    1 => Ok(Color::HSB(HSB { /* spec mapping */ })),
    2 => Ok(Color::CMYK(CMYK { c: c1, m: c2, y: c3, k: c4 })),
    7 => Ok(Color::LAB(LAB { l: c1 as f64 / 10000.0, a: (c2 as i16) as f64 / 100.0, b: (c3 as i16) as f64 / 100.0 })),
    8 => Ok(Color::Grayscale(Grayscale { k: c1 })),
    _ => Ok(Color::OpaqueColorSpace {
        color_space,
        components: [c1, c2, c3, c4],
    }),
}
```

- [ ] **Step 4: Implement spec-correct color writing**

```rust
// src/writer.rs
Some(Color::CMYK(c)) => {
    self.write_u16(2)?;
    self.write_u16(c.c)?;
    self.write_u16(c.m)?;
    self.write_u16(c.y)?;
    self.write_u16(c.k)?;
}
Some(Color::Grayscale(c)) => {
    self.write_u16(8)?;
    self.write_u16(c.k)?;
    self.write_zeros(6)?;
}
Some(Color::OpaqueColorSpace { color_space, components }) => {
    self.write_u16(*color_space)?;
    for component in components {
        self.write_u16(*component)?;
    }
}
```

- [ ] **Step 5: Run focused tests, then the full suite**

Run: `cargo test --quiet read_color_preserves_grayscale_0_to_10000`

Expected: PASS

Run: `cargo test --quiet`

Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add src/types.rs src/reader.rs src/writer.rs tests/integration_test.rs
git commit -m "fix: make Photoshop color structures spec correct"
```

## Self-Review

- Spec coverage:
  - Public model expansion: Task 1
  - Thumbnail and color samplers: Task 2
  - Version-aware slices: Task 3
  - Linked-layer fidelity: Task 4
  - Color structure semantics: Task 5

- Placeholder scan:
  - No `TODO`, `TBD`, or deferred placeholders remain in the plan steps.

- Type consistency:
  - `DocumentSlices`, `ColorSampler.version`, `ColorSampler.color_space`, `ColorSampler.depth`, `LinkedFile.item_version`, `LinkedFile.open_descriptor`, and `Color::OpaqueColorSpace` are named consistently across later tasks.
