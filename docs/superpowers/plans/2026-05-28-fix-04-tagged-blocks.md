# Fix 04 — Tagged Blocks (additional_info.rs): TySh, Anno, FEid, lnk2, shmd, Single-byte Blocks

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fix 8 bugs in `src/additional_info.rs`: spurious `u32` before text descriptor in `TySh`, wrong `Anno` color byte count, wrong pascal-string format for annotation author/date, missing initial `u32` in `FEid` rect, undecompressed `FEid` slot data, `lnk2` open-descriptor skip corruption, raw-bytes `shmd` (should parse descriptors), and single-byte blocks missing 3 padding bytes. Also add missing `uV` block after layer mask data in the writer.

**Architecture:** All changes are in `src/additional_info.rs` and `src/writer.rs`. No struct changes for most fixes; `Anno` needs color fields expanded.

**Tech Stack:** Rust, `cargo test`

**TS reference:** `photoshop/psd/src/psd/tagged-block-reader.ts`, `tagged-block-writer.ts`, `layer-record-writer.ts`

---

### Task 1: Fix `TySh` — remove extra `u32` before text descriptor

**Bug:** `additional_info.rs:1343` uses `read_version_and_descriptor()` for the text descriptor, which reads an extra `u32` version prefix. TS does NOT write a version prefix before the text descriptor (only before the warp descriptor).

**Files:**
- Modify: `src/additional_info.rs` (the `TySh` read block)

- [ ] **Step 1: Find the TySh read code**

```bash
grep -n "TySh\|read_version_and_descriptor\|text.*descriptor\|warp.*descriptor" /Users/jakubkolcar/projects/customs/ag-psd-rust/src/additional_info.rs | head -20
```

- [ ] **Step 2: Fix the text descriptor read**

Find the TySh block. It should read:
```
u16 version
6 × f64 (transform matrix)
u16 (should be 50)
[text descriptor — NO u32 prefix]
u16 warp version
u32 (16)
[warp descriptor — WITH u32 prefix via read_version_and_descriptor]
```

Change the text descriptor call from:
```rust
let text_descriptor = self.read_version_and_descriptor()?;
```
To:
```rust
let text_descriptor = self.read_descriptor()?;  // no version u32 prefix
```

- [ ] **Step 3: Fix the TySh writer similarly**

Find the write side. Change from:
```rust
self.write_version_and_descriptor(text.descriptor_version, &td)?;
```
To:
```rust
self.write_descriptor(&td)?;  // no u32 version prefix
```

- [ ] **Step 4: Run all tests**

```bash
cargo test 2>&1
```
Expected: all pass.

- [ ] **Step 5: Commit**

```bash
git add src/additional_info.rs
git commit -m "fix: TySh text descriptor has no u32 version prefix (only warp descriptor does)"
```

---

### Task 2: Fix `Anno` color — read/write 8 bytes with proper scaling

**Bug:** `additional_info.rs:932-934` reads `Anno` color as 3×u16 (6 bytes). TS reads 4×u16 (8 bytes): `u16 colorSpace=0` + 3 scaled u16 channels + `u16 trailing zero`. TS also scales from `0–255` to `0–65535`.

**Files:**
- Modify: `src/additional_info.rs` (Anno color read/write)
- Modify: `src/layer.rs` (or wherever `AnnotationColor` is defined)

- [ ] **Step 1: Find the current `AnnotationColor` struct and read/write code**

```bash
grep -n "AnnotationColor\|annotation.*color\|color_l\|color_r\|color_g\|color_b" /Users/jakubkolcar/projects/customs/ag-psd-rust/src/additional_info.rs | head -20
grep -n "AnnotationColor" /Users/jakubkolcar/projects/customs/ag-psd-rust/src/layer.rs | head -10
```

- [ ] **Step 2: Fix reader — read 8 bytes with correct scaling**

Replace the 3×`read_u16` color read:
```rust
fn read_annotation_color(reader: &mut PsdReader<impl Read + Seek>) -> Result<[u8; 3]> {
    let _color_space = reader.read_u16()?;  // always 0
    let r = ((reader.read_u16()? as u32 * 255 + 32767) / 65535) as u8;
    let g = ((reader.read_u16()? as u32 * 255 + 32767) / 65535) as u8;
    let b = ((reader.read_u16()? as u32 * 255 + 32767) / 65535) as u8;
    let _padding = reader.read_u16()?;  // trailing zero
    Ok([r, g, b])
}
```

- [ ] **Step 3: Fix writer — write 8 bytes with proper scaling**

```rust
fn write_annotation_color(writer: &mut PsdWriter, color: [u8; 3]) -> Result<()> {
    writer.write_u16(0)?;  // colorSpace = 0
    writer.write_u16(((color[0] as u32 * 65535 + 127) / 255) as u16)?;
    writer.write_u16(((color[1] as u32 * 65535 + 127) / 255) as u16)?;
    writer.write_u16(((color[2] as u32 * 65535 + 127) / 255) as u16)?;
    writer.write_u16(0)?;  // trailing zero
    Ok(())
}
```

- [ ] **Step 4: Run all tests**

```bash
cargo test 2>&1
```

- [ ] **Step 5: Commit**

```bash
git add src/additional_info.rs
git commit -m "fix: Anno color reads/writes 8 bytes (colorSpace + 3 scaled channels + padding)"
```

---

### Task 3: Fix `Anno` pascal strings — use 4-byte length prefix format

**Bug:** `additional_info.rs:935-937` reads annotation author and date strings using `read_pascal_string(2)` (1-byte length prefix). TS uses `readPascalStringWithPadding`: `u32(length+1) + ascii(length) + u8(null)`.

**Files:**
- Modify: `src/additional_info.rs` (Anno string read/write)

- [ ] **Step 1: Add a helper that matches TS `readPascalStringWithPadding`**

```rust
fn read_pascal_string_with_padding(reader: &mut PsdReader<impl Read + Seek>) -> Result<String> {
    let length_plus_one = reader.read_u32()? as usize;
    if length_plus_one == 0 {
        return Ok(String::new());
    }
    let length = length_plus_one - 1;
    let bytes = reader.read_bytes(length)?;
    let _null = reader.read_u8()?;  // null terminator
    Ok(String::from_utf8_lossy(&bytes).to_string())
}

fn write_pascal_string_with_padding(writer: &mut PsdWriter, text: &str) -> Result<()> {
    let bytes = text.as_bytes();
    writer.write_u32((bytes.len() + 1) as u32)?;  // length + 1 for null
    writer.write_bytes(bytes)?;
    writer.write_u8(0)?;  // null terminator
    Ok(())
}
```

- [ ] **Step 2: Replace annotation author/date string reads**

In the `Anno` reader, replace:
```rust
let author = self.read_pascal_string(2)?;
// (and similar for empty string and date)
```
With:
```rust
let author = read_pascal_string_with_padding(self)?;
```

Do the same for the empty string slot and the date field.

- [ ] **Step 3: Replace annotation author/date string writes**

In the `Anno` writer, replace the corresponding write calls to use `write_pascal_string_with_padding`.

- [ ] **Step 4: Run all tests**

```bash
cargo test 2>&1
```

- [ ] **Step 5: Commit**

```bash
git add src/additional_info.rs
git commit -m "fix: Anno author/date strings use 4-byte length prefix format (u32 len+1 + bytes + null)"
```

---

### Task 4: Fix `FEid` — read initial `u32` before rect, decompress slot data

**Bugs:**
1. `additional_info.rs:988-996` — missing initial `u32` (constant 8) before the slot rect.
2. `additional_info.rs:1010` — slot data read raw without decompressing.

**Files:**
- Modify: `src/additional_info.rs` (FEid reader block)

- [ ] **Step 1: Find FEid block**

```bash
grep -n "FEid\|feid\|slot.*length\|channel.*slot\|interleave_feid" /Users/jakubkolcar/projects/customs/ag-psd-rust/src/additional_info.rs | head -20
```

- [ ] **Step 2: Add the missing initial `u32` read**

After the chunk header read, before reading rect:
```rust
let _constant_8 = self.read_u32()?;  // always 8
let top    = self.read_i32()?;
let left   = self.read_i32()?;
let bottom = self.read_i32()?;
let right  = self.read_i32()?;
let depth  = self.read_u32()?;
let channel_count = self.read_u32()?;
```

- [ ] **Step 3: Decompress slot data**

Replace:
```rust
let slot_raw = self.read_bytes(slot_length as usize)?;
```
With decompression using the 2-byte compression header:
```rust
let slot_bytes = self.read_bytes(slot_length as usize)?;
let compression = u16::from_be_bytes([slot_bytes[0], slot_bytes[1]]);
let pixel_data = match Compression::from_u16(compression)? {
    Compression::RawData => slot_bytes[2..].to_vec(),
    Compression::PackBits => {
        let width = (right - left) as usize;
        let height = (bottom - top) as usize;
        decompress_rle(&slot_bytes[2..], width, height, depth as usize)?
    }
    Compression::ZipWithoutPrediction => {
        use flate2::read::ZlibDecoder;
        use std::io::Read as IoRead;
        let mut dec = ZlibDecoder::new(&slot_bytes[2..]);
        let mut out = Vec::new();
        dec.read_to_end(&mut out)?;
        out
    }
    Compression::ZipWithPrediction => {
        // same as ZipWithoutPrediction then apply delta
        use flate2::read::ZlibDecoder;
        use std::io::Read as IoRead;
        let mut dec = ZlibDecoder::new(&slot_bytes[2..]);
        let mut out = Vec::new();
        dec.read_to_end(&mut out)?;
        let width = (right - left) as usize;
        let height = (bottom - top) as usize;
        apply_prediction(&mut out, width, height, depth as usize);
        out
    }
};
```

- [ ] **Step 4: Run all tests**

```bash
cargo test 2>&1
```

- [ ] **Step 5: Commit**

```bash
git add src/additional_info.rs
git commit -m "fix: FEid reads initial u32 before rect; slot data now decompressed"
```

---

### Task 5: Fix `lnk2`/`lnkD` — skip open-descriptor when `open != 0`

**Bug:** `additional_info.rs:846-847` — when the `open` byte is non-zero, TS reads and discards a `u32` + descriptor before reading the payload. Rust just reads `payload_length` bytes from the wrong position.

**Files:**
- Modify: `src/additional_info.rs` (lnk2/lnkD reader block)

- [ ] **Step 1: Find the linked layer open-byte handling**

```bash
grep -n "lnk2\|lnkD\|_open\|open.*payload\|payload_length" /Users/jakubkolcar/projects/customs/ag-psd-rust/src/additional_info.rs | head -20
```

- [ ] **Step 2: Fix by consuming the open-descriptor when present**

```rust
let open = self.read_u8()?;
if open != 0 {
    let _version = self.read_u32()?;
    let descriptor_start = self.offset();
    self.read_descriptor()?;
    // 2-byte alignment after descriptor
    if (self.offset() - descriptor_start) % 2 != 0 {
        self.skip_bytes(1)?;
    }
}
let data = self.read_bytes(payload_length as usize)?;
```

- [ ] **Step 3: Run all tests**

```bash
cargo test 2>&1
```

- [ ] **Step 4: Commit**

```bash
git add src/additional_info.rs
git commit -m "fix: lnk2/lnkD skip open-descriptor block when open != 0"
```

---

### Task 6: Fix `shmd` — parse entries as versioned descriptors

**Bug:** `additional_info.rs:1783-1799` stores each `shmd` metadata entry's data as raw bytes. TS parses each entry as `u32(version=16) + descriptor`.

**Files:**
- Modify: `src/additional_info.rs` (shmd reader block)
- Modify: `src/layer.rs` (MetadataEntry.data type)

- [ ] **Step 1: Change `MetadataEntry.data` to hold a parsed descriptor**

Find `MetadataEntry` in `src/layer.rs`. Change `data: Vec<u8>` to:
```rust
pub struct MetadataEntry {
    pub key: String,
    pub copy_on_sheet_change: bool,
    pub descriptor: Option<Vec<(String, crate::descriptor::DescriptorValue)>>,
    pub raw_data: Vec<u8>,  // fallback for non-version-16 entries
}
```

- [ ] **Step 2: Fix the shmd reader**

```rust
// For each entry:
let entry_key = self.read_signature()?;
let copy_on_sheet = self.read_u8()? != 0;
self.skip_bytes(3)?;  // padding
let data_length = self.read_u32()? as usize;
let entry_start = self.offset();
let version = self.read_u32()?;
let (descriptor, raw_data) = if version == 16 {
    (Some(self.read_descriptor()?), Vec::new())
} else {
    self.skip_bytes(entry_start + data_length - self.offset())?;
    (None, Vec::new())
};
entries.push(MetadataEntry { key: entry_key, copy_on_sheet_change: copy_on_sheet, descriptor, raw_data });
```

- [ ] **Step 3: Fix the shmd writer to emit versioned descriptors**

For entries with a descriptor:
```rust
let entry_start = writer.offset();
writer.write_u32(16)?;  // version
if let Some(ref desc) = entry.descriptor {
    writer.write_descriptor(desc)?;
}
```

- [ ] **Step 4: Run all tests**

```bash
cargo test 2>&1
```

- [ ] **Step 5: Commit**

```bash
git add src/additional_info.rs src/layer.rs
git commit -m "fix: shmd metadata entries parsed as versioned descriptors not raw bytes"
```

---

### Task 7: Fix single-byte tagged blocks — add 3 padding bytes

**Bug:** `iOpa`, `knko`, `infx`, `clbl`, `lmgm`, `vmgm`, `fcmy` blocks are written as 1 byte but Photoshop/TS expects 4 bytes (value byte + 3 zero bytes). The block's `length` field ends up as 1 instead of 4.

**Files:**
- Modify: `src/additional_info.rs` (writers for those blocks ~lines 2002-2127)

- [ ] **Step 1: Find all single-byte block writers**

```bash
grep -n "iOpa\|knko\|infx\|clbl\|lmgm\|vmgm\|fcmy" /Users/jakubkolcar/projects/customs/ag-psd-rust/src/additional_info.rs | head -30
```

- [ ] **Step 2: Add 3 padding bytes after each single-byte value**

For each block like:
```rust
// BEFORE:
temp_writer.write_u8(value)?;
```
Change to:
```rust
// AFTER:
temp_writer.write_u8(value)?;
temp_writer.write_zeros(3)?;  // pad to 4 bytes as Photoshop expects
```

Apply this to: `iOpa`, `knko`, `infx`, `clbl`, `lmgm`, `vmgm`, `fcmy`.

The readers only read 1 byte (`self.read_u8()`) and the remaining 3 bytes are consumed by the outer `consume_remaining_to` guard, so the read side already works — only the write side needs fixing.

- [ ] **Step 3: Run all tests**

```bash
cargo test 2>&1
```

- [ ] **Step 4: Commit**

```bash
git add src/additional_info.rs
git commit -m "fix: single-byte tagged blocks (iOpa/knko/infx/clbl etc.) padded to 4 bytes"
```

---

### Task 8: Add missing `uV` block after layer mask data in writer

**Bug:** TS `writeLayerRecord` appends a 40-byte `uV` block (`0x00 0x06` + 38 zeros) inside the mask data section when a mask is present. Rust never writes it, causing compatibility issues with some Photoshop versions.

**Files:**
- Modify: `src/writer.rs` (layer mask write section ~line 586-650)

- [ ] **Step 1: Find the layer mask write section**

```bash
grep -n "uV\|uv_block\|layer.*mask.*write\|write.*mask" /Users/jakubkolcar/projects/customs/ag-psd-rust/src/writer.rs | head -20
```

- [ ] **Step 2: Add the uV block after mask data**

After writing the mask bytes and before computing the section length, add:

```rust
// Write uV filler block (required by some Photoshop versions)
if layer.additional_info.mask.is_some() {
    writer.write_u16(0x0006)?;  // type marker
    writer.write_zeros(38)?;    // 38 zero bytes = 40 total
}
```

The TS code writes this inside the `extraWriter` that is then length-prefixed, so the 40 bytes are included in the mask section's total length.

- [ ] **Step 3: Run all tests**

```bash
cargo test 2>&1
```

- [ ] **Step 4: Commit**

```bash
git add src/writer.rs
git commit -m "fix: add 40-byte uV filler block after layer mask data for Photoshop compatibility"
```
