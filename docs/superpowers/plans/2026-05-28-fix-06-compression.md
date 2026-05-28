# Fix 06 — Compression: PackBits 0x80 and ZIP+Prediction 16-bit

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fix two bugs in `src/compression.rs`: (1) PackBits byte `0x80` is treated as a NOP (Apple spec) but TS treats it as a 129-byte repeat run — any Photoshop file using this code will silently lose 129 output bytes; (2) ZIP+prediction 16-bit uses a non-standard BE16 delta approach instead of the byte-level delta that Photoshop uses, making Rust-written files unreadable by Photoshop.

**Architecture:** Both fixes are isolated to `src/compression.rs`. The PackBits fix changes one match arm. The ZIP+prediction 16-bit fix replaces the `encode_prediction_16` and `decode_prediction_16` functions to use the byte-level (8-bit delta across the raw byte stream, same as the 8-bit path).

**Tech Stack:** Rust, `cargo test`

**TS reference:** `photoshop/psd/src/io/packbits.ts`, `zip-prediction.ts`

---

### Task 1: Fix PackBits — byte `0x80` is a 129-byte repeat, not a NOP

**Bug:** `compression.rs` decodes `header == 128` as NOP. TS sign-extends the header byte and uses `count = 1 - header` for negative values; signed byte `-128` gives `count = 1 - (-128) = 129`.

**Files:**
- Modify: `src/compression.rs` (PackBits decode loop)

- [ ] **Step 1: Write a failing test**

Add to `src/compression.rs` test module:

```rust
#[test]
fn packbits_0x80_decodes_as_129_byte_repeat() {
    // 0x80 followed by 0xFF means: repeat 0xFF for 129 bytes
    let encoded = vec![0x80u8, 0xFF];
    let decoded = decode_pack_bits(&encoded, 129).unwrap();
    assert_eq!(decoded.len(), 129);
    assert!(decoded.iter().all(|&b| b == 0xFF),
        "all 129 bytes should be 0xFF");
}
```

- [ ] **Step 2: Run to verify failure**

```bash
cargo test packbits_0x80_decodes_as_129_byte_repeat 2>&1
```
Expected: FAIL — decoded.len() is 0 or wrong.

- [ ] **Step 3: Fix the decode loop**

Find the match on `header` in the PackBits decoder. Replace the `128 => {}` arm:

```rust
// BEFORE:
} else {
    // 128 is a NOP
}

// AFTER:
} else {
    // header == 128 (signed: -128): 129-byte repeat run
    let count = 257usize - header as usize; // = 1 - (-128 as i8) = 129
    if input_pos >= input.len() {
        return Err(PsdError::InvalidFormat("PackBits: missing repeat byte at 0x80".to_string()));
    }
    let byte = input[input_pos];
    input_pos += 1;
    for _ in 0..count {
        if output.len() < expected_len {
            output.push(byte);
        }
    }
}
```

Equivalently, since the current code already handles `header > 128` with `count = 257 - header`, simply change the condition so that `128` is included:

```rust
if header > 127 {
    // Repeat: count = 257 - header (for header 128, count = 129)
    let count = 257usize - header as usize;
    ...
} else {
    // Literal: count = header + 1
    ...
}
```

Remove the `else { // NOP }` branch entirely.

- [ ] **Step 4: Run tests**

```bash
cargo test packbits_0x80_decodes_as_129_byte_repeat 2>&1
```
Expected: PASS.

- [ ] **Step 5: Run all tests**

```bash
cargo test 2>&1
```
Expected: all pass.

- [ ] **Step 6: Commit**

```bash
git add src/compression.rs
git commit -m "fix: PackBits byte 0x80 decodes as 129-byte repeat run (not NOP)

Apple spec says 0x80 is NOP; Photoshop and TS treat it as a 129-byte
repeat. Any real PSD using this code was silently losing 129 bytes."
```

---

### Task 2: Fix ZIP+prediction 16-bit — use byte-level delta (same as 8-bit path)

**Bug:** The current 16-bit prediction uses a BE16 delta (reads big-endian u16 pairs, computes their difference). Photoshop uses a **byte-level** delta: the prediction is applied across the raw byte stream byte-by-byte, identical to the 8-bit path but applied to the full `width * 2 * height` bytes. The TS implementation is also broken (decode is not the inverse of encode), so neither tool can round-trip 16-bit ZIP+prediction files correctly with Photoshop.

**Files:**
- Modify: `src/compression.rs` (`encode_prediction_16` and `decode_prediction_16` / the 16-bit ZIP+prediction encode/decode paths)

- [ ] **Step 1: Write a roundtrip test**

```rust
#[test]
fn zip_prediction_16bit_roundtrips() {
    // Width=4 pixels, 1 channel, each pixel is a u16 big-endian value
    // Pixel values: 100, 200, 300, 400 (as BE u16)
    let mut original = Vec::new();
    for v in [100u16, 200, 300, 400] {
        original.extend_from_slice(&v.to_be_bytes());
    }
    let encoded = apply_prediction_16(&original, 4, 1);
    let mut decoded = encoded.clone();
    undo_prediction_16(&mut decoded, 4, 1);
    assert_eq!(decoded, original, "16-bit prediction must roundtrip");
}
```

- [ ] **Step 2: Run to verify current failure**

```bash
cargo test zip_prediction_16bit_roundtrips 2>&1
```
Expected: FAIL.

- [ ] **Step 3: Rewrite `apply_prediction_16` using byte-level delta**

The correct algorithm (matching Photoshop) for a row of `width` pixels (big-endian u16):
- Treat the row as `width * 2` raw bytes.
- Apply right-to-left byte delta: `byte[i] = byte[i] - byte[i-1]` for `i` from end to 1.

```rust
/// Apply delta prediction to 16-bit channel data (byte-level, same as 8-bit path).
/// Input: raw big-endian bytes for `height` rows of `width` u16 pixels.
pub fn apply_prediction_16(data: &[u8], width: usize, height: usize) -> Vec<u8> {
    let row_bytes = width * 2;
    let mut out = data.to_vec();
    for row in 0..height {
        let start = row * row_bytes;
        // Right-to-left byte delta
        for i in (start + 1..start + row_bytes).rev() {
            out[i] = out[i].wrapping_sub(out[i - 1]);
        }
    }
    out
}

/// Undo delta prediction from 16-bit channel data (byte-level, same as 8-bit path).
pub fn undo_prediction_16(data: &mut [u8], width: usize, height: usize) {
    let row_bytes = width * 2;
    for row in 0..height {
        let start = row * row_bytes;
        // Left-to-right prefix sum
        for i in start + 1..start + row_bytes {
            data[i] = data[i].wrapping_add(data[i - 1]);
        }
    }
}
```

- [ ] **Step 4: Wire these functions into the ZIP+prediction encode/decode paths**

In `src/compression.rs`, find where 16-bit prediction is applied during:
- Encoding (ZIP with prediction, 16-bit): call `apply_prediction_16` before deflating.
- Decoding (ZIP with prediction, 16-bit): call `undo_prediction_16` after inflating.

Replace any existing `encode_prediction_16` / `decode_prediction_16` calls.

- [ ] **Step 5: Run tests**

```bash
cargo test zip_prediction_16bit_roundtrips 2>&1
```
Expected: PASS.

- [ ] **Step 6: Run all tests**

```bash
cargo test 2>&1
```
Expected: all pass.

- [ ] **Step 7: Commit**

```bash
git add src/compression.rs
git commit -m "fix: ZIP+prediction 16-bit uses byte-level delta (same as 8-bit path)

Previous BE16 approach was not the inverse of itself and did not match
Photoshop's format. Now uses right-to-left byte delta on raw byte stream."
```
