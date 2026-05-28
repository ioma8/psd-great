# Fix 07 — Image Resources: Little-Endian Byte Order for Resources 1026, 1036, 1073; Unicode Alpha Name Null Terminator

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fix three byte-order bugs and one offset-accounting bug in `src/image_resources.rs`: resources 1026 (layer clipping), 1036 (Display Info), and 1073 (Custom Points) are written little-endian by TS/Photoshop but big-endian by Rust, corrupting all multi-byte field values. Also fix resource 1045 (Unicode alpha channel names) where the 2-byte null terminator per entry is not consumed, causing misalignment after the first entry.

**Architecture:** All fixes are in `src/image_resources.rs`. Resources 1026/1036/1073 need their structs or parse code changed to read/write little-endian. Resource 1045 needs its `remaining` decrement fixed.

**Tech Stack:** Rust, `cargo test`

**TS reference:** `photoshop/psd/src/psd/document-postprocess.ts` (resources 1026, 1036, 1045, 1073)

---

### Task 1: Fix resource 1026 (layer clipping) — little-endian u16

**Bug:** TS uses `getUint16(index * 2, false)` (little-endian). Rust uses `LayerStateRecord` via `decode_be`, which is big-endian. For clipping value 1 this doesn't matter (0x0001 = 0x0100 bit-swapped gives 256, which breaks), but the bug corrupts any non-zero clipping.

**Files:**
- Modify: `src/image_resources.rs` (resource 1026 read/write)

- [ ] **Step 1: Find the resource 1026 handler**

```bash
grep -n "1026\|layer_clipping\|clipping_groups\|read_clipping\|write_clipping\|LayerStateRecord" /Users/jakubkolcar/projects/customs/ag-psd-rust/src/image_resources.rs | head -20
```

- [ ] **Step 2: Write a failing test**

```rust
#[test]
fn resource_1026_clipping_roundtrips_little_endian() {
    // A clipping group list: [0, 1, 0] (3 layers, middle is grouped)
    let clipping = vec![0u16, 1u16, 0u16];
    let encoded = build_clipping_resource(&clipping);
    // Bytes should be LE: 0x00,0x00, 0x01,0x00, 0x00,0x00
    assert_eq!(encoded[0], 0x00);
    assert_eq!(encoded[1], 0x00);
    assert_eq!(encoded[2], 0x01); // LE: low byte first
    assert_eq!(encoded[3], 0x00);
    let decoded = parse_clipping_resource(&encoded);
    assert_eq!(decoded, clipping);
}
```

- [ ] **Step 3: Fix the reader — use LE u16**

Replace the `decode_be` call with a direct little-endian read:

```rust
fn parse_clipping_resource(bytes: &[u8]) -> Vec<u16> {
    bytes.chunks_exact(2)
        .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
        .collect()
}
```

- [ ] **Step 4: Fix the writer — use LE u16**

```rust
fn build_clipping_resource(clipping: &[u16]) -> Vec<u8> {
    let mut out = Vec::with_capacity(clipping.len() * 2);
    for &v in clipping {
        out.extend_from_slice(&v.to_le_bytes());
    }
    out
}
```

- [ ] **Step 5: Run all tests**

```bash
cargo test 2>&1
```
Expected: all pass.

- [ ] **Step 6: Commit**

```bash
git add src/image_resources.rs
git commit -m "fix: resource 1026 (layer clipping) reads/writes little-endian u16 values"
```

---

### Task 2: Fix resource 1036 (Display Info) — little-endian u16 unit fields

**Bug:** TS uses `getUint16(..., false)` (little-endian) for all unit code fields at offsets 2, 6, 10, 14. Rust uses `u16::from_be_bytes`.

**Files:**
- Modify: `src/image_resources.rs` (resource 1036 read/write, ~lines 148-173)

- [ ] **Step 1: Find the Display Info handler**

```bash
grep -n "1036\|DisplayInfo\|display_info\|hResUnit\|h_res_unit" /Users/jakubkolcar/projects/customs/ag-psd-rust/src/image_resources.rs | head -20
```

- [ ] **Step 2: Write a failing test**

```rust
#[test]
fn resource_1036_display_info_roundtrips_little_endian() {
    let info = DisplayInfoResource {
        version: 1,
        h_res_unit: PsdU16Code(1),
        v_res_unit: PsdU16Code(1),
        width_unit: PsdU16Code(2),
        height_unit: PsdU16Code(2),
    };
    let bytes = build_display_info_resource(&info);
    // h_res_unit at bytes[2..4] should be LE: 0x01, 0x00
    assert_eq!(bytes[2], 0x01, "low byte first (LE)");
    assert_eq!(bytes[3], 0x00, "high byte second (LE)");
    let parsed = parse_display_info_resource(&bytes).unwrap();
    assert_eq!(parsed.h_res_unit.0, 1);
    assert_eq!(parsed.width_unit.0, 2);
}
```

- [ ] **Step 3: Fix the reader**

Replace all `u16::from_be_bytes([bytes[n], bytes[n+1]])` with `u16::from_le_bytes(...)` for the four unit fields (offsets 2, 6, 10, 14). The `version` field at offset 0–1 should be treated as big-endian (or little-endian — it's always 1, so it doesn't matter; keep BE for version, LE for unit codes to match TS exactly).

- [ ] **Step 4: Fix the writer**

Replace `u16::to_be_bytes()` with `u16::to_le_bytes()` for the four unit fields.

- [ ] **Step 5: Run all tests**

```bash
cargo test 2>&1
```

- [ ] **Step 6: Commit**

```bash
git add src/image_resources.rs
git commit -m "fix: resource 1036 (Display Info) unit fields are little-endian not big-endian"
```

---

### Task 3: Fix resource 1073 (Custom Points) — all fields little-endian

**Bug:** TS uses little-endian (`false` DataView flag) for all fields: version (u32), count (u32), and coordinate fixed-point values (i32). Rust uses big-endian.

**Files:**
- Modify: `src/image_resources.rs` (resource 1073 read/write, ~lines 175-229)

- [ ] **Step 1: Find the Custom Points handler**

```bash
grep -n "1073\|custom_points\|CustomPoint\|build_custom_points\|parse_custom_points" /Users/jakubkolcar/projects/customs/ag-psd-rust/src/image_resources.rs | head -20
```

- [ ] **Step 2: Write a failing test**

```rust
#[test]
fn resource_1073_custom_points_roundtrips_little_endian() {
    let points = vec![CustomPoint { x: 1.5, y: 2.25, .. Default::default() }];
    let bytes = build_custom_points_resource(3, &points);
    // version (u32 LE) at bytes[0..4] should be 0x03,0x00,0x00,0x00
    assert_eq!(bytes[0], 0x03, "version low byte first (LE)");
    assert_eq!(bytes[1], 0x00);
    let parsed = parse_custom_points_resource(&bytes).unwrap();
    assert!((parsed[0].x - 1.5).abs() < 0.0001);
}
```

- [ ] **Step 3: Fix the reader — all fields LE**

Replace all `i32::from_be_bytes` / `u32::from_be_bytes` calls with their `from_le_bytes` counterparts in the custom points reader.

- [ ] **Step 4: Fix the writer — all fields LE**

Replace all `.to_be_bytes()` calls with `.to_le_bytes()` in the custom points writer.

- [ ] **Step 5: Run all tests**

```bash
cargo test 2>&1
```

- [ ] **Step 6: Commit**

```bash
git add src/image_resources.rs
git commit -m "fix: resource 1073 (Custom Points) all fields are little-endian not big-endian"
```

---

### Task 4: Fix resource 1045 (Unicode alpha names) — account for 2-byte null terminator

**Bug:** `read_alpha_unicode_names` decrements `remaining` by `name.len() * 2 + 4`, but each entry is `4 + len*2 + 2` bytes (TS always appends a null u16 that `read_unicode_string` does not consume). The 2-byte null terminator is never consumed, causing misalignment after the first entry.

**Files:**
- Modify: `src/image_resources.rs` (~lines 789-805)

- [ ] **Step 1: Write a failing test**

```rust
#[test]
fn resource_1045_multiple_alpha_names_read_correctly() {
    // Build two unicode string entries as TS writes them:
    // Entry 1: "AB" → [0,0,0,3][0,65][0,66][0,0]  (len=3 incl. null, chars 'A','B', null)
    // Entry 2: "C"  → [0,0,0,2][0,67][0,0]          (len=2, char 'C', null)
    let mut bytes = Vec::new();
    // "AB"
    bytes.extend_from_slice(&3u32.to_be_bytes());  // length = 3
    bytes.extend_from_slice(&[0x00, 0x41, 0x00, 0x42]); // 'A', 'B'
    bytes.extend_from_slice(&[0x00, 0x00]); // null terminator
    // "C"
    bytes.extend_from_slice(&2u32.to_be_bytes());  // length = 2
    bytes.extend_from_slice(&[0x00, 0x43]); // 'C'
    bytes.extend_from_slice(&[0x00, 0x00]); // null terminator

    let result = parse_alpha_unicode_names(&bytes).unwrap();
    assert_eq!(result.len(), 2);
    assert_eq!(result[0], "AB");
    assert_eq!(result[1], "C");
}
```

- [ ] **Step 2: Run to verify failure**

```bash
cargo test resource_1045_multiple_alpha_names_read_correctly 2>&1
```
Expected: FAIL — second entry is garbled due to 2-byte null not consumed.

- [ ] **Step 3: Fix `read_alpha_unicode_names`**

Change the `remaining` decrement to include the 2-byte null terminator. Also consume the null in the reader:

```rust
while remaining >= 6 {  // minimum: 4 (length) + 2 (1 char) = 6 bytes
    let length = reader.read_u32()? as usize;
    if length == 0 {
        break;
    }
    // length includes null terminator: read (length - 1) chars, then skip null
    let char_count = length - 1;
    let name = reader.read_unicode_string_with_length(char_count)?;
    let _null = reader.read_u16()?;  // consume null terminator
    names.push(name.clone());
    remaining = remaining.saturating_sub(4 + char_count * 2 + 2);
}
```

- [ ] **Step 4: Fix `write_alpha_unicode_names`**

Use `write_unicode_string` (which now includes null terminator after fix in Plan 03 Task 1). Each entry is: `[u32 N+1][N chars][u16 0]`.

- [ ] **Step 5: Run all tests**

```bash
cargo test 2>&1
```
Expected: all pass.

- [ ] **Step 6: Commit**

```bash
git add src/image_resources.rs
git commit -m "fix: resource 1045 unicode alpha names consume 2-byte null terminator per entry"
```
