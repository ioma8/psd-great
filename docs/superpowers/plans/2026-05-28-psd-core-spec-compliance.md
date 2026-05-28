# PSD Core Spec Compliance Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make the PSD/PSB reader-writer comply with the audited Adobe PSD core spec requirements for header color modes, PackBits RLE, additional-info framing, PSB length rules, merged alpha signaling, high-bit-depth composite image handling, and saved path resource coordinates.

**Architecture:** Keep the public API stable and fix compliance at the binary-format boundary. Centralize framing logic in the PSD reader/writer helpers and targeted format modules so tests can verify exact on-disk behavior. Use focused regression tests first, then minimal production fixes.

**Tech Stack:** Rust 2021, `cargo test`, existing unit/integration tests, `binrw`, `byteorder`

---

### Task 1: Header Color Mode Codes

**Files:**
- Modify: `src/types.rs`
- Modify: `tests/integration_test.rs`

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn spec_color_mode_codes_match_adobe_header_values() {
    use psd_great::ColorMode;

    assert_eq!(ColorMode::Bitmap as u16, 0);
    assert_eq!(ColorMode::Grayscale as u16, 1);
    assert_eq!(ColorMode::Indexed as u16, 2);
    assert_eq!(ColorMode::RGB as u16, 3);
    assert_eq!(ColorMode::CMYK as u16, 4);
    assert_eq!(ColorMode::Multichannel as u16, 7);
    assert_eq!(ColorMode::Duotone as u16, 8);
    assert_eq!(ColorMode::Lab as u16, 9);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test spec_color_mode_codes_match_adobe_header_values -- --nocapture`
Expected: FAIL because the enum currently uses `5/6/7`.

- [ ] **Step 3: Write minimal implementation**

```rust
#[repr(u16)]
pub enum ColorMode {
    Bitmap = 0,
    Grayscale = 1,
    Indexed = 2,
    RGB = 3,
    CMYK = 4,
    Multichannel = 7,
    Duotone = 8,
    Lab = 9,
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test spec_color_mode_codes_match_adobe_header_values -- --nocapture`
Expected: PASS

### Task 2: PackBits No-Op Byte

**Files:**
- Modify: `src/compression.rs`
- Modify: `tests/integration_test.rs`

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn packbits_0x80_is_a_noop() {
    let compressed = [0x80, 0x02, b'A', b'B', b'C'];
    let mut output = [0u8; 3];

    psd_great::decompress_rle(&compressed, &mut output, 3, 1, &[5]).unwrap();

    assert_eq!(&output, b"ABC");
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test packbits_0x80_is_a_noop -- --nocapture`
Expected: FAIL because `0x80` is currently treated as a repeat run.

- [ ] **Step 3: Write minimal implementation**

```rust
if header == 128 {
    continue;
} else if header > 128 {
    let count = 257usize - header as usize;
    // existing repeat logic
} else {
    let count = header as usize + 1;
    // existing literal logic
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test packbits_0x80_is_a_noop -- --nocapture`
Expected: PASS

### Task 3: Additional-Info Padding And PSB Framing

**Files:**
- Modify: `src/additional_info.rs`
- Modify: `tests/integration_test.rs`

- [ ] **Step 1: Write the failing padding test**

```rust
#[test]
fn additional_info_uses_even_padding_not_four_byte_padding() {
    use psd_great::{LayerAdditionalInfo, PsdWriter};

    let mut info = LayerAdditionalInfo::default();
    info.blend_clipped_elements = Some(true);

    let mut writer = PsdWriter::new(64);
    psd_great::additional_info::write_layer_additional_info(&mut writer, &info).unwrap();
    let buf = writer.into_buffer();

    assert_eq!(buf.len(), 14);
}
```

- [ ] **Step 2: Run padding test to verify it fails**

Run: `cargo test additional_info_uses_even_padding_not_four_byte_padding -- --nocapture`
Expected: FAIL because the writer currently pads tagged blocks to 4 bytes.

- [ ] **Step 3: Write the failing PSB framing test**

```rust
#[test]
fn psb_large_tagged_blocks_use_8b64_length_headers() {
    use psd_great::{Layer, LayerAdditionalInfo, Psd, PsdStringCode, ReadOptions, WriteOptions};

    let mut layer = Layer::default();
    layer.top = Some(0);
    layer.left = Some(0);
    layer.bottom = Some(1);
    layer.right = Some(1);
    layer.additional_info.high_depth_layer_data = Some(psd_great::additional_info::HighDepthLayerInfo {
        key: PsdStringCode::from("Lr16"),
        layers: vec![],
    });

    let psd = Psd {
        width: 1,
        height: 1,
        children: Some(vec![layer]),
        ..Default::default()
    };

    let bytes = psd_great::write_psd(&psd, &WriteOptions { psb: Some(true), ..Default::default() }).unwrap();

    assert!(bytes.windows(4).any(|w| w == b"8B64"));
}
```

- [ ] **Step 4: Run PSB framing test to verify it fails**

Run: `cargo test psb_large_tagged_blocks_use_8b64_length_headers -- --nocapture`
Expected: FAIL because the writer always emits `8BIM` + `u32`.

- [ ] **Step 5: Write minimal implementation**

```rust
fn tagged_block_uses_u64_length(key: &str, large: bool) -> bool {
    large && matches!(key, "LMsk" | "Lr16" | "Lr32" | "Layr" | "Mt16" | "Mt32" | "Mtrn" | "Alph" | "FMsk" | "lnk2" | "FEid" | "FXid" | "PxSD")
}
```

Apply it to both read and write framing, and change block padding to even-byte alignment.

- [ ] **Step 6: Run the two tests to verify they pass**

Run: `cargo test additional_info_uses_even_padding_not_four_byte_padding -- --nocapture`
Expected: PASS

Run: `cargo test psb_large_tagged_blocks_use_8b64_length_headers -- --nocapture`
Expected: PASS

### Task 4: Merged Alpha Negative Layer Count

**Files:**
- Modify: `src/writer.rs`
- Modify: `tests/integration_test.rs`

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn merged_alpha_writes_negative_layer_count() {
    use std::io::Cursor;
    use psd_great::{read_psd, write_psd, ColorMode, PixelData, Psd, ReadOptions, WriteOptions};

    let psd = Psd {
        width: 1,
        height: 1,
        channels: Some(4),
        color_mode: Some(ColorMode::RGB),
        image_data: Some(PixelData { data: vec![1, 2, 3, 4], width: 1, height: 1 }),
        children: Some(vec![]),
        ..Default::default()
    };

    let bytes = write_psd(&psd, &WriteOptions::default()).unwrap();
    let reparsed = read_psd(Cursor::new(bytes), ReadOptions::default()).unwrap();
    assert_eq!(reparsed.channels, Some(4));
}
```

- [ ] **Step 2: Run test to verify it fails in a targeted byte assertion you add while implementing**

Run: `cargo test merged_alpha_writes_negative_layer_count -- --nocapture`
Expected: FAIL after adding an assertion that the layer-count field is `0xFFFF` for zero layers with merged alpha.

- [ ] **Step 3: Write minimal implementation**

```rust
let merged_alpha = psd.channels.unwrap_or(channel_count) > base_channels as u16;
let layer_count = if merged_alpha { -(layers.len() as i16) } else { layers.len() as i16 };
writer.write_i16(layer_count)?;
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test merged_alpha_writes_negative_layer_count -- --nocapture`
Expected: PASS

### Task 5: Depth-Aware Composite Image Data

**Files:**
- Modify: `src/reader.rs`
- Modify: `src/writer.rs`
- Modify: `tests/integration_test.rs`

- [ ] **Step 1: Write the failing 16-bit composite test**

```rust
#[test]
fn composite_16bit_raw_roundtrip_reads_correct_plane_sizes() {
    use std::io::Cursor;
    use psd_great::{read_psd, write_psd, ColorMode, PixelData, Psd, ReadOptions, WriteOptions};

    let psd = Psd {
        width: 1,
        height: 1,
        channels: Some(3),
        bits_per_channel: Some(16),
        color_mode: Some(ColorMode::RGB),
        image_data: Some(PixelData { data: vec![0x12, 0x34, 0x56, 0xFF], width: 1, height: 1 }),
        ..Default::default()
    };

    let bytes = write_psd(&psd, &WriteOptions { compress: Some(false), ..Default::default() }).unwrap();
    let reparsed = read_psd(Cursor::new(bytes), ReadOptions::default()).unwrap();

    assert_eq!(reparsed.image_data.unwrap().data.len(), 4);
}
```

- [ ] **Step 2: Write the failing 32-bit ZIP prediction test**

```rust
#[test]
fn composite_32bit_zip_prediction_roundtrip_reads_correct_plane_sizes() {
    use std::io::Cursor;
    use psd_great::{read_psd, write_psd, ColorMode, PixelData, Psd, ReadOptions, WriteOptions};

    let psd = Psd {
        width: 1,
        height: 1,
        channels: Some(1),
        bits_per_channel: Some(32),
        color_mode: Some(ColorMode::Grayscale),
        image_data: Some(PixelData { data: vec![0x40, 0x40, 0x40, 0xFF], width: 1, height: 1 }),
        ..Default::default()
    };

    let bytes = write_psd(&psd, &WriteOptions { compress: Some(true), ..Default::default() }).unwrap();
    let reparsed = read_psd(Cursor::new(bytes), ReadOptions::default()).unwrap();

    assert_eq!(reparsed.image_data.unwrap().data.len(), 4);
}
```

- [ ] **Step 3: Run tests to verify they fail**

Run: `cargo test composite_16bit_raw_roundtrip_reads_correct_plane_sizes -- --nocapture`
Expected: FAIL because composite channel sizing is currently 8-bit-based.

Run: `cargo test composite_32bit_zip_prediction_roundtrip_reads_correct_plane_sizes -- --nocapture`
Expected: FAIL for the same reason.

- [ ] **Step 4: Write minimal implementation**

```rust
let bytes_per_sample = match psd.bits_per_channel.unwrap_or(8) {
    8 => 1,
    16 => 2,
    32 => 4,
    _ => return Err(...),
};
let channel_len_bytes = width * height * bytes_per_sample;
```

Use depth-aware sizing for raw, RLE, ZIP, and ZIP-with-prediction composite paths, then convert decoded planes down to `u8` samples when assembling public `PixelData`.

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo test composite_16bit_raw_roundtrip_reads_correct_plane_sizes -- --nocapture`
Expected: PASS

Run: `cargo test composite_32bit_zip_prediction_roundtrip_reads_correct_plane_sizes -- --nocapture`
Expected: PASS

### Task 6: Path Resource 8.24 Fixed Point

**Files:**
- Modify: `src/image_resources.rs`
- Modify: `tests/integration_test.rs`

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn path_resources_use_8_24_fixed_point() {
    use psd_great::image_resources::{ImageResources, PathResourceRecord};
    use psd_great::types::Point;

    let mut resources = ImageResources::default();
    resources.path_resources.insert(2000, vec![PathResourceRecord {
        record_type: 1,
        closed: true,
        points: vec![
            Point { x: 1.0, y: 1.0 },
            Point { x: 0.5, y: 0.5 },
            Point { x: 0.25, y: 0.25 },
            Point { x: 0.0, y: 0.0 },
        ],
    }]);

    let mut writer = psd_great::PsdWriter::new(256);
    psd_great::image_resources::write_image_resources(&mut writer, &resources).unwrap();
    let bytes = writer.into_buffer();

    assert!(bytes.windows(4).any(|w| w == [0x01, 0x00, 0x00, 0x00]));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test path_resources_use_8_24_fixed_point -- --nocapture`
Expected: FAIL because the writer currently uses 16.16 encoding.

- [ ] **Step 3: Write minimal implementation**

```rust
let fixed = (point.x * 16777216.0).round() as i32;
```

Apply the same 8.24 scaling to both read and write paths.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test path_resources_use_8_24_fixed_point -- --nocapture`
Expected: PASS

### Task 7: Full Verification

**Files:**
- Modify: `tests/integration_test.rs`
- Modify: `tests/ts_parity_test.rs`

- [ ] **Step 1: Run focused spec-compliance tests**

Run: `cargo test spec_color_mode_codes_match_adobe_header_values packbits_0x80_is_a_noop additional_info_uses_even_padding_not_four_byte_padding psb_large_tagged_blocks_use_8b64_length_headers merged_alpha_writes_negative_layer_count composite_16bit_raw_roundtrip_reads_correct_plane_sizes composite_32bit_zip_prediction_roundtrip_reads_correct_plane_sizes path_resources_use_8_24_fixed_point -- --nocapture`
Expected: PASS

- [ ] **Step 2: Run full test suite**

Run: `cargo test --quiet`
Expected: PASS

- [ ] **Step 3: Inspect changed files**

Run: `git diff -- src/types.rs src/compression.rs src/reader.rs src/writer.rs src/additional_info.rs src/image_resources.rs tests/integration_test.rs tests/ts_parity_test.rs`
Expected: Diff only for spec-compliance changes and related tests.
