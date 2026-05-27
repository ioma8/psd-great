# Full Model Consolidation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Collapse the remaining duplicate PSD model types so the crate exposes one canonical public model per PSD concept, with no internal-vs-external wrappers and no duplicate public type names for the same domain object.

**Architecture:** Keep the current live parser/writer path centered on `Psd`, `Layer`, `additional_info::LayerAdditionalInfo`, and `image_resources::ImageResources`, then fold all remaining duplicate types into those canonical homes. Use TDD with a broad duplicate-type audit plus focused behavior tests after each model family migration so the refactor stays honest and parity-safe.

**Tech Stack:** Rust, Cargo, existing crate test suite, `apply_patch`, existing PSD parser/writer modules

---

## File Responsibilities

- `src/lib.rs`
  - Crate-root re-exports. After consolidation, this should expose only the canonical types for each PSD concept.

- `src/types.rs`
  - Primitive shared domain types. Canonical home for cross-cutting atoms like `Point`, `Fraction`, `RenderingIntent`, `LayerColor`, code wrappers, and color/value enums.

- `src/adjustments.rs`
  - Canonical home for `AdjustmentLayer`. `src/layer.rs` should stop duplicating this enum.

- `src/image_resources.rs`
  - Canonical home for document image-resource domain types: `ImageResources`, `ResolutionInfo`, `PrintInformation`, `PrintFlags`, `PrintScale`, `ProofSetup`, `Slice`, `LayerComps`, `OnionSkins`, `Timeline`, `Bounds`, and resource-only support structs.

- `src/additional_info.rs`
  - Canonical home for tagged-block-specific layer/document metadata that is actually stored in PSD additional-info blocks. Any overlapping types in `src/layer.rs` should be removed or replaced with aliases to this module.

- `src/layer.rs`
  - Canonical home for `Layer` and layer-only support structs that are not really tagged-block models. After consolidation it should stop defining duplicate concept types that already live in `src/additional_info.rs`, `src/image_resources.rs`, `src/adjustments.rs`, or `src/types.rs`.

- `src/psd.rs`
  - Canonical home for `Psd` and truly document-only structs. It should not redefine image resource or shared primitives.

- `src/reader.rs`
  - Parser wiring into the canonical model. No shape conversion layers should remain after consolidation.

- `src/writer.rs`
  - Writer wiring out of the canonical model. No shadow model translation should remain after consolidation.

- `tests/integration_test.rs`
  - Broad API/model integration tests, including the duplicate-public-type audit.

- `tests/ts_parity_test.rs`
  - TS reference parity tests. These are the guardrail for behavioral equivalence while consolidating models.

- `examples/*.rs`
  - Canonical public API examples. These must compile against the consolidated single model.

---

### Task 1: Broaden the Duplicate-Type Audit

**Files:**
- Modify: `tests/integration_test.rs`
- Test: `tests/integration_test.rs`

- [ ] **Step 1: Write the failing audit test for all remaining duplicate public model names**

Add this test near the existing duplicate-type audit in `tests/integration_test.rs`:

```rust
#[test]
fn test_no_remaining_duplicate_public_model_type_names() {
    let source_files = [
        ("src/additional_info.rs", include_str!("../src/additional_info.rs")),
        ("src/adjustments.rs", include_str!("../src/adjustments.rs")),
        ("src/image_resources.rs", include_str!("../src/image_resources.rs")),
        ("src/layer.rs", include_str!("../src/layer.rs")),
        ("src/psd.rs", include_str!("../src/psd.rs")),
        ("src/types.rs", include_str!("../src/types.rs")),
    ];

    let audited_names = [
        "AdjustmentLayer",
        "Bounds",
        "Fraction",
        "LayerComps",
        "OnionSkins",
        "PlacedLayer",
        "Point",
        "PrintScale",
        "ProofSetup",
        "RenderingIntent",
        "SectionDivider",
        "Slice",
        "Timeline",
        "VectorStroke",
    ];

    let mut duplicates = Vec::new();

    for name in audited_names {
        let mut hits = Vec::new();
        for (path, contents) in &source_files {
            let needle_struct = format!("pub struct {}", name);
            let needle_enum = format!("pub enum {}", name);
            let needle_type = format!("pub type {}", name);
            if contents.contains(&needle_struct)
                || contents.contains(&needle_enum)
                || contents.contains(&needle_type)
            {
                hits.push(*path);
            }
        }
        if hits.len() > 1 {
            duplicates.push((name, hits));
        }
    }

    assert!(
        duplicates.is_empty(),
        "duplicate public model types remain: {:?}",
        duplicates
    );
}
```

- [ ] **Step 2: Run the audit test to verify it fails**

Run:

```bash
cargo test test_no_remaining_duplicate_public_model_type_names -- --nocapture
```

Expected: FAIL with duplicate names including `AdjustmentLayer`, `Point`, `PlacedLayer`, `VectorStroke`, `SectionDivider`, `PrintScale`, `ProofSetup`, `LayerComps`, `OnionSkins`, `Slice`, `Timeline`, `Fraction`, `Bounds`, or `RenderingIntent`.

- [ ] **Step 3: Commit the failing audit before implementation**

```bash
git add tests/integration_test.rs
git commit -m "test: broaden duplicate model audit"
```

---

### Task 2: Canonicalize Shared Primitive Types in `src/types.rs`

**Files:**
- Modify: `src/types.rs`
- Modify: `src/image_resources.rs`
- Modify: `src/additional_info.rs`
- Modify: `src/layer.rs`
- Modify: `src/lib.rs`
- Test: `tests/integration_test.rs`

- [ ] **Step 1: Write a focused failing test that proves canonical shared types are used**

Add this to `tests/integration_test.rs`:

```rust
#[test]
fn test_canonical_shared_types_are_constructible_from_public_api() {
    let point = Point { x: 1.0, y: 2.0 };
    let fraction = Fraction {
        numerator: 1,
        denominator: 24,
    };
    let rendering_intent = RenderingIntent::Perceptual;

    assert_eq!(point.x, 1.0);
    assert_eq!(fraction.denominator, 24);
    assert_eq!(rendering_intent, RenderingIntent::Perceptual);
}
```

- [ ] **Step 2: Run the focused test and the broad audit**

Run:

```bash
cargo test test_canonical_shared_types_are_constructible_from_public_api -- --nocapture
cargo test test_no_remaining_duplicate_public_model_type_names -- --nocapture
```

Expected: first test passes already, second still fails. This locks behavior before type moves.

- [ ] **Step 3: Replace duplicate shared primitive definitions with canonical imports**

Make these code changes:

In `src/image_resources.rs`, replace local duplicates with canonical aliases/imports:

```rust
use crate::types::{Fraction, Point, RenderingIntent};

pub type Bounds = crate::additional_info::Bounds;
```

Then delete the local public definitions for:

```rust
pub struct Point { ... }
pub struct Fraction { ... }
pub enum RenderingIntent { ... }
pub struct Bounds { ... }
```

In `src/additional_info.rs`, stop defining local `Point` and use `crate::types::Point`:

```rust
use crate::types::Point;
```

Delete:

```rust
pub struct Point {
    pub x: f64,
    pub y: f64,
}
```

Update any explicit `additional_info::Point` construction to `crate::types::Point`.

- [ ] **Step 4: Run the shared-type test and duplicate audit**

Run:

```bash
cargo test test_canonical_shared_types_are_constructible_from_public_api -- --nocapture
cargo test test_no_remaining_duplicate_public_model_type_names -- --nocapture
```

Expected: shared-type test PASS, duplicate audit now reports fewer names, and `Point` / `Fraction` / `RenderingIntent` / `Bounds` are no longer duplicated.

- [ ] **Step 5: Commit the shared-type consolidation**

```bash
git add src/types.rs src/image_resources.rs src/additional_info.rs src/layer.rs src/lib.rs tests/integration_test.rs
git commit -m "refactor: consolidate shared primitive model types"
```

---

### Task 3: Canonicalize `AdjustmentLayer`

**Files:**
- Modify: `src/layer.rs`
- Modify: `src/lib.rs`
- Test: `tests/integration_test.rs`
- Test: `tests/ts_parity_test.rs`

- [ ] **Step 1: Write a failing compile-level test that uses the canonical `AdjustmentLayer`**

Add to `tests/integration_test.rs`:

```rust
#[test]
fn test_adjustment_layer_comes_from_canonical_module() {
    let adjustment = AdjustmentLayer::Invert;
    match adjustment {
        AdjustmentLayer::Invert => {}
        _ => panic!("wrong adjustment layer variant"),
    }
}
```

- [ ] **Step 2: Run the focused test and duplicate audit**

Run:

```bash
cargo test test_adjustment_layer_comes_from_canonical_module -- --nocapture
cargo test test_no_remaining_duplicate_public_model_type_names -- --nocapture
```

Expected: first test PASS, duplicate audit still FAIL because `src/layer.rs` still exports a second `AdjustmentLayer`.

- [ ] **Step 3: Delete the duplicate `AdjustmentLayer` enum from `src/layer.rs` and re-export the canonical one**

At the top of `src/layer.rs`, ensure:

```rust
use crate::adjustments::AdjustmentLayer;
```

Delete the local public enum:

```rust
pub enum AdjustmentLayer { ... }
```

Leave all field types referring to `AdjustmentLayer` unchanged so they now resolve to `crate::adjustments::AdjustmentLayer`.

- [ ] **Step 4: Run focused tests plus one adjustment parity test**

Run:

```bash
cargo test test_adjustment_layer_comes_from_canonical_module -- --nocapture
cargo test test_no_remaining_duplicate_public_model_type_names -- --nocapture
cargo test test_levels_roundtrip -- --nocapture
```

Expected: all PASS, duplicate audit reports one fewer family.

- [ ] **Step 5: Commit the adjustment-layer consolidation**

```bash
git add src/layer.rs src/lib.rs tests/integration_test.rs
git commit -m "refactor: canonicalize adjustment layer model"
```

---

### Task 4: Canonicalize Tagged-Block Layer Subtypes

**Files:**
- Modify: `src/layer.rs`
- Modify: `src/additional_info.rs`
- Modify: `src/lib.rs`
- Modify: `examples/create_psd.rs`
- Test: `tests/ts_parity_test.rs`
- Test: `tests/integration_test.rs`

- [ ] **Step 1: Write a focused failing test for canonical tagged-block types**

Add to `tests/integration_test.rs`:

```rust
#[test]
fn test_canonical_tagged_block_types_are_used_by_layer_additional_info() {
    let divider = psd_great::additional_info::SectionDivider {
        divider_type: SectionDividerType::OpenFolder,
        blend_mode: None,
        sub_type: None,
    };
    let stroke = psd_great::additional_info::VectorStroke {
        version: 16,
        descriptor: psd_great::descriptor::Descriptor {
            name: String::new(),
            class_id: "null".to_string(),
            items: std::collections::HashMap::new(),
        },
    };
    let placed = psd_great::additional_info::PlacedLayer {
        id: "id".to_string(),
        page: None,
        total_pages: None,
        anti_alias_policy: None,
        placed_layer_type: None,
        transform: vec![1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
        warp: None,
        placed: None,
    };

    let info = LayerAdditionalInfo {
        section_divider: Some(divider),
        vector_stroke: Some(stroke),
        placed_layer: Some(placed),
        ..Default::default()
    };

    assert!(info.section_divider.is_some());
    assert!(info.vector_stroke.is_some());
    assert!(info.placed_layer.is_some());
}
```

- [ ] **Step 2: Run the focused test and the duplicate audit**

Run:

```bash
cargo test test_canonical_tagged_block_types_are_used_by_layer_additional_info -- --nocapture
cargo test test_no_remaining_duplicate_public_model_type_names -- --nocapture
```

Expected: first test PASS, duplicate audit still FAIL because `src/layer.rs` still exports `PlacedLayer`, `VectorStroke`, and `SectionDivider`.

- [ ] **Step 3: Remove duplicate tagged-block types from `src/layer.rs` and use canonical imports**

Delete these duplicate public definitions from `src/layer.rs`:

```rust
pub struct PlacedLayer { ... }
pub struct VectorStroke { ... }
pub struct SectionDivider { ... }
```

At the top of `src/layer.rs`, import the canonical versions:

```rust
use crate::additional_info::{PlacedLayer, SectionDivider, VectorStroke};
```

Update `src/lib.rs` re-exports to stop exporting the `layer` copies and export the canonical ones:

```rust
pub use additional_info::{PlacedLayer, SectionDivider, VectorStroke};
```

- [ ] **Step 4: Run tagged-block parity tests**

Run:

```bash
cargo test roundtrip_vscg_matches_vstk_wrapped_descriptor -- --nocapture
cargo test roundtrip_plld_semantic_descriptor -- --nocapture
cargo test test_canonical_tagged_block_types_are_used_by_layer_additional_info -- --nocapture
cargo test test_no_remaining_duplicate_public_model_type_names -- --nocapture
```

Expected: all PASS, duplicate audit reports those three families removed.

- [ ] **Step 5: Commit the tagged-block type consolidation**

```bash
git add src/layer.rs src/additional_info.rs src/lib.rs examples/create_psd.rs tests/integration_test.rs tests/ts_parity_test.rs
git commit -m "refactor: consolidate tagged block layer subtypes"
```

---

### Task 5: Canonicalize Image Resource Domain Types

**Files:**
- Modify: `src/psd.rs`
- Modify: `src/image_resources.rs`
- Modify: `src/lib.rs`
- Test: `tests/integration_test.rs`
- Test: `tests/ts_parity_test.rs`

- [ ] **Step 1: Write a focused failing test that exercises canonical image resource types**

Add to `tests/integration_test.rs`:

```rust
#[test]
fn test_canonical_image_resource_types_are_public() {
    let resources = ImageResources {
        resolution_info: Some(psd_great::image_resources::ResolutionInfo {
            horizontal_res: 300.0,
            horizontal_res_unit: psd_great::image_resources::ResolutionUnit::PixelsPerInch,
            width_unit: psd_great::image_resources::MeasurementUnit::Inches,
            vertical_res: 300.0,
            vertical_res_unit: psd_great::image_resources::ResolutionUnit::PixelsPerInch,
            height_unit: psd_great::image_resources::MeasurementUnit::Inches,
        }),
        print_scale: Some(psd_great::image_resources::PrintScale {
            style: Some(PsdStringCode::from("centered")),
            x: Some(0.0),
            y: Some(0.0),
            scale: Some(100.0),
        }),
        ..Default::default()
    };

    assert!(resources.resolution_info.is_some());
    assert!(resources.print_scale.is_some());
}
```

- [ ] **Step 2: Run the focused test and the duplicate audit**

Run:

```bash
cargo test test_canonical_image_resource_types_are_public -- --nocapture
cargo test test_no_remaining_duplicate_public_model_type_names -- --nocapture
```

Expected: first test PASS, duplicate audit still FAIL for `PrintScale`, `ProofSetup`, `LayerComps`, `OnionSkins`, `Slice`, and `Timeline`.

- [ ] **Step 3: Remove duplicate document-resource model structs from `src/psd.rs`**

Delete these duplicate public types from `src/psd.rs`:

```rust
pub struct PrintScale { ... }
pub enum ProofSetup { ... }
pub struct LayerComps { ... }
pub struct OnionSkins { ... }
pub struct SlicesInfo { ... } // if this is the duplicate Slice family root
pub struct TimelineInformation { ... } // if this is the duplicate Timeline family root
```

Then update `Psd` or document-only structs to reference the canonical `src/image_resources.rs` types directly:

```rust
pub artboards: Option<ArtboardsInfo>,
pub image_resources: Option<crate::image_resources::ImageResources>,
```

and for any embedded references:

```rust
pub print_scale: Option<crate::image_resources::PrintScale>;
pub onion_skins: Option<crate::image_resources::OnionSkins>;
pub layer_comps: Option<crate::image_resources::LayerComps>;
```

Use the actual canonical names already defined in `src/image_resources.rs`.

- [ ] **Step 4: Run image resource parity tests**

Run:

```bash
cargo test preserve_image_resources_on_roundtrip -- --nocapture
cargo test roundtrip_combined_document_resource_parity -- --nocapture
cargo test test_canonical_image_resource_types_are_public -- --nocapture
cargo test test_no_remaining_duplicate_public_model_type_names -- --nocapture
```

Expected: all PASS, duplicate audit no longer reports those image-resource families.

- [ ] **Step 5: Commit the image resource consolidation**

```bash
git add src/psd.rs src/image_resources.rs src/lib.rs tests/integration_test.rs tests/ts_parity_test.rs
git commit -m "refactor: consolidate image resource model types"
```

---

### Task 6: Canonicalize Remaining Overlap Types or Rename Truly Different Concepts

**Files:**
- Modify: `src/image_resources.rs`
- Modify: `src/psd.rs`
- Modify: `src/layer.rs`
- Modify: `src/additional_info.rs`
- Modify: `src/lib.rs`
- Test: `tests/integration_test.rs`

- [ ] **Step 1: Decide per remaining duplicate whether it is the same concept or a naming collision**

Use this rule:

```text
Same PSD concept -> one canonical public type name
Different PSD concepts that only share a vague name -> rename one side to a precise domain name
```

The remaining likely cases are:

```text
Bounds
LayerComps
OnionSkins
PrintScale
ProofSetup
SectionDivider
Slice
Timeline
```

- [ ] **Step 2: Apply precise renames where a concept is actually different**

Examples of allowed precise renames if the underlying concepts are not the same object:

```rust
pub struct VectorPathBounds { ... }
pub struct SliceResource { ... }
pub struct TimelineResource { ... }
```

Do not leave a duplicate public name in place just because the types are “kind of related.”

- [ ] **Step 3: Run the broad duplicate audit until it is completely clean**

Run:

```bash
cargo test test_no_duplicate_public_type_names_for_canonical_models -- --nocapture
cargo test test_no_remaining_duplicate_public_model_type_names -- --nocapture
```

Expected: both PASS with no duplicate public type names left in the audited source files.

- [ ] **Step 4: Commit the final naming/type cleanup**

```bash
git add src/image_resources.rs src/psd.rs src/layer.rs src/additional_info.rs src/lib.rs tests/integration_test.rs
git commit -m "refactor: eliminate remaining duplicate public model names"
```

---

### Task 7: Remove Any Remaining Serialization Coupling that Reintroduces Shadow Shapes

**Files:**
- Modify: `Cargo.toml`
- Modify: `src/README.md`
- Modify: `README.md`
- Modify: `examples/basic_usage.rs`
- Test: `tests/integration_test.rs`

- [ ] **Step 1: Write a regression test proving the canonical API no longer depends on serializing `Psd`**

Add to `tests/integration_test.rs`:

```rust
#[test]
fn test_psd_roundtrip_does_not_depend_on_serde() {
    let psd = Psd {
        width: 4,
        height: 4,
        channels: Some(4),
        bits_per_channel: Some(8),
        color_mode: Some(ColorMode::RGB),
        ..Default::default()
    };

    let bytes = write_psd(&psd, &WriteOptions::default()).expect("write");
    let reparsed = read_psd(std::io::Cursor::new(bytes), ReadOptions::default()).expect("read");
    assert_eq!(reparsed.width, 4);
    assert_eq!(reparsed.height, 4);
}
```

- [ ] **Step 2: Run the focused test**

Run:

```bash
cargo test test_psd_roundtrip_does_not_depend_on_serde -- --nocapture
```

Expected: PASS.

- [ ] **Step 3: If `serde` is no longer needed for the document model, make it optional or remove it**

In `Cargo.toml`, if only helper value types still need serde, gate it behind a feature:

```toml
[features]
default = []
serde = ["dep:serde", "dep:serde_json"]

[dependencies]
serde = { version = "1.0", features = ["derive"], optional = true }
serde_json = { version = "1.0", optional = true }
```

If you keep serde for small leaf types, explicitly document that it is not part of the main PSD document model shape anymore.

- [ ] **Step 4: Update docs and examples to describe the single canonical model**

Replace any doc text like:

```text
flattened additional info
JSON representation of Psd
outer/public model
internal tagged blocks
```

with:

```text
single canonical PSD document model
typed parser/writer API
document serialization is not the primary contract
```

- [ ] **Step 5: Commit the serialization cleanup**

```bash
git add Cargo.toml README.md src/README.md examples/basic_usage.rs tests/integration_test.rs
git commit -m "docs: align serde and docs with canonical PSD model"
```

---

### Task 8: Final Verification Sweep

**Files:**
- Modify: `tests/integration_test.rs` (only if verification reveals a missing audit case)
- Test: full repo

- [ ] **Step 1: Run the duplicate audits**

Run:

```bash
cargo test test_no_duplicate_public_type_names_for_canonical_models -- --nocapture
cargo test test_no_remaining_duplicate_public_model_type_names -- --nocapture
```

Expected: both PASS.

- [ ] **Step 2: Run the full crate test suite**

Run:

```bash
cargo test -- --nocapture
```

Expected:

```text
85 unit tests passed
18 integration tests passed
47 TS parity tests passed
```

Adjust the numbers only if the suite legitimately grows during implementation.

- [ ] **Step 3: Run the quiet suite to check for warning regressions**

Run:

```bash
cargo test -q
```

Expected: PASS with no dead-code warnings from removed shadow model types.

- [ ] **Step 4: Commit the final verification state**

```bash
git add .
git commit -m "refactor: finish full PSD model consolidation"
```

---

## Self-Review

- Spec coverage:
  - One canonical `Psd` / `Layer` path: already preserved and verified during Tasks 4-8
  - No duplicate public models: covered by Tasks 1, 2, 3, 4, 5, 6, and final audits in Task 8
  - No internal/external wrappers: covered by Tasks 4, 5, and 7
  - Optional serde simplification: covered by Task 7

- Placeholder scan:
  - No `TBD`, `TODO`, or “appropriate handling” placeholders remain
  - Each code step contains concrete test code, code shapes, and commands

- Type consistency:
  - Canonical homes are fixed up front
  - All later tasks refer back to those canonical homes consistently

