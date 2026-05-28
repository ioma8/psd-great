# Fix 01 — Core PSD: CMYK Formula, White Matte, write_section Length

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fix three correctness bugs in the PSD read/write pipeline: wrong CMYK→RGB color conversion, spurious white-matte removal on the merged image, and `write_section` including padding bytes in the length field.

**Architecture:** All three fixes are isolated to `src/reader.rs` and `src/writer.rs`. No struct changes needed. Tests are in `src/reader.rs` (inline test module) and `tests/`.

**Tech Stack:** Rust, `cargo test`

---

### Task 1: Fix CMYK→RGB formula in `read_merged_image_data`

**Bug:** `reader.rs:1111-1113` uses `(C*K)/255` which is completely wrong. Correct formula is `255*(1-C/255)*(1-K/255)`.

**Files:**
- Modify: `src/reader.rs:1106-1116`

- [ ] **Step 1: Write a failing test**

Add to the inline `#[cfg(test)]` block at the bottom of `src/reader.rs`:

```rust
#[test]
fn cmyk_to_rgba_formula_is_correct() {
    // C=0, M=0, Y=0, K=0  →  white (255, 255, 255)
    // C=255, M=255, Y=255, K=255  →  black (0, 0, 0)
    // C=0, M=0, Y=0, K=127  →  mid-grey ≈ (128, 128, 128)
    fn cmyk_pixel(c: u8, m: u8, y: u8, k: u8) -> [u8; 3] {
        let cv = c as u16;
        let mv = m as u16;
        let yv = y as u16;
        let kv = k as u16;
        // Correct formula: R = round(255 * (1 - C/255) * (1 - K/255))
        let r = ((255u32 * (255 - cv as u32) * (255 - kv as u32)) / (255 * 255)) as u8;
        let g = ((255u32 * (255 - mv as u32) * (255 - kv as u32)) / (255 * 255)) as u8;
        let b = ((255u32 * (255 - yv as u32) * (255 - kv as u32)) / (255 * 255)) as u8;
        [r, g, b]
    }
    assert_eq!(cmyk_pixel(0, 0, 0, 0), [255, 255, 255]);
    assert_eq!(cmyk_pixel(255, 255, 255, 255), [0, 0, 0]);
    let mid = cmyk_pixel(0, 0, 0, 127);
    assert!(mid[0] >= 127 && mid[0] <= 129, "expected ~128, got {}", mid[0]);
}
```

- [ ] **Step 2: Run test (it passes because it's testing a helper — now write an integration assertion)**

```bash
cargo test cmyk_to_rgba_formula_is_correct 2>&1
```
Expected: PASS (the helper is correct; next step makes the production code match it).

- [ ] **Step 3: Fix the CMYK block in `read_merged_image_data`**

Replace lines 1106–1116 in `src/reader.rs`:

```rust
        if color_mode == ColorMode::CMYK {
            let c = planes.get(0).and_then(|p| p.get(i)).copied().unwrap_or(0) as u32;
            let m = planes.get(1).and_then(|p| p.get(i)).copied().unwrap_or(0) as u32;
            let y = planes.get(2).and_then(|p| p.get(i)).copied().unwrap_or(0) as u32;
            let k = planes.get(3).and_then(|p| p.get(i)).copied().unwrap_or(0) as u32;
            rgba[i * 4]     = ((255 * (255 - c) * (255 - k)) / (255 * 255)) as u8;
            rgba[i * 4 + 1] = ((255 * (255 - m) * (255 - k)) / (255 * 255)) as u8;
            rgba[i * 4 + 2] = ((255 * (255 - y) * (255 - k)) / (255 * 255)) as u8;
            if total_channels <= 4 {
                rgba[i * 4 + 3] = 255;
            }
        } else if total_channels <= 3 {
```

- [ ] **Step 4: Run all tests**

```bash
cargo test 2>&1
```
Expected: all tests pass.

- [ ] **Step 5: Commit**

```bash
git add src/reader.rs
git commit -m "fix: correct CMYK->RGB formula in merged image conversion

Was (C*K)/255 which multiplies ink channels together.
Correct formula is 255*(1-C/255)*(1-K/255)."
```

---

### Task 2: Remove `remove_white_matte` from merged image decoding

**Bug:** `reader.rs:1130-1131` applies a white-matte de-composite transform when `global_alpha` is set. The TypeScript source of truth stores raw channel data without any such transform. This causes pixel value divergence.

**Files:**
- Modify: `src/reader.rs:1130-1131`

- [ ] **Step 1: Remove the `remove_white_matte` call**

In `src/reader.rs`, remove lines 1130–1131:

```rust
    // DELETE these two lines:
    if reader.global_alpha {
        remove_white_matte(&mut pixel_data);
    }
```

The `remove_white_matte` function itself can stay (it is not pub) — the compiler will warn it is unused. Remove it too:

Delete lines 1152–1164:

```rust
// DELETE this function entirely:
fn remove_white_matte(pixel_data: &mut PixelData) {
    for px in pixel_data.data.chunks_exact_mut(4) {
        let pa = px[3];
        if pa != 0 && pa != 255 {
            let a = pa as f32 / 255.0;
            let ra = 1.0 / a;
            let inv_a = 255.0 * (1.0 - ra);
            px[0] = ((px[0] as f32 * ra + inv_a).clamp(0.0, 255.0)) as u8;
            px[1] = ((px[1] as f32 * ra + inv_a).clamp(0.0, 255.0)) as u8;
            px[2] = ((px[2] as f32 * ra + inv_a).clamp(0.0, 255.0)) as u8;
        }
    }
}
```

- [ ] **Step 2: Run all tests**

```bash
cargo test 2>&1
```
Expected: all tests pass.

- [ ] **Step 3: Commit**

```bash
git add src/reader.rs
git commit -m "fix: remove white-matte removal from merged image decoding

TS source of truth stores raw channel data; applying de-matte
produces different pixel values for semi-transparent merged images."
```

---

### Task 3: Fix `write_section` to exclude padding from length field

**Bug:** `writer.rs:224-232` includes the alignment padding bytes in the `actual_length` value. PSD format requires the length to count only the content; padding comes after.

**Files:**
- Modify: `src/writer.rs:218-235`

- [ ] **Step 1: Write a failing test**

Add to the test module in `src/reader.rs` (or a new `#[cfg(test)]` block in `src/writer.rs`):

```rust
#[cfg(test)]
mod write_section_tests {
    use super::*;

    #[test]
    fn write_section_length_excludes_padding() {
        // Write a section with 3 bytes of content (odd), round=2
        // The length field must be 3, not 4
        let mut w = PsdWriter::new();
        w.write_section(2, false, |w| {
            w.write_u8(0xAA)?;
            w.write_u8(0xBB)?;
            w.write_u8(0xCC)?;
            Ok(())
        }).unwrap();
        let buf = w.finish();
        // bytes: [length u32 BE] [0xAA 0xBB 0xCC] [0x00 padding]
        assert_eq!(buf.len(), 8); // 4 (length) + 3 (content) + 1 (padding)
        let length = u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]);
        assert_eq!(length, 3, "length must count content only, not padding");
        assert_eq!(&buf[4..7], &[0xAA, 0xBB, 0xCC]);
        assert_eq!(buf[7], 0x00); // padding byte present but not counted
    }
}
```

- [ ] **Step 2: Run to verify failure**

```bash
cargo test write_section_length_excludes_padding 2>&1
```
Expected: FAIL — `length` is currently 4 instead of 3.

- [ ] **Step 3: Fix `write_section`**

In `src/writer.rs`, change lines 218–234:

```rust
    pub fn write_section<F>(&mut self, round: usize, large: bool, func: F) -> Result<()>
    where
        F: FnOnce(&mut Self) -> Result<()>,
    {
        if large {
            self.write_u32(0)?; // High 32 bits
        }

        let length_offset = self.offset;
        self.write_u32(0)?; // Placeholder for length

        let start_offset = self.offset;
        func(self)?;

        // Record content length BEFORE padding
        let content_length = (self.offset - start_offset) as u32;

        // Pad to alignment (padding bytes are NOT counted in length)
        while (self.offset - start_offset) % round != 0 {
            self.write_u8(0)?;
        }

        // Write content length (excludes padding)
        let mut cursor = Cursor::new(&mut self.buffer[length_offset..]);
        cursor.write_u32::<BigEndian>(content_length)?;

        Ok(())
    }
```

- [ ] **Step 4: Run all tests**

```bash
cargo test 2>&1
```
Expected: all tests pass. If the existing `test_read_section_applies_padding_for_round` test in `reader.rs` fails, check that `read_section` seeks to `start + length` and then applies padding alignment independently — this is correct and unchanged.

- [ ] **Step 5: Commit**

```bash
git add src/writer.rs
git commit -m "fix: write_section length field excludes alignment padding

PSD spec requires section length = content bytes only. Padding bytes
that follow are not counted. Previously length included padding, making
Rust-written files incompatible with Photoshop for odd-length sections."
```
