# Typed Public API And Fixed-Layout Binrw Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace avoidable magic strings and numbers in the public API with typed semantic values, and convert fixed-layout binary records to `binrw` where it clearly improves correctness and maintainability.

**Architecture:** Split the work into two coordinated layers. First, tighten the public semantic API so known PSD concepts are represented by enums and structs instead of raw wrapper codes. Second, move fixed-layout binary fragments behind `binrw` record structs while keeping variable-layout parsing manual and preserving the existing high-level reader/writer control flow.

**Tech Stack:** Rust, `cargo test`, `binrw`, existing `binrw_support.rs`, PSD reader/writer modules

---

### Task 1: Replace Public Magic-Value Hotspots With Semantic Types

**Files:**
- Modify: `src/types.rs`
- Modify: `src/psd.rs`
- Modify: `src/layer.rs`
- Modify: `src/lib.rs`
- Test: `tests/integration_test.rs`
- Test: `tests/ts_parity_test.rs`

- [ ] **Step 1: Write the failing public-API typing tests**

Add tests that prove callers can use semantic values instead of raw code wrappers for the first cleanup batch:

```rust
#[test]
fn linked_file_kind_is_typed_publicly() {
    use psd_great::{LinkedFileDataKind, LinkedFile};

    let file = LinkedFile {
        id: "id".into(),
        name: "name".into(),
        data_kind: Some(LinkedFileDataKind::External),
        ..Default::default()
    };

    assert_eq!(file.data_kind, Some(LinkedFileDataKind::External));
}

#[test]
fn guide_direction_and_display_units_are_typed_publicly() {
    use psd_great::{DisplayUnit, GuideDirection, GuideInfo, Psd};

    let guide = GuideInfo {
        location: 10.0,
        direction: GuideDirection::Vertical,
    };
    let mut psd = Psd::default();
    psd.display_info = Some(psd_great::DisplayInfo {
        h_res_unit: DisplayUnit::PixelsPerInch,
        v_res_unit: DisplayUnit::PixelsPerCentimeter,
        width_unit: DisplayUnit::Inches,
        height_unit: DisplayUnit::Centimeters,
    });

    assert_eq!(guide.direction, GuideDirection::Vertical);
    assert_eq!(
        psd.display_info.unwrap().h_res_unit,
        DisplayUnit::PixelsPerInch
    );
}
```

- [ ] **Step 2: Run the targeted typing tests to verify they fail**

Run: `cargo test --quiet linked_file_kind_is_typed_publicly guide_direction_and_display_units_are_typed_publicly`

Expected: FAIL because the public API still uses `PsdStringCode` / `PsdU16Code` / other raw wrapper fields for those concepts.

- [ ] **Step 3: Introduce semantic enums and switch the first public fields over**

Update `src/types.rs`, `src/psd.rs`, `src/layer.rs`, and `src/lib.rs` with explicit semantic types for the known-value public surface:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkedFileDataKind {
    Data,
    External,
    Alias,
    Other([u8; 4]),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuideDirection {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayUnit {
    PixelsPerInch,
    PixelsPerCentimeter,
    Points,
    Picas,
    Columns,
    Inches,
    Centimeters,
    Millimeters,
}
```

Then replace the relevant public fields with those semantic types:

```rust
pub struct GuideInfo {
    pub location: f64,
    pub direction: GuideDirection,
}

pub struct DisplayInfo {
    pub h_res_unit: DisplayUnit,
    pub v_res_unit: DisplayUnit,
    pub width_unit: DisplayUnit,
    pub height_unit: DisplayUnit,
}

pub struct LinkedFile {
    pub data_kind: Option<LinkedFileDataKind>,
    // ...
}
```

Keep raw wrappers only where the field is intentionally open-ended.

- [ ] **Step 4: Add explicit raw-to-semantic mapping helpers**

Implement the raw code conversions in one place, not ad hoc across readers/writers:

```rust
impl LinkedFileDataKind {
    pub fn from_code(code: &str) -> Self {
        match code {
            "liFD" => Self::Data,
            "liFE" => Self::External,
            "liFA" => Self::Alias,
            _ => Self::Other(code.as_bytes().try_into().unwrap_or(*b"????")),
        }
    }

    pub fn to_code(self) -> [u8; 4] {
        match self {
            Self::Data => *b"liFD",
            Self::External => *b"liFE",
            Self::Alias => *b"liFA",
            Self::Other(code) => code,
        }
    }
}
```

Add equivalent conversion helpers for guide directions and display units.

- [ ] **Step 5: Run the targeted public-API tests and affected suites**

Run: `cargo test --quiet linked_file_kind_is_typed_publicly guide_direction_and_display_units_are_typed_publicly`
Expected: PASS

Run: `cargo test --quiet --test integration_test`
Expected: PASS

Run: `cargo test --quiet --test ts_parity_test`
Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add src/types.rs src/psd.rs src/layer.rs src/lib.rs tests/integration_test.rs tests/ts_parity_test.rs
git commit -m "feat: type known public PSD API values"
```

### Task 2: Replace Remaining Known Public Wrapper Fields With Semantic Values

**Files:**
- Modify: `src/additional_info.rs`
- Modify: `src/image_resources.rs`
- Modify: `src/psd.rs`
- Modify: `src/layer.rs`
- Test: `tests/integration_test.rs`
- Test: `tests/ts_parity_test.rs`

- [ ] **Step 1: Write the failing semantic roundtrip tests**

Add tests that force typed public values to serialize to the correct wire codes:

```rust
#[test]
fn typed_linked_file_kind_roundtrips_to_wire_codes() {
    let mut layer = psd_great::Layer::default();
    layer.additional_info.linked_files = Some(psd_great::additional_info::LinkedFilesBlock {
        key: psd_great::PsdStringCode::from("lnkD"),
        items: vec![psd_great::LinkedFile {
            id: "id".into(),
            name: "name".into(),
            data_kind: Some(psd_great::LinkedFileDataKind::External),
            ..Default::default()
        }],
    });

    let mut psd = psd_great::Psd::default();
    psd.width = 1;
    psd.height = 1;
    psd.children = Some(vec![layer]);

    let bytes = psd_great::write_psd(&psd, &psd_great::WriteOptions::default()).unwrap();
    let reparsed = psd_great::read_psd(std::io::Cursor::new(bytes), psd_great::ReadOptions::default()).unwrap();

    assert_eq!(
        reparsed.children.unwrap()[0]
            .additional_info
            .linked_files
            .unwrap()
            .items[0]
            .data_kind,
        Some(psd_great::LinkedFileDataKind::External)
    );
}
```

- [ ] **Step 2: Run the targeted semantic roundtrip test to verify it fails**

Run: `cargo test --quiet typed_linked_file_kind_roundtrips_to_wire_codes`

Expected: FAIL because the reader/writer paths still materialize or expect raw code wrapper values in those public fields.

- [ ] **Step 3: Update readers and writers to use the semantic public values**

Replace public-facing raw wrapper handling in `src/additional_info.rs` and `src/image_resources.rs` with explicit semantic conversion:

```rust
let kind = LinkedFileDataKind::from_code(&self.read_signature()?);
// ...
data_kind: Some(kind),
```

```rust
let kind = item
    .data_kind
    .unwrap_or(LinkedFileDataKind::Data)
    .to_code();
item_writer.write_signature(std::str::from_utf8(&kind).unwrap())?;
```

Do the same for display-unit records, guide direction records, and other closed public domains touched by this pass.

- [ ] **Step 4: Remove stale public raw-code usage from tests and examples**

Update `tests/integration_test.rs` and `tests/ts_parity_test.rs` so the public API is exercised through typed values, not `PsdStringCode("...")` or numeric code wrappers, except where the test explicitly verifies raw passthrough behavior.

- [ ] **Step 5: Run the targeted test and affected suites**

Run: `cargo test --quiet typed_linked_file_kind_roundtrips_to_wire_codes`
Expected: PASS

Run: `cargo test --quiet --test integration_test`
Expected: PASS

Run: `cargo test --quiet --test ts_parity_test`
Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add src/additional_info.rs src/image_resources.rs src/psd.rs src/layer.rs tests/integration_test.rs tests/ts_parity_test.rs
git commit -m "refactor: use semantic values in public PSD models"
```

### Task 3: Convert Fixed-Layout Effect And Additional-Info Records To Binrw

**Files:**
- Modify: `src/binrw_support.rs`
- Modify: `src/effects_helpers.rs`
- Modify: `src/additional_info.rs`
- Test: `tests/integration_test.rs`

- [ ] **Step 1: Write the failing fixed-layout decode tests**

Add tests around specific hand-rolled fixed-layout records that should move to `binrw`:

```rust
#[test]
fn effect_common_state_record_decodes_via_binrw() {
    let bytes = [0, 0, 0, 7, 0, 0, 0, 0, 1, 0, 0];
    let record: psd_great::binrw_support::EffectsCommonStateRecord =
        psd_great::binrw_support::decode_be(&bytes, "effects common state").unwrap();

    assert_eq!(record.size, 7);
    assert_eq!(record.version, 0);
    assert_eq!(record.visible, 1);
}
```

- [ ] **Step 2: Run the targeted decode test to verify it fails where the record is still manual**

Run: `cargo test --quiet effect_common_state_record_decodes_via_binrw`

Expected: FAIL because the current path still decodes the targeted fixed-layout record manually or does not expose a corresponding `binrw` record yet.

- [ ] **Step 3: Add the fixed-layout `binrw` record structs**

Expand `src/binrw_support.rs` with the next fixed-layout records now still decoded manually in `effects_helpers.rs` and `additional_info.rs`:

```rust
#[derive(binrw::BinRead, binrw::BinWrite, Debug, Clone, PartialEq, Eq)]
#[brw(big)]
pub(crate) struct AnnotationHeaderRecord {
    pub major: u16,
    pub minor: u16,
    pub count: u32,
}

#[derive(binrw::BinRead, binrw::BinWrite, Debug, Clone, PartialEq, Eq)]
#[brw(big)]
pub(crate) struct UsingAlignedRenderingRecord {
    pub value: u32,
}
```

Only add records whose field layout is truly static.

- [ ] **Step 4: Route existing fixed-layout reads and writes through the new records**

Replace manual `read_u16` / `read_u32` clusters for those records:

```rust
let record: AnnotationHeaderRecord =
    decode_be(&self.read_bytes(8)?, "annotation header")?;
```

```rust
temp_writer.write_bytes(&encode_be(
    &UsingAlignedRenderingRecord {
        value: if using { 1 } else { 0 },
    },
    "using aligned rendering",
)?)?;
```

Keep variable-layout tails manual.

- [ ] **Step 5: Run the targeted test and affected suites**

Run: `cargo test --quiet effect_common_state_record_decodes_via_binrw`
Expected: PASS

Run: `cargo test --quiet --test integration_test`
Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add src/binrw_support.rs src/effects_helpers.rs src/additional_info.rs tests/integration_test.rs
git commit -m "refactor: use binrw for fixed-layout effect and tagged-block records"
```

### Task 4: Convert Fixed-Layout Image Resource Records To Binrw

**Files:**
- Modify: `src/binrw_support.rs`
- Modify: `src/image_resources.rs`
- Modify: `src/reader.rs`
- Modify: `src/writer.rs`
- Test: `tests/integration_test.rs`
- Test: `tests/ts_parity_test.rs`

- [ ] **Step 1: Write the failing image-resource fixed-record tests**

Add tests for a fixed-layout image-resource record still parsed manually:

```rust
#[test]
fn display_info_fixed_record_roundtrips_via_binrw_record() {
    let record = psd_great::binrw_support::DisplayInfoRecord {
        version: 1,
        h_res_unit: 1,
        width_unit: 2,
        v_res_unit: 3,
        height_unit: 4,
    };

    let bytes = psd_great::binrw_support::encode_be(&record, "display info").unwrap();
    let reparsed: psd_great::binrw_support::DisplayInfoRecord =
        psd_great::binrw_support::decode_be(&bytes, "display info").unwrap();

    assert_eq!(reparsed, record);
}
```

- [ ] **Step 2: Run the targeted image-resource test to verify it fails**

Run: `cargo test --quiet display_info_fixed_record_roundtrips_via_binrw_record`

Expected: FAIL because the fixed-layout record does not exist yet or the path is still fully manual.

- [ ] **Step 3: Add the fixed-layout image-resource records to `binrw_support.rs`**

Add records for static resource payloads that are still hand-assembled:

```rust
#[derive(binrw::BinRead, binrw::BinWrite, Debug, Clone, PartialEq, Eq)]
#[brw(big)]
pub(crate) struct DisplayInfoRecord {
    pub version: u16,
    #[br(map = u16::from_le, map = |v: &u16| v.to_le())]
    pub h_res_unit: u16,
    pub h_width_unit_tag: u16,
    #[br(map = u16::from_le, map = |v: &u16| v.to_le())]
    pub v_res_unit: u16,
    pub v_width_unit_tag: u16,
    #[br(map = u16::from_le, map = |v: &u16| v.to_le())]
    pub width_unit: u16,
    pub width_unit_tag: u16,
    #[br(map = u16::from_le, map = |v: &u16| v.to_le())]
    pub height_unit: u16,
    pub height_unit_tag: u16,
    pub padding: [u8; 10],
}
```

If a record mixes endianness in a stable way, encode that in the record rather than leaving it hand-assembled.

- [ ] **Step 4: Switch resource parsing/writing to those `binrw` records**

Use the new records in `src/image_resources.rs` and any matching helper path in `src/reader.rs` / `src/writer.rs`, while leaving variable-layout resources manual.

- [ ] **Step 5: Run the targeted test and affected suites**

Run: `cargo test --quiet display_info_fixed_record_roundtrips_via_binrw_record`
Expected: PASS

Run: `cargo test --quiet --test integration_test`
Expected: PASS

Run: `cargo test --quiet --test ts_parity_test`
Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add src/binrw_support.rs src/image_resources.rs src/reader.rs src/writer.rs tests/integration_test.rs tests/ts_parity_test.rs
git commit -m "refactor: use binrw for fixed-layout image resource records"
```

### Task 5: Final Public API Sweep And Verification

**Files:**
- Modify: `src/lib.rs`
- Modify: `tests/integration_test.rs`
- Modify: `tests/ts_parity_test.rs`
- Review: `src/types.rs`
- Review: `src/psd.rs`
- Review: `src/layer.rs`
- Review: `src/image_resources.rs`
- Review: `src/additional_info.rs`
- Review: `src/binrw_support.rs`

- [ ] **Step 1: Add the final public-API regression tests**

Add one compact regression that fails if known public semantics drift back to raw wrappers:

```rust
#[test]
fn canonical_image_resource_types_are_public() {
    let guide = psd_great::GuideInfo {
        location: 10.0,
        direction: psd_great::GuideDirection::Horizontal,
    };
    let sampler = psd_great::ColorSampler {
        position: psd_great::ColorSamplerPosition::V2 {
            horizontal: 1,
            vertical: 2,
            depth: 16,
        },
        color_space: 8,
    };

    assert_eq!(guide.direction, psd_great::GuideDirection::Horizontal);
    assert_eq!(sampler.position.version(), 2);
}
```

- [ ] **Step 2: Run the final targeted regression to verify it passes**

Run: `cargo test --quiet canonical_image_resource_types_are_public`

Expected: PASS

- [ ] **Step 3: Run the full test suite**

Run: `cargo test --quiet`

Expected: PASS

- [ ] **Step 4: Audit the public API and fixed-layout boundary**

Run:

```bash
rg -n "pub .*PsdStringCode|pub .*PsdIntCode|pub .*PsdU16Code|pub .*PsdU32Code|write_signature\\(|read_signature\\(|read_u16\\(\\)|write_u16\\(" src
```

Expected:
- known public semantics use typed enums/structs instead of raw wrappers
- remaining raw wrappers are clearly open-ended or passthrough
- remaining manual `read_u16` / `write_u16` clusters are in variable-layout code paths, not overlooked fixed-layout records

- [ ] **Step 5: Commit the final sweep**

```bash
git add src/lib.rs tests/integration_test.rs tests/ts_parity_test.rs src/types.rs src/psd.rs src/layer.rs src/image_resources.rs src/additional_info.rs src/binrw_support.rs src/reader.rs src/writer.rs src/effects_helpers.rs
git commit -m "test: verify typed public API and binrw cleanup"
```
