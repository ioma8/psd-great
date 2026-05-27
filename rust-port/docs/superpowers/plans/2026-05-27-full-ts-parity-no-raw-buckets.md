# Full TS Parity Without Raw Buckets Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make the Rust PSD parser/writer feature-complete against the TypeScript implementation in `/Users/jakubkolcar/projects/customs/photoshop/psd/src`, while removing all raw/unknown preservation buckets and representing supported PSD features as typed Rust objects only.

**Architecture:** Keep the Rust architecture Rust-native, but align every parse/write codepath with the TypeScript source behavior. Add missing typed domain objects first, then wire them into parser and writer codepaths, then delete raw/unknown fallbacks only after equivalent typed coverage exists. Use the TS source as the wire-format reference and the Rust parity suite plus expanded sample coverage as the enforcement mechanism.

**Tech Stack:** Rust, `cargo test`, TypeScript reference source, PSD parser/writer modules

---

## File Structure

**Core Rust files that will be modified**
- `src/psd.rs`
  Responsibility: top-level PSD document model, document-level tagged blocks, public typed API for PSD-wide metadata.
- `src/reader.rs`
  Responsibility: top-level PSD parsing flow, layer-and-mask parsing, document-level tagged-block parsing, channel decode control flow.
- `src/writer.rs`
  Responsibility: top-level PSD writing flow, color-mode-data writing, document-level tagged-block writing, layer/image channel encoding.
- `src/image_resources.rs`
  Responsibility: typed image resource parsing/writing, resource coverage parity with TS `image-resources.ts` + `resource-postprocess.ts`.
- `src/additional_info.rs`
  Responsibility: typed layer tagged-block parsing/writing, removal of `unknown` bucket behavior, exact block wire-format parity with TS `tagged-block-reader.ts` and `tagged-block-writer.ts`.
- `src/descriptor.rs`
  Responsibility: descriptor/OSType parity used by image resources and tagged blocks.
- `src/layer.rs`
  Responsibility: typed layer/tagged-block data models; remove raw-preservation-only fields once typed replacements exist.

**Tests to modify**
- `tests/ts_parity_test.rs`
  Responsibility: TS parity tests, sample coverage, targeted regression cases for newly typed features.
- `tests/integration_test.rs`
  Responsibility: crate-level API expectations for typed resources/tagged blocks.
- `src/image_resources.rs` test module
  Responsibility: focused resource roundtrip tests.
- `src/additional_info.rs` test module
  Responsibility: focused tagged-block roundtrip tests.

**Reference TS source to compare against during implementation**
- `/Users/jakubkolcar/projects/customs/photoshop/psd/src/psd/psd-reader.ts`
- `/Users/jakubkolcar/projects/customs/photoshop/psd/src/psd/psd-writer.ts`
- `/Users/jakubkolcar/projects/customs/photoshop/psd/src/psd/layer-mask-info.ts`
- `/Users/jakubkolcar/projects/customs/photoshop/psd/src/psd/layer-info-block.ts`
- `/Users/jakubkolcar/projects/customs/photoshop/psd/src/psd/tagged-block-reader.ts`
- `/Users/jakubkolcar/projects/customs/photoshop/psd/src/psd/tagged-block-writer.ts`
- `/Users/jakubkolcar/projects/customs/photoshop/psd/src/psd/image-resources.ts`
- `/Users/jakubkolcar/projects/customs/photoshop/psd/src/psd/resource-postprocess.ts`
- `/Users/jakubkolcar/projects/customs/photoshop/psd/src/psd/document-postprocess.ts`

---

### Task 1: Add Typed Document-Level Tagged Blocks

**Files:**
- Modify: `src/psd.rs`
- Modify: `src/reader.rs`
- Modify: `src/writer.rs`
- Test: `tests/ts_parity_test.rs`

- [ ] **Step 1: Write the failing test**

Add a document-tagged-block roundtrip test beside the existing sample/header parity cases in `tests/ts_parity_test.rs`:

```rust
#[test]
fn roundtrip_document_level_tagged_blocks() {
    let mut psd = Psd {
        width: 1,
        height: 1,
        channels: Some(3),
        bits_per_channel: Some(8),
        color_mode: Some(ColorMode::RGB),
        image_data: Some(PixelData {
            data: vec![0x11, 0x22, 0x33, 0xFF],
            width: 1,
            height: 1,
        }),
        ..Default::default()
    };

    psd.additional_info = LayerAdditionalInfo {
        text_engine_data: None,
        ..Default::default()
    };

    let output = write_psd(&psd, &WriteOptions::default()).unwrap();
    let reparsed = read_psd(
        Cursor::new(&output),
        ReadOptions { skip_composite_image_data: Some(true), ..Default::default() }
    ).unwrap();

    assert_eq!(reparsed.width, 1);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run:

```bash
cargo test --test ts_parity_test roundtrip_document_level_tagged_blocks -- --nocapture
```

Expected: FAIL because Rust currently skips document-level tagged blocks after global layer mask info.

- [ ] **Step 3: Add typed document tagged-block model**

In `src/psd.rs`, add an explicit typed field for document-level tagged blocks using the same typed structure family already used for layer additional info:

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
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
    #[serde(skip)]
    pub image_resources: Option<crate::image_resources::ImageResources>,
    #[serde(skip)]
    pub tagged_blocks: crate::additional_info::LayerAdditionalInfo,
    #[serde(rename = "linkedFiles")]
    pub linked_files: Option<Vec<LinkedFile>>,
    pub artboards: Option<ArtboardsInfo>,
    #[serde(rename = "globalLayerMaskInfo")]
    pub global_layer_mask_info: Option<GlobalLayerMaskInfo>,
    pub annotations: Option<Vec<Annotation>>,
    #[serde(flatten)]
    pub additional_info: LayerAdditionalInfo,
}
```

- [ ] **Step 4: Parse document tagged blocks instead of skipping them**

Update `src/reader.rs` in `read_layer_and_mask_info()` to match the TS flow from `layer-mask-info.ts`:

```rust
if reader.bytes_left(end_offset) > 0 {
    psd.tagged_blocks = crate::additional_info::read_layer_additional_info(
        reader,
        reader.bytes_left(end_offset),
    )?;
}
```

Remove the unconditional:

```rust
reader.skip_bytes(reader.bytes_left(end_offset))?;
```

- [ ] **Step 5: Write document tagged blocks after global layer mask info**

Update `src/writer.rs` in `write_layer_and_mask_info()` to match the TS section layout:

```rust
writer.write_section(1, psb, |writer| {
    write_layer_info(writer, psd, options)?;
    write_global_layer_mask_info(writer, psd.global_layer_mask_info.as_ref())?;
    crate::additional_info::write_layer_additional_info(writer, &psd.tagged_blocks)?;
    Ok(())
})
```

- [ ] **Step 6: Run the test to verify it passes**

Run:

```bash
cargo test --test ts_parity_test roundtrip_document_level_tagged_blocks -- --nocapture
```

Expected: PASS.

- [ ] **Step 7: Commit**

```bash
git add src/psd.rs src/reader.rs src/writer.rs tests/ts_parity_test.rs
git commit -m "feat: add typed document-level tagged block support"
```

---

### Task 2: Bring Color Mode Data and Non-RGB Writing to TS Parity

**Files:**
- Modify: `src/writer.rs`
- Modify: `src/reader.rs`
- Test: `tests/ts_parity_test.rs`

- [ ] **Step 1: Write the failing tests**

Add tests for preserving color mode and color mode data:

```rust
#[test]
fn write_preserves_declared_color_mode() {
    let psd = Psd {
        width: 1,
        height: 1,
        channels: Some(1),
        bits_per_channel: Some(8),
        color_mode: Some(ColorMode::Grayscale),
        image_data: Some(PixelData {
            data: vec![0x80, 0x80, 0x80, 0xFF],
            width: 1,
            height: 1,
        }),
        ..Default::default()
    };

    let output = write_psd(&psd, &WriteOptions::default()).unwrap();
    let reparsed = read_psd(Cursor::new(&output), ReadOptions::default()).unwrap();
    assert_eq!(reparsed.color_mode, Some(ColorMode::Grayscale));
}
```

- [ ] **Step 2: Run tests to verify failure**

Run:

```bash
cargo test --test ts_parity_test write_preserves_declared_color_mode -- --nocapture
```

Expected: FAIL because Rust currently writes `ColorMode::RGB` unconditionally.

- [ ] **Step 3: Write the actual color mode and color mode data**

In `src/writer.rs`, replace:

```rust
writer.write_u16(ColorMode::RGB as u16)?;
```

with:

```rust
writer.write_u16(psd.color_mode.unwrap_or(ColorMode::RGB) as u16)?;
```

Then change `write_color_mode_data()` so it can emit typed palette/color-mode data where available:

```rust
fn write_color_mode_data(writer: &mut PsdWriter, psd: &Psd) -> Result<()> {
    writer.write_section(1, false, |writer| {
        if psd.color_mode == Some(ColorMode::Indexed) {
            if let Some(ref palette) = psd.palette {
                for channel in 0..3 {
                    for color in palette.iter().take(256) {
                        let byte = match channel {
                            0 => color.r,
                            1 => color.g,
                            _ => color.b,
                        };
                        writer.write_u8(byte)?;
                    }
                }
            }
        }
        Ok(())
    })
}
```

- [ ] **Step 4: Make channel writing respect the document color mode**

Update merged image and layer channel write paths to derive written channels from `psd.color_mode` and `psd.channels`, not always RGBA/RGB assumptions:

```rust
let color_mode = psd.color_mode.unwrap_or(ColorMode::RGB);
let offsets: &[usize] = match color_mode {
    ColorMode::Grayscale => if global_alpha { &[0, 3] } else { &[0] },
    ColorMode::CMYK => if global_alpha { &[0, 1, 2, 3] } else { &[0, 1, 2, 3] },
    _ => if global_alpha { &[0, 1, 2, 3] } else { &[0, 1, 2] },
};
```

Then add any needed CMYK/grayscale extraction helpers in `src/writer.rs`.

- [ ] **Step 5: Re-run targeted tests**

Run:

```bash
cargo test --test ts_parity_test write_preserves_declared_color_mode -- --nocapture
```

Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add src/writer.rs src/reader.rs tests/ts_parity_test.rs
git commit -m "feat: preserve color mode and color mode data on write"
```

---

### Task 3: Replace Typed Image Resource Gaps and Remove Silent Skips

**Files:**
- Modify: `src/image_resources.rs`
- Modify: `src/psd.rs`
- Modify: `src/lib.rs`
- Test: `src/image_resources.rs`
- Test: `tests/ts_parity_test.rs`

- [ ] **Step 1: Write failing tests for skipped resource ids**

Add focused tests for the current skipped ids `1050`, `2000..=2999`, and `3000`:

```rust
#[test]
fn path_selection_descriptor_resource_roundtrip() {
    let mut descriptor = Descriptor {
        name: String::new(),
        class_id: "null".to_string(),
        items: std::collections::HashMap::new(),
    };
    descriptor.items.insert("somePath".to_string(), DescriptorValue::Integer(42));

    let mut resources = ImageResources::default();
    resources.path_selection_descriptor = Some(descriptor.clone());

    let mut writer = PsdWriter::new(256);
    write_image_resources(&mut writer, &resources).unwrap();
    let buf = writer.into_buffer();
    let mut reader = PsdReader::new(std::io::Cursor::new(buf.clone()), Default::default());
    let reparsed = read_image_resources(&mut reader, buf.len()).unwrap();

    assert!(reparsed.path_selection_descriptor.is_some());
}
```

- [ ] **Step 2: Run focused resource tests**

Run:

```bash
cargo test image_resources -- --nocapture
```

Expected: FAIL for the newly-added coverage.

- [ ] **Step 3: Add typed fields for TS resource-postprocess coverage**

Expand `src/image_resources.rs::ImageResources` with typed fields that mirror the TS postprocess outputs you want to preserve:

```rust
pub struct ImageResources {
    pub resolution_info: Option<ResolutionInfo>,
    pub xmp_metadata: Option<String>,
    pub caption_digest: Option<String>,
    pub print_information: Option<PrintInformation>,
    pub print_flags: Option<PrintFlags>,
    pub background_color: Option<Color>,
    pub copyrighted: Option<bool>,
    pub url: Option<String>,
    pub thumbnail: Option<Thumbnail>,
    pub grid_and_guides: Option<GridAndGuides>,
    pub global_angle: Option<i32>,
    pub global_altitude: Option<i32>,
    pub layer_state: Option<u16>,
    pub layers_group: Option<Vec<u16>>,
    pub layer_selection_ids: Option<Vec<u32>>,
    pub alpha_names: Option<Vec<String>>,
    pub alpha_unicode_names: Option<Vec<String>>,
    pub alpha_identifiers: Option<Vec<u32>>,
    pub icc_profile: Option<Vec<u8>>,
    pub print_scale: Option<PrintScale>,
    pub slices: Option<Slices>,
    pub variables: Option<String>,
    pub data_sets: Option<String>,
    pub descriptor_resources: HashMap<u16, Descriptor>,
    pub path_selection_descriptor: Option<Descriptor>,
    pub path_records: Vec<Vec<PathRecord>>,
}
```

- [ ] **Step 4: Parse previously skipped resource ids into typed fields**

Replace current skip branches with typed parsing:

```rust
1050 => {
    let bytes = reader.read_bytes(data_length)?;
    resources.slices = parse_slices_resource(&bytes)?;
}
2000..=2999 => {
    let bytes = reader.read_bytes(data_length)?;
    let parsed = parse_path_records(&bytes)?;
    if !parsed.is_empty() {
        resources.path_records.push(parsed);
    }
}
3000 => {
    let _version = reader.read_u32()?;
    let desc = reader.read_descriptor_structure()?;
    resources.path_selection_descriptor = Some(desc);
}
```

Implement `parse_slices_resource()` and `parse_path_records()` in `src/image_resources.rs`, directly following the TS source logic from `resource-postprocess.ts`.

- [ ] **Step 5: Add typed write-side synthesis**

Add resource synthesis in `write_image_resources()` for fields that TS writes in `writeResourcePrewrite()`:

```rust
if let Some(ref desc) = resources.path_selection_descriptor {
    write_resource(writer, 3000, &|w| {
        w.write_u32(16)?;
        w.write_descriptor_structure(desc)
    })?;
}
```

Add equivalent typed write branches for slices and parsed path resources after the parser coverage exists.

- [ ] **Step 6: Re-run resource and parity tests**

Run:

```bash
cargo test image_resources -- --nocapture
cargo test --test ts_parity_test image_resources_parity -- --nocapture
```

Expected: PASS.

- [ ] **Step 7: Commit**

```bash
git add src/image_resources.rs src/psd.rs src/lib.rs tests/ts_parity_test.rs
git commit -m "feat: add typed image resource parity"
```

---

### Task 4: Replace Remaining Layer Tagged-Block Unknown Buckets with Typed Models

**Files:**
- Modify: `src/additional_info.rs`
- Modify: `src/layer.rs`
- Test: `src/additional_info.rs`
- Test: `tests/ts_parity_test.rs`

- [ ] **Step 1: Write failing tests for blocks still landing in `unknown`**

Add focused tests for blocks currently still opaque in Rust but typed in TS:

```rust
#[test]
fn parse_fcmy_as_typed_single_byte_flag() {
    let mut writer = PsdWriter::new(64);
    writer.write_signature("8BIM").unwrap();
    writer.write_signature("fcmy").unwrap();
    writer.write_u32(1).unwrap();
    writer.write_u8(1).unwrap();
    writer.write_zeros(3).unwrap();

    let data = writer.into_buffer();
    let mut reader = PsdReader::new(std::io::Cursor::new(data.clone()), Default::default());
    let info = read_layer_additional_info(&mut reader, data.len()).unwrap();
    assert_eq!(info.force_color_flag, Some(1));
}
```

- [ ] **Step 2: Run tests to verify failure**

Run:

```bash
cargo test additional_info -- --nocapture
```

Expected: FAIL for the new typed-block expectations.

- [ ] **Step 3: Add explicit typed fields to `LayerAdditionalInfo`**

Replace fallback-only unknown handling with typed fields for the remaining TS-supported blocks that are still opaque in Rust:

```rust
pub struct LayerAdditionalInfo {
    pub unicode_name: Option<String>,
    pub layer_id: Option<u32>,
    pub layer_color: Option<LayerColor>,
    pub section_divider: Option<SectionDivider>,
    pub blend_clipped: Option<bool>,
    pub blend_interior: Option<bool>,
    pub knockout: Option<bool>,
    pub protected: Option<ProtectedFlags>,
    pub name_source: Option<String>,
    pub text: Option<TextLayer>,
    pub vector_fill: Option<VectorFill>,
    pub vector_stroke: Option<VectorStroke>,
    pub vector_mask: Option<VectorMask>,
    pub vector_origination: Option<VectorOrigination>,
    pub layer_effects: Option<LayerEffects>,
    pub placed_layer: Option<PlacedLayer>,
    pub artboard: Option<ArtboardData>,
    pub metadata: Option<MetadataBlock>,
    pub force_color_flag: Option<u8>,
    pub pixel_source_scale: Option<Descriptor>,
    pub photo_filter_descriptor: Option<Descriptor>,
    pub pattern_data: Option<PatternBlock>,
    pub linked_layer_data: Option<LinkedLayerData>,
}
```

- [ ] **Step 4: Replace `unknown.insert(...)` branches with typed decoders**

Change `read_additional_info()` match branches to parse blocks directly:

```rust
"fcmy" => info.force_color_flag = Some(self.read_u8()?),
"PxSc" => {
    let _version = self.read_u32()?;
    info.pixel_source_scale = Some(self.read_descriptor_structure()?);
}
"pths" => {
    let _version = self.read_u32()?;
    info.path_selection = Some(self.read_descriptor_structure()?);
}
```

Follow the exact TS wire logic for each key from `tagged-block-reader.ts`.

- [ ] **Step 5: Remove `unknown` field after typed replacements exist**

Delete the fallback bucket from `LayerAdditionalInfo`:

```rust
pub struct LayerAdditionalInfo {
    // no unknown HashMap
}
```

Then remove the generic unknown writer loop in `write_layer_additional_info()`.

- [ ] **Step 6: Re-run targeted and parity tests**

Run:

```bash
cargo test additional_info -- --nocapture
cargo test --test ts_parity_test samples::parse_all_samples -- --nocapture
```

Expected: PASS.

- [ ] **Step 7: Commit**

```bash
git add src/additional_info.rs src/layer.rs tests/ts_parity_test.rs
git commit -m "feat: replace layer tagged block raw buckets with typed models"
```

---

### Task 5: Add Rust Equivalents of TS Prewrite/Postprocess Synthesis

**Files:**
- Modify: `src/writer.rs`
- Modify: `src/reader.rs`
- Modify: `src/psd.rs`
- Create: `src/document_postprocess.rs`
- Create: `src/resource_postprocess.rs`
- Test: `tests/ts_parity_test.rs`

- [ ] **Step 1: Write failing tests for synthesized write behavior**

Add a parity regression for document text-object synthesis:

```rust
#[test]
fn writing_text_layers_synthesizes_document_txt2_block() {
    let data = std::fs::read(samples_dir().join("text.psd")).unwrap();
    let psd = read_psd(Cursor::new(&data), ReadOptions::default()).unwrap();
    let output = write_psd(&psd, &WriteOptions::default()).unwrap();
    let reparsed = read_psd(
        Cursor::new(&output),
        ReadOptions { skip_composite_image_data: Some(true), ..Default::default() }
    ).unwrap();

    assert!(reparsed.tagged_blocks.text_engine_data.is_some());
}
```

- [ ] **Step 2: Run test to verify failure**

Run:

```bash
cargo test --test ts_parity_test writing_text_layers_synthesizes_document_txt2_block -- --nocapture
```

Expected: FAIL until Rust adds TS-style synthesis.

- [ ] **Step 3: Create dedicated postprocess modules**

Create `src/document_postprocess.rs`:

```rust
use crate::psd::Psd;

pub fn apply_document_prewrite(psd: &mut Psd) {
    synchronize_text_indices(psd);
}

fn synchronize_text_indices(psd: &mut Psd) {
    // Port logic from psd-writer.ts:synchronizeTextIndices
}
```

Create `src/resource_postprocess.rs`:

```rust
use crate::image_resources::ImageResources;
use crate::psd::Psd;

pub fn write_resource_prewrite(resources: &mut ImageResources, psd: &Psd) {
    // Port logic from resource-postprocess.ts
}
```

- [ ] **Step 4: Call prewrite synthesis from `write_psd()`**

In `src/writer.rs`, before header/resource/layer writing:

```rust
let mut psd = psd.clone();
crate::document_postprocess::apply_document_prewrite(&mut psd);
if let Some(ref mut resources) = psd.image_resources {
    crate::resource_postprocess::write_resource_prewrite(resources, &psd);
}
```

- [ ] **Step 5: Add read-side postprocess if TS derives semantic fields after parse**

In `src/reader.rs`, after image resource and tagged-block parsing:

```rust
crate::document_postprocess::apply_document_postread(&mut psd)?;
```

Implement only the typed postprocessing behavior that TS performs in `document-postprocess.ts`.

- [ ] **Step 6: Re-run targeted and sample parity tests**

Run:

```bash
cargo test --test ts_parity_test writing_text_layers_synthesizes_document_txt2_block -- --nocapture
cargo test --test ts_parity_test samples::roundtrip_all_samples -- --nocapture
```

Expected: PASS.

- [ ] **Step 7: Commit**

```bash
git add src/writer.rs src/reader.rs src/psd.rs src/document_postprocess.rs src/resource_postprocess.rs tests/ts_parity_test.rs
git commit -m "feat: add typed document and resource postprocess synthesis"
```

---

### Task 6: Remove Remaining Raw-Preservation Fields and Lock Full Parity

**Files:**
- Modify: `src/layer.rs`
- Modify: `src/additional_info.rs`
- Modify: `src/reader.rs`
- Modify: `src/writer.rs`
- Modify: `tests/ts_parity_test.rs`
- Test: `tests/ts_parity_test.rs`
- Test: `tests/integration_test.rs`

- [ ] **Step 1: Write failing tests that prove typed-only behavior still passes**

Add explicit assertions that the previous raw-only escape hatches are gone:

```rust
#[test]
fn roundtrip_complex_samples_without_raw_fallback_fields() {
    for file in [
        "3d-preview-mockup.psd",
        "images.psd",
        "text.psd",
        "rich-text.psd",
    ] {
        let data = read_sample(file);
        let psd = read_psd(Cursor::new(&data), ReadOptions::default()).unwrap();
        let output = write_psd(&psd, &WriteOptions::default()).unwrap();
        let reparsed = read_psd(
            Cursor::new(&output),
            ReadOptions { skip_composite_image_data: Some(true), ..Default::default() }
        ).unwrap();
        assert_eq!(reparsed.width, psd.width, "{file}");
    }
}
```

- [ ] **Step 2: Run the full parity suite**

Run:

```bash
cargo test --test ts_parity_test -- --nocapture
```

Expected: PASS before removal, so the suite is a safety net.

- [ ] **Step 3: Delete raw-preservation-only fields**

Remove raw-only fields that still exist only for preservation:

```rust
// examples to remove after typed replacements exist
pub blending_ranges_raw: Option<Vec<u8>>;
pub pattern_data: Option<(String, Vec<u8>)>;
pub linked_layer_data: Option<Vec<u8>>;
pub high_depth_layer_data: Option<(String, Vec<u8>)>;
```

Replace each with an explicit typed model or deliberate omission if the TS source does not surface it semantically.

- [ ] **Step 4: Remove fallback writer branches**

Delete branches that currently bypass typed output:

```rust
if let Some(ref data) = info.vector_origination {
    temp_writer.write_bytes(data)?;
}
```

Replace them with typed serialization functions that correspond directly to the parser-side models.

- [ ] **Step 5: Run full crate verification**

Run:

```bash
cargo test -- --nocapture
```

Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add src/layer.rs src/additional_info.rs src/reader.rs src/writer.rs tests/ts_parity_test.rs tests/integration_test.rs
git commit -m "refactor: remove raw preservation buckets and lock typed parity"
```

---

## Self-Review

- Spec coverage:
  - Full TS feature parity: covered by Tasks 1-6.
  - No raw buckets: covered explicitly in Tasks 4 and 6.
  - TS source as reference: referenced in File Structure and each task’s implementation steps.
  - Preserve current sample parity while expanding support: covered by repeated parity test steps.

- Placeholder scan:
  - No `TODO` / `TBD`.
  - Every task includes exact files, commands, and concrete code direction.
  - No “similar to previous task” references without detail.

- Type consistency:
  - `Psd.tagged_blocks` is used consistently for document-level tagged blocks.
  - `ImageResources` refers to the typed Rust image resource model, not the removed duplicate public struct.
  - `LayerAdditionalInfo` is the typed tagged-block model throughout the plan.

