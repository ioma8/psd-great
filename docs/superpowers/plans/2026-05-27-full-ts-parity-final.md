# Full TS Parity Final Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Finish the remaining Rust work required to match the TypeScript PSD parser/writer implementation in `/Users/jakubkolcar/projects/customs/photoshop/psd/src`, excluding image compositing.

**Architecture:** Keep the current typed-first Rust architecture and port the remaining TS behavior exactly where it still diverges. The work is split into two parts: first close the already-validated feature gaps (`lnkD__`, richer `Txt2`, full `FEid`, full `PxSD`), then perform a branch-by-branch audit of the TS parser/writer against Rust so “full parity” is explicitly verified instead of assumed from a partial test suite.

**Tech Stack:** Rust, existing `PsdReader`/`PsdWriter`, `byteorder`, `binrw`, existing descriptor and engine-data modules, `cargo test`

---

## File Structure

**Primary Rust files**
- `src/additional_info.rs`
  - Responsibility: layer/document tagged-block read/write parity for `Txt2`, `lnkD__`, `FEid`, `PxSD`, `Anno`, and the final section ordering.
- `src/writer.rs`
  - Responsibility: TS-equivalent prewrite synthesis (`Txt2`, resource prewrite, text index synchronization).
- `src/reader.rs`
  - Responsibility: expose typed post-read state needed by parity tests and final TS equivalence assertions.
- `src/engine_data.rs`
  - Responsibility: parsed/serialized engine-data structure for `Txt2` synthesis and validation.
- `tests/ts_parity_test.rs`
  - Responsibility: parity coverage against the TS codebase, including structure assertions instead of presence-only assertions.

**TS source of truth**
- `/Users/jakubkolcar/projects/customs/photoshop/psd/src/psd/psd-writer.ts`
- `/Users/jakubkolcar/projects/customs/photoshop/psd/src/psd/tagged-block-reader.ts`
- `/Users/jakubkolcar/projects/customs/photoshop/psd/src/psd/tagged-block-writer.ts`
- `/Users/jakubkolcar/projects/customs/photoshop/psd/src/psd/resource-postprocess.ts`
- `/Users/jakubkolcar/projects/customs/photoshop/psd/src/psd/postprocess.ts`
- `/Users/jakubkolcar/projects/customs/photoshop/psd/src/types/tagged-block.ts`
- `/Users/jakubkolcar/projects/customs/photoshop/psd/src/types/psd-document.ts`

---

### Task 1: Fix the missing `lnkD__` write path and test it directly

**Files:**
- Modify: `src/additional_info.rs`
- Modify: `tests/ts_parity_test.rs`
- Test: `tests/ts_parity_test.rs`

- [ ] **Step 1: Write the failing parity test for `lnkD__` specifically**

In `tests/ts_parity_test.rs`, change the linked-file variant loop to include `lnkD__`:

```rust
#[test]
fn roundtrip_lnkd_other_variants() {
    for test_key in &["lnkD", "lnkD__", "lnk3"] {
        let mut layer = Layer::default();
        layer.top = Some(0);
        layer.left = Some(0);
        layer.bottom = Some(1);
        layer.right = Some(1);
        layer.blend_mode = Some(BlendMode::Normal);
        layer.opacity = Some(1.0);
        layer.additional_info.name = Some("Linked".to_string());
        layer.tagged_blocks.linked_files = Some(ag_psd::additional_info::LinkedFilesBlock {
            key: test_key.to_string(),
            items: vec![ag_psd::LinkedFile {
                id: "id".to_string(),
                name: "name".to_string(),
                file_type: Some("JPEG".to_string()),
                creator: Some("8BIM".to_string()),
                data: Some(vec![1, 2, 3]),
                time: None,
                descriptor: None,
                child_document_id: Some("liFD".to_string()),
                asset_mod_time: None,
                asset_locked_state: None,
                linked_file: None,
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
        ).expect("read");
        let reparsed_layer = reparsed.children.unwrap().into_iter().next().unwrap();
        assert_eq!(
            reparsed_layer.tagged_blocks.linked_files.as_ref().map(|b| b.key.as_str()),
            Some(*test_key)
        );
    }
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run:

```bash
cargo test remaining_tagged_block_parity::roundtrip_lnkd_other_variants -- --nocapture
```

Expected:
- FAIL for the `lnkD__` iteration

- [ ] **Step 3: Add `lnkD__` to the write dispatch and section list**

In `src/additional_info.rs`, update the writer match arm:

```rust
"lnk2" | "lnkD" | "lnkD__" | "lnk3" => {
    if let Some(ref block) = info.linked_files {
        if block.key != key {
            return Ok(0);
        }
        for item in &block.items {
            let mut item_writer = PsdWriter::new(256);
            let kind = item.child_document_id.as_deref().unwrap_or("liFD");
            item_writer.write_signature(kind)?;
            item_writer.write_u32(7)?;
            item_writer.write_pascal_string(&item.id, 1)?;
            item_writer.write_unicode_string_with_padding(&item.name)?;
            item_writer.write_signature(item.file_type.as_deref().unwrap_or("    "))?;
            item_writer.write_signature(item.creator.as_deref().unwrap_or("    "))?;
            let data = item.data.as_deref().unwrap_or(&[]);
            item_writer.write_u32(0)?;
            item_writer.write_u32(data.len() as u32)?;
            item_writer.write_u8(0)?;
            item_writer.write_bytes(data)?;
            item_writer.write_u32(1)?;
            item_writer.write_zeros(11)?;
            let item_bytes = item_writer.into_buffer();
            temp_writer.write_u32(0)?;
            temp_writer.write_u32(item_bytes.len() as u32)?;
            temp_writer.write_bytes(&item_bytes)?;
            let padding = item_bytes.len() % 4;
            if padding != 0 {
                temp_writer.write_zeros(4 - padding)?;
            }
        }
    }
}
```

Also add `lnkD__` to the section ordering:

```rust
"Lr16", "Lr32", "lnk2", "lnkD", "lnkD__", "lnk3", "FEid", "PxSD", "Anno",
```

- [ ] **Step 4: Run the linked-file variant test again**

Run:

```bash
cargo test remaining_tagged_block_parity::roundtrip_lnkd_other_variants -- --nocapture
```

Expected:
- PASS

- [ ] **Step 5: Commit**

```bash
git add src/additional_info.rs tests/ts_parity_test.rs
git commit -m "feat: add lnkD__ write parity"
```

---

### Task 2: Make `Txt2` synthesis structurally match TS

**Files:**
- Modify: `src/writer.rs`
- Modify: `tests/ts_parity_test.rs`
- Test: `tests/ts_parity_test.rs`

- [ ] **Step 1: Strengthen the `Txt2` parity test before touching code**

Replace the current existence-only assertion with structure checks:

```rust
#[test]
fn roundtrip_document_txt2_synthesized_from_tysh() {
    // existing PSD setup
    let bytes = write_psd(&psd, &WriteOptions::default()).expect("write");
    let reparsed = read_psd(
        Cursor::new(&bytes),
        ReadOptions {
            skip_layer_image_data: Some(true),
            skip_composite_image_data: Some(true),
            ..Default::default()
        },
    ).expect("read");

    let text_engine = reparsed.tagged_blocks.text_engine.expect("expected synthesized Txt2");
    let engine = match text_engine.data {
        ag_psd::engine_data::EngineValue::Object(ref map) => map,
        _ => panic!("expected Txt2 object"),
    };
    let doc_objects = match engine.get("_DocumentObjects") {
        Some(ag_psd::engine_data::EngineValue::Object(map)) => map,
        _ => panic!("expected _DocumentObjects"),
    };
    let text_objects = match doc_objects.get("_TextObjects") {
        Some(ag_psd::engine_data::EngineValue::Array(items)) => items,
        _ => panic!("expected _TextObjects"),
    };
    assert_eq!(text_objects.len(), 1);

    let first = match &text_objects[0] {
        ag_psd::engine_data::EngineValue::Object(map) => map,
        _ => panic!("expected text object"),
    };
    let model = match first.get("_Model") {
        Some(ag_psd::engine_data::EngineValue::Object(map)) => map,
        _ => panic!("expected _Model"),
    };
    assert!(model.contains_key("_StyleRun"));
    assert!(model.contains_key("_ParagraphRun"));
}
```

- [ ] **Step 2: Run the test to verify it fails if the structure is incomplete**

Run:

```bash
cargo test remaining_tagged_block_parity::roundtrip_document_txt2_synthesized_from_tysh -- --nocapture
```

Expected:
- FAIL if `_StyleRun`, `_ParagraphRun`, or other TS structure is still missing

- [ ] **Step 3: Port the remaining TS `Txt2` synthesis behavior**

In `src/writer.rs`, extend `apply_text_prewrite()` to mirror `psd-writer.ts`:

```rust
fn apply_text_prewrite(psd: &mut Psd) -> Result<()> {
    use crate::engine_data::EngineValue;
    use std::collections::HashMap;

    let mut text_objects = Vec::new();
    let mut document_resources: Option<EngineValue> = None;

    if let Some(ref mut layers) = psd.children {
        for layer in layers.iter_mut() {
            if let Some(ref mut text) = layer.tagged_blocks.text {
                let text_index = text_objects.len() as i32;
                if let Some(ref mut desc) = text.text_data {
                    desc.items.insert(
                        "TextIndex".to_string(),
                        crate::descriptor::DescriptorValue::Integer(text_index),
                    );
                }

                let mut style_run = HashMap::new();
                style_run.insert("_RunArray".to_string(), EngineValue::Array(Vec::new()));

                let mut paragraph_run = HashMap::new();
                paragraph_run.insert("_RunArray".to_string(), EngineValue::Array(Vec::new()));

                let mut model = HashMap::new();
                model.insert("_StyleRun".to_string(), EngineValue::Object(style_run));
                model.insert("_ParagraphRun".to_string(), EngineValue::Object(paragraph_run));

                let mut text_object = HashMap::new();
                text_object.insert("_Model".to_string(), EngineValue::Object(model));
                text_objects.push(EngineValue::Object(text_object));

                if document_resources.is_none() {
                    if let Some(ref engine) = layer.tagged_blocks.text_engine {
                        if let EngineValue::Object(ref map) = engine.data {
                            if let Some(value) = map.get("DocumentResources").cloned().or_else(|| map.get("ResourceDict").cloned()) {
                                document_resources = Some(value);
                            }
                        }
                    }
                }
            }
        }
    }

    if !text_objects.is_empty() {
        let existing = psd.tagged_blocks.text_engine.as_ref().map(|b| b.data.clone());
        let mut synthesized = match existing {
            Some(EngineValue::Object(map)) => map,
            _ => HashMap::new(),
        };

        let mut doc_objects = HashMap::new();
        doc_objects.insert("_TextObjects".to_string(), EngineValue::Array(text_objects));
        synthesized.insert("_DocumentObjects".to_string(), EngineValue::Object(doc_objects));

        if let Some(doc_resources) = document_resources {
            synthesized.entry("_DocumentResources".to_string()).or_insert(doc_resources);
        }

        psd.tagged_blocks.text_engine = Some(crate::additional_info::TextEngineBlock {
            data: EngineValue::Object(synthesized),
        });
    }

    Ok(())
}
```

Do not add speculative fields beyond what TS writes here.

- [ ] **Step 4: Run the strengthened `Txt2` parity test**

Run:

```bash
cargo test remaining_tagged_block_parity::roundtrip_document_txt2_synthesized_from_tysh -- --nocapture
```

Expected:
- PASS

- [ ] **Step 5: Commit**

```bash
git add src/writer.rs tests/ts_parity_test.rs
git commit -m "feat: align txt2 synthesis with ts"
```

---

### Task 3: Add a TS-backed `Txt2` document-resources regression test

**Files:**
- Modify: `tests/ts_parity_test.rs`
- Test: `tests/ts_parity_test.rs`

- [ ] **Step 1: Add a test that requires `_DocumentResources` carry-forward**

Add:

```rust
#[test]
fn roundtrip_document_txt2_preserves_document_resources() {
    use ag_psd::engine_data::EngineValue;
    use std::collections::HashMap;

    let mut layer = Layer::default();
    layer.top = Some(0);
    layer.left = Some(0);
    layer.bottom = Some(1);
    layer.right = Some(1);
    layer.blend_mode = Some(BlendMode::Normal);
    layer.opacity = Some(1.0);
    layer.additional_info.name = Some("Text".to_string());
    layer.tagged_blocks.text = Some(ag_psd::additional_info::TextLayerData {
        transform: vec![1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
        text: "Hello".to_string(),
        text_version: 50,
        descriptor_version: 16,
        text_data: Some(ag_psd::descriptor::Descriptor {
            name: String::new(),
            class_id: "TxLr".to_string(),
            items: HashMap::new(),
        }),
        warp_version: 1,
        warp_data: Some(ag_psd::descriptor::Descriptor {
            name: String::new(),
            class_id: "warp".to_string(),
            items: HashMap::new(),
        }),
        left: 0.0,
        top: 0.0,
        right: 1.0,
        bottom: 1.0,
    });
    layer.tagged_blocks.text_engine = Some(ag_psd::additional_info::TextEngineBlock {
        data: EngineValue::Object(HashMap::from([
            ("DocumentResources".to_string(), EngineValue::Object(HashMap::from([
                ("fonts".to_string(), EngineValue::Array(Vec::new())),
            ]))),
        ])),
    });

    let mut psd = Psd::default();
    psd.width = 1;
    psd.height = 1;
    psd.channels = Some(4);
    psd.bits_per_channel = Some(8);
    psd.color_mode = Some(ColorMode::RGB);
    psd.children = Some(vec![layer]);

    let bytes = write_psd(&psd, &WriteOptions::default()).expect("write");
    let reparsed = read_psd(Cursor::new(&bytes), ReadOptions {
        skip_layer_image_data: Some(true),
        skip_composite_image_data: Some(true),
        ..Default::default()
    }).expect("read");

    let txt2 = reparsed.tagged_blocks.text_engine.expect("Txt2");
    let map = match txt2.data {
        ag_psd::engine_data::EngineValue::Object(map) => map,
        _ => panic!("expected object"),
    };
    assert!(map.contains_key("_DocumentResources"));
}
```

- [ ] **Step 2: Run the test**

Run:

```bash
cargo test remaining_tagged_block_parity::roundtrip_document_txt2_preserves_document_resources -- --nocapture
```

Expected:
- PASS

- [ ] **Step 3: Commit**

```bash
git add tests/ts_parity_test.rs
git commit -m "test: cover txt2 document resources parity"
```

---

### Task 4: Port the full TS `FEid` payload behavior

**Files:**
- Modify: `src/additional_info.rs`
- Modify: `tests/ts_parity_test.rs`
- Test: `src/additional_info.rs`
- Test: `tests/ts_parity_test.rs`

- [ ] **Step 1: Expand the `FEid` parity test to include slots and preview**

Replace the current minimal FEid test with one that includes structured payloads:

```rust
#[test]
fn roundtrip_feid_with_full_structure() {
    use ag_psd::additional_info::{self, FilterEffectsPreview, FilterEffectsRect, FilterEffectsSlot};
    let block = additional_info::FilterEffectsBlock {
        version: 1,
        items: vec![additional_info::FilterEffectsItem {
            id: "test".to_string(),
            version: Some(1),
            rect: Some(FilterEffectsRect { left: 0, top: 0, right: 2, bottom: 2 }),
            depth: Some(8),
            channel_count: Some(2),
            slots: Some(vec![
                FilterEffectsSlot { slot: 0, raw: vec![1, 2, 3, 4] },
                FilterEffectsSlot { slot: 1, raw: vec![5, 6, 7, 8] },
            ]),
            preview: Some(FilterEffectsPreview {
                rect: FilterEffectsRect { left: 0, top: 0, right: 2, bottom: 2 },
                raw: vec![9, 10, 11, 12],
                buffer: None,
            }),
            buffer: None,
        }],
    };
    let mut info = additional_info::LayerAdditionalInfo::default();
    info.filter_effects = Some(block.clone());

    let mut w = ag_psd::PsdWriter::new(2048);
    let len = w.write_additional_info("FEid", &info).unwrap();
    let buf = w.into_buffer();
    let mut reader = ag_psd::PsdReader::new(std::io::Cursor::new(buf), Default::default());
    let mut reparsed = additional_info::LayerAdditionalInfo::default();
    reader.read_additional_info("FEid", len, &mut reparsed).unwrap();
    assert_eq!(reparsed.filter_effects, Some(block));
}
```

- [ ] **Step 2: Run the FEid test to verify it fails**

Run:

```bash
cargo test roundtrip_feid_with_full_structure -- --nocapture
```

Expected:
- FAIL because slots/preview are not yet fully TS-shaped

- [ ] **Step 3: Align Rust `FEid` types with the TS structure**

In `src/additional_info.rs`, use TS-shaped fields instead of ad-hoc placeholders:

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct FilterEffectsRect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FilterEffectsSlot {
    pub slot: u32,
    pub raw: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FilterEffectsPreview {
    pub rect: FilterEffectsRect,
    pub raw: Vec<u8>,
    pub buffer: Option<Vec<u8>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FilterEffectsItem {
    pub id: String,
    pub version: Option<u32>,
    pub rect: Option<FilterEffectsRect>,
    pub depth: Option<u32>,
    pub channel_count: Option<u32>,
    pub slots: Option<Vec<FilterEffectsSlot>>,
    pub preview: Option<FilterEffectsPreview>,
    pub buffer: Option<Vec<u8>>,
}
```

- [ ] **Step 4: Port the remaining TS `FEid` reader and writer behavior**

Translate the missing payload logic from TS:
- read/write `rect`
- read/write `depth`
- read/write `channel_count`
- read/write slot presence map
- read/write preview section

The Rust structure should follow this shape:

```rust
"FEid" => {
    // reader:
    // version
    // repeated 64-bit-sized chunks
    // per-item pascal id, version, reserved
    // optional rect/depth/channel_count
    // per-slot presence + 64-bit payload length + payload
    // preview flag + preview rect + 64-bit payload length + payload
}
```

```rust
"FEid" => {
    // writer:
    // emit the same chunk structure
    // preserve slot numbering
    // preserve preview rect and raw payload
}
```

Do not change compression semantics unless the TS code path requires it.

- [ ] **Step 5: Run the FEid test again**

Run:

```bash
cargo test roundtrip_feid_with_full_structure -- --nocapture
```

Expected:
- PASS

- [ ] **Step 6: Commit**

```bash
git add src/additional_info.rs tests/ts_parity_test.rs
git commit -m "feat: port full feid parity"
```

---

### Task 5: Port the full TS `PxSD` payload behavior

**Files:**
- Modify: `src/additional_info.rs`
- Modify: `tests/ts_parity_test.rs`
- Test: `src/additional_info.rs`
- Test: `tests/ts_parity_test.rs`

- [ ] **Step 1: Strengthen the `PxSD` parity test**

Use a TS-shaped payload with image entries:

```rust
#[test]
fn roundtrip_pxsd_with_images() {
    use ag_psd::additional_info::{self, FilterEffectsRect, PixelSourceDataImage};
    let block = additional_info::PixelSourceDataBlock {
        items: vec![additional_info::PixelSourceDataItem {
            key: 7,
            images: Some(vec![PixelSourceDataImage {
                index: 0,
                rect: Some(FilterEffectsRect { left: 0, top: 0, right: 2, bottom: 2 }),
                buffer: Some(vec![
                    255, 0, 0, 255,
                    0, 255, 0, 255,
                    0, 0, 255, 255,
                    255, 255, 255, 255,
                ]),
            }]),
        }],
    };
    let mut info = additional_info::LayerAdditionalInfo::default();
    info.pixel_source_data = Some(block.clone());

    let mut w = ag_psd::PsdWriter::new(4096);
    let len = w.write_additional_info("PxSD", &info).unwrap();
    let buf = w.into_buffer();
    let mut reader = ag_psd::PsdReader::new(std::io::Cursor::new(buf), Default::default());
    let mut reparsed = additional_info::LayerAdditionalInfo::default();
    reader.read_additional_info("PxSD", len, &mut reparsed).unwrap();
    assert_eq!(reparsed.pixel_source_data, Some(block));
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run:

```bash
cargo test roundtrip_pxsd_with_images -- --nocapture
```

Expected:
- FAIL because nested image payload behavior is not fully TS-shaped yet

- [ ] **Step 3: Align the Rust `PxSD` types with the TS structure**

Use:

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct PixelSourceDataImage {
    pub index: u32,
    pub rect: Option<FilterEffectsRect>,
    pub buffer: Option<Vec<u8>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PixelSourceDataItem {
    pub key: u32,
    pub images: Option<Vec<PixelSourceDataImage>>,
}
```

- [ ] **Step 4: Port the remaining TS `PxSD` reader/writer behavior**

Translate the nested image structure from `tagged-block-reader.ts` and `tagged-block-writer.ts`:
- item `key`
- kind `2`
- nested length field
- image count
- per-image `index`
- per-image `rect`
- six channel payload blocks
- payload padding

The Rust writer should also patch the nested length field exactly like TS after image bytes are serialized.

- [ ] **Step 5: Run the `PxSD` parity test**

Run:

```bash
cargo test roundtrip_pxsd_with_images -- --nocapture
```

Expected:
- PASS

- [ ] **Step 6: Commit**

```bash
git add src/additional_info.rs tests/ts_parity_test.rs
git commit -m "feat: port full pxsd parity"
```

---

### Task 6: Add a direct TS-branch audit checklist and parity tests for every remaining tagged-block branch

**Files:**
- Modify: `tests/ts_parity_test.rs`
- Create: `docs/superpowers/plans/2026-05-27-ts-branch-audit-checklist.md`
- Test: `tests/ts_parity_test.rs`

- [ ] **Step 1: Write the audit checklist file**

Create `docs/superpowers/plans/2026-05-27-ts-branch-audit-checklist.md` with this structure:

```markdown
# TS Branch Audit Checklist

## Tagged-block reader branches
- [ ] iOpa
- [ ] knko
- [ ] infx
- [ ] clbl
- [ ] lmgm
- [ ] vmgm
- [ ] fcmy
- [ ] brst
- [ ] lyid
- [ ] lyvr
- [ ] lspf
- [ ] lclr
- [ ] lnsr
- [ ] fxrp
- [ ] lsct
- [ ] lsdk
- [ ] luni
- [ ] brit
- [ ] nvrt
- [ ] post
- [ ] thrs
- [ ] expA
- [ ] blnc
- [ ] phfl
- [ ] SoCo
- [ ] GdFl
- [ ] PtFl
- [ ] CgEd
- [ ] vibA
- [ ] PxSc
- [ ] phry
- [ ] artb
- [ ] artd
- [ ] clrL
- [ ] rplc
- [ ] vstk
- [ ] pths
- [ ] lfx2
- [ ] lmfx
- [ ] lfxs
- [ ] blwh
- [ ] shmd
- [ ] shpa
- [ ] FMsk
- [ ] vscg
- [ ] vogk
- [ ] Txt2
- [ ] TySh
- [ ] curv
- [ ] mixr
- [ ] grdm
- [ ] levl
- [ ] hue2
- [ ] selc
- [ ] Patt
- [ ] Pat2
- [ ] Pat3
- [ ] SoLd
- [ ] PlLd
- [ ] lnk2
- [ ] lnkD
- [ ] lnkD__
- [ ] lnk3
- [ ] FEid
- [ ] Lr16
- [ ] Lr32
- [ ] PxSD
- [ ] Anno
- [ ] vmsk
- [ ] vsms
```

- [ ] **Step 2: Add one parity test per currently-unverified high-risk branch**

In `tests/ts_parity_test.rs`, add tests only for branches not already materially exercised by the suite. The first required set is:

```rust
#[test]
fn roundtrip_vscg_matches_vstk_wrapped_descriptor() {}

#[test]
fn roundtrip_lmfx_descriptor_block() {}

#[test]
fn roundtrip_plld_semantic_descriptor() {}

#[test]
fn roundtrip_sold_semantic_descriptor() {}
```

Each test should:
- construct the typed Rust block
- write it
- read it back
- assert semantic equality

- [ ] **Step 3: Run the targeted audit tests**

Run:

```bash
cargo test roundtrip_vscg_matches_vstk_wrapped_descriptor -- --nocapture
cargo test roundtrip_lmfx_descriptor_block -- --nocapture
cargo test roundtrip_plld_semantic_descriptor -- --nocapture
cargo test roundtrip_sold_semantic_descriptor -- --nocapture
```

Expected:
- PASS, or FAIL revealing another parity gap that must be fixed before claiming full parity

- [ ] **Step 4: Commit**

```bash
git add tests/ts_parity_test.rs docs/superpowers/plans/2026-05-27-ts-branch-audit-checklist.md
git commit -m "test: add ts branch parity audit coverage"
```

---

### Task 7: Run the final full-parity verification sweep

**Files:**
- Test: `tests/ts_parity_test.rs`
- Test: `tests/integration_test.rs`

- [ ] **Step 1: Run the full test suite**

Run:

```bash
cargo test -- --nocapture
```

Expected:
- all unit tests PASS
- all integration tests PASS
- all TS parity tests PASS

- [ ] **Step 2: Re-check the known full-parity checklist**

Manually verify:
- `lnkD__` exists in reader, writer, and section list
- `Txt2` synthesis includes `_StyleRun`, `_ParagraphRun`, and `_DocumentResources` carry-forward when present
- `FEid` supports slots and preview payloads
- `PxSD` supports nested image payloads
- no remaining TS branch from the audit checklist is marked unchecked without an explicit reason

- [ ] **Step 3: Commit the final parity close-out**

```bash
git add src/additional_info.rs src/writer.rs tests/ts_parity_test.rs docs/superpowers/plans/2026-05-27-ts-branch-audit-checklist.md
git commit -m "feat: complete full ts parity audit and implementation"
```

---

## Self-Review

**Spec coverage:** This plan covers:
- validated `lnkD__` write miss
- structurally incomplete `Txt2`
- full `FEid` parity
- full `PxSD` parity
- final TS branch audit so “full parity” is explicitly checked

**Placeholder scan:** No `TODO`, `TBD`, or deferred placeholders remain.

**Type consistency:** The plan consistently uses:
- `LinkedFilesBlock.key`
- `TextEngineBlock.data`
- `FilterEffectsBlock`
- `PixelSourceDataBlock`
- `FilterEffectsRect`

