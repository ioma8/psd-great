# PSD Lossless Final Compliance Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Finish the last PSD/PSB spec-compliance gaps by making Photoshop color structures lossless, modeling version-aware color sampler coordinates, fully serializing linked-layer metadata, and tightening legacy slice descriptor framing.

**Architecture:** Keep the existing typed-first reader/writer architecture, but replace the remaining lossy public shapes with spec-shaped data where needed. Execute in narrow TDD slices: public model and color serialization first, then `1073`, then linked-layer blocks, then `1050` v6 framing, followed by full verification and a fresh source/spec audit.

**Tech Stack:** Rust, `cargo test`, existing PSD reader/writer modules, Adobe PSD spec at `/tmp/psd-spec.md`

---

### Task 1: Make Photoshop Color Structures Lossless

**Files:**
- Modify: `src/types.rs`
- Modify: `src/reader.rs`
- Modify: `src/writer.rs`
- Modify: `src/lib.rs`
- Test: `tests/integration_test.rs`
- Test: `tests/ts_parity_test.rs`

- [ ] **Step 1: Write the failing public-model and round-trip tests**

Add tests that prove raw PSD color-structure values survive read/write exactly instead of being normalized or truncated:

```rust
#[test]
fn read_color_preserves_raw_rgb_hsb_lab_values() {
    use psd_great::{Color, PsdReader, ReadOptions};

    let rgb_bytes = vec![
        0x00, 0x00,
        0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0x00, 0x00,
    ];
    let mut rgb_reader = PsdReader::new(std::io::Cursor::new(rgb_bytes), ReadOptions::default());
    assert_eq!(
        rgb_reader.read_color().unwrap(),
        Color::Rgb48 {
            red: 0x1234,
            green: 0x5678,
            blue: 0x9abc,
        }
    );

    let hsb_bytes = vec![
        0x00, 0x01,
        0x11, 0x11, 0x22, 0x22, 0x33, 0x33, 0x00, 0x00,
    ];
    let mut hsb_reader = PsdReader::new(std::io::Cursor::new(hsb_bytes), ReadOptions::default());
    assert_eq!(
        hsb_reader.read_color().unwrap(),
        Color::Hsb {
            hue: 0x1111,
            saturation: 0x2222,
            brightness: 0x3333,
        }
    );

    let lab_bytes = vec![
        0x00, 0x07,
        0x27, 0x10, 0xff, 0x9c, 0x00, 0x64, 0x00, 0x00,
    ];
    let mut lab_reader = PsdReader::new(std::io::Cursor::new(lab_bytes), ReadOptions::default());
    assert_eq!(
        lab_reader.read_color().unwrap(),
        Color::Lab {
            lightness: 10000,
            a: -100,
            b: 100,
        }
    );
}

#[test]
fn write_color_roundtrips_raw_color_structures_exactly() {
    use psd_great::{Color, PsdReader, PsdWriter, ReadOptions};

    let colors = [
        Color::Rgb48 {
            red: 0x1234,
            green: 0x5678,
            blue: 0x9abc,
        },
        Color::Hsb {
            hue: 0x1111,
            saturation: 0x2222,
            brightness: 0x3333,
        },
        Color::Lab {
            lightness: 10000,
            a: -100,
            b: 100,
        },
        Color::OpaqueColorSpace {
            color_space: 42,
            components: [1, 2, 3, 4],
        },
    ];

    for color in colors {
        let mut writer = PsdWriter::new(32);
        writer.write_color(Some(&color)).unwrap();
        let bytes = writer.into_buffer();
        let mut reader = PsdReader::new(std::io::Cursor::new(bytes), ReadOptions::default());
        assert_eq!(reader.read_color().unwrap(), color);
    }
}
```

- [ ] **Step 2: Run the targeted tests to verify they fail**

Run: `cargo test --quiet read_color_preserves_raw_rgb_hsb_lab_values write_color_roundtrips_raw_color_structures_exactly`

Expected: FAIL because `Color::RGB`, `Color::HSB`, and `Color::LAB` still use lossy public shapes and `read_color()` / `write_color()` still normalize or truncate.

- [ ] **Step 3: Replace the lossy PSD color variants with raw spec-shaped variants**

Update `src/types.rs` and `src/lib.rs` so PSD color structures use exact raw values:

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Color {
    RGBA(RGBA),
    RGB(RGB),
    FRGB(FRGB),
    Rgb48 {
        red: u16,
        green: u16,
        blue: u16,
    },
    Hsb {
        hue: u16,
        saturation: u16,
        brightness: u16,
    },
    CMYK(CMYK),
    Lab {
        lightness: u16,
        a: i16,
        b: i16,
    },
    Grayscale(Grayscale),
    OpaqueColorSpace {
        color_space: u16,
        components: [u16; 4],
    },
}
```

Export the new variants from `src/lib.rs` and remove any now-dead PSD-color-specific normalized helpers that only served `read_color()` / `write_color()`.

- [ ] **Step 4: Implement exact color parsing and serialization**

Update `src/reader.rs` and `src/writer.rs` so PSD color structures are read and written exactly:

```rust
match color_space {
    0 => Ok(Color::Rgb48 {
        red: c1,
        green: c2,
        blue: c3,
    }),
    1 => Ok(Color::Hsb {
        hue: c1,
        saturation: c2,
        brightness: c3,
    }),
    2 => Ok(Color::CMYK(CMYK {
        c: c1,
        m: c2,
        y: c3,
        k: c4,
    })),
    7 => Ok(Color::Lab {
        lightness: c1,
        a: i16::from_be_bytes(c2.to_be_bytes()),
        b: i16::from_be_bytes(c3.to_be_bytes()),
    }),
    8 => Ok(Color::Grayscale(Grayscale { k: c1 })),
    _ => Ok(Color::OpaqueColorSpace {
        color_space,
        components: [c1, c2, c3, c4],
    }),
}
```

```rust
Some(Color::Rgb48 { red, green, blue }) => {
    self.write_u16(0)?;
    self.write_u16(*red)?;
    self.write_u16(*green)?;
    self.write_u16(*blue)?;
    self.write_u16(0)?;
}
Some(Color::Hsb {
    hue,
    saturation,
    brightness,
}) => {
    self.write_u16(1)?;
    self.write_u16(*hue)?;
    self.write_u16(*saturation)?;
    self.write_u16(*brightness)?;
    self.write_u16(0)?;
}
Some(Color::Lab { lightness, a, b }) => {
    self.write_u16(7)?;
    self.write_u16(*lightness)?;
    self.write_i16(*a)?;
    self.write_i16(*b)?;
    self.write_u16(0)?;
}
```

- [ ] **Step 5: Run the targeted color tests and the affected suite**

Run: `cargo test --quiet read_color_preserves_raw_rgb_hsb_lab_values write_color_roundtrips_raw_color_structures_exactly`
Expected: PASS

Run: `cargo test --quiet integration_test ts_parity_test`
Expected: PASS after updating any parity expectations for the new raw color variants.

- [ ] **Step 6: Commit**

```bash
git add src/types.rs src/reader.rs src/writer.rs src/lib.rs tests/integration_test.rs tests/ts_parity_test.rs
git commit -m "fix: make PSD color structures lossless"
```

### Task 2: Make Color Samplers `1073` Version-Aware

**Files:**
- Modify: `src/psd.rs`
- Modify: `src/image_resources.rs`
- Modify: `src/document_resource_postprocess.rs`
- Test: `tests/integration_test.rs`
- Test: `tests/ts_parity_test.rs`

- [ ] **Step 1: Write the failing sampler tests**

Add tests that force the public model and resource serializer to preserve version-specific coordinates:

```rust
#[test]
fn color_sampler_resource_v1_roundtrips_versioned_position() {
    use psd_great::psd::{ColorSampler, ColorSamplerPosition};

    let sampler = ColorSampler {
        version: 1,
        position: ColorSamplerPosition::V1 {
            horizontal: 0x01020304,
            vertical: -0x0102030,
        },
        color_space: 7,
        depth: None,
    };

    let bytes = psd_great::image_resources::build_color_samplers_resource_for_test(1, &[sampler.clone()]);
    let parsed = psd_great::image_resources::parse_color_samplers_resource_for_test(&bytes);
    assert_eq!(parsed.samplers, vec![sampler]);
}

#[test]
fn color_sampler_resource_v2_roundtrips_depth_and_position() {
    use psd_great::psd::{ColorSampler, ColorSamplerPosition};

    let sampler = ColorSampler {
        version: 2,
        position: ColorSamplerPosition::V2 {
            horizontal: -200,
            vertical: 400,
        },
        color_space: 8,
        depth: Some(16),
    };

    let bytes = psd_great::image_resources::build_color_samplers_resource_for_test(2, &[sampler.clone()]);
    let parsed = psd_great::image_resources::parse_color_samplers_resource_for_test(&bytes);
    assert_eq!(parsed.samplers, vec![sampler]);
}
```

- [ ] **Step 2: Run the targeted sampler tests to verify they fail**

Run: `cargo test --quiet color_sampler_resource_v1_roundtrips_versioned_position color_sampler_resource_v2_roundtrips_depth_and_position`

Expected: FAIL because `ColorSampler` still exposes `horizontal` / `vertical` only and the resource helpers flatten all versions into plain `i32` pairs.

- [ ] **Step 3: Expand the public sampler model**

Update `src/psd.rs`:

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum ColorSamplerPosition {
    V1 { horizontal: i32, vertical: i32 },
    V2 { horizontal: i32, vertical: i32 },
}

#[derive(Debug, Clone, PartialEq)]
pub struct ColorSampler {
    pub version: u32,
    pub position: ColorSamplerPosition,
    pub color_space: i16,
    pub depth: Option<u16>,
}
```

Adjust document postprocess so `psd.color_samplers` keeps the version-aware shape without collapsing it.

- [ ] **Step 4: Make `1073` parse/write honor the versioned position type**

Update `src/image_resources.rs` so parser and writer match on the sampler version explicitly:

```rust
let position = match version {
    1 => ColorSamplerPosition::V1 { horizontal, vertical },
    2 => ColorSamplerPosition::V2 { horizontal, vertical },
    other => {
        return ColorSamplersResource {
            version: other,
            samplers,
        };
    }
};
```

```rust
match (&sampler.position, samplers.version) {
    (ColorSamplerPosition::V1 { horizontal, vertical }, 1) => {
        bytes.extend_from_slice(&horizontal.to_be_bytes());
        bytes.extend_from_slice(&vertical.to_be_bytes());
    }
    (ColorSamplerPosition::V2 { horizontal, vertical }, 2) => {
        bytes.extend_from_slice(&horizontal.to_be_bytes());
        bytes.extend_from_slice(&vertical.to_be_bytes());
    }
    _ => panic!("sampler version and position variant must match"),
}
```

Add `#[cfg(test)] pub(crate)` helper wrappers named `build_color_samplers_resource_for_test()` and `parse_color_samplers_resource_for_test()` so the tests can target the resource codec without widening the public production API.

- [ ] **Step 5: Run the targeted sampler tests and related suite**

Run: `cargo test --quiet color_sampler_resource_v1_roundtrips_versioned_position color_sampler_resource_v2_roundtrips_depth_and_position`
Expected: PASS

Run: `cargo test --quiet integration_test ts_parity_test`
Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add src/psd.rs src/image_resources.rs src/document_resource_postprocess.rs tests/integration_test.rs tests/ts_parity_test.rs
git commit -m "fix: preserve version-aware color samplers"
```

### Task 3: Finish Linked-Layer Metadata For `lnkD` / `lnk2` / `lnk3`

**Files:**
- Modify: `src/layer.rs`
- Modify: `src/additional_info.rs`
- Test: `tests/integration_test.rs`
- Test: `tests/ts_parity_test.rs`

- [ ] **Step 1: Write the failing linked-layer metadata test**

Add a targeted round-trip test that includes every still-missing field:

```rust
#[test]
fn linked_file_roundtrips_real_metadata_fields() {
    use psd_great::additional_info::LinkedFilesBlock;
    use psd_great::{LayerAdditionalInfo, LinkedFile, LinkedFileInfo, PsdIntCode, PsdStringCode};

    let linked = LinkedFile {
        id: "asset-id".into(),
        name: "Placed Asset".into(),
        item_version: Some(7),
        data_kind: Some(PsdStringCode::from("liFD")),
        file_type: Some(PsdStringCode::from("JPEG")),
        creator: Some(PsdStringCode::from("8BIM")),
        data: Some(vec![1, 2, 3, 4]),
        time: None,
        descriptor: None,
        child_document_id: Some(PsdStringCode::from("chd1")),
        asset_mod_time: Some(1234.5),
        asset_locked_state: Some(PsdIntCode(3)),
        linked_file: Some(LinkedFileInfo {
            file_size: 99,
            name: "asset.jpg".into(),
            full_path: "/tmp/asset.jpg".into(),
            original_path: "/orig/asset.jpg".into(),
            relative_path: "asset.jpg".into(),
        }),
        open_descriptor: Some(psd_great::descriptor::Descriptor::default()),
    };

    let mut info = LayerAdditionalInfo::default();
    info.linked_files = Some(LinkedFilesBlock {
        key: PsdStringCode::from("lnkD"),
        items: vec![linked.clone()],
    });

    let mut writer = psd_great::PsdWriter::new(1024);
    let len = writer.write_additional_info("lnkD", &info).unwrap();
    let mut reader = psd_great::PsdReader::new(std::io::Cursor::new(writer.into_buffer()), psd_great::ReadOptions::default());
    let mut reparsed = LayerAdditionalInfo::default();
    reader.read_additional_info("lnkD", len, &mut reparsed).unwrap();

    assert_eq!(reparsed.linked_files.unwrap().items[0], linked);
}
```

- [ ] **Step 2: Run the linked-layer test to verify it fails**

Run: `cargo test --quiet linked_file_roundtrips_real_metadata_fields`

Expected: FAIL because the reader still hardcodes `"chid"` and drops asset metadata and linked-file info, while the writer still synthesizes placeholders.

- [ ] **Step 3: Expand the linked-layer model to fit the spec fields**

Update `src/layer.rs` so the linked-layer shape can carry the exact metadata that the spec version includes:

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct LinkedFile {
    pub id: String,
    pub name: String,
    pub item_version: Option<u32>,
    pub data_kind: Option<PsdStringCode>,
    pub file_type: Option<PsdStringCode>,
    pub creator: Option<PsdStringCode>,
    pub data: Option<Vec<u8>>,
    pub time: Option<String>,
    pub descriptor: Option<LinkedFileDescriptor>,
    pub child_document_id: Option<PsdStringCode>,
    pub asset_mod_time: Option<f64>,
    pub asset_locked_state: Option<PsdIntCode>,
    pub linked_file: Option<LinkedFileInfo>,
    pub open_descriptor: Option<crate::descriptor::Descriptor>,
}
```

If the spec layout needs more than this current shape exposes, add the missing first-class fields rather than hiding them in raw bytes.

- [ ] **Step 4: Parse and write the real linked-layer metadata fields**

Replace the placeholder linked-layer handling in `src/additional_info.rs` with actual version-aware reads/writes:

```rust
let child_document_id = match item_version {
    Some(version) if version >= 5 => Some(PsdStringCode(self.read_signature()?)),
    _ => None,
};
let asset_mod_time = if item_version.unwrap_or(0) >= 6 {
    Some(self.read_f64()?)
} else {
    None
};
let asset_locked_state = if item_version.unwrap_or(0) >= 7 {
    Some(PsdIntCode(self.read_i32()?))
} else {
    None
};
let linked_file = if item_version.unwrap_or(0) >= 7 {
    Some(LinkedFileInfo {
        file_size: self.read_u64()?,
        name: self.read_unicode_string()?,
        full_path: self.read_unicode_string()?,
        original_path: self.read_unicode_string()?,
        relative_path: self.read_unicode_string()?,
    })
} else {
    None
};
```

```rust
if let Some(child_document_id) = item.child_document_id.as_ref() {
    item_writer.write_signature(child_document_id.as_ref())?;
}
if let Some(asset_mod_time) = item.asset_mod_time {
    item_writer.write_f64(asset_mod_time)?;
}
if let Some(asset_locked_state) = item.asset_locked_state.as_ref() {
    item_writer.write_i32(asset_locked_state.0)?;
}
if let Some(linked_file) = item.linked_file.as_ref() {
    item_writer.write_u64(linked_file.file_size)?;
    item_writer.write_unicode_string_with_padding(&linked_file.name)?;
    item_writer.write_unicode_string_with_padding(&linked_file.full_path)?;
    item_writer.write_unicode_string_with_padding(&linked_file.original_path)?;
    item_writer.write_unicode_string_with_padding(&linked_file.relative_path)?;
}
```

Match the exact conditional layout from the spec while preserving `lnkD__` normalization behavior already established elsewhere in the file.

- [ ] **Step 5: Run the targeted linked-layer test and related suite**

Run: `cargo test --quiet linked_file_roundtrips_real_metadata_fields`
Expected: PASS

Run: `cargo test --quiet lnk2_roundtrip`
Expected: PASS

Run: `cargo test --quiet integration_test ts_parity_test`
Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add src/layer.rs src/additional_info.rs tests/integration_test.rs tests/ts_parity_test.rs
git commit -m "fix: serialize linked-layer metadata fully"
```

### Task 4: Tighten Legacy Slice `1050` V6 Descriptor Framing

**Files:**
- Modify: `src/image_resources.rs`
- Modify: `src/document_resource_postprocess.rs`
- Test: `tests/integration_test.rs`

- [ ] **Step 1: Write the failing legacy-slice test**

Add a low-level round-trip test that proves per-slice descriptor tails are framed deterministically:

```rust
#[test]
fn slices_v6_roundtrip_preserves_per_slice_descriptor_tail() {
    let descriptor = psd_great::descriptor::Descriptor::default();
    let slices = psd_great::image_resources::Slices {
        version: 6,
        bounds: [0, 0, 100, 100],
        group_name: "group".into(),
        slices: vec![psd_great::image_resources::Slice {
            id: 1,
            group_id: 1,
            origin: 0,
            associated_layer_id: None,
            name: "slice".into(),
            slice_type: 0,
            left: 0,
            top: 0,
            right: 100,
            bottom: 100,
            url: String::new(),
            target: String::new(),
            message: String::new(),
            alt_tag: String::new(),
            cell_text_is_html: false,
            cell_text: String::new(),
            horizontal_align: 0,
            vertical_align: 0,
            alpha: 255,
            red: 0,
            green: 0,
            blue: 0,
            descriptor: Some(descriptor.clone()),
        }],
        descriptor: None,
    };

    let bytes = psd_great::image_resources::build_slices_resource_for_test(&slices);
    let parsed = psd_great::image_resources::parse_slices_resource_for_test(&bytes).unwrap();
    assert_eq!(parsed, slices);
}
```

- [ ] **Step 2: Run the targeted slice test to verify it fails**

Run: `cargo test --quiet slices_v6_roundtrip_preserves_per_slice_descriptor_tail`

Expected: FAIL because the current reader still infers the descriptor tail from leftover bytes instead of a stricter record interpretation.

- [ ] **Step 3: Replace the heuristic tail detection with explicit v6 record framing**

Update `src/image_resources.rs` so the version `6` parser and writer use a single deterministic record contract:

```rust
fn parse_legacy_slice(reader: &mut PsdReader<impl Read + Seek>, end: u64) -> Result<Slice> {
    let id = reader.read_u32()?;
    let group_id = reader.read_u32()?;
    let origin = reader.read_u32()?;
    let associated_layer_id = if origin == 1 {
        Some(reader.read_u32()?)
    } else {
        None
    };
    let name = reader.read_unicode_string()?;
    let slice_type = reader.read_u32()?;
    let left = reader.read_u32()?;
    let top = reader.read_u32()?;
    let right = reader.read_u32()?;
    let bottom = reader.read_u32()?;
    let url = reader.read_unicode_string()?;
    let target = reader.read_unicode_string()?;
    let message = reader.read_unicode_string()?;
    let alt_tag = reader.read_unicode_string()?;
    let cell_text_is_html = reader.read_u8()? != 0;
    let cell_text = reader.read_unicode_string()?;
    let horizontal_align = reader.read_u32()?;
    let vertical_align = reader.read_u32()?;
    let alpha = reader.read_u8()?;
    let red = reader.read_u8()?;
    let green = reader.read_u8()?;
    let blue = reader.read_u8()?;
    let descriptor = if reader.bytes_left(end) >= 8 && next_bytes_start_descriptor(reader)? {
        Some(reader.read_descriptor_structure()?)
    } else {
        None
    };
    Ok(Slice {
        id,
        group_id,
        origin,
        associated_layer_id,
        name,
        slice_type,
        left,
        top,
        right,
        bottom,
        url,
        target,
        message,
        alt_tag,
        cell_text_is_html,
        cell_text,
        horizontal_align,
        vertical_align,
        alpha,
        red,
        green,
        blue,
        descriptor,
    })
}
```

Use a deterministic “slice record end” boundary and a concrete descriptor-start test based on the descriptor’s empty Unicode-name prefix and class-ID framing instead of “any bytes left means descriptor”.

- [ ] **Step 4: Run the targeted slice test and affected integration tests**

Run: `cargo test --quiet slices_v6_roundtrip_preserves_per_slice_descriptor_tail`
Expected: PASS

Run: `cargo test --quiet integration_test`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/image_resources.rs src/document_resource_postprocess.rs tests/integration_test.rs
git commit -m "fix: tighten legacy slice descriptor framing"
```

### Task 5: Final Verification And Fresh Spec Audit

**Files:**
- Modify: `tests/integration_test.rs`
- Modify: `tests/ts_parity_test.rs`
- Review: `src/types.rs`
- Review: `src/reader.rs`
- Review: `src/writer.rs`
- Review: `src/psd.rs`
- Review: `src/image_resources.rs`
- Review: `src/document_resource_postprocess.rs`
- Review: `src/layer.rs`
- Review: `src/additional_info.rs`

- [ ] **Step 1: Run the full test suite**

Run: `cargo test --quiet`

Expected: PASS

- [ ] **Step 2: Audit the remaining four spec sections against the final code**

Run:

```bash
rg -n "read_color\\(|write_color\\(|ColorSampler|ColorSamplerPosition|lnkD|lnk2|lnk3|child_document_id|asset_mod_time|asset_locked_state|linked_file|descriptor" src
```

Expected: every remaining audited area points to real typed parsing/writing instead of placeholder defaults or heuristic-only framing.

- [ ] **Step 3: Inspect the diff for unintended API damage**

Run:

```bash
git diff -- src/types.rs src/reader.rs src/writer.rs src/psd.rs src/image_resources.rs src/document_resource_postprocess.rs src/layer.rs src/additional_info.rs tests/integration_test.rs tests/ts_parity_test.rs
```

Expected: only the planned lossless-model and wire-format changes appear.

- [ ] **Step 4: Commit the final verification touch-ups**

```bash
git add tests/integration_test.rs tests/ts_parity_test.rs src/types.rs src/reader.rs src/writer.rs src/psd.rs src/image_resources.rs src/document_resource_postprocess.rs src/layer.rs src/additional_info.rs
git commit -m "test: verify final PSD spec compliance pass"
```
