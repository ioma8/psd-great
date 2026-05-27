# Remaining TS Parity Gaps Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement the remaining TypeScript PSD parser/writer features that still exist in `/Users/jakubkolcar/projects/customs/photoshop/psd/src` but are not yet implemented in the Rust port, excluding image compositing.

**Architecture:** Keep the current typed-first Rust architecture. Add typed models for the remaining tagged-block families, port the TS read/write logic block-by-block, and add the missing document/resource prewrite synthesis as explicit Rust prewrite passes instead of raw preservation. Reuse the existing `Psd`, `Layer`, `Descriptor`, `engine_data`, `image_resources`, `reader`, and `writer` infrastructure rather than introducing another binary abstraction layer.

**Tech Stack:** Rust, `byteorder`, `binrw`, existing `PsdReader`/`PsdWriter`, existing descriptor and engine-data modules, `cargo test`

---

## File Structure

**Primary files to modify**
- `src/additional_info.rs`
  - Add typed support for `Txt2`, `FEid`, `PxSD`, `Anno`, and the missing linked-file variant `lnkD__`.
  - Wire read/write dispatch to those typed structures.
- `src/psd.rs`
  - Add typed document-level fields for generic `color_mode_data`, path-selection descriptor exposure, and any text/document-resource state needed for TS-style prewrite synthesis.
- `src/reader.rs`
  - Preserve generic color-mode-data bytes in the typed PSD model for non-indexed modes.
  - Map document-level data into the new `Psd` fields.
- `src/writer.rs`
  - Write generic `color_mode_data` from the PSD model.
  - Add prewrite orchestration hooks equivalent to TS `synchronizeTextIndices()` and `writeResourcePrewrite()`.
- `src/image_resources.rs`
  - Support write-side path-selection descriptor prewrite from the typed PSD model.
- `src/engine_data.rs`
  - Reuse for typed `Txt2` parsing/writing; extend only if current serializer/parser cannot represent TS `w6` payloads cleanly.
- `tests/ts_parity_test.rs`
  - Add parity tests covering each missing family.
- `tests/integration_test.rs`
  - Add narrow round-trip tests for internal Rust types if needed.

**TS source of truth to compare against while implementing**
- `/Users/jakubkolcar/projects/customs/photoshop/psd/src/psd/psd-writer.ts`
- `/Users/jakubkolcar/projects/customs/photoshop/psd/src/psd/resource-postprocess.ts`
- `/Users/jakubkolcar/projects/customs/photoshop/psd/src/psd/tagged-block-reader.ts`
- `/Users/jakubkolcar/projects/customs/photoshop/psd/src/psd/tagged-block-writer.ts`
- `/Users/jakubkolcar/projects/customs/photoshop/psd/src/psd/postprocess.ts`
- `/Users/jakubkolcar/projects/customs/photoshop/psd/src/types/tagged-block.ts`
- `/Users/jakubkolcar/projects/customs/photoshop/psd/src/types/psd-document.ts`

---

### Task 1: Add parity tests for the missing feature families before implementation

**Files:**
- Modify: `tests/ts_parity_test.rs`
- Test: `tests/ts_parity_test.rs`

- [ ] **Step 1: Write the failing tests**

Add a new parity module near the end of `tests/ts_parity_test.rs` with these tests:

```rust
#[cfg(test)]
mod remaining_tagged_block_parity {
    use psd_great::{
        read_psd, write_psd, BlendMode, ChannelID, ColorMode, Compression,
        Descriptor, DescriptorValue, Layer, LayerAdditionalInfo, LayerRawData,
        LayerRawDataChannel, Psd, ReadOptions, RGB, WriteOptions,
    };
    use std::collections::HashMap;
    use std::io::Cursor;

    #[test]
    fn roundtrip_generic_color_mode_data() {
        let mut psd = Psd::default();
        psd.width = 1;
        psd.height = 1;
        psd.channels = Some(1);
        psd.bits_per_channel = Some(8);
        psd.color_mode = Some(ColorMode::Duotone);
        psd.color_mode_data = Some(vec![0xAA, 0xBB, 0xCC, 0xDD]);

        let bytes = write_psd(&psd, &WriteOptions::default()).expect("write");
        let reparsed = read_psd(&bytes, &ReadOptions::default()).expect("read");
        assert_eq!(reparsed.color_mode, Some(ColorMode::Duotone));
        assert_eq!(reparsed.color_mode_data, Some(vec![0xAA, 0xBB, 0xCC, 0xDD]));
    }

    #[test]
    fn roundtrip_document_path_selection_descriptor_prewrite() {
        let mut psd = Psd::default();
        psd.width = 1;
        psd.height = 1;
        psd.channels = Some(3);
        psd.bits_per_channel = Some(8);
        psd.color_mode = Some(ColorMode::RGB);
        psd.path_selection_descriptor = Some(Descriptor {
            name: String::new(),
            class_id: "null".to_string(),
            items: HashMap::from([(
                "name".to_string(),
                DescriptorValue::Text("path".to_string()),
            )]),
        });

        let bytes = write_psd(&psd, &WriteOptions::default()).expect("write");
        let reparsed = read_psd(&bytes, &ReadOptions::default()).expect("read");
        assert!(reparsed.path_selection_descriptor.is_some());
    }

    #[test]
    fn roundtrip_document_txt2_synthesized_from_tysh() {
        let mut text_desc = Descriptor {
            name: String::new(),
            class_id: "TxLr".to_string(),
            items: HashMap::new(),
        };
        text_desc.items.insert("Txt ".to_string(), DescriptorValue::Text("Hello".to_string()));

        let mut layer = Layer::default();
        layer.top = Some(0);
        layer.left = Some(0);
        layer.bottom = Some(1);
        layer.right = Some(1);
        layer.blend_mode = Some(BlendMode::Normal);
        layer.opacity = Some(1.0);
        layer.additional_info.name = Some("Text".to_string());
        layer.tagged_blocks.text = Some(psd_great::additional_info::TextLayerData {
            transform: vec![1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
            text: "Hello".to_string(),
            text_version: 50,
            descriptor_version: 16,
            text_data: Some(text_desc),
            warp_version: 1,
            warp_data: Some(Descriptor {
                name: String::new(),
                class_id: "warp".to_string(),
                items: HashMap::new(),
            }),
            left: 0.0,
            top: 0.0,
            right: 1.0,
            bottom: 1.0,
        });

        let mut psd = Psd::default();
        psd.width = 1;
        psd.height = 1;
        psd.channels = Some(4);
        psd.bits_per_channel = Some(8);
        psd.color_mode = Some(ColorMode::RGB);
        psd.children = Some(vec![layer]);

        let bytes = write_psd(&psd, &WriteOptions::default()).expect("write");
        let reparsed = read_psd(&bytes, &ReadOptions::default()).expect("read");
        assert!(reparsed.tagged_blocks.text_engine.is_some(), "expected synthesized Txt2");
    }

    #[test]
    fn roundtrip_annotation_tagged_block() {
        let mut layer = Layer::default();
        layer.top = Some(0);
        layer.left = Some(0);
        layer.bottom = Some(1);
        layer.right = Some(1);
        layer.blend_mode = Some(BlendMode::Normal);
        layer.opacity = Some(1.0);
        layer.additional_info.name = Some("Annotated".to_string());
        layer.tagged_blocks.annotations = Some(vec![
            psd_great::additional_info::AnnotationItem {
                x: 10,
                y: 20,
                color_l: 1,
                color_o: 2,
                color_c: 3,
                author: "author".to_string(),
                text: "note".to_string(),
            }
        ]);

        let mut psd = Psd::default();
        psd.width = 1;
        psd.height = 1;
        psd.channels = Some(4);
        psd.bits_per_channel = Some(8);
        psd.color_mode = Some(ColorMode::RGB);
        psd.children = Some(vec![layer]);

        let bytes = write_psd(&psd, &WriteOptions::default()).expect("write");
        let reparsed = read_psd(&bytes, &ReadOptions::default()).expect("read");
        let layer = reparsed.children.unwrap().into_iter().next().unwrap();
        assert!(layer.tagged_blocks.annotations.is_some());
    }

    #[test]
    fn roundtrip_lnkd_dunder_variant() {
        let mut layer = Layer::default();
        layer.top = Some(0);
        layer.left = Some(0);
        layer.bottom = Some(1);
        layer.right = Some(1);
        layer.blend_mode = Some(BlendMode::Normal);
        layer.opacity = Some(1.0);
        layer.additional_info.name = Some("Linked".to_string());
        layer.tagged_blocks.linked_files = Some(psd_great::additional_info::LinkedFilesBlock {
            key: "lnkD__".to_string(),
            items: vec![psd_great::LinkedFile {
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
        let reparsed = read_psd(&bytes, &ReadOptions::default()).expect("read");
        let layer = reparsed.children.unwrap().into_iter().next().unwrap();
        assert_eq!(
            layer.tagged_blocks.linked_files.as_ref().map(|b| b.key.as_str()),
            Some("lnkD__")
        );
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run:

```bash
cargo test remaining_tagged_block_parity -- --nocapture
```

Expected:
- FAIL on missing fields/types like `color_mode_data`, `path_selection_descriptor`, `text_engine`, `annotations`
- Or FAIL on assertions because current writer/reader does not preserve these behaviors yet

- [ ] **Step 3: Commit the failing tests**

```bash
git add tests/ts_parity_test.rs
git commit -m "test: add parity coverage for remaining ts gaps"
```

---

### Task 2: Add typed PSD model fields for generic color-mode data and document prewrite state

**Files:**
- Modify: `src/psd.rs`
- Modify: `src/lib.rs`
- Test: `tests/ts_parity_test.rs`

- [ ] **Step 1: Add the missing typed PSD fields**

Update `src/psd.rs` by adding these fields to `Psd`:

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Psd {
    // existing fields...
    #[serde(rename = "colorModeData")]
    pub color_mode_data: Option<Vec<u8>>,

    #[serde(rename = "pathSelectionDescriptor")]
    pub path_selection_descriptor: Option<crate::descriptor::Descriptor>,
}
```

If there is already a semantically-overlapping field, keep one public canonical field and remove the duplicate instead of carrying two parallel representations.

- [ ] **Step 2: Re-export any newly-needed public types**

Update `src/lib.rs` only if tests or downstream callers need new public types exported:

```rust
pub use crate::psd::Psd;
```

If `Descriptor` is already exported, do not add duplicate exports.

- [ ] **Step 3: Run the failing test module again**

Run:

```bash
cargo test remaining_tagged_block_parity::roundtrip_generic_color_mode_data -- --nocapture
```

Expected:
- still FAIL, but now on read/write behavior rather than missing struct fields

- [ ] **Step 4: Commit**

```bash
git add src/psd.rs src/lib.rs
git commit -m "feat: add typed psd fields for remaining parity data"
```

---

### Task 3: Preserve generic color-mode-data like TS, not just indexed palettes

**Files:**
- Modify: `src/reader.rs`
- Modify: `src/writer.rs`
- Test: `tests/ts_parity_test.rs`

- [ ] **Step 1: Make the read-side test fail for the right reason**

Run:

```bash
cargo test remaining_tagged_block_parity::roundtrip_generic_color_mode_data -- --nocapture
```

Expected:
- FAIL because `color_mode_data` is not currently read/written generically

- [ ] **Step 2: Update the reader to preserve non-indexed color-mode data**

Change `read_color_mode_data()` in `src/reader.rs` so the non-indexed branch stores the bytes:

```rust
fn read_color_mode_data<R: Read + Seek>(
    reader: &mut PsdReader<R>,
    psd: &mut Psd,
) -> Result<()> {
    reader.read_section(1, |reader, end_offset| {
        let remaining = reader.bytes_left(end_offset);
        if remaining == 0 {
            return Ok(());
        }

        if psd.color_mode == Some(ColorMode::Indexed) {
            if remaining != 768 {
                return Err(PsdError::InvalidFormat("Invalid color palette size".to_string()));
            }
            let mut palette = Vec::with_capacity(256);
            for _ in 0..256 {
                let r = reader.read_u8()?;
                palette.push(crate::types::RGB { r, g: 0, b: 0 });
            }
            for i in 0..256 {
                palette[i].g = reader.read_u8()?;
            }
            for i in 0..256 {
                palette[i].b = reader.read_u8()?;
            }
            psd.palette = Some(palette);
            psd.color_mode_data = Some(Vec::new());
        } else {
            psd.color_mode_data = Some(reader.read_bytes(remaining as usize)?);
        }

        Ok(())
    })
}
```

- [ ] **Step 3: Update the writer to preserve non-indexed color-mode data**

Change `write_color_mode_data()` in `src/writer.rs`:

```rust
fn write_color_mode_data(writer: &mut PsdWriter, psd: &Psd) -> Result<()> {
    writer.write_section(1, false, |writer| {
        if psd.color_mode == Some(ColorMode::Indexed) {
            let palette = psd.palette.as_ref().ok_or_else(|| {
                PsdError::InvalidFormat("Indexed color mode requires palette".to_string())
            })?;
            if palette.len() != 256 {
                return Err(PsdError::InvalidFormat(
                    "Indexed color mode requires 256 palette entries".to_string(),
                ));
            }
            for entry in palette {
                writer.write_u8(entry.r)?;
            }
            for entry in palette {
                writer.write_u8(entry.g)?;
            }
            for entry in palette {
                writer.write_u8(entry.b)?;
            }
        } else if let Some(ref data) = psd.color_mode_data {
            writer.write_bytes(data)?;
        }
        Ok(())
    })
}
```

- [ ] **Step 4: Run the color-mode-data test**

Run:

```bash
cargo test remaining_tagged_block_parity::roundtrip_generic_color_mode_data -- --nocapture
```

Expected:
- PASS

- [ ] **Step 5: Commit**

```bash
git add src/reader.rs src/writer.rs
git commit -m "feat: preserve generic color mode data"
```

---

### Task 4: Port TS path-selection descriptor prewrite behavior

**Files:**
- Modify: `src/image_resources.rs`
- Modify: `src/writer.rs`
- Modify: `src/psd.rs`
- Test: `tests/ts_parity_test.rs`

- [ ] **Step 1: Reproduce the missing prewrite behavior**

Run:

```bash
cargo test remaining_tagged_block_parity::roundtrip_document_path_selection_descriptor_prewrite -- --nocapture
```

Expected:
- FAIL because the writer is not synthesizing resource `3000` from `psd.path_selection_descriptor`

- [ ] **Step 2: Add a small prewrite helper in `src/writer.rs`**

Add a helper mirroring TS `writeResourcePrewrite()`:

```rust
fn apply_resource_prewrite(psd: &mut Psd) {
    if let Some(ref descriptor) = psd.path_selection_descriptor {
        let resources = psd.image_resources.get_or_insert_with(Default::default);
        resources.descriptor_resources.insert(3000, descriptor.clone());
    }
}
```

Call it in `write_psd()` before `write_image_resources()`:

```rust
let mut psd = psd.clone();
apply_resource_prewrite(&mut psd);
write_image_resources(&mut writer, &psd, options)?;
```

- [ ] **Step 3: Make sure the read side exposes `3000` back to the PSD**

In the image-resource post-read path, map descriptor resource `3000` back to `psd.path_selection_descriptor`:

```rust
if let Some(ref resources) = psd.image_resources {
    if let Some(descriptor) = resources.descriptor_resources.get(&3000) {
        psd.path_selection_descriptor = Some(descriptor.clone());
    }
}
```

Place this immediately after image-resource parsing in `src/reader.rs`, not inside the low-level resource decoder.

- [ ] **Step 4: Run the path-selection test**

Run:

```bash
cargo test remaining_tagged_block_parity::roundtrip_document_path_selection_descriptor_prewrite -- --nocapture
```

Expected:
- PASS

- [ ] **Step 5: Commit**

```bash
git add src/writer.rs src/reader.rs src/psd.rs src/image_resources.rs
git commit -m "feat: add path selection descriptor prewrite parity"
```

---

### Task 5: Add typed `Txt2` support and document-level text-engine storage

**Files:**
- Modify: `src/additional_info.rs`
- Modify: `src/psd.rs`
- Modify: `src/lib.rs`
- Test: `tests/ts_parity_test.rs`

- [ ] **Step 1: Add typed `Txt2` storage to the Rust models**

In `src/additional_info.rs`, add a typed block:

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct TextEngineBlock {
    pub data: crate::engine_data::EngineValue,
}
```

Add document-level storage to `LayerAdditionalInfo`:

```rust
pub text_engine: Option<TextEngineBlock>,
```

Add the same public semantic exposure to `Psd` if document-level tagged blocks are mirrored there.

- [ ] **Step 2: Add a failing read/write test for raw `Txt2` payload parsing**

Add a unit test in `src/additional_info.rs`:

```rust
#[test]
fn txt2_roundtrip_engine_data() {
    use crate::engine_data::{EngineValue, parse_engine_data, serialize_engine_data};
    let engine = EngineValue::Object(std::collections::HashMap::from([
        ("_DocumentObjects".to_string(), EngineValue::Object(std::collections::HashMap::new())),
    ]));
    let mut info = LayerAdditionalInfo::default();
    info.text_engine = Some(TextEngineBlock { data: engine.clone() });

    let mut writer = PsdWriter::new(256);
    let len = writer.write_additional_info("Txt2", &info).unwrap();
    let buf = writer.into_buffer();
    let mut reader = PsdReader::new(std::io::Cursor::new(buf), Default::default());
    let mut reparsed = LayerAdditionalInfo::default();
    reader.read_additional_info("Txt2", len, &mut reparsed).unwrap();
    assert_eq!(reparsed.text_engine, info.text_engine);
}
```

- [ ] **Step 3: Implement `Txt2` reader/writer in `src/additional_info.rs`**

Reader branch:

```rust
"Txt2" => {
    let raw = self.read_bytes(length)?;
    let parsed = crate::engine_data::parse_engine_data(&raw)
        .map_err(|e| PsdError::InvalidFormat(e.to_string()))?;
    info.text_engine = Some(TextEngineBlock { data: parsed });
}
```

Writer branch:

```rust
"Txt2" => {
    if let Some(ref text_engine) = info.text_engine {
        let bytes = crate::engine_data::serialize_engine_data(&text_engine.data)
            .map_err(|e| PsdError::InvalidFormat(e.to_string()))?;
        temp_writer.write_bytes(&bytes)?;
    }
}
```

Only add serializer API to `engine_data.rs` if it does not already exist. If the existing serializer returns bytes under a different function name, reuse it.

- [ ] **Step 4: Run the focused `Txt2` tests**

Run:

```bash
cargo test txt2_roundtrip_engine_data -- --nocapture
```

Expected:
- PASS

- [ ] **Step 5: Commit**

```bash
git add src/additional_info.rs src/psd.rs src/lib.rs
git commit -m "feat: add typed txt2 support"
```

---

### Task 6: Port TS text-index and document-resource synthesis for `Txt2`

**Files:**
- Modify: `src/writer.rs`
- Modify: `src/additional_info.rs`
- Modify: `src/engine_data.rs`
- Test: `tests/ts_parity_test.rs`

- [ ] **Step 1: Reproduce the missing synthesis behavior**

Run:

```bash
cargo test remaining_tagged_block_parity::roundtrip_document_txt2_synthesized_from_tysh -- --nocapture
```

Expected:
- FAIL because a `TySh` layer does not currently cause document-level `Txt2` generation

- [ ] **Step 2: Add a prewrite pass equivalent to TS `synchronizeTextIndices()`**

In `src/writer.rs`, add a helper like:

```rust
fn apply_text_prewrite(psd: &mut Psd) -> Result<()> {
    let mut text_index = 0i32;
    let mut text_objects = Vec::new();
    let mut document_resources: Option<crate::engine_data::EngineValue> = None;

    if let Some(ref mut layers) = psd.children {
        for layer in layers.iter_mut() {
            if let Some(ref mut text) = layer.tagged_blocks.text {
                if let Some(ref mut desc) = text.text_data {
                    desc.items.insert("TextIndex".to_string(), DescriptorValue::Integer(text_index));
                }

                text_objects.push(crate::engine_data::EngineValue::Object(std::collections::HashMap::new()));
                text_index += 1;
            }
        }
    }

    if !text_objects.is_empty() {
        let engine = crate::engine_data::EngineValue::Object(std::collections::HashMap::from([
            (
                "_DocumentObjects".to_string(),
                crate::engine_data::EngineValue::Object(std::collections::HashMap::from([
                    (
                        "_TextObjects".to_string(),
                        crate::engine_data::EngineValue::Array(text_objects),
                    )
                ])),
            ),
        ]));
        psd.tagged_blocks.text_engine = Some(crate::additional_info::TextEngineBlock { data: engine });
    }

    Ok(())
}
```

Do not overgeneralize this pass. Match only the TS behavior needed for text-object index synchronization and `_DocumentObjects` synthesis.

- [ ] **Step 3: Call the prewrite helper before writing tagged blocks**

In `write_psd()`:

```rust
let mut psd = psd.clone();
apply_text_prewrite(&mut psd)?;
apply_resource_prewrite(&mut psd);
```

Make sure the later write path uses the cloned `psd`.

- [ ] **Step 4: Run the synthesized `Txt2` parity test**

Run:

```bash
cargo test remaining_tagged_block_parity::roundtrip_document_txt2_synthesized_from_tysh -- --nocapture
```

Expected:
- PASS

- [ ] **Step 5: Commit**

```bash
git add src/writer.rs src/additional_info.rs src/engine_data.rs
git commit -m "feat: synthesize txt2 from tysh data"
```

---

### Task 7: Add typed `Anno` tagged-block support

**Files:**
- Modify: `src/additional_info.rs`
- Modify: `src/lib.rs`
- Test: `tests/ts_parity_test.rs`

- [ ] **Step 1: Add typed annotation structures**

In `src/additional_info.rs`, add:

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct AnnotationItem {
    pub x: i32,
    pub y: i32,
    pub color_l: u16,
    pub color_o: u16,
    pub color_c: u16,
    pub author: String,
    pub text: String,
}
```

And add the field:

```rust
pub annotations: Option<Vec<AnnotationItem>>,
```

- [ ] **Step 2: Port the TS reader logic for `Anno`**

In the `read_additional_info()` dispatch, add a dedicated branch for `Anno` following the TS logic in `tagged-block-reader.ts`:

```rust
"Anno" => {
    let major = self.read_u16()?;
    let minor = self.read_u16()?;
    let count = self.read_u32()? as usize;
    let mut items = Vec::with_capacity(count);
    for _ in 0..count {
        let item_length = self.read_u32()? as usize;
        let item_start = self.offset;
        let signature = self.read_signature()?;
        if signature != "txtA" {
            return Err(PsdError::InvalidFormat(format!("Unexpected annotation signature: {}", signature)));
        }
        self.skip_bytes(4)?;
        let x = self.read_i32()?;
        let y = self.read_i32()?;
        self.skip_bytes(24)?;
        let color_l = self.read_u16()?;
        let color_o = self.read_u16()?;
        let color_c = self.read_u16()?;
        let author = self.read_pascal_string(2)?;
        let _ = self.read_pascal_string(2)?;
        let _ = self.read_pascal_string(2)?;
        let text_len = self.read_u32()? as usize;
        let block = self.read_signature()?;
        if block != "txtC" {
            return Err(PsdError::InvalidFormat(format!("Unexpected annotation text block: {}", block)));
        }
        let chars_len = self.read_u32()? as usize;
        self.skip_bytes(2)?;
        let text = self.read_unicode_string_with_length(chars_len / 2)?;
        let consumed = (self.offset - item_start) as usize;
        if consumed < item_length {
            self.skip_bytes(item_length - consumed)?;
        }
        items.push(AnnotationItem { x, y, color_l, color_o, color_c, author, text });
    }
    info.annotations = Some(items);
}
```

- [ ] **Step 3: Port the TS writer logic for `Anno`**

In `write_additional_info()`:

```rust
"Anno" => {
    if let Some(ref annotations) = info.annotations {
        temp_writer.write_u16(2)?;
        temp_writer.write_u16(1)?;
        temp_writer.write_u32(annotations.len() as u32)?;
        for item in annotations {
            let mut item_writer = PsdWriter::new(256);
            item_writer.write_signature("txtA")?;
            item_writer.write_u8(1)?;
            item_writer.write_u8(28)?;
            item_writer.write_u16(1)?;
            item_writer.write_i32(item.x)?;
            item_writer.write_i32(item.y)?;
            item_writer.write_i32(17)?;
            item_writer.write_i32(21)?;
            item_writer.write_i32(item.x + 8)?;
            item_writer.write_i32(item.y + 10)?;
            item_writer.write_i32(241)?;
            item_writer.write_i32(141)?;
            item_writer.write_u16(item.color_l)?;
            item_writer.write_u16(item.color_o)?;
            item_writer.write_u16(item.color_c)?;
            item_writer.write_pascal_string(&item.author, 2)?;
            item_writer.write_pascal_string("", 2)?;
            item_writer.write_pascal_string("D:20211012120233+01'00'", 2)?;
            item_writer.write_u32((12 + 2 + item.text.len() * 2) as u32)?;
            item_writer.write_signature("txtC")?;
            item_writer.write_u32((2 + item.text.len() * 2) as u32)?;
            item_writer.write_u8(254)?;
            item_writer.write_u8(255)?;
            for ch in item.text.chars() {
                item_writer.write_u16(ch as u16)?;
            }
            let bytes = item_writer.into_buffer();
            temp_writer.write_u32(bytes.len() as u32)?;
            temp_writer.write_bytes(&bytes)?;
        }
    }
}
```

- [ ] **Step 4: Run the annotation parity test**

Run:

```bash
cargo test remaining_tagged_block_parity::roundtrip_annotation_tagged_block -- --nocapture
```

Expected:
- PASS

- [ ] **Step 5: Commit**

```bash
git add src/additional_info.rs src/lib.rs
git commit -m "feat: add typed anno tagged block support"
```

---

### Task 8: Add typed `lnkD__` parity to linked-file handling

**Files:**
- Modify: `src/additional_info.rs`
- Test: `tests/ts_parity_test.rs`

- [ ] **Step 1: Reproduce the variant-gap failure**

Run:

```bash
cargo test remaining_tagged_block_parity::roundtrip_lnkd_dunder_variant -- --nocapture
```

Expected:
- FAIL because `lnkD__` is not recognized or not written back with the same key

- [ ] **Step 2: Extend the dispatch to support `lnkD__`**

Update both read and write paths in `src/additional_info.rs`:

```rust
"lnk2" | "lnkD" | "lnkD__" | "lnk3" => {
    // existing typed linked-file parsing
}
```

And in the writer:

```rust
"lnk2" | "lnkD" | "lnkD__" | "lnk3" => {
    if let Some(ref block) = info.linked_files {
        if block.key == key {
            // existing typed linked-file writer body
        }
    }
}
```

Do not special-case the payload structure unless the test proves the `lnkD__` body differs. Start by preserving the typed body with the original block key.

- [ ] **Step 3: Run the linked-file variant test**

Run:

```bash
cargo test remaining_tagged_block_parity::roundtrip_lnkd_dunder_variant -- --nocapture
```

Expected:
- PASS

- [ ] **Step 4: Commit**

```bash
git add src/additional_info.rs
git commit -m "feat: support lnkD__ linked file tagged block"
```

---

### Task 9: Add typed `FEid` filter-effects support

**Files:**
- Modify: `src/additional_info.rs`
- Modify: `src/lib.rs`
- Test: `tests/ts_parity_test.rs`

- [ ] **Step 1: Add typed filter-effects models**

Port the minimum typed shape from TS `FilterEffectsBlock` into `src/additional_info.rs`:

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct FilterEffectsBlock {
    pub version: u32,
    pub items: Vec<FilterEffectsItem>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FilterEffectsItem {
    pub id: String,
    pub version: Option<u32>,
    pub rect: Option<(i32, i32, i32, i32)>,
    pub depth: Option<u32>,
    pub channel_count: Option<u32>,
    pub slots: Option<Vec<FilterEffectsSlot>>,
    pub preview: Option<FilterEffectsPreview>,
}
```

Add the field:

```rust
pub filter_effects: Option<FilterEffectsBlock>,
```

- [ ] **Step 2: Add a narrow unit test before implementing**

In `src/additional_info.rs`, add:

```rust
#[test]
fn feid_roundtrip_minimal_item() {
    let mut info = LayerAdditionalInfo::default();
    info.filter_effects = Some(FilterEffectsBlock {
        version: 1,
        items: vec![FilterEffectsItem {
            id: "test".to_string(),
            version: Some(1),
            rect: None,
            depth: None,
            channel_count: None,
            slots: None,
            preview: None,
        }],
    });

    let mut writer = PsdWriter::new(256);
    let len = writer.write_additional_info("FEid", &info).unwrap();
    let buf = writer.into_buffer();
    let mut reader = PsdReader::new(std::io::Cursor::new(buf), Default::default());
    let mut reparsed = LayerAdditionalInfo::default();
    reader.read_additional_info("FEid", len, &mut reparsed).unwrap();
    assert_eq!(reparsed.filter_effects, info.filter_effects);
}
```

- [ ] **Step 3: Port the TS reader/writer incrementally**

Add `FEid` branches by translating the TS chunk layout from `tagged-block-reader.ts:771` and `tagged-block-writer.ts:751`.

Reader skeleton:

```rust
"FEid" => {
    let block_end = start_offset + length as u64;
    let version = self.read_u32()?;
    let mut items = Vec::new();
    while self.offset < block_end {
        let high = self.read_u32()? as u64;
        let low = self.read_u32()? as u64;
        let chunk_length = (high << 32) | low;
        let chunk_start = self.offset;
        let id = self.read_pascal_string(1)?;
        let item_version = if self.offset + 4 <= chunk_start + chunk_length { Some(self.read_u32()?) } else { None };
        if self.offset + 4 <= chunk_start + chunk_length {
            let _ = self.read_u32()?;
        }
        self.skip_bytes((chunk_start + chunk_length).saturating_sub(self.offset) as usize)?;
        if chunk_length % 4 != 0 {
            self.skip_bytes((4 - (chunk_length % 4)) as usize)?;
        }
        items.push(FilterEffectsItem {
            id,
            version: item_version,
            rect: None,
            depth: None,
            channel_count: None,
            slots: None,
            preview: None,
        });
    }
    info.filter_effects = Some(FilterEffectsBlock { version, items });
}
```

Writer skeleton:

```rust
"FEid" => {
    if let Some(ref block) = info.filter_effects {
        temp_writer.write_u32(block.version)?;
        for item in &block.items {
            let mut item_writer = PsdWriter::new(256);
            item_writer.write_pascal_string(&item.id, 1)?;
            item_writer.write_u32(item.version.unwrap_or(1))?;
            item_writer.write_u32(0)?;
            let bytes = item_writer.into_buffer();
            temp_writer.write_u32(0)?;
            temp_writer.write_u32(bytes.len() as u32)?;
            temp_writer.write_bytes(&bytes)?;
            let remainder = bytes.len() % 4;
            if remainder != 0 {
                temp_writer.write_zeros(4 - remainder)?;
            }
        }
    }
}
```

Start minimal, make the test pass, then extend with typed slots/preview only when additional tests require them.

- [ ] **Step 4: Run the focused `FEid` test**

Run:

```bash
cargo test feid_roundtrip_minimal_item -- --nocapture
```

Expected:
- PASS

- [ ] **Step 5: Commit**

```bash
git add src/additional_info.rs src/lib.rs
git commit -m "feat: add typed feid tagged block support"
```

---

### Task 10: Add typed `PxSD` pixel-source tagged-block support

**Files:**
- Modify: `src/additional_info.rs`
- Modify: `src/lib.rs`
- Test: `tests/ts_parity_test.rs`

- [ ] **Step 1: Add typed `PxSD` models**

Add minimal TS-shaped models:

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct PixelSourceDataBlock {
    pub items: Vec<PixelSourceDataItem>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PixelSourceDataItem {
    pub key: u32,
}
```

Add the field:

```rust
pub pixel_source_data: Option<PixelSourceDataBlock>,
```

- [ ] **Step 2: Add a minimal failing unit test**

```rust
#[test]
fn pxsd_roundtrip_minimal_item() {
    let mut info = LayerAdditionalInfo::default();
    info.pixel_source_data = Some(PixelSourceDataBlock {
        items: vec![PixelSourceDataItem { key: 7 }],
    });

    let mut writer = PsdWriter::new(256);
    let len = writer.write_additional_info("PxSD", &info).unwrap();
    let buf = writer.into_buffer();
    let mut reader = PsdReader::new(std::io::Cursor::new(buf), Default::default());
    let mut reparsed = LayerAdditionalInfo::default();
    reader.read_additional_info("PxSD", len, &mut reparsed).unwrap();
    assert_eq!(reparsed.pixel_source_data, info.pixel_source_data);
}
```

- [ ] **Step 3: Implement the minimal typed `PxSD` chunk reader/writer**

Reader:

```rust
"PxSD" => {
    let block_end = start_offset + length as u64;
    let mut items = Vec::new();
    while self.offset + 8 <= block_end {
        let high = self.read_u32()? as u64;
        let low = self.read_u32()? as u64;
        let chunk_length = (high << 32) | low;
        let chunk_start = self.offset;
        let key = self.read_u32()?;
        self.skip_bytes((chunk_start + chunk_length).saturating_sub(self.offset) as usize)?;
        items.push(PixelSourceDataItem { key });
    }
    info.pixel_source_data = Some(PixelSourceDataBlock { items });
}
```

Writer:

```rust
"PxSD" => {
    if let Some(ref block) = info.pixel_source_data {
        for item in &block.items {
            let mut item_writer = PsdWriter::new(64);
            item_writer.write_u32(item.key)?;
            item_writer.write_u32(2)?;
            let bytes = item_writer.into_buffer();
            temp_writer.write_u32(0)?;
            temp_writer.write_u32(bytes.len() as u32)?;
            temp_writer.write_bytes(&bytes)?;
        }
    }
}
```

This is intentionally the smallest typed pass. Only add nested image payload support if a failing parity test or sample requires it.

- [ ] **Step 4: Run the focused `PxSD` test**

Run:

```bash
cargo test pxsd_roundtrip_minimal_item -- --nocapture
```

Expected:
- PASS

- [ ] **Step 5: Commit**

```bash
git add src/additional_info.rs src/lib.rs
git commit -m "feat: add typed pxsd tagged block support"
```

---

### Task 11: Wire the new tagged blocks into the public write/read section lists

**Files:**
- Modify: `src/additional_info.rs`
- Test: `tests/ts_parity_test.rs`

- [ ] **Step 1: Add the new keys to the section ordering**

Update `write_layer_additional_info()` to include the missing keys:

```rust
let sections = vec![
    "luni", "lyid", "lclr", "iOpa", "lsct", "clbl", "infx", "knko", "lspf", "lnsr",
    "lyvr", "lmgm", "vmgm", "fcmy", "brst", "fxrp",
    "TySh", "Txt2", "SoCo", "GdFl", "PtFl", "vstk", "vscg", "vmsk", "vogk",
    "lfx2", "lrFX", "clrL", "rplc", "PlLd", "SoLd", "artb", "sn2P", "shmd",
    "FMsk", "shpa", "pths", "CgEd", "vibA", "PxSc", "phry",
    "Lr16", "Lr32", "lnk2", "lnkD", "lnkD__", "lnk3", "FEid", "PxSD", "Anno",
];
```

Only include keys that actually have read/write branches implemented by this point.

- [ ] **Step 2: Run the full additional-info unit test slice**

Run:

```bash
cargo test additional_info:: -- --nocapture
```

Expected:
- PASS

- [ ] **Step 3: Commit**

```bash
git add src/additional_info.rs
git commit -m "chore: wire remaining tagged blocks into section ordering"
```

---

### Task 12: Run the full Rust and TS-parity verification sweep

**Files:**
- Test: `tests/ts_parity_test.rs`
- Test: `tests/integration_test.rs`

- [ ] **Step 1: Run the remaining parity module**

Run:

```bash
cargo test remaining_tagged_block_parity -- --nocapture
```

Expected:
- PASS

- [ ] **Step 2: Run the full crate tests**

Run:

```bash
cargo test -- --nocapture
```

Expected:
- PASS

- [ ] **Step 3: Run formatting if needed**

Run:

```bash
cargo fmt --all
cargo test -- --nocapture
```

Expected:
- `cargo fmt --all` makes no semantic changes
- test suite stays PASS

- [ ] **Step 4: Commit the finished parity pass**

```bash
git add src/additional_info.rs src/psd.rs src/reader.rs src/writer.rs src/image_resources.rs src/engine_data.rs src/lib.rs tests/ts_parity_test.rs
git commit -m "feat: close remaining ts parity gaps"
```

---

## Self-Review

**Spec coverage:** This plan covers the concrete remaining gaps identified from the TS source:
- generic `colorModeData`
- document path-selection prewrite (`3000`)
- `Txt2` parsing/writing and TS-style text synthesis
- `Anno`
- `lnkD__`
- `FEid`
- `PxSD`

**Known deliberate boundary:** The plan stays typed-first and does not reintroduce raw preservation. For `FEid` and `PxSD`, it starts with the smallest typed shape that can round-trip a defined structure, then extends only if failing parity tests or sample PSDs require more payload coverage.

**Placeholder scan:** No `TODO`, `TBD`, or “implement later” placeholders remain. Every task contains exact files, commands, and concrete code skeletons.

**Type consistency:** The plan consistently uses:
- `Psd.color_mode_data`
- `Psd.path_selection_descriptor`
- `LayerAdditionalInfo.text_engine`
- `LayerAdditionalInfo.annotations`
- `LayerAdditionalInfo.filter_effects`
- `LayerAdditionalInfo.pixel_source_data`

