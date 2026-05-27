# PSD Data Layer Completion Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Close all remaining data-layer gaps between the Rust PSD library and the reference TypeScript implementation, giving every adjustment layer type a structured read/write API and fixing three protocol-level bugs.

**Architecture:** New `src/adjustments.rs` holds all adjustment-layer types and their byte-level serialisers; `src/additional_info.rs` is updated to use them. Three bug fixes (vstk/vscg, PlLd UUID) are applied to `src/additional_info.rs` before the new features land so later tasks compile cleanly. Text-engine-data parsing (EngineData inside TySh) is explicitly out of scope — the raw bytes are already preserved.

**Tech Stack:** Rust 2021, `byteorder` (already in tree via `PsdReader`/`PsdWriter`), `cargo test`.

---

## File Map

| File | Change |
|------|--------|
| `src/adjustments.rs` | **Create** — all adjustment-layer structs + `AdjustmentLayer` enum + read/write |
| `src/additional_info.rs` | **Modify** — use `AdjustmentLayer`, fix vstk/vscg/PlLd bugs, add lmfx/Patt routing |
| `src/lib.rs` | **Modify** — `pub mod adjustments;` |

---

## Task 1: Fix `vstk` read/write (reads one too many u32)

**Files:**
- Modify: `src/additional_info.rs`

TS reads `vstk` as: `u32 (skip) + parseDescriptor`. The current Rust calls `read_u32()` (version) **then** `read_version_and_descriptor()` (another u32 + descriptor). That reads an extra u32.

- [ ] **Step 1.1: Write a failing read/write roundtrip test**

Add inside `#[cfg(test)]` in `src/additional_info.rs`:

```rust
#[test]
fn vstk_descriptor_roundtrip() {
    use crate::descriptor::{Descriptor, DescriptorValue};
    let mut desc = Descriptor { name: String::new(), class_id: "vstk".to_string(), items: std::collections::HashMap::new() };
    desc.items.insert("strokeStyleVersion".to_string(), DescriptorValue::Integer(2));

    let mut info = LayerAdditionalInfo::default();
    info.vector_stroke = Some(VectorStroke { version: 1, descriptor: desc.clone() });

    let mut w = PsdWriter::new(256);
    let len = w.write_additional_info("vstk", &info).unwrap();
    let buf = w.into_buffer();

    let cursor = std::io::Cursor::new(buf);
    let mut reader = PsdReader::new(cursor, Default::default());
    let mut read_info = LayerAdditionalInfo::default();
    reader.read_additional_info("vstk", len, &mut read_info).unwrap();

    let vs = read_info.vector_stroke.unwrap();
    assert!(vs.descriptor.items.contains_key("strokeStyleVersion"));
}
```

- [ ] **Step 1.2: Run to confirm failure**

```bash
cargo test vstk_descriptor_roundtrip 2>&1
```
Expected: FAIL (corrupt descriptor parse due to extra u32).

- [ ] **Step 1.3: Fix `read_vector_stroke` in `src/additional_info.rs`**

Replace:
```rust
fn read_vector_stroke(&mut self, info: &mut LayerAdditionalInfo, _length: usize) -> Result<()> {
    let version = self.read_u32()?;
    let descriptor = self.read_version_and_descriptor()?;
    info.vector_stroke = Some(VectorStroke { version, descriptor });
    Ok(())
}
```
With:
```rust
fn read_vector_stroke(&mut self, info: &mut LayerAdditionalInfo, _length: usize) -> Result<()> {
    let version = self.read_u32()?;
    let descriptor = self.read_descriptor_structure()?;
    info.vector_stroke = Some(VectorStroke { version, descriptor });
    Ok(())
}
```

- [ ] **Step 1.4: Fix the write arm for `vstk` in `write_additional_info`**

Replace:
```rust
"vscg" | "vstk" => {
    if let Some(ref vs) = info.vector_stroke {
        temp_writer.write_u32(vs.version)?;
        temp_writer.write_version_and_descriptor(16, &vs.descriptor)?;
    }
}
```
With:
```rust
"vscg" | "vstk" => {
    if let Some(ref vs) = info.vector_stroke {
        temp_writer.write_u32(vs.version)?;
        temp_writer.write_descriptor_structure(&vs.descriptor)?;
    }
}
```

- [ ] **Step 1.5: Run tests**

```bash
cargo test 2>&1
```
Expected: all pass, including `vstk_descriptor_roundtrip`.

- [ ] **Step 1.6: Commit**

```bash
git add src/additional_info.rs
git commit -m "fix: vstk read consumed one extra u32 (version-and-descriptor vs descriptor)"
```

---

## Task 2: Fix `vscg` read (missing wrapped-key prefix)

**Files:**
- Modify: `src/additional_info.rs`

TS reads `vscg` as: `wrappedKey (4 ASCII bytes) + u32 (skip) + descriptor`, stores result under `wrappedKey`. Rust routes `vscg` to `read_vector_stroke`, which reads nothing for the wrapped key.

- [ ] **Step 2.1: Add `vscg_wraps_vstk` field to `VectorStroke`** (no struct change needed — just fix the reader to skip the key)

Actually the TS just stores the descriptor under the inner key. For round-trip, we only need to store the raw bytes — the wrapped key is always `vstk` in practice. Fix by reading and discarding the 4-byte key, then reading u32+descriptor.

Replace `read_vector_stroke` (as fixed in Task 1) with a separate `read_vscg` helper:

```rust
fn read_vector_stroke(&mut self, info: &mut LayerAdditionalInfo, _length: usize) -> Result<()> {
    let version = self.read_u32()?;
    let descriptor = self.read_descriptor_structure()?;
    info.vector_stroke = Some(VectorStroke { version, descriptor });
    Ok(())
}

fn read_vscg(&mut self, info: &mut LayerAdditionalInfo, _length: usize) -> Result<()> {
    let _wrapped_key = self.read_signature()?; // always "vstk" in practice
    let version = self.read_u32()?;
    let descriptor = self.read_descriptor_structure()?;
    info.vector_stroke = Some(VectorStroke { version, descriptor });
    Ok(())
}
```

- [ ] **Step 2.2: Update the match arm in `read_additional_info`**

Replace:
```rust
"vscg" | "vstk" => self.read_vector_stroke(info, length)?,
```
With:
```rust
"vstk" => self.read_vector_stroke(info, length)?,
"vscg" => self.read_vscg(info, length)?,
```

- [ ] **Step 2.3: Run tests**

```bash
cargo test 2>&1
```
Expected: all pass.

- [ ] **Step 2.4: Commit**

```bash
git add src/additional_info.rs
git commit -m "fix: vscg read now skips 4-byte wrapped key before version+descriptor"
```

---

## Task 3: Fix `PlLd` UUID parsing (pascal string, not fixed 32 bytes)

**Files:**
- Modify: `src/additional_info.rs`

TS reads the UUID as: `u8 length + that many bytes`. Rust reads fixed 32 bytes.

- [ ] **Step 3.1: Fix `read_placed_layer` in `src/additional_info.rs`**

Replace the id-reading block:
```rust
// Read UUID
let id_length = 32;
let id_bytes = self.read_bytes(id_length)?;
let id = String::from_utf8_lossy(&id_bytes).to_string();
```
With:
```rust
// UUID is a pascal string: u8 length then that many bytes
let id_len = self.read_u8()? as usize;
let id_bytes = self.read_bytes(id_len)?;
let id = String::from_utf8_lossy(&id_bytes).to_string();
```

- [ ] **Step 3.2: Fix the write arm for `PlLd`/`SoLd` in `write_additional_info`**

Replace:
```rust
let id_bytes = pl.id.as_bytes();
temp_writer.write_bytes(id_bytes)?;
// Pad id to 32 bytes
if id_bytes.len() < 32 {
    temp_writer.write_zeros(32 - id_bytes.len())?;
}
```
With:
```rust
let id_bytes = pl.id.as_bytes();
temp_writer.write_u8(id_bytes.len() as u8)?;
temp_writer.write_bytes(id_bytes)?;
```

- [ ] **Step 3.3: Write a roundtrip test**

```rust
#[test]
fn plld_uuid_pascal_string_roundtrip() {
    let mut info = LayerAdditionalInfo::default();
    info.placed_layer = Some(PlacedLayer {
        id: "abc".to_string(),
        page: Some(1),
        total_pages: Some(1),
        anti_alias_policy: Some(0),
        placed_layer_type: Some(1),
        transform: vec![1.0; 8],
        warp: None,
        placed: None,
    });

    let mut w = PsdWriter::new(256);
    let len = w.write_additional_info("PlLd", &info).unwrap();
    let buf = w.into_buffer();

    let cursor = std::io::Cursor::new(buf);
    let mut reader = PsdReader::new(cursor, Default::default());
    let mut read_info = LayerAdditionalInfo::default();
    reader.read_additional_info("PlLd", len, &mut read_info).unwrap();

    assert_eq!(read_info.placed_layer.unwrap().id, "abc");
}
```

- [ ] **Step 3.4: Run tests**

```bash
cargo test 2>&1
```
Expected: all pass.

- [ ] **Step 3.5: Commit**

```bash
git add src/additional_info.rs
git commit -m "fix: PlLd/SoLd UUID is a pascal u8-length string, not fixed 32 bytes"
```

---

## Task 4: Create `src/adjustments.rs` with simple adjustment types

**Files:**
- Create: `src/adjustments.rs`
- Modify: `src/lib.rs`

Implement the five simplest adjustment layer types: BrightnessContrast (`brit`), Invert (`nvrt`), Posterize (`post`), Threshold (`thrs`), Exposure (`expA`), ColorBalance (`blnc`).

- [ ] **Step 4.1: Add `pub mod adjustments;` to `src/lib.rs`**

Find the existing `pub mod` block in `src/lib.rs` and add:
```rust
pub mod adjustments;
```

- [ ] **Step 4.2: Create `src/adjustments.rs`**

```rust
//! Adjustment layer typed data for all PSD adjustment block types.
//!
//! Each adjustment layer type has a struct, a `read(&[u8])` function, and a
//! `write(&self) -> Vec<u8>` function that match the on-disk binary format
//! documented in the PSD spec and verified against the reference TS implementation.

use crate::error::{PsdError, Result};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io::Cursor;

fn read_i16(data: &[u8], offset: &mut usize) -> Result<i16> {
    if *offset + 2 > data.len() {
        return Err(PsdError::InvalidFormat("adjustment: unexpected end".to_string()));
    }
    let v = i16::from_be_bytes([data[*offset], data[*offset + 1]]);
    *offset += 2;
    Ok(v)
}

fn read_u16(data: &[u8], offset: &mut usize) -> Result<u16> {
    if *offset + 2 > data.len() {
        return Err(PsdError::InvalidFormat("adjustment: unexpected end".to_string()));
    }
    let v = u16::from_be_bytes([data[*offset], data[*offset + 1]]);
    *offset += 2;
    Ok(v)
}

fn read_u32(data: &[u8], offset: &mut usize) -> Result<u32> {
    if *offset + 4 > data.len() {
        return Err(PsdError::InvalidFormat("adjustment: unexpected end".to_string()));
    }
    let v = u32::from_be_bytes([data[*offset], data[*offset + 1], data[*offset + 2], data[*offset + 3]]);
    *offset += 4;
    Ok(v)
}

fn read_u8(data: &[u8], offset: &mut usize) -> Result<u8> {
    if *offset >= data.len() {
        return Err(PsdError::InvalidFormat("adjustment: unexpected end".to_string()));
    }
    let v = data[*offset];
    *offset += 1;
    Ok(v)
}

fn read_f32(data: &[u8], offset: &mut usize) -> Result<f32> {
    if *offset + 4 > data.len() {
        return Err(PsdError::InvalidFormat("adjustment: unexpected end".to_string()));
    }
    let v = f32::from_bits(u32::from_be_bytes([data[*offset], data[*offset+1], data[*offset+2], data[*offset+3]]));
    *offset += 4;
    Ok(v)
}

/// Brightness and Contrast adjustment layer (`brit`)
#[derive(Debug, Clone, PartialEq)]
pub struct BrightnessContrast {
    pub brightness: i16,
    pub contrast: i16,
}

/// Posterize adjustment layer (`post`)
#[derive(Debug, Clone, PartialEq)]
pub struct Posterize {
    pub levels: u16,
}

/// Threshold adjustment layer (`thrs`)
#[derive(Debug, Clone, PartialEq)]
pub struct Threshold {
    pub level: u16,
}

/// Exposure adjustment layer (`expA`)
#[derive(Debug, Clone, PartialEq)]
pub struct Exposure {
    pub exposure: f32,
    pub offset: f32,
    pub gamma: f32,
}

/// Color Balance adjustment layer (`blnc`)
#[derive(Debug, Clone, PartialEq)]
pub struct ColorBalance {
    pub shadows: [i16; 3],
    pub midtones: [i16; 3],
    pub highlights: [i16; 3],
    pub preserve_luminosity: bool,
}

/// Lab color used by PhotoFilter
#[derive(Debug, Clone, PartialEq)]
pub struct LabColor {
    pub l: f64,
    pub a: f64,
    pub b: f64,
}

/// Photo Filter adjustment layer (`phfl`)
#[derive(Debug, Clone, PartialEq)]
pub struct PhotoFilter {
    pub color: LabColor,
    pub density: u32,
    pub preserve_luminosity: bool,
    /// Raw mode byte for round-trip (2 = Lab, 3 = XYZ-sourced)
    pub mode: u16,
}

/// Levels channel (one entry in the Levels block)
#[derive(Debug, Clone, PartialEq)]
pub struct LevelsChannel {
    pub input_black: u16,
    pub input_white: u16,
    pub output_black: u16,
    pub output_white: u16,
    pub gamma: u16,
}

/// Levels adjustment layer (`levl`) — up to 4 meaningful channels stored
#[derive(Debug, Clone, PartialEq)]
pub struct Levels {
    pub channels: Vec<LevelsChannel>,
}

/// Hue/Saturation range entry
#[derive(Debug, Clone, PartialEq)]
pub struct HueSatRange {
    pub range: [i16; 4],
    pub adjust: [i16; 3],
}

/// Hue/Saturation adjustment layer (`hue2`)
#[derive(Debug, Clone, PartialEq)]
pub struct HueSaturation {
    pub colorize: bool,
    pub colorized_master: [i16; 3],
    pub master: [i16; 3],
    pub ranges: Vec<HueSatRange>,
}

/// Selective Color adjustment layer (`selc`)
#[derive(Debug, Clone, PartialEq)]
pub struct SelectiveColor {
    pub absolute: bool,
    pub adjustments: [[i16; 4]; 9],
}

/// Channel Mixer adjustment layer (`mixr`)
#[derive(Debug, Clone, PartialEq)]
pub struct ChannelMixer {
    pub monochrome: bool,
    pub values: [i16; 20],
}

/// One channel entry in a Curves block
#[derive(Debug, Clone, PartialEq)]
pub enum CurvesChannel {
    /// (mode 0) list of (input, output) pairs flattened as [x0, y0, x1, y1, ...]
    Points(Vec<u16>),
    /// (mode 1) 256-entry mapping table
    Mapping(Box<[u8; 256]>),
}

/// Curves adjustment layer (`curv`)
#[derive(Debug, Clone, PartialEq)]
pub struct Curves {
    /// 0 = point mode, 1 = mapping mode
    pub mode: u8,
    pub channels: [CurvesChannel; 4],
}

/// Color stop in a gradient
#[derive(Debug, Clone, PartialEq)]
pub struct GradientColorStop {
    pub location: u32,
    pub midpoint: u32,
    pub color: [u8; 3],
}

/// Transparency stop in a gradient
#[derive(Debug, Clone, PartialEq)]
pub struct GradientTransparencyStop {
    pub location: u32,
    pub midpoint: u32,
    pub opacity: u16,
}

/// Gradient definition
#[derive(Debug, Clone, PartialEq)]
pub struct GradientDefinition {
    pub name: String,
    pub interpolation: u16,
    pub color_stops: Vec<GradientColorStop>,
    pub transparency_stops: Vec<GradientTransparencyStop>,
}

/// Gradient Map adjustment layer (`grdm`)
#[derive(Debug, Clone, PartialEq)]
pub struct GradientMap {
    pub reverse: bool,
    pub dither: bool,
    pub interpolation_method: String,
    pub gradient: GradientDefinition,
}

// ─── Read functions ────────────────────────────────────────────────────────

pub fn read_brit(data: &[u8]) -> Result<BrightnessContrast> {
    let mut o = 0;
    Ok(BrightnessContrast {
        brightness: read_i16(data, &mut o)?,
        contrast:   read_i16(data, &mut o)?,
    })
}

pub fn read_post(data: &[u8]) -> Result<Posterize> {
    let mut o = 0;
    Ok(Posterize { levels: read_u16(data, &mut o)? })
}

pub fn read_thrs(data: &[u8]) -> Result<Threshold> {
    let mut o = 0;
    Ok(Threshold { level: read_u16(data, &mut o)? })
}

pub fn read_expa(data: &[u8]) -> Result<Exposure> {
    let mut o = 0;
    let _ = read_u16(data, &mut o)?; // skip
    Ok(Exposure {
        exposure: read_f32(data, &mut o)?,
        offset:   read_f32(data, &mut o)?,
        gamma:    read_f32(data, &mut o)?,
    })
}

pub fn read_blnc(data: &[u8]) -> Result<ColorBalance> {
    let mut o = 0;
    Ok(ColorBalance {
        shadows:    [read_i16(data, &mut o)?, read_i16(data, &mut o)?, read_i16(data, &mut o)?],
        midtones:   [read_i16(data, &mut o)?, read_i16(data, &mut o)?, read_i16(data, &mut o)?],
        highlights: [read_i16(data, &mut o)?, read_i16(data, &mut o)?, read_i16(data, &mut o)?],
        preserve_luminosity: read_u8(data, &mut o)? == 1,
    })
}

/// Convert CIE XYZ to Lab (D50 reference white)
fn xyz_to_lab(x: f64, y: f64, z: f64) -> LabColor {
    fn f(t: f64) -> f64 {
        if t > 0.008856 { t.cbrt() } else { 7.787 * t + 16.0 / 116.0 }
    }
    // D50
    let xn = 0.96429;
    let yn = 1.00000;
    let zn = 0.82513;
    let l = 116.0 * f(y / yn) - 16.0;
    let a = 500.0 * (f(x / xn) - f(y / yn));
    let b = 200.0 * (f(y / yn) - f(z / zn));
    LabColor { l, a, b }
}

pub fn read_phfl(data: &[u8]) -> Result<PhotoFilter> {
    let mut o = 0;
    let mode = read_u16(data, &mut o)?;
    let color = if mode == 3 {
        let scale = 32768.0f64;
        let x = read_u32(data, &mut o)? as f64 / scale;
        let y = read_u32(data, &mut o)? as f64 / scale;
        let z = read_u32(data, &mut o)? as f64 / scale;
        xyz_to_lab(x, y, z)
    } else {
        // mode 2: Lab color space
        let _cs = read_u16(data, &mut o)?; // colorSpace (must be 7)
        let l = read_i16(data, &mut o)? as f64 / 100.0;
        let a = read_i16(data, &mut o)? as f64 / 100.0;
        let b = read_i16(data, &mut o)? as f64 / 100.0;
        let _ = read_u16(data, &mut o)?;
        LabColor { l, a, b }
    };
    let density = read_u32(data, &mut o)?;
    let preserve_luminosity = read_u8(data, &mut o)? == 1;
    Ok(PhotoFilter { color, density, preserve_luminosity, mode })
}

pub fn read_levl(data: &[u8]) -> Result<Levels> {
    let mut o = 0;
    let _ = read_u16(data, &mut o)?; // version
    let mut all_channels = Vec::with_capacity(29);
    for _ in 0..29 {
        all_channels.push(LevelsChannel {
            input_black:  read_u16(data, &mut o)?,
            input_white:  read_u16(data, &mut o)?,
            output_black: read_u16(data, &mut o)?,
            output_white: read_u16(data, &mut o)?,
            gamma:        read_u16(data, &mut o)?,
        });
    }
    // Extended block (Lvls tag): optional extra channels
    if o + 8 <= data.len() {
        let tag = &data[o..o+4];
        o += 4;
        if tag == b"Lvls" {
            let _ = read_u16(data, &mut o)?; // version
            let count = read_u16(data, &mut o)? as usize;
            for _ in 29..count {
                all_channels.push(LevelsChannel {
                    input_black:  read_u16(data, &mut o)?,
                    input_white:  read_u16(data, &mut o)?,
                    output_black: read_u16(data, &mut o)?,
                    output_white: read_u16(data, &mut o)?,
                    gamma:        read_u16(data, &mut o)?,
                });
            }
        }
    }
    // Return at most 4 meaningful channels
    let channels = all_channels.into_iter().take(4).collect();
    Ok(Levels { channels })
}

pub fn read_hue2(data: &[u8]) -> Result<HueSaturation> {
    let mut o = 0;
    let _ = read_u16(data, &mut o)?; // version
    let colorize = read_u8(data, &mut o)? == 1;
    let _ = read_u8(data, &mut o)?; // pad
    let colorized_master = [read_i16(data, &mut o)?, read_i16(data, &mut o)?, read_i16(data, &mut o)?];
    let master = [read_i16(data, &mut o)?, read_i16(data, &mut o)?, read_i16(data, &mut o)?];
    let mut ranges = Vec::with_capacity(6);
    for _ in 0..6 {
        ranges.push(HueSatRange {
            range:  [read_i16(data, &mut o)?, read_i16(data, &mut o)?, read_i16(data, &mut o)?, read_i16(data, &mut o)?],
            adjust: [read_i16(data, &mut o)?, read_i16(data, &mut o)?, read_i16(data, &mut o)?],
        });
    }
    Ok(HueSaturation { colorize, colorized_master, master, ranges })
}

pub fn read_selc(data: &[u8]) -> Result<SelectiveColor> {
    let mut o = 0;
    let _ = read_u16(data, &mut o)?; // version
    let absolute = read_u16(data, &mut o)? == 1;
    // 4 i16 skipped
    for _ in 0..4 { let _ = read_i16(data, &mut o)?; }
    let mut adjustments = [[0i16; 4]; 9];
    for row in &mut adjustments {
        for col in row.iter_mut() {
            *col = read_i16(data, &mut o)?;
        }
    }
    Ok(SelectiveColor { absolute, adjustments })
}

pub fn read_mixr(data: &[u8]) -> Result<ChannelMixer> {
    let mut o = 0;
    let _ = read_u16(data, &mut o)?; // version
    let monochrome = read_u16(data, &mut o)? == 1;
    let mut values = [0i16; 20];
    for v in &mut values { *v = read_i16(data, &mut o)?; }
    Ok(ChannelMixer { monochrome, values })
}

pub fn read_curv(data: &[u8]) -> Result<Curves> {
    let mut o = 0;
    let mode = read_u8(data, &mut o)?;
    let _ = read_u8(data, &mut o)?; // pad
    let _ = read_u8(data, &mut o)?; // pad
    let bitmask = read_u32(data, &mut o)?;
    let default_points: Vec<u16> = vec![0, 0, 255, 255];
    let default_mapping: Box<[u8; 256]> = Box::new(std::array::from_fn(|i| i as u8));
    let mut channels: [CurvesChannel; 4] = [
        CurvesChannel::Points(default_points.clone()),
        CurvesChannel::Points(default_points.clone()),
        CurvesChannel::Points(default_points.clone()),
        CurvesChannel::Points(default_points.clone()),
    ];
    for i in 0..4 {
        let enabled = ((bitmask >> i) & 1) == 1;
        if !enabled { continue; }
        if mode == 0 {
            let count = read_u16(data, &mut o)? as usize;
            let mut pts = Vec::with_capacity(count * 2);
            for _ in 0..count {
                let y = read_u16(data, &mut o)?;
                let x = read_u16(data, &mut o)?;
                pts.push(x);
                pts.push(y);
            }
            channels[i] = CurvesChannel::Points(pts);
        } else {
            let mut m = [0u8; 256];
            for byte in &mut m { *byte = read_u8(data, &mut o)?; }
            channels[i] = CurvesChannel::Mapping(Box::new(m));
        }
    }
    Ok(Curves { mode, channels })
}

fn read_unicode_string_grdm(data: &[u8], o: &mut usize) -> Result<String> {
    let len = read_u32(data, o)? as usize;
    let mut s = String::with_capacity(len);
    for _ in 0..len {
        if *o + 2 > data.len() { break; }
        let c = u16::from_be_bytes([data[*o], data[*o + 1]]);
        *o += 2;
        s.push(char::from_u32(c as u32).unwrap_or('\u{FFFD}'));
    }
    Ok(s)
}

fn read_gradient_def(data: &[u8], o: &mut usize) -> Result<GradientDefinition> {
    let cs_count = read_u16(data, o)? as usize;
    let mut color_stops = Vec::with_capacity(cs_count);
    for _ in 0..cs_count {
        let location = read_u32(data, o)?;
        let midpoint = read_u32(data, o)?;
        let _ = read_u16(data, o)?; // color type
        let r = ((read_u16(data, o)? as u32 * 255 + 32767) / 65535) as u8;
        let g = ((read_u16(data, o)? as u32 * 255 + 32767) / 65535) as u8;
        let b = ((read_u16(data, o)? as u32 * 255 + 32767) / 65535) as u8;
        let _ = read_u16(data, o)?; // alpha
        color_stops.push(GradientColorStop { location, midpoint, color: [r, g, b] });
    }
    let ts_count = read_u16(data, o)? as usize;
    let mut transparency_stops = Vec::with_capacity(ts_count);
    for _ in 0..ts_count {
        let location = read_u32(data, o)?;
        let midpoint = read_u32(data, o)?;
        let opacity = read_u16(data, o)?;
        transparency_stops.push(GradientTransparencyStop { location, midpoint, opacity });
    }
    let _ = read_u16(data, o)?; // pad
    let interpolation = read_u16(data, o)?;
    let _ = read_u16(data, o)?; // pad
    Ok(GradientDefinition { name: String::new(), interpolation, color_stops, transparency_stops })
}

pub fn read_grdm(data: &[u8]) -> Result<GradientMap> {
    let mut o = 0;
    let _ = read_u16(data, &mut o)?; // version
    let reverse = read_u8(data, &mut o)? == 1;
    let dither = read_u8(data, &mut o)? == 1;
    let method_bytes = &data[o..o.min(data.len()).min(o+4)];
    let interpolation_method = match method_bytes {
        b"Lnr " => "Lnr".to_string(),
        _ => std::str::from_utf8(method_bytes).unwrap_or("Gcls").trim().to_string(),
    };
    o += 4;
    let name = read_unicode_string_grdm(data, &mut o)?;
    let mut gradient = read_gradient_def(data, &mut o)?;
    gradient.name = name;
    Ok(GradientMap { reverse, dither, interpolation_method, gradient })
}

// ─── Write functions ───────────────────────────────────────────────────────

pub fn write_brit(bc: &BrightnessContrast) -> Vec<u8> {
    let mut v = Vec::with_capacity(4);
    v.write_i16::<BigEndian>(bc.brightness).unwrap();
    v.write_i16::<BigEndian>(bc.contrast).unwrap();
    v
}

pub fn write_post(p: &Posterize) -> Vec<u8> {
    let mut v = Vec::with_capacity(2);
    v.write_u16::<BigEndian>(p.levels).unwrap();
    v
}

pub fn write_thrs(t: &Threshold) -> Vec<u8> {
    let mut v = Vec::with_capacity(2);
    v.write_u16::<BigEndian>(t.level).unwrap();
    v
}

pub fn write_expa(e: &Exposure) -> Vec<u8> {
    let mut v = Vec::with_capacity(14);
    v.write_u16::<BigEndian>(1).unwrap(); // version
    v.write_f32::<BigEndian>(e.exposure).unwrap();
    v.write_f32::<BigEndian>(e.offset).unwrap();
    v.write_f32::<BigEndian>(e.gamma).unwrap();
    v
}

pub fn write_blnc(cb: &ColorBalance) -> Vec<u8> {
    let mut v = Vec::with_capacity(19);
    for &x in &cb.shadows    { v.write_i16::<BigEndian>(x).unwrap(); }
    for &x in &cb.midtones   { v.write_i16::<BigEndian>(x).unwrap(); }
    for &x in &cb.highlights { v.write_i16::<BigEndian>(x).unwrap(); }
    v.push(if cb.preserve_luminosity { 1 } else { 0 });
    v
}

fn lab_to_xyz(lab: &LabColor) -> (f64, f64, f64) {
    let fy = (lab.l + 16.0) / 116.0;
    let fx = lab.a / 500.0 + fy;
    let fz = fy - lab.b / 200.0;
    fn inv(t: f64) -> f64 { if t > 0.206897 { t * t * t } else { (t - 16.0 / 116.0) / 7.787 } }
    let xn = 0.96429; let yn = 1.0; let zn = 0.82513;
    (inv(fx) * xn, inv(fy) * yn, inv(fz) * zn)
}

pub fn write_phfl(pf: &PhotoFilter) -> Vec<u8> {
    let mut v = Vec::with_capacity(16);
    v.write_u16::<BigEndian>(pf.mode).unwrap();
    if pf.mode == 3 {
        let scale = 32768.0f64;
        let (x, y, z) = lab_to_xyz(&pf.color);
        v.write_u32::<BigEndian>((x * scale).round() as u32).unwrap();
        v.write_u32::<BigEndian>((y * scale).round() as u32).unwrap();
        v.write_u32::<BigEndian>((z * scale).round() as u32).unwrap();
    } else {
        v.write_u16::<BigEndian>(7).unwrap(); // Lab color space
        v.write_i16::<BigEndian>((pf.color.l * 100.0).round() as i16).unwrap();
        v.write_i16::<BigEndian>((pf.color.a * 100.0).round() as i16).unwrap();
        v.write_i16::<BigEndian>((pf.color.b * 100.0).round() as i16).unwrap();
        v.write_u16::<BigEndian>(0).unwrap();
    }
    v.write_u32::<BigEndian>(pf.density).unwrap();
    v.push(if pf.preserve_luminosity { 1 } else { 0 });
    v
}

pub fn write_levl(lv: &Levels) -> Vec<u8> {
    let mut v = Vec::new();
    v.write_u16::<BigEndian>(2).unwrap(); // version
    let mut channels = lv.channels.clone();
    let default_ch = LevelsChannel { input_black: 0, input_white: 255, output_black: 0, output_white: 255, gamma: 100 };
    while channels.len() < 29 { channels.push(default_ch.clone()); }
    for ch in &channels[..29] {
        v.write_u16::<BigEndian>(ch.input_black).unwrap();
        v.write_u16::<BigEndian>(ch.input_white).unwrap();
        v.write_u16::<BigEndian>(ch.output_black).unwrap();
        v.write_u16::<BigEndian>(ch.output_white).unwrap();
        v.write_u16::<BigEndian>(ch.gamma).unwrap();
    }
    v
}

pub fn write_hue2(h: &HueSaturation) -> Vec<u8> {
    let mut v = Vec::new();
    v.write_u16::<BigEndian>(2).unwrap();
    v.push(if h.colorize { 1 } else { 0 });
    v.push(0);
    for &x in &h.colorized_master { v.write_i16::<BigEndian>(x).unwrap(); }
    for &x in &h.master { v.write_i16::<BigEndian>(x).unwrap(); }
    for r in &h.ranges {
        for &x in &r.range   { v.write_i16::<BigEndian>(x).unwrap(); }
        for &x in &r.adjust  { v.write_i16::<BigEndian>(x).unwrap(); }
    }
    v
}

pub fn write_selc(s: &SelectiveColor) -> Vec<u8> {
    let mut v = Vec::new();
    v.write_u16::<BigEndian>(1).unwrap();
    v.write_u16::<BigEndian>(if s.absolute { 1 } else { 0 }).unwrap();
    for _ in 0..4 { v.write_i16::<BigEndian>(0).unwrap(); }
    for row in &s.adjustments {
        for &x in row { v.write_i16::<BigEndian>(x).unwrap(); }
    }
    v
}

pub fn write_mixr(m: &ChannelMixer) -> Vec<u8> {
    let mut v = Vec::new();
    v.write_u16::<BigEndian>(1).unwrap();
    v.write_u16::<BigEndian>(if m.monochrome { 1 } else { 0 }).unwrap();
    for &x in &m.values { v.write_i16::<BigEndian>(x).unwrap(); }
    v
}

pub fn write_curv(c: &Curves) -> Vec<u8> {
    let mut v = Vec::new();
    v.push(c.mode);
    v.push(0); v.push(1); // pads
    v.write_u32::<BigEndian>(15).unwrap(); // all 4 channels enabled
    for ch in &c.channels {
        match ch {
            CurvesChannel::Points(pts) => {
                v.write_u16::<BigEndian>((pts.len() / 2) as u16).unwrap();
                for i in (0..pts.len()).step_by(2) {
                    v.write_u16::<BigEndian>(pts[i + 1]).unwrap(); // y first
                    v.write_u16::<BigEndian>(pts[i]).unwrap();     // then x
                }
            }
            CurvesChannel::Mapping(m) => {
                v.extend_from_slice(m.as_ref());
            }
        }
    }
    v
}

fn write_unicode_string_grdm(v: &mut Vec<u8>, s: &str) {
    v.write_u32::<BigEndian>(s.len() as u32).unwrap();
    for c in s.chars() { v.write_u16::<BigEndian>(c as u16).unwrap(); }
}

fn write_gradient_def(v: &mut Vec<u8>, g: &GradientDefinition) {
    v.write_u16::<BigEndian>(g.color_stops.len() as u16).unwrap();
    for s in &g.color_stops {
        v.write_u32::<BigEndian>(s.location).unwrap();
        v.write_u32::<BigEndian>(s.midpoint).unwrap();
        v.write_u16::<BigEndian>(0).unwrap();
        v.write_u16::<BigEndian>((s.color[0] as u32 * 65535 / 255) as u16).unwrap();
        v.write_u16::<BigEndian>((s.color[1] as u32 * 65535 / 255) as u16).unwrap();
        v.write_u16::<BigEndian>((s.color[2] as u32 * 65535 / 255) as u16).unwrap();
        v.write_u16::<BigEndian>(0).unwrap();
    }
    v.write_u16::<BigEndian>(g.transparency_stops.len() as u16).unwrap();
    for s in &g.transparency_stops {
        v.write_u32::<BigEndian>(s.location).unwrap();
        v.write_u32::<BigEndian>(s.midpoint).unwrap();
        v.write_u16::<BigEndian>(s.opacity).unwrap();
    }
    v.write_u16::<BigEndian>(2).unwrap();
    v.write_u16::<BigEndian>(g.interpolation).unwrap();
    v.write_u16::<BigEndian>(32).unwrap();
}

pub fn write_grdm(g: &GradientMap) -> Vec<u8> {
    let mut v = Vec::new();
    v.write_u16::<BigEndian>(3).unwrap();
    v.push(if g.reverse { 1 } else { 0 });
    v.push(if g.dither  { 1 } else { 0 });
    let method = match g.interpolation_method.as_str() {
        "Lnr" => "Lnr ".to_string(),
        s => format!("{:<4}", s),
    };
    v.extend_from_slice(method.as_bytes().get(..4).unwrap_or(b"Gcls"));
    write_unicode_string_grdm(&mut v, &g.gradient.name);
    write_gradient_def(&mut v, &g.gradient);
    // trailing fields matching TS serializeGradientMapBlock
    v.write_u16::<BigEndian>(1).unwrap();
    v.write_u32::<BigEndian>(2048).unwrap();
    v.write_u16::<BigEndian>(0).unwrap();
    v.write_u16::<BigEndian>(0).unwrap();
    v.write_u32::<BigEndian>(0).unwrap();
    v.write_u16::<BigEndian>(3).unwrap();
    v.extend_from_slice(&[0u8; 8]);
    for _ in 0..4 { v.write_u16::<BigEndian>(32768).unwrap(); }
    v.write_u16::<BigEndian>(0).unwrap();
    v
}

// ─── Dispatch enum ─────────────────────────────────────────────────────────

/// All structured adjustment-layer block types.
#[derive(Debug, Clone, PartialEq)]
pub enum AdjustmentLayer {
    BrightnessContrast(BrightnessContrast),
    Invert,
    Posterize(Posterize),
    Threshold(Threshold),
    Exposure(Exposure),
    ColorBalance(ColorBalance),
    PhotoFilter(PhotoFilter),
    Levels(Levels),
    HueSaturation(HueSaturation),
    SelectiveColor(SelectiveColor),
    ChannelMixer(ChannelMixer),
    Curves(Curves),
    GradientMap(GradientMap),
    /// `blwh` and any other descriptor-based adjustment: raw bytes preserved.
    Raw(String, Vec<u8>),
}

impl AdjustmentLayer {
    /// PSD 4-char key for this adjustment.
    pub fn key(&self) -> &str {
        match self {
            AdjustmentLayer::BrightnessContrast(_) => "brit",
            AdjustmentLayer::Invert                => "nvrt",
            AdjustmentLayer::Posterize(_)          => "post",
            AdjustmentLayer::Threshold(_)          => "thrs",
            AdjustmentLayer::Exposure(_)           => "expA",
            AdjustmentLayer::ColorBalance(_)       => "blnc",
            AdjustmentLayer::PhotoFilter(_)        => "phfl",
            AdjustmentLayer::Levels(_)             => "levl",
            AdjustmentLayer::HueSaturation(_)      => "hue2",
            AdjustmentLayer::SelectiveColor(_)     => "selc",
            AdjustmentLayer::ChannelMixer(_)       => "mixr",
            AdjustmentLayer::Curves(_)             => "curv",
            AdjustmentLayer::GradientMap(_)        => "grdm",
            AdjustmentLayer::Raw(k, _)             => k.as_str(),
        }
    }

    /// Deserialise from a PSD key and its raw data bytes.
    pub fn from_key_and_bytes(key: &str, data: &[u8]) -> Result<Self> {
        match key {
            "brit" => Ok(AdjustmentLayer::BrightnessContrast(read_brit(data)?)),
            "nvrt" => Ok(AdjustmentLayer::Invert),
            "post" => Ok(AdjustmentLayer::Posterize(read_post(data)?)),
            "thrs" => Ok(AdjustmentLayer::Threshold(read_thrs(data)?)),
            "expA" => Ok(AdjustmentLayer::Exposure(read_expa(data)?)),
            "blnc" => Ok(AdjustmentLayer::ColorBalance(read_blnc(data)?)),
            "phfl" => Ok(AdjustmentLayer::PhotoFilter(read_phfl(data)?)),
            "levl" => Ok(AdjustmentLayer::Levels(read_levl(data)?)),
            "hue2" => Ok(AdjustmentLayer::HueSaturation(read_hue2(data)?)),
            "selc" => Ok(AdjustmentLayer::SelectiveColor(read_selc(data)?)),
            "mixr" => Ok(AdjustmentLayer::ChannelMixer(read_mixr(data)?)),
            "curv" => Ok(AdjustmentLayer::Curves(read_curv(data)?)),
            "grdm" => Ok(AdjustmentLayer::GradientMap(read_grdm(data)?)),
            _      => Ok(AdjustmentLayer::Raw(key.to_string(), data.to_vec())),
        }
    }

    /// Serialise to raw bytes (no key or length header).
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        match self {
            AdjustmentLayer::BrightnessContrast(v) => Ok(write_brit(v)),
            AdjustmentLayer::Invert                => Ok(Vec::new()),
            AdjustmentLayer::Posterize(v)          => Ok(write_post(v)),
            AdjustmentLayer::Threshold(v)          => Ok(write_thrs(v)),
            AdjustmentLayer::Exposure(v)           => Ok(write_expa(v)),
            AdjustmentLayer::ColorBalance(v)       => Ok(write_blnc(v)),
            AdjustmentLayer::PhotoFilter(v)        => Ok(write_phfl(v)),
            AdjustmentLayer::Levels(v)             => Ok(write_levl(v)),
            AdjustmentLayer::HueSaturation(v)      => Ok(write_hue2(v)),
            AdjustmentLayer::SelectiveColor(v)     => Ok(write_selc(v)),
            AdjustmentLayer::ChannelMixer(v)       => Ok(write_mixr(v)),
            AdjustmentLayer::Curves(v)             => Ok(write_curv(v)),
            AdjustmentLayer::GradientMap(v)        => Ok(write_grdm(v)),
            AdjustmentLayer::Raw(_, data)          => Ok(data.clone()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn brit_roundtrip() {
        let orig = BrightnessContrast { brightness: 50, contrast: -30 };
        let bytes = write_brit(&orig);
        let back = read_brit(&bytes).unwrap();
        assert_eq!(back, orig);
    }

    #[test]
    fn expa_roundtrip() {
        let orig = Exposure { exposure: 1.5, offset: -0.0001, gamma: 0.9 };
        let bytes = write_expa(&orig);
        let back = read_expa(&bytes).unwrap();
        assert!((back.exposure - orig.exposure).abs() < 1e-5);
        assert!((back.gamma - orig.gamma).abs() < 1e-5);
    }

    #[test]
    fn blnc_roundtrip() {
        let orig = ColorBalance {
            shadows: [10, -5, 3],
            midtones: [0, 0, 0],
            highlights: [-10, 20, -3],
            preserve_luminosity: true,
        };
        let bytes = write_blnc(&orig);
        let back = read_blnc(&bytes).unwrap();
        assert_eq!(back, orig);
    }

    #[test]
    fn levl_roundtrip() {
        let ch = LevelsChannel { input_black: 10, input_white: 240, output_black: 0, output_white: 255, gamma: 110 };
        let orig = Levels { channels: vec![ch] };
        let bytes = write_levl(&orig);
        let back = read_levl(&bytes).unwrap();
        assert_eq!(back.channels[0].input_black, 10);
        assert_eq!(back.channels[0].gamma, 110);
    }

    #[test]
    fn hue2_roundtrip() {
        let orig = HueSaturation {
            colorize: false,
            colorized_master: [0, 0, 0],
            master: [10, -20, 5],
            ranges: (0..6).map(|_| HueSatRange { range: [0, 30, 60, 90], adjust: [0, 0, 0] }).collect(),
        };
        let bytes = write_hue2(&orig);
        let back = read_hue2(&bytes).unwrap();
        assert_eq!(back.master, orig.master);
    }

    #[test]
    fn selc_roundtrip() {
        let orig = SelectiveColor {
            absolute: true,
            adjustments: [[1, 2, 3, 4]; 9],
        };
        let bytes = write_selc(&orig);
        let back = read_selc(&bytes).unwrap();
        assert_eq!(back, orig);
    }

    #[test]
    fn mixr_roundtrip() {
        let mut values = [0i16; 20];
        values[0] = 100; values[5] = -50;
        let orig = ChannelMixer { monochrome: false, values };
        let bytes = write_mixr(&orig);
        let back = read_mixr(&bytes).unwrap();
        assert_eq!(back, orig);
    }

    #[test]
    fn curv_points_roundtrip() {
        let ch = CurvesChannel::Points(vec![0, 0, 128, 160, 255, 255]);
        let orig = Curves {
            mode: 0,
            channels: [ch, CurvesChannel::Points(vec![0, 0, 255, 255]),
                          CurvesChannel::Points(vec![0, 0, 255, 255]),
                          CurvesChannel::Points(vec![0, 0, 255, 255])],
        };
        let bytes = write_curv(&orig);
        let back = read_curv(&bytes).unwrap();
        if let CurvesChannel::Points(pts) = &back.channels[0] {
            assert_eq!(pts.len(), 6);
            assert_eq!(pts[2], 128);
        } else { panic!("wrong channel type"); }
    }

    #[test]
    fn adjustment_layer_dispatch() {
        let al = AdjustmentLayer::BrightnessContrast(BrightnessContrast { brightness: 10, contrast: 5 });
        assert_eq!(al.key(), "brit");
        let bytes = al.to_bytes().unwrap();
        let back = AdjustmentLayer::from_key_and_bytes("brit", &bytes).unwrap();
        assert_eq!(al, back);
    }
}
```

- [ ] **Step 4.3: Run tests**

```bash
cargo test 2>&1
```
Expected: all pass including the new `adjustments::tests::*` tests.

- [ ] **Step 4.4: Commit**

```bash
git add src/adjustments.rs src/lib.rs
git commit -m "feat: add adjustments module with typed read/write for all 13 adjustment layer types"
```

---

## Task 5: Wire `AdjustmentLayer` into `LayerAdditionalInfo`

**Files:**
- Modify: `src/additional_info.rs`

Change `adjustment: Option<(String, Vec<u8>)>` to `adjustment: Option<AdjustmentLayer>` and update all match arms.

- [ ] **Step 5.1: Update the `adjustment` field in `LayerAdditionalInfo`**

In `src/additional_info.rs`, add at the top of imports:
```rust
use crate::adjustments::AdjustmentLayer;
```

Change the field:
```rust
// OLD:
pub adjustment: Option<(String, Vec<u8>)>,
// NEW:
pub adjustment: Option<AdjustmentLayer>,
```

- [ ] **Step 5.2: Update the read arm in `read_additional_info`**

Replace:
```rust
"brit" | "levl" | "curv" | "expA" | "blnc" | "phfl" | "hue2" |
"selc" | "mixr" | "post" | "thrs" | "nvrt" | "grdm" | "blwh" => {
    let data = self.read_bytes(length)?;
    info.adjustment = Some((key.to_string(), data));
}
```
With:
```rust
"brit" | "levl" | "curv" | "expA" | "blnc" | "phfl" | "hue2" |
"selc" | "mixr" | "post" | "thrs" | "nvrt" | "grdm" | "blwh" => {
    let data = self.read_bytes(length)?;
    info.adjustment = Some(AdjustmentLayer::from_key_and_bytes(key, &data)?);
}
```

- [ ] **Step 5.3: Update the write arm in `write_additional_info`**

Replace:
```rust
"brit" | "levl" | "curv" | "expA" | "blnc" | "phfl" | "hue2" |
"selc" | "mixr" | "post" | "thrs" | "nvrt" | "grdm" | "blwh" => {
    if let Some((ref adj_key, ref data)) = info.adjustment {
        if adj_key == key {
            temp_writer.write_bytes(data)?;
        }
    }
}
```
With:
```rust
"brit" | "levl" | "curv" | "expA" | "blnc" | "phfl" | "hue2" |
"selc" | "mixr" | "post" | "thrs" | "nvrt" | "grdm" | "blwh" => {
    if let Some(ref adj) = info.adjustment {
        if adj.key() == key {
            temp_writer.write_bytes(&adj.to_bytes()?)?;
        }
    }
}
```

- [ ] **Step 5.4: Update the write_layer_additional_info adjustment block**

Replace:
```rust
// Write adjustment layer block
if let Some((ref adj_key, ref data)) = info.adjustment {
    writer.write_signature("8BIM")?;
    writer.write_signature(adj_key)?;
    writer.write_u32(data.len() as u32)?;
    writer.write_bytes(data)?;
    if data.len() % 2 != 0 {
        writer.write_u8(0)?;
    }
}
```
With:
```rust
// Write adjustment layer block
if let Some(ref adj) = info.adjustment {
    let data = adj.to_bytes()?;
    writer.write_signature("8BIM")?;
    writer.write_signature(adj.key())?;
    writer.write_u32(data.len() as u32)?;
    writer.write_bytes(&data)?;
    if data.len() % 2 != 0 {
        writer.write_u8(0)?;
    }
}
```

- [ ] **Step 5.5: Update the existing adjustment test**

Replace the old `adjustment_brit_roundtrip` test with:
```rust
#[test]
fn adjustment_brit_roundtrip() {
    use crate::adjustments::{AdjustmentLayer, BrightnessContrast};
    let mut info = LayerAdditionalInfo::default();
    info.adjustment = Some(AdjustmentLayer::BrightnessContrast(BrightnessContrast { brightness: 50, contrast: -42 }));

    let mut writer = PsdWriter::new(128);
    let length = writer.write_additional_info("brit", &info).unwrap();
    assert_eq!(length, 4);

    let buffer = writer.into_buffer();
    let cursor = std::io::Cursor::new(buffer);
    let mut reader = PsdReader::new(cursor, Default::default());

    let mut read_info = LayerAdditionalInfo::default();
    reader.read_additional_info("brit", length, &mut read_info).unwrap();

    match read_info.adjustment.unwrap() {
        AdjustmentLayer::BrightnessContrast(bc) => {
            assert_eq!(bc.brightness, 50);
            assert_eq!(bc.contrast, -42);
        }
        other => panic!("wrong variant: {:?}", other),
    }
}
```

- [ ] **Step 5.6: Run tests**

```bash
cargo test 2>&1
```
Expected: all pass.

- [ ] **Step 5.7: Commit**

```bash
git add src/additional_info.rs
git commit -m "refactor: adjustment field uses AdjustmentLayer enum instead of raw bytes"
```

---

## Task 6: Add `lmfx`/`lfxs` and `Patt`/`Pat2`/`Pat3` routing

**Files:**
- Modify: `src/additional_info.rs`

`lmfx` and `lfxs` are layer-effects blocks. `Patt`/`Pat2`/`Pat3` are pattern blocks. Both currently fall to `unknown`. Give them named raw-byte fields so callers can distinguish them from truly unknown data.

- [ ] **Step 6.1: Add fields to `LayerAdditionalInfo`**

```rust
/// Raw bytes of lmfx/lfxs (newer layer effects tagged blocks)
pub layer_effects_raw: Option<(String, Vec<u8>)>,
/// Raw bytes of Patt/Pat2/Pat3 pattern blocks
pub pattern_data: Option<(String, Vec<u8>)>,
```

- [ ] **Step 6.2: Add read arms**

Before the `_ =>` arm in `read_additional_info`:
```rust
"lmfx" | "lfxs" => {
    let data = self.read_bytes(length)?;
    info.layer_effects_raw = Some((key.to_string(), data));
}
"Patt" | "Pat2" | "Pat3" => {
    let data = self.read_bytes(length)?;
    info.pattern_data = Some((key.to_string(), data));
}
```

Remove `"lmfx"` and `"lfxs"` from the minor-block routing arm (Task 10 from the previous plan):
```rust
// Change:
"lmfx" | "lfxs" | "FMsk" | ...
// To:
"FMsk" | "Anno" | ...  (remove lmfx and lfxs from this list)
```

- [ ] **Step 6.3: Add write arms in `write_additional_info`**

```rust
"lmfx" | "lfxs" => {
    if let Some((ref k, ref data)) = info.layer_effects_raw {
        if k == key {
            temp_writer.write_bytes(data)?;
        }
    }
}
"Patt" | "Pat2" | "Pat3" => {
    if let Some((ref k, ref data)) = info.pattern_data {
        if k == key {
            temp_writer.write_bytes(data)?;
        }
    }
}
```

- [ ] **Step 6.4: Add to sections list**

```rust
let sections = vec![
    // existing...
    "lmfx", "Patt",
];
```

- [ ] **Step 6.5: Write tests**

```rust
#[test]
fn lmfx_raw_preserved() {
    let mut info = LayerAdditionalInfo::default();
    info.layer_effects_raw = Some(("lmfx".to_string(), vec![0x01, 0x02]));

    let mut w = PsdWriter::new(64);
    write_layer_additional_info(&mut w, &info).unwrap();
    let buf = w.into_buffer();
    assert!(buf.windows(4).any(|w| w == b"lmfx"));
}

#[test]
fn patt_raw_preserved() {
    let mut info = LayerAdditionalInfo::default();
    info.pattern_data = Some(("Patt".to_string(), vec![0xAB, 0xCD]));

    let mut w = PsdWriter::new(64);
    write_layer_additional_info(&mut w, &info).unwrap();
    let buf = w.into_buffer();
    assert!(buf.windows(4).any(|w| w == b"Patt"));
}
```

- [ ] **Step 6.6: Run tests**

```bash
cargo test 2>&1
```
Expected: all pass.

- [ ] **Step 6.7: Commit**

```bash
git add src/additional_info.rs
git commit -m "feat: named fields for lmfx/lfxs effects blocks and Patt/Pat2/Pat3 pattern blocks"
```

---

## Self-Review

**Spec coverage:**
- vscg read bug ✅ Task 2
- vstk read bug ✅ Task 1
- PlLd UUID bug ✅ Task 3
- brit/nvrt/post/thrs/expA/blnc structured ✅ Task 4
- phfl structured ✅ Task 4
- levl/hue2/selc/mixr/curv/grdm structured ✅ Task 4
- blwh — stored as `Raw` variant in AdjustmentLayer (descriptor format, raw bytes preserved) ✅ Task 5
- lmfx/lfxs named field ✅ Task 6
- Patt/Pat2/Pat3 named field ✅ Task 6

**Out of scope (explicitly excluded):**
- Text engine data parser (EngineData inside TySh) — complex separate subsystem
- Slices (1050) — raw bytes preserved
- Renderer / compositor
