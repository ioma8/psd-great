# Fix 09 — Effects and Adjustments: Legacy Effects Errors, Intensity Loss, Gradient Rounding

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fix 4 bugs in `src/effects_helpers.rs` and `src/adjustments.rs`: (1) legacy effects reject valid PSD files where `visible == 0`; (2) shadow/glow `intensity` field is silently discarded on read and always written as 0; (3) satin/gradient-overlay/pattern-overlay/stroke effects trigger a parse error instead of being skipped gracefully; (4) gradient color stop u8→u16 scaling rounds differently than TS `Math.round` at odd byte values (±1 unit).

**Architecture:** All fixes are isolated to `src/effects_helpers.rs` (bugs 1–3) and `src/adjustments.rs` (bug 4). No struct changes for bug 1/3; bug 2 needs an `intensity` field added to shadow/glow structs; bug 4 is a formula change.

**Tech Stack:** Rust, `cargo test`

**TS reference:** `photoshop/psd/src/psd/layer-effects.ts`, `photoshop/psd/src/psd/adjustments.ts`, `photoshop/psd/src/psd/gradient-map.ts`

---

### Task 1: Fix `visible == 0` — mark effects disabled, don't error

**Bug:** `effects_helpers.rs:98` returns `Err` when `common.visible == 0`. A valid PSD where the user has turned off all layer effects triggers a parse error.

**Files:**
- Modify: `src/effects_helpers.rs` (~line 98)

- [ ] **Step 1: Find the check**

```bash
grep -n "visible\|cmnS\|common.*visible\|Invalid effects" /Users/jakubkolcar/projects/customs/ag-psd-rust/src/effects_helpers.rs | head -20
```

- [ ] **Step 2: Write a failing test**

```rust
#[test]
fn effects_with_visible_zero_does_not_error() {
    // Build minimal cmnS record with visible=0
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"8BIM");
    bytes.extend_from_slice(b"lrFX");
    // ... minimal valid lrFX block with visible=0 in cmnS
    // This test verifies parse does not return Err
}
```

For now, the test intent is: any effects block with `visible=0` in the common state record should parse successfully and set `effects.all_disabled = true` (or equivalent).

- [ ] **Step 3: Fix the check**

In `src/effects_helpers.rs`, change:

```rust
// BEFORE:
if common.size != 7 || common.version != 0 || common.visible == 0 {
    return Err(PsdError::InvalidFormat("Invalid effects common state".to_string()));
}

// AFTER:
if common.size != 7 || common.version != 0 {
    return Err(PsdError::InvalidFormat("Invalid effects common state".to_string()));
}
// visible == 0 means all effects are disabled — store this
effects.all_effects_disabled = common.visible == 0;
```

Add `all_effects_disabled: bool` to the effects struct if it doesn't exist.

- [ ] **Step 4: Run all tests**

```bash
cargo test 2>&1
```
Expected: all pass.

- [ ] **Step 5: Commit**

```bash
git add src/effects_helpers.rs
git commit -m "fix: legacy effects visible=0 means disabled, not a parse error"
```

---

### Task 2: Preserve `intensity` field in shadow/glow effects

**Bug:** The `intensity` fixed-point value is read and discarded (`let _intensity = reader.read_fixed_point_32()?`), and always written back as `0.0`. This silently mutates files on round-trip.

**Files:**
- Modify: `src/effects_helpers.rs` (drop shadow, inner shadow, outer glow, inner glow read/write)
- Modify: `src/layer.rs` or wherever the effect structs are defined

- [ ] **Step 1: Find the intensity reads**

```bash
grep -n "_intensity\|intensity" /Users/jakubkolcar/projects/customs/ag-psd-rust/src/effects_helpers.rs | head -20
grep -n "struct.*Shadow\|struct.*Glow\|intensity" /Users/jakubkolcar/projects/customs/ag-psd-rust/src/layer.rs | head -20
```

- [ ] **Step 2: Add `intensity` field to affected structs**

For each of `DropShadow`, `InnerShadow`, `OuterGlow`, `InnerGlow` (or however they're named), add:

```rust
pub intensity: f64,
```

Default value: `75.0` (matching Photoshop's default).

- [ ] **Step 3: Fix readers to store intensity**

Replace:
```rust
let _intensity = reader.read_fixed_point_32()?;
```
With:
```rust
effect.intensity = reader.read_fixed_point_32()?;
```

- [ ] **Step 4: Fix writers to emit stored intensity**

Replace:
```rust
writer.write_fixed_point_32(0.0)?; // intensity
```
With:
```rust
writer.write_fixed_point_32(effect.intensity)?;
```

- [ ] **Step 5: Run all tests**

```bash
cargo test 2>&1
```
Expected: all pass.

- [ ] **Step 6: Commit**

```bash
git add src/effects_helpers.rs src/layer.rs
git commit -m "fix: legacy effects intensity field preserved on read/write (was always discarded as 0)"
```

---

### Task 3: Handle unknown legacy effect blocks gracefully instead of erroring

**Bug:** `effects_helpers.rs:378-385` returns `Err` for any unknown effect key (e.g., `ChFX` satin, stroke, gradient overlay, pattern overlay). This causes parsing to fail on PSD files from Photoshop 7+.

**Files:**
- Modify: `src/effects_helpers.rs` (the effect-type match `_` arm)

- [ ] **Step 1: Find the unknown effect arm**

```bash
grep -n "Unknown effect\|_ =>\|InvalidFormat.*effect\|ChFX\|sofi\|oglw" /Users/jakubkolcar/projects/customs/ag-psd-rust/src/effects_helpers.rs | head -20
```

- [ ] **Step 2: Replace the error arm with a skip**

The block length is available before the match. Use it to skip unknown blocks:

```rust
// BEFORE:
_ => {
    return Err(PsdError::InvalidFormat(format!("Unknown effect type: {}", key)));
}

// AFTER:
_ => {
    // Unknown effect type (e.g. ChFX satin, stroke, gradient overlay added in PS 7+)
    // Skip the block rather than failing — block_size was read before the match
    reader.skip_bytes(block_size - /* already consumed header bytes */ N)?;
}
```

The exact number of already-consumed bytes depends on the structure. Check what has been read before the match (typically `size`, `version`, and sometimes a `blend_mode` block). Skip the remaining `block_size - consumed` bytes.

Alternatively, if the outer loop has a size guard, just break cleanly:
```rust
_ => {
    // Skip unknown effect block
    break;
}
```

- [ ] **Step 3: Run all tests**

```bash
cargo test 2>&1
```

- [ ] **Step 4: Commit**

```bash
git add src/effects_helpers.rs
git commit -m "fix: unknown legacy effect blocks (satin/stroke etc.) skipped gracefully not error"
```

---

### Task 4: Fix gradient color stop rounding — use `Math.round` equivalent

**Bug:** `adjustments.rs:885-887` uses integer rounding `(x * 65535 + 127) / 255` for u8→u16 color scaling. TS uses `Math.round((x / 255) * 65535)`. For odd values like `x=127`, Rust gives 32639 and TS gives 32640 (±1 unit difference).

**Files:**
- Modify: `src/adjustments.rs` (gradient color stop write, ~line 885)

- [ ] **Step 1: Write a test**

```rust
#[test]
fn gradient_color_stop_scaling_matches_ts_math_round() {
    // TS: Math.round((127 / 255) * 65535) = Math.round(32639.88) = 32640
    let scale_to_u16 = |x: u8| -> u16 {
        ((x as f64 / 255.0) * 65535.0).round() as u16
    };
    assert_eq!(scale_to_u16(127), 32640);
    assert_eq!(scale_to_u16(128), 32896);
    assert_eq!(scale_to_u16(0),   0);
    assert_eq!(scale_to_u16(255), 65535);
}
```

- [ ] **Step 2: Fix the write formula**

Replace:
```rust
// BEFORE:
((stop.color[0] as u32 * 65535 + 127) / 255) as u16
```
With:
```rust
// AFTER:
((stop.color[0] as f64 / 255.0) * 65535.0).round() as u16
```

Apply the same change to channels 1, 2, 3.

- [ ] **Step 3: Fix the read formula to match**

Find the corresponding read formula (line ~630):
```rust
// BEFORE:
((c.read_u16()? as u32 * 255 + 32767) / 65535) as u8
```
Change to:
```rust
// AFTER:
((c.read_u16()? as f64 / 65535.0) * 255.0).round() as u8
```

- [ ] **Step 4: Run all tests**

```bash
cargo test 2>&1
```

- [ ] **Step 5: Commit**

```bash
git add src/adjustments.rs
git commit -m "fix: gradient color stop u8<->u16 scaling uses floating-point round to match TS Math.round"
```
