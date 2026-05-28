# Fix 03 — Descriptor Parsing: Tag Round-trips, Binary Types, Null Terminator

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fix 11 bugs in `src/descriptor.rs` and `src/writer.rs`: wrong OSType tags on write for several value types (`Enmr`, `obj `/`Reference`, `indx`/`Idnt`, `name`, `GlbO`), off-by-one in `Pth ` string read, discarded `rele` and `prop` reference data, `alis` stored as lossy UTF-8, missing null terminator in `write_unicode_string`, and space-trimming for short class IDs.

**Architecture:** Fixes are all in `src/descriptor.rs` (read side) and `src/writer.rs` (`write_unicode_string`, `write_ascii_string_or_class_id`). The `DescriptorValue` enum in `src/descriptor.rs` needs new variants for the currently-losing types.

**Tech Stack:** Rust, `cargo test`

**TS reference:** `photoshop/psd/src/psd/descriptor.ts`

---

### Task 1: Fix `write_unicode_string` — add null terminator

**Bug:** `writer.rs:186-193` emits `[u32 N][N chars]`. TS and PSD spec require `[u32 N+1][N chars][u16 0x0000]`.

**Files:**
- Modify: `src/writer.rs:186-196`

- [ ] **Step 1: Write a failing test**

Add to `src/writer.rs` test module:

```rust
#[test]
fn write_unicode_string_includes_null_terminator() {
    let mut w = PsdWriter::new();
    w.write_unicode_string("hi").unwrap();
    let buf = w.finish();
    // Expected: [0,0,0,3] (length=3 incl. null) [0,'h'] [0,'i'] [0,0] (null)
    assert_eq!(buf.len(), 4 + 3 * 2); // 4 length + 3 u16s
    let len = u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]);
    assert_eq!(len, 3); // N+1
    // last two bytes are null terminator
    assert_eq!(buf[buf.len()-2], 0);
    assert_eq!(buf[buf.len()-1], 0);
}
```

- [ ] **Step 2: Run to verify failure**

```bash
cargo test write_unicode_string_includes_null_terminator 2>&1
```
Expected: FAIL.

- [ ] **Step 3: Fix `write_unicode_string`**

Replace lines 186–196 in `src/writer.rs`:

```rust
    pub fn write_unicode_string(&mut self, text: &str) -> Result<()> {
        let chars: Vec<u16> = text.encode_utf16().collect();
        self.write_u32((chars.len() + 1) as u32)?; // length includes null terminator
        for ch in &chars {
            self.write_u16(*ch)?;
        }
        self.write_u16(0)?; // null terminator
        Ok(())
    }
```

- [ ] **Step 4: Run all tests**

```bash
cargo test 2>&1
```
Expected: all tests pass.

- [ ] **Step 5: Commit**

```bash
git add src/writer.rs
git commit -m "fix: write_unicode_string now includes null terminator in length and bytes

PSD spec and TS source write [N+1][chars*N][0x0000]. Was writing [N][chars*N]."
```

---

### Task 2: Fix `Pth ` path string off-by-one

**Bug:** `descriptor.rs` reads `Pth ` string length as N+1 instead of N, consuming one extra byte.

**Files:**
- Modify: `src/descriptor.rs` (find `"Pth "` match arm in `read_descriptor_value`)

- [ ] **Step 1: Find the exact lines**

```bash
grep -n "Pth" /Users/jakubkolcar/projects/customs/ag-psd-rust/src/descriptor.rs
```

- [ ] **Step 2: Write a failing test**

Add to descriptor test module (or `tests/descriptor_roundtrip.rs`):

```rust
#[test]
fn pth_string_roundtrip_correct_length() {
    use crate::descriptor::{DescriptorValue, write_descriptor_value, read_descriptor_value};
    let val = DescriptorValue::Path("hello".to_string());
    let mut w = PsdWriter::new();
    write_descriptor_value(&mut w, &val).unwrap();
    let buf = w.finish();
    let mut r = PsdReader::from_bytes(&buf);
    // read the ostype tag first (4 bytes "Pth ")
    let _tag = r.read_signature().unwrap();
    let result = read_descriptor_value(&mut r, "Pth ").unwrap();
    match result {
        DescriptorValue::Path(s) => assert_eq!(s, "hello"),
        other => panic!("unexpected: {:?}", other),
    }
}
```

- [ ] **Step 3: Run to verify failure**

```bash
cargo test pth_string_roundtrip_correct_length 2>&1
```
Expected: FAIL — reads 6 chars or corrupts stream.

- [ ] **Step 4: Fix the reader**

In `src/descriptor.rs`, find the `"Pth "` arm. The length prefix for a `Pth ` value is a `u32` counting the number of Unicode chars (not including null). Fix to read exactly `length` chars:

```rust
"Pth " => {
    let length = self.read_u32()? as usize;
    let s = self.read_unicode_string_with_length(length)?;
    Ok(DescriptorValue::Path(s))
}
```

Also fix the writer for `DescriptorValue::Path` to use `write_unicode_string` (which now correctly writes `[N+1][chars][null]`).

- [ ] **Step 5: Run all tests**

```bash
cargo test 2>&1
```
Expected: all tests pass.

- [ ] **Step 6: Commit**

```bash
git add src/descriptor.rs
git commit -m "fix: Pth descriptor value reads correct char count (not N+1)"
```

---

### Task 3: Fix `Enmr` tag round-trip (`enum` → `Enmr`)

**Bug:** Rust reads `Enmr` values and stores them as `DescriptorValue::Enum`, but the writer emits the tag `"enum"` instead of `"Enmr"`.

**Files:**
- Modify: `src/descriptor.rs` (writer `ostype_sig` or `write_descriptor_value`)

- [ ] **Step 1: Find the write tag for Enum**

```bash
grep -n "enum\|Enmr\|Enum" /Users/jakubkolcar/projects/customs/ag-psd-rust/src/descriptor.rs | head -30
```

- [ ] **Step 2: Write a failing test**

```rust
#[test]
fn enum_descriptor_roundtrips_correct_tag() {
    // Write an Enum value, read the raw 4-byte tag back, assert it's "Enmr"
    use crate::descriptor::{DescriptorValue, write_descriptor, read_descriptor};
    let desc = vec![("type".to_string(), DescriptorValue::Enum {
        type_id: "Algn".to_string(),
        value: "Lft ".to_string(),
    })];
    let mut w = PsdWriter::new();
    write_descriptor(&mut w, "", &desc).unwrap();
    let buf = w.finish();
    // tag for Enum should be "Enmr" at the appropriate offset
    let tag_pos = /* find via parsing */ 0;
    // simpler: roundtrip and check value survives
    let mut r = PsdReader::from_bytes(&buf);
    let result = read_descriptor(&mut r).unwrap();
    assert_eq!(result.len(), 1);
    match &result[0].1 {
        DescriptorValue::Enum { type_id, value } => {
            assert_eq!(type_id, "Algn");
            assert_eq!(value, "Lft ");
        }
        other => panic!("unexpected: {:?}", other),
    }
}
```

- [ ] **Step 3: Run to verify current state**

```bash
cargo test enum_descriptor_roundtrips_correct_tag 2>&1
```
Expected: if roundtrip works already it passes; if tag `"enum"` vs `"Enmr"` breaks cross-tool reading the test is still useful as a regression guard.

- [ ] **Step 4: Fix the write tag**

Find the `ostype_sig` function or match arm for `Enum` in the writer path. Change the emitted tag from `"enum"` to `"Enmr"`:

```rust
DescriptorValue::Enum { .. } => "Enmr",
```

- [ ] **Step 5: Run all tests**

```bash
cargo test 2>&1
```
Expected: all pass.

- [ ] **Step 6: Commit**

```bash
git add src/descriptor.rs
git commit -m "fix: Enum descriptor value written with tag 'Enmr' not 'enum'"
```

---

### Task 4: Fix `obj ` (Reference) tag and `indx`/`Idnt`/`name` reference item tags

**Bugs:**
- `obj ` / `Reference` values are written as `"VlLs"` (List) instead of `"obj "`.
- `indx`/`Idnt` reference items written as `"long"` instead of `"indx"`/`"Idnt"`.
- `name` reference items written as `"TEXT"` instead of `"name"`.

**Files:**
- Modify: `src/descriptor.rs`

- [ ] **Step 1: Find all write tags for reference types**

```bash
grep -n "VlLs\|obj \|indx\|Idnt\|name\|ReferenceItem\|Reference" /Users/jakubkolcar/projects/customs/ag-psd-rust/src/descriptor.rs | head -40
```

- [ ] **Step 2: Write a roundtrip test for Reference**

```rust
#[test]
fn reference_descriptor_roundtrips() {
    use crate::descriptor::{DescriptorValue, ReferenceItem, write_descriptor, read_descriptor};
    let desc = vec![("ref".to_string(), DescriptorValue::Reference(vec![
        ReferenceItem::Index { class_id: "Lyr ".to_string(), index: 2 },
    ]))];
    let mut w = PsdWriter::new();
    write_descriptor(&mut w, "", &desc).unwrap();
    let buf = w.finish();
    let mut r = PsdReader::from_bytes(&buf);
    let result = read_descriptor(&mut r).unwrap();
    match &result[0].1 {
        DescriptorValue::Reference(items) => {
            assert_eq!(items.len(), 1);
            match &items[0] {
                ReferenceItem::Index { class_id, index } => {
                    assert_eq!(class_id, "Lyr ");
                    assert_eq!(*index, 2);
                }
                other => panic!("unexpected: {:?}", other),
            }
        }
        other => panic!("unexpected: {:?}", other),
    }
}
```

- [ ] **Step 3: Run to verify current failure**

```bash
cargo test reference_descriptor_roundtrips 2>&1
```

- [ ] **Step 4: Fix all write tags**

In the writer path for descriptor values:
- `DescriptorValue::Reference(_)` → emit `"obj "` (not `"VlLs"`)
- `ReferenceItem::Index { .. }` → emit `"indx"` (not `"long"`)
- `ReferenceItem::Identifier { .. }` → emit `"Idnt"` (not `"long"`)
- `ReferenceItem::Name { .. }` → emit `"name"` (not `"TEXT"`)

- [ ] **Step 5: Run all tests**

```bash
cargo test 2>&1
```
Expected: all pass.

- [ ] **Step 6: Commit**

```bash
git add src/descriptor.rs
git commit -m "fix: Reference/Index/Idnt/Name descriptor tags corrected for write path"
```

---

### Task 5: Fix `GlbO` written as `Objc`, fix `rele`/`prop` data loss

**Bugs:**
- `GlbO` descriptor values written back as `"Objc"`.
- `rele` reference items: offset field is discarded on read; written back with wrong tag `"type"`.
- `prop` reference items: class context discarded on read.

**Files:**
- Modify: `src/descriptor.rs`

- [ ] **Step 1: Find `GlbO` and `rele` in descriptor.rs**

```bash
grep -n "GlbO\|rele\|prop\|GlobalObject\|Property\|RelativeOffset" /Users/jakubkolcar/projects/customs/ag-psd-rust/src/descriptor.rs | head -30
```

- [ ] **Step 2: Add `GlobalObject` variant if missing**

In `DescriptorValue`, ensure a `GlobalObject` variant exists (or reuse `Objc` with a flag). The simplest fix matching TS: `GlobalObject(Vec<(String, DescriptorValue)>)`. If the variant already exists, skip struct change.

- [ ] **Step 3: Fix `GlbO` write tag**

In the writer, match `DescriptorValue::GlobalObject(_)` and emit `"GlbO"`:

```rust
DescriptorValue::GlobalObject(_) => "GlbO",
```

- [ ] **Step 4: Fix `rele` — preserve offset field**

In `ReferenceItem`, add `RelativeOffset` variant if missing:

```rust
pub enum ReferenceItem {
    // ... existing ...
    RelativeOffset { class_id: String, offset: i32 },
}
```

On read, when tag is `"rele"`:
```rust
"rele" => {
    let class_id = self.read_class_id()?;
    let _class_name = self.read_unicode_string()?;
    let offset = self.read_i32()?;
    ReferenceItem::RelativeOffset { class_id, offset }
}
```

On write:
```rust
ReferenceItem::RelativeOffset { class_id, offset } => {
    w.write_signature("rele")?;
    w.write_class_id(class_id)?;
    w.write_unicode_string("")?;
    w.write_i32(*offset)?;
}
```

- [ ] **Step 5: Fix `prop` — preserve class context**

Similarly update `ReferenceItem::Property` to store both `class_id` and `key_id`, and fix read/write.

- [ ] **Step 6: Run all tests**

```bash
cargo test 2>&1
```
Expected: all pass.

- [ ] **Step 7: Commit**

```bash
git add src/descriptor.rs
git commit -m "fix: GlbO tag, rele offset, prop class context in descriptor reference items"
```

---

### Task 6: Fix `alis` stored as lossy UTF-8 — use `Vec<u8>`

**Bug:** `alis` descriptor values contain binary path data (Mac alias records), stored as a Rust `String` which silently corrupts any non-UTF-8 bytes.

**Files:**
- Modify: `src/descriptor.rs` (`DescriptorValue` enum, read/write for `alis`)

- [ ] **Step 1: Change `Alias` variant to hold `Vec<u8>`**

In `DescriptorValue`:
```rust
// Change from:
Alias(String),
// To:
Alias(Vec<u8>),
```

- [ ] **Step 2: Fix reader**

```rust
"alis" => {
    let length = self.read_u32()? as usize;
    let bytes = self.read_bytes(length)?;
    Ok(DescriptorValue::Alias(bytes))
}
```

- [ ] **Step 3: Fix writer**

```rust
DescriptorValue::Alias(bytes) => {
    w.write_signature("alis")?;
    w.write_u32(bytes.len() as u32)?;
    w.write_bytes(bytes)?;
}
```

- [ ] **Step 4: Fix any compilation errors** (callers that pattern-match `Alias(String)`)

```bash
cargo build 2>&1 | grep "Alias"
```

Update all match arms that destructure `Alias`.

- [ ] **Step 5: Run all tests**

```bash
cargo test 2>&1
```
Expected: all pass.

- [ ] **Step 6: Commit**

```bash
git add src/descriptor.rs
git commit -m "fix: alis descriptor value stored as Vec<u8> not String to preserve binary data"
```

---

### Task 7: Fix `write_ascii_string_or_class_id` — pad short IDs to 4 bytes

**Bug:** IDs shorter than 4 chars are written as length-prefixed variable-length strings. TS pads them with spaces to 4 bytes with a zero-length prefix.

**Files:**
- Modify: `src/writer.rs` (`write_ascii_string_or_class_id`)

- [ ] **Step 1: Find the function**

```bash
grep -n "write_ascii_string_or_class_id\|is_long_descriptor_id" /Users/jakubkolcar/projects/customs/ag-psd-rust/src/writer.rs | head -10
```

- [ ] **Step 2: Fix the function**

```rust
pub fn write_ascii_string_or_class_id(&mut self, value: &str) -> Result<()> {
    if value.len() == 4 && !is_long_descriptor_id(value) {
        self.write_u32(0)?;
        self.write_signature(value)?;
    } else if value.len() < 4 && !is_long_descriptor_id(value) {
        // Short IDs: zero-length prefix + pad to 4 bytes with spaces
        self.write_u32(0)?;
        let mut padded = value.to_string();
        while padded.len() < 4 {
            padded.push(' ');
        }
        self.write_signature(&padded)?;
    } else {
        self.write_u32(value.len() as u32)?;
        self.write_bytes(value.as_bytes())?;
    }
    Ok(())
}
```

- [ ] **Step 3: Run all tests**

```bash
cargo test 2>&1
```

- [ ] **Step 4: Commit**

```bash
git add src/writer.rs
git commit -m "fix: short class IDs (<4 chars) padded to 4 bytes with spaces in descriptor writer"
```
