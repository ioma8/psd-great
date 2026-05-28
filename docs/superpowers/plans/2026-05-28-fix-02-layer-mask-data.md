# Fix 02 — Layer Mask Data: Real-Mask Field Reading and Writing

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fix three related bugs in layer mask data parsing: (1) Rust unconditionally reads 18 bytes of "real mask" before parameter flags regardless of whether channel `-3` is present — corrupting all subsequent reads when the channel is absent; (2) Rust never reads real-mask fields for the older >20-byte mask format that lacks `HAS_PARAMETERS`; (3) the writer always emits 18 zero bytes for the real-mask block even when no real mask exists.

**Architecture:** All fixes are in `src/reader.rs` (`read_layer_mask_data` function) and `src/writer.rs` (the mask-write block). The `LayerMaskData` struct in `src/layer.rs` needs two new optional fields for real-mask data.

**Tech Stack:** Rust, `cargo test`

**TS reference:** `photoshop/psd/src/psd/layer-mask-data.ts`

---

### Task 1: Add `real_mask` fields to `LayerMaskData`

**Files:**
- Modify: `src/layer.rs` (find `pub struct LayerMaskData`)

- [ ] **Step 1: Add optional real-mask fields to the struct**

Find `pub struct LayerMaskData` in `src/layer.rs` and add two fields:

```rust
pub struct LayerMaskData {
    // ... existing fields ...
    pub real_flags_byte: Option<u8>,
    pub real_default_color: Option<u8>,
    pub real_top: Option<i32>,
    pub real_left: Option<i32>,
    pub real_bottom: Option<i32>,
    pub real_right: Option<i32>,
}
```

- [ ] **Step 2: Run all tests to confirm nothing is broken**

```bash
cargo test 2>&1
```
Expected: all tests pass (new fields have `Option` defaults so `Default::default()` still works).

- [ ] **Step 3: Commit**

```bash
git add src/layer.rs
git commit -m "feat: add real_mask fields to LayerMaskData struct"
```

---

### Task 2: Fix reader — condition real-mask on channel -3 presence

**Bug:** `reader.rs:663-688` — when `HAS_PARAMETERS` is set, Rust unconditionally reads 18 bytes (real flags + real default + real rect) before reading the parameter-flags byte. TS only reads those 18 bytes when channel ID `-3` (RealUserMask) is present in the layer's channel list.

**Files:**
- Modify: `src/reader.rs:636-697` (`read_layer_mask_data` function)
- The function signature needs access to channel IDs — check what is already passed in.

- [ ] **Step 1: Check the call site of `read_layer_mask_data`**

Search for `read_layer_mask_data(` in `src/reader.rs`. Note whether `channels: &[ChannelInfo]` is already available at the call site. If not, thread it through.

```bash
grep -n "read_layer_mask_data" /Users/jakubkolcar/projects/customs/ag-psd-rust/src/reader.rs
```

- [ ] **Step 2: Update the function signature to accept channel IDs**

If channels aren't already passed in, update the signature:

```rust
fn read_layer_mask_data<R: Read + Seek>(
    reader: &mut PsdReader<R>,
    layer: &mut Layer,
    channel_ids: &[i16],  // ADD THIS
) -> Result<()> {
```

And update the call site to pass `&channels.iter().map(|c| c.id as i16).collect::<Vec<_>>()`.

- [ ] **Step 3: Fix the `HAS_PARAMETERS` branch**

Replace the current `HAS_PARAMETERS` block (lines ~663–688) with:

```rust
        let remaining = reader.bytes_left(end_offset) as usize;
        if remaining >= 18 {
            if flags.contains(LayerMaskStateBits::HAS_PARAMETERS) {
                // Real mask fields are only present when channel -3 (RealUserMask) exists
                let has_real_mask_channel = channel_ids.contains(&-3);
                if has_real_mask_channel && reader.bytes_left(end_offset) >= 18 {
                    mask.real_flags_byte = Some(reader.read_u8()?);
                    mask.real_default_color = Some(reader.read_u8()?);
                    mask.real_top    = Some(reader.read_i32()?);
                    mask.real_left   = Some(reader.read_i32()?);
                    mask.real_bottom = Some(reader.read_i32()?);
                    mask.real_right  = Some(reader.read_i32()?);
                }
                if reader.bytes_left(end_offset) > 0 {
                    let param_flags = LayerMaskParameterFlags::from_bits_retain(reader.read_u8()?);
                    if param_flags.contains(LayerMaskParameterFlags::USER_MASK_DENSITY)
                        && reader.bytes_left(end_offset) > 0
                    {
                        mask.user_mask_density = Some(reader.read_u8()? as f64);
                    }
                    if param_flags.contains(LayerMaskParameterFlags::USER_MASK_FEATHER)
                        && reader.bytes_left(end_offset) >= 8
                    {
                        mask.user_mask_feather = Some(reader.read_f64()?);
                    }
                    if param_flags.contains(LayerMaskParameterFlags::VECTOR_MASK_DENSITY)
                        && reader.bytes_left(end_offset) > 0
                    {
                        mask.vector_mask_density = Some(reader.read_u8()? as f64);
                    }
                    if param_flags.contains(LayerMaskParameterFlags::VECTOR_MASK_FEATHER)
                        && reader.bytes_left(end_offset) >= 8
                    {
                        mask.vector_mask_feather = Some(reader.read_f64()?);
                    }
                    // 2-byte alignment after parameter flags block
                    if reader.bytes_left(end_offset) % 2 != 0 {
                        reader.skip_bytes(1)?;
                    }
                }
            } else if remaining > 20 && reader.bytes_left(end_offset) >= 18 {
                // Old format (pre-HAS_PARAMETERS): extra bytes are the real mask
                mask.real_flags_byte    = Some(reader.read_u8()?);
                mask.real_default_color = Some(reader.read_u8()?);
                mask.real_top    = Some(reader.read_i32()?);
                mask.real_left   = Some(reader.read_i32()?);
                mask.real_bottom = Some(reader.read_i32()?);
                mask.real_right  = Some(reader.read_i32()?);
            }
        }
```

- [ ] **Step 4: Run all tests**

```bash
cargo test 2>&1
```
Expected: all tests pass.

- [ ] **Step 5: Commit**

```bash
git add src/reader.rs
git commit -m "fix: condition real-mask read on channel -3 presence in layer mask data

Rust was unconditionally reading 18 bytes of real-mask fields before
parameter-flags regardless of channel list, causing stream misalignment
when channel -3 is absent. Also handle old pre-HAS_PARAMETERS format."
```

---

### Task 3: Fix writer — only emit real-mask block when real mask exists

**Bug:** `writer.rs:619-621` unconditionally writes 18 zero bytes for the real-mask block when `has_params` is true. TS only writes real-mask bytes when `mask.realMask` is non-null.

**Files:**
- Modify: `src/writer.rs` (the mask write block around line 619)

- [ ] **Step 1: Update the writer to conditionally write real-mask**

Replace lines 619–620:

```rust
                if has_params {
                    writer.write_zeros(18)?; // real mask rect + real flags (not stored currently)
```

With:

```rust
                if has_params {
                    // Only write real-mask block when real mask data is present
                    if mask.real_flags_byte.is_some() {
                        writer.write_u8(mask.real_flags_byte.unwrap_or(0))?;
                        writer.write_u8(mask.real_default_color.unwrap_or(0))?;
                        writer.write_i32(mask.real_top.unwrap_or(0))?;
                        writer.write_i32(mask.real_left.unwrap_or(0))?;
                        writer.write_i32(mask.real_bottom.unwrap_or(0))?;
                        writer.write_i32(mask.real_right.unwrap_or(0))?;
                    }
```

- [ ] **Step 2: Run all tests**

```bash
cargo test 2>&1
```
Expected: all tests pass.

- [ ] **Step 3: Commit**

```bash
git add src/writer.rs
git commit -m "fix: only write real-mask block in layer mask when real mask exists

Previously always wrote 18 zero bytes for real-mask when has_params
was true, producing an invalid block for layers with no real mask."
```
