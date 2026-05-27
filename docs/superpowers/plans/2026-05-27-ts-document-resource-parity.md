# TS Document Resource Parity Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Finish the remaining Rust work required for full TypeScript parity in document-level image resource parsing, postprocess mapping, and write-side prewrite behavior.

**Architecture:** Keep the Rust parser/writer typed-first, but add an explicit document resource postprocess/prewrite layer equivalent to the TypeScript `document-postprocess.ts` and `resource-postprocess.ts` logic. Fix the current semantic mismatch for resource `1026`, add the missing typed resource families and document-facing fields, then verify those mappings with focused parity tests plus the full existing suite.

**Tech Stack:** Rust, existing `PsdReader`/`PsdWriter`, `byteorder`, `binrw`, existing `descriptor` and `image_resources` modules, `cargo test`

---

## File Structure

**Primary Rust files**
- `src/psd.rs`
  - Responsibility: document-facing typed fields that correspond to TS `PsdDocument` resource-derived properties.
- `src/image_resources.rs`
  - Responsibility: low-level typed parsing/writing for individual image resource blocks.
- `src/reader.rs`
  - Responsibility: call Rust-side document resource postprocess after reading image resources and map typed resources onto `Psd`.
- `src/writer.rs`
  - Responsibility: call Rust-side document resource prewrite before writing the image resources section.
- `tests/ts_parity_test.rs`
  - Responsibility: parity tests for the missing TS document resource behaviors.

**New Rust helper file**
- `src/document_resource_postprocess.rs`
  - Responsibility: mirror the TS `document-postprocess.ts` and `resource-postprocess.ts` read/write mapping layer without polluting the binary IO layer.

**TS source of truth**
- `/Users/jakubkolcar/projects/customs/photoshop/psd/src/psd/document-postprocess.ts`
- `/Users/jakubkolcar/projects/customs/photoshop/psd/src/psd/resource-postprocess.ts`
- `/Users/jakubkolcar/projects/customs/photoshop/psd/src/psd/psd-writer.ts`
- `/Users/jakubkolcar/projects/customs/photoshop/psd/src/psd/image-resources.ts`

---

### Task 1: Add missing document-facing resource fields to the Rust PSD model

**Files:**
- Create: `src/document_resource_postprocess.rs`
- Modify: `src/lib.rs`
- Modify: `src/psd.rs`
- Test: `tests/ts_parity_test.rs`

- [ ] **Step 1: Write the failing parity test for document resource field mapping**

Add this test module to `tests/ts_parity_test.rs`:

```rust
#[test]
fn roundtrip_document_resource_postprocess_fields() {
    let mut psd = Psd::default();
    psd.width = 1;
    psd.height = 1;
    psd.channels = Some(4);
    psd.bits_per_channel = Some(8);
    psd.color_mode = Some(ColorMode::RGB);
    psd.children = Some(vec![Layer::default()]);

    psd.variable_sets = Some(vec![psd_great::psd::VariableSet {
        var_name: Some("title".to_string()),
        trait_name: Some("textcontent".to_string()),
        doc_ref: None,
        placement_method: None,
        align: None,
        valign: None,
        clip: None,
    }]);
    psd.data_sets = Some(vec![
        vec!["title".to_string()],
        vec!["Hello".to_string()],
    ]);
    psd.descriptor_1065 = Some(psd_great::descriptor::Descriptor {
        name: String::new(),
        class_id: "test".to_string(),
        items: std::collections::HashMap::new(),
    });
    psd.descriptor_1074 = psd.descriptor_1065.clone();
    psd.descriptor_1075 = psd.descriptor_1065.clone();
    psd.custom_points = Some(vec![psd_great::psd::CustomPoint { x: 10.5, y: 20.25 }]);
    psd.display_info = Some(psd_great::psd::DisplayInfo {
        h_res_unit: 1,
        v_res_unit: 2,
        width_unit: 3,
        height_unit: 4,
    });

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

    assert_eq!(reparsed.variable_sets, psd.variable_sets);
    assert_eq!(reparsed.data_sets, psd.data_sets);
    assert_eq!(reparsed.descriptor_1065, psd.descriptor_1065);
    assert_eq!(reparsed.descriptor_1074, psd.descriptor_1074);
    assert_eq!(reparsed.descriptor_1075, psd.descriptor_1075);
    assert_eq!(reparsed.custom_points, psd.custom_points);
    assert_eq!(reparsed.display_info, psd.display_info);
}
```

- [ ] **Step 2: Run the new test to verify it fails**

Run:

```bash
cargo test roundtrip_document_resource_postprocess_fields -- --nocapture
```

Expected:
- FAIL because `Psd` does not yet expose the required typed fields and/or the read/write mapping is missing

- [ ] **Step 3: Add the missing typed document fields to `src/psd.rs`**

Add these types near the other document model structs in `src/psd.rs`:

```rust
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CustomPoint {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DisplayInfo {
    #[serde(rename = "hResUnit")]
    pub h_res_unit: u16,
    #[serde(rename = "vResUnit")]
    pub v_res_unit: u16,
    #[serde(rename = "widthUnit")]
    pub width_unit: u16,
    #[serde(rename = "heightUnit")]
    pub height_unit: u16,
}
```

Add these fields to `Psd`:

```rust
#[serde(rename = "variableSets")]
pub variable_sets: Option<Vec<VariableSet>>,
#[serde(rename = "dataSets")]
pub data_sets: Option<Vec<Vec<String>>>,
#[serde(rename = "descriptor1065")]
pub descriptor_1065: Option<crate::descriptor::Descriptor>,
#[serde(rename = "descriptor1074")]
pub descriptor_1074: Option<crate::descriptor::Descriptor>,
#[serde(rename = "descriptor1075")]
pub descriptor_1075: Option<crate::descriptor::Descriptor>,
#[serde(rename = "customPoints")]
pub custom_points: Option<Vec<CustomPoint>>,
#[serde(rename = "displayInfo")]
pub display_info: Option<DisplayInfo>,
```

- [ ] **Step 4: Export the new helper module**

In `src/lib.rs`, add:

```rust
pub mod document_resource_postprocess;
```

- [ ] **Step 5: Run the test again to verify it still fails for implementation reasons, not compile reasons**

Run:

```bash
cargo test roundtrip_document_resource_postprocess_fields -- --nocapture
```

Expected:
- FAIL in assertions, but compile cleanly

- [ ] **Step 6: Commit**

```bash
git add src/lib.rs src/psd.rs tests/ts_parity_test.rs
git commit -m "feat: add typed document resource parity model"
```

---

### Task 2: Fix the `1026` semantic mismatch and map it to per-layer clipping values

**Files:**
- Modify: `src/image_resources.rs`
- Modify: `src/document_resource_postprocess.rs`
- Modify: `src/reader.rs`
- Modify: `src/writer.rs`
- Modify: `tests/ts_parity_test.rs`
- Test: `tests/ts_parity_test.rs`

- [ ] **Step 1: Write the failing clipping parity test**

Add this test to `tests/ts_parity_test.rs`:

```rust
#[test]
fn roundtrip_resource_1026_maps_layer_clipping() {
    let mut layer_a = Layer::default();
    layer_a.top = Some(0);
    layer_a.left = Some(0);
    layer_a.bottom = Some(1);
    layer_a.right = Some(1);
    layer_a.clipping = Some(0);

    let mut layer_b = layer_a.clone();
    layer_b.clipping = Some(1);

    let mut psd = Psd::default();
    psd.width = 1;
    psd.height = 1;
    psd.channels = Some(4);
    psd.bits_per_channel = Some(8);
    psd.color_mode = Some(ColorMode::RGB);
    psd.children = Some(vec![layer_a, layer_b]);

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

    let layers = reparsed.children.expect("layers");
    assert_eq!(layers[0].clipping, Some(0));
    assert_eq!(layers[1].clipping, Some(1));
    assert!(
        reparsed
            .image_resources
            .as_ref()
            .and_then(|r| r.layers_group.as_ref())
            .is_none()
    );
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run:

```bash
cargo test roundtrip_resource_1026_maps_layer_clipping -- --nocapture
```

Expected:
- FAIL because resource `1026` is currently exposed as `layers_group`

- [ ] **Step 3: Replace `layers_group` with typed clipping values in `src/image_resources.rs`**

Change the `ImageResources` field:

```rust
pub clipping: Option<Vec<u16>>,
```

Replace the read/write helpers:

```rust
pub fn read_clipping(&mut self, resources: &mut ImageResources, length: usize) -> Result<()> {
    let mut clipping = Vec::new();
    let count = length / 2;
    for _ in 0..count {
        clipping.push(
            decode_be::<LayerStateRecord>(&self.read_bytes(2)?, "clipping entry")?.state,
        );
    }
    resources.clipping = Some(clipping);
    Ok(())
}

pub fn write_clipping(&mut self, clipping: &[u16]) -> Result<()> {
    for value in clipping {
        self.write_bytes(&encode_be(&LayerStateRecord { state: *value }, "clipping entry")?)?;
    }
    Ok(())
}
```

Update the resource dispatch:

```rust
1026 => reader.read_clipping(&mut resources, data_length)?,
```

And the writer:

```rust
if let Some(ref clipping) = resources.clipping {
    write_resource(writer, 1026, &|w| w.write_clipping(clipping))?;
}
```

- [ ] **Step 4: Implement document-level read/write mapping for clipping**

Create `src/document_resource_postprocess.rs` with:

```rust
use crate::error::Result;
use crate::psd::Psd;

pub fn apply_document_postprocess(psd: &mut Psd) -> Result<()> {
    if let (Some(resources), Some(layers)) = (psd.image_resources.as_ref(), psd.children.as_mut()) {
        if let Some(clipping) = resources.clipping.as_ref() {
            for (layer, value) in layers.iter_mut().zip(clipping.iter()) {
                layer.clipping = Some(*value);
            }
        }
    }
    Ok(())
}

pub fn apply_document_prewrite(psd: &mut Psd) -> Result<()> {
    if let Some(layers) = psd.children.as_ref() {
        let clipping_values: Vec<u16> = layers
            .iter()
            .map(|layer| layer.clipping.unwrap_or(0))
            .collect();
        if clipping_values.iter().any(|value| *value > 0) {
            let resources = psd.image_resources.get_or_insert_with(Default::default);
            resources.clipping = Some(clipping_values);
        }
    }
    Ok(())
}
```

- [ ] **Step 5: Wire the postprocess and prewrite calls**

In `src/reader.rs`, after `psd.image_resources = Some(resources);`, call:

```rust
crate::document_resource_postprocess::apply_document_postprocess(psd)?;
```

In `src/writer.rs`, before writing the image resources section, call:

```rust
apply_resource_prewrite(&mut psd);
crate::document_resource_postprocess::apply_document_prewrite(&mut psd)?;
```

- [ ] **Step 6: Run the clipping parity test again**

Run:

```bash
cargo test roundtrip_resource_1026_maps_layer_clipping -- --nocapture
```

Expected:
- PASS

- [ ] **Step 7: Commit**

```bash
git add src/image_resources.rs src/document_resource_postprocess.rs src/reader.rs src/writer.rs tests/ts_parity_test.rs
git commit -m "fix: map resource 1026 to layer clipping parity"
```

---

### Task 3: Add TS-equivalent typed parsing and prewrite for `7000`, `7001`, `1065`, `1074`, `1075`, `1073`, and `1036`

**Files:**
- Modify: `src/image_resources.rs`
- Modify: `src/document_resource_postprocess.rs`
- Modify: `src/psd.rs`
- Modify: `tests/ts_parity_test.rs`
- Test: `tests/ts_parity_test.rs`

- [ ] **Step 1: Write the failing targeted tests**

Add these tests to `tests/ts_parity_test.rs`:

```rust
#[test]
fn roundtrip_variables_and_data_sets_are_typed() {
    let mut psd = Psd::default();
    psd.width = 1;
    psd.height = 1;
    psd.channels = Some(4);
    psd.bits_per_channel = Some(8);
    psd.color_mode = Some(ColorMode::RGB);
    psd.variable_sets = Some(vec![psd_great::psd::VariableSet {
        var_name: Some("title".to_string()),
        trait_name: Some("textcontent".to_string()),
        doc_ref: Some("doc".to_string()),
        placement_method: None,
        align: None,
        valign: None,
        clip: None,
    }]);
    psd.data_sets = Some(vec![
        vec!["title".to_string()],
        vec!["Hello".to_string()],
    ]);

    let bytes = write_psd(&psd, &WriteOptions::default()).expect("write");
    let reparsed = read_psd(Cursor::new(&bytes), ReadOptions::default()).expect("read");

    assert_eq!(reparsed.variable_sets, psd.variable_sets);
    assert_eq!(reparsed.data_sets, psd.data_sets);
}

#[test]
fn roundtrip_display_info_and_custom_points_are_typed() {
    let mut psd = Psd::default();
    psd.width = 1;
    psd.height = 1;
    psd.channels = Some(4);
    psd.bits_per_channel = Some(8);
    psd.color_mode = Some(ColorMode::RGB);
    psd.display_info = Some(psd_great::psd::DisplayInfo {
        h_res_unit: 1,
        v_res_unit: 2,
        width_unit: 3,
        height_unit: 4,
    });
    psd.custom_points = Some(vec![psd_great::psd::CustomPoint { x: 1.5, y: 2.5 }]);

    let bytes = write_psd(&psd, &WriteOptions::default()).expect("write");
    let reparsed = read_psd(Cursor::new(&bytes), ReadOptions::default()).expect("read");

    assert_eq!(reparsed.display_info, psd.display_info);
    assert_eq!(reparsed.custom_points, psd.custom_points);
}
```

- [ ] **Step 2: Run the new tests to verify they fail**

Run:

```bash
cargo test roundtrip_variables_and_data_sets_are_typed -- --nocapture
cargo test roundtrip_display_info_and_custom_points_are_typed -- --nocapture
```

Expected:
- FAIL because the TS-equivalent typed mapping does not exist yet

- [ ] **Step 3: Implement typed XML parsing/building and custom-point/display-info codecs in `src/document_resource_postprocess.rs`**

Add focused helpers mirroring TS behavior:

```rust
fn parse_variables_xml(xml: &str) -> Vec<crate::psd::VariableSet> { /* port TS regex behavior */ }

fn build_variables_xml(variables: &[crate::psd::VariableSet]) -> String { /* emit TS-shaped XML */ }

fn parse_data_sets_xml(xml: &str) -> Vec<Vec<String>> { /* header row + sampleDataSet rows */ }

fn build_data_sets_xml(table: &[Vec<String>]) -> String { /* emit TS-shaped XML */ }

fn parse_custom_points(bytes: &[u8]) -> Vec<crate::psd::CustomPoint> { /* version 3 fixed16.16 */ }

fn build_custom_points(points: &[crate::psd::CustomPoint]) -> Vec<u8> { /* 14-byte entries */ }

fn parse_display_info(bytes: &[u8]) -> Option<crate::psd::DisplayInfo> { /* units at offsets 2/6/10/14 */ }

fn build_display_info(info: &crate::psd::DisplayInfo) -> Vec<u8> { /* 28-byte TS-shaped payload */ }
```

- [ ] **Step 4: Implement read-side document postprocess mappings**

Extend `apply_document_postprocess` with:

```rust
if let Some(resources) = psd.image_resources.as_ref() {
    if let Some(xml) = resources.variables.as_ref() {
        psd.variable_sets = Some(parse_variables_xml(xml));
    }
    if let Some(xml) = resources.data_sets.as_ref() {
        psd.data_sets = Some(parse_data_sets_xml(xml));
    }
    if let Some(desc) = resources.descriptor_resources.get(&1065) {
        psd.descriptor_1065 = Some(desc.clone());
    }
    if let Some(desc) = resources.descriptor_resources.get(&1074) {
        psd.descriptor_1074 = Some(desc.clone());
    }
    if let Some(desc) = resources.descriptor_resources.get(&1075) {
        psd.descriptor_1075 = Some(desc.clone());
    }
    if let Some(bytes) = resources.custom_points.as_ref() {
        psd.custom_points = Some(parse_custom_points(bytes));
    }
    if let Some(bytes) = resources.display_info.as_ref() {
        psd.display_info = parse_display_info(bytes);
    }
}
```

- [ ] **Step 5: Implement write-side document prewrite mappings**

Extend `apply_document_prewrite` with:

```rust
let resources = psd.image_resources.get_or_insert_with(Default::default);

if let Some(variable_sets) = psd.variable_sets.as_ref() {
    resources.variables = Some(build_variables_xml(variable_sets));
}
if let Some(data_sets) = psd.data_sets.as_ref() {
    resources.data_sets = Some(build_data_sets_xml(data_sets));
}
if let Some(desc) = psd.descriptor_1065.as_ref() {
    resources.descriptor_resources.insert(1065, desc.clone());
}
if let Some(desc) = psd.descriptor_1074.as_ref() {
    resources.descriptor_resources.insert(1074, desc.clone());
}
if let Some(desc) = psd.descriptor_1075.as_ref() {
    resources.descriptor_resources.insert(1075, desc.clone());
}
if let Some(points) = psd.custom_points.as_ref() {
    resources.custom_points = Some(build_custom_points(points));
}
if let Some(info) = psd.display_info.as_ref() {
    resources.display_info = Some(build_display_info(info));
}
```

- [ ] **Step 6: Add the missing low-level resource fields to `src/image_resources.rs`**

Add:

```rust
pub custom_points: Option<Vec<u8>>,
pub display_info: Option<Vec<u8>>,
```

Add dispatch:

```rust
1036 => {
    resources.display_info = Some(reader.read_bytes(data_length)?);
}
1073 => {
    resources.custom_points = Some(reader.read_bytes(data_length)?);
}
```

And writer output:

```rust
if let Some(ref bytes) = resources.display_info {
    write_resource(writer, 1036, &|w| w.write_bytes(bytes))?;
}
if let Some(ref bytes) = resources.custom_points {
    write_resource(writer, 1073, &|w| w.write_bytes(bytes))?;
}
```

- [ ] **Step 7: Run the targeted tests again**

Run:

```bash
cargo test roundtrip_variables_and_data_sets_are_typed -- --nocapture
cargo test roundtrip_display_info_and_custom_points_are_typed -- --nocapture
cargo test roundtrip_document_resource_postprocess_fields -- --nocapture
```

Expected:
- PASS

- [ ] **Step 8: Commit**

```bash
git add src/image_resources.rs src/document_resource_postprocess.rs src/psd.rs tests/ts_parity_test.rs
git commit -m "feat: add typed document resource postprocess parity"
```

---

### Task 4: Add TS-equivalent resource visibility handling for `1072`

**Files:**
- Modify: `src/layer.rs`
- Modify: `src/image_resources.rs`
- Modify: `src/document_resource_postprocess.rs`
- Modify: `tests/ts_parity_test.rs`
- Test: `tests/ts_parity_test.rs`

- [ ] **Step 1: Write the failing resource visibility parity test**

Add this test:

```rust
#[test]
fn roundtrip_resource_visibility_1072_maps_layers() {
    let mut layer_a = Layer::default();
    layer_a.top = Some(0);
    layer_a.left = Some(0);
    layer_a.bottom = Some(1);
    layer_a.right = Some(1);
    layer_a.resource_visible = Some(true);

    let mut layer_b = layer_a.clone();
    layer_b.resource_visible = Some(false);

    let mut psd = Psd::default();
    psd.width = 1;
    psd.height = 1;
    psd.channels = Some(4);
    psd.bits_per_channel = Some(8);
    psd.color_mode = Some(ColorMode::RGB);
    psd.children = Some(vec![layer_a, layer_b]);

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

    let layers = reparsed.children.expect("layers");
    assert_eq!(layers[0].resource_visible, Some(true));
    assert_eq!(layers[1].resource_visible, Some(false));
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run:

```bash
cargo test roundtrip_resource_visibility_1072_maps_layers -- --nocapture
```

Expected:
- FAIL because `Layer` does not yet expose `resource_visible` and Rust ignores `1072`

- [ ] **Step 3: Add the typed field and the raw resource storage**

In `src/layer.rs`, add:

```rust
#[serde(rename = "resourceVisible")]
pub resource_visible: Option<bool>,
```

In `src/image_resources.rs`, add:

```rust
pub resource_visibility: Option<Vec<u8>>,
```

Add dispatch and write output:

```rust
1072 => {
    resources.resource_visibility = Some(reader.read_bytes(data_length)?);
}
```

```rust
if let Some(ref bytes) = resources.resource_visibility {
    write_resource(writer, 1072, &|w| w.write_bytes(bytes))?;
}
```

- [ ] **Step 4: Map `1072` through document postprocess and prewrite**

In `apply_document_postprocess`:

```rust
if let (Some(bytes), Some(layers)) = (
    resources.resource_visibility.as_ref(),
    psd.children.as_mut(),
) {
    for (layer, value) in layers.iter_mut().zip(bytes.iter()) {
        layer.resource_visible = Some(*value == 1);
    }
}
```

In `apply_document_prewrite`:

```rust
if let Some(layers) = psd.children.as_ref() {
    let values: Vec<u8> = layers
        .iter()
        .map(|layer| if layer.resource_visible == Some(false) { 0 } else { 1 })
        .collect();
    if values.iter().any(|value| *value == 0) {
        let resources = psd.image_resources.get_or_insert_with(Default::default);
        resources.resource_visibility = Some(values);
    }
}
```

- [ ] **Step 5: Run the test again**

Run:

```bash
cargo test roundtrip_resource_visibility_1072_maps_layers -- --nocapture
```

Expected:
- PASS

- [ ] **Step 6: Commit**

```bash
git add src/layer.rs src/image_resources.rs src/document_resource_postprocess.rs tests/ts_parity_test.rs
git commit -m "feat: add resource visibility parity"
```

---

### Task 5: Run the final parity audit and lock it in with regression coverage

**Files:**
- Modify: `tests/ts_parity_test.rs`
- Modify: `docs/superpowers/plans/2026-05-27-ts-document-resource-parity.md`
- Test: `tests/ts_parity_test.rs`

- [ ] **Step 1: Add a single end-to-end regression test that combines the remaining document resource features**

Add this test:

```rust
#[test]
fn roundtrip_combined_document_resource_parity() {
    let mut layer_a = Layer::default();
    layer_a.top = Some(0);
    layer_a.left = Some(0);
    layer_a.bottom = Some(1);
    layer_a.right = Some(1);
    layer_a.clipping = Some(0);
    layer_a.resource_visible = Some(true);

    let mut layer_b = layer_a.clone();
    layer_b.clipping = Some(1);
    layer_b.resource_visible = Some(false);

    let mut psd = Psd::default();
    psd.width = 1;
    psd.height = 1;
    psd.channels = Some(4);
    psd.bits_per_channel = Some(8);
    psd.color_mode = Some(ColorMode::RGB);
    psd.children = Some(vec![layer_a, layer_b]);
    psd.variable_sets = Some(vec![psd_great::psd::VariableSet {
        var_name: Some("title".to_string()),
        trait_name: Some("textcontent".to_string()),
        doc_ref: None,
        placement_method: None,
        align: None,
        valign: None,
        clip: None,
    }]);
    psd.data_sets = Some(vec![
        vec!["title".to_string()],
        vec!["Hello".to_string()],
    ]);
    psd.custom_points = Some(vec![psd_great::psd::CustomPoint { x: 4.0, y: 8.0 }]);
    psd.display_info = Some(psd_great::psd::DisplayInfo {
        h_res_unit: 1,
        v_res_unit: 1,
        width_unit: 1,
        height_unit: 1,
    });

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

    let layers = reparsed.children.as_ref().expect("layers");
    assert_eq!(layers[0].clipping, Some(0));
    assert_eq!(layers[1].clipping, Some(1));
    assert_eq!(layers[0].resource_visible, Some(true));
    assert_eq!(layers[1].resource_visible, Some(false));
    assert_eq!(reparsed.variable_sets, psd.variable_sets);
    assert_eq!(reparsed.data_sets, psd.data_sets);
    assert_eq!(reparsed.custom_points, psd.custom_points);
    assert_eq!(reparsed.display_info, psd.display_info);
}
```

- [ ] **Step 2: Run the focused parity tests**

Run:

```bash
cargo test roundtrip_resource_1026_maps_layer_clipping -- --nocapture
cargo test roundtrip_resource_visibility_1072_maps_layers -- --nocapture
cargo test roundtrip_variables_and_data_sets_are_typed -- --nocapture
cargo test roundtrip_display_info_and_custom_points_are_typed -- --nocapture
cargo test roundtrip_combined_document_resource_parity -- --nocapture
```

Expected:
- PASS on all targeted tests

- [ ] **Step 3: Run the full Rust suite**

Run:

```bash
cargo test -- --nocapture
```

Expected:
- All unit tests pass
- All integration tests pass
- All TS parity tests pass

- [ ] **Step 4: Update the plan file with completion notes**

Append this section to the bottom of this plan file once implementation is complete:

```markdown
## Completion Notes

- Resource `1026` now maps to per-layer clipping values, matching TS `document-postprocess.ts`.
- Rust now implements typed document postprocess/prewrite coverage for `7000`, `7001`, `1065`, `1072`, `1073`, `1074`, `1075`, and `1036`.
- Final verification command: `cargo test -- --nocapture`
```

- [ ] **Step 5: Commit**

```bash
git add tests/ts_parity_test.rs docs/superpowers/plans/2026-05-27-ts-document-resource-parity.md
git commit -m "test: lock in remaining document resource parity"
```
