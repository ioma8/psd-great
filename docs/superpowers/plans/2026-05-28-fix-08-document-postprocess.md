# Fix 08 — Document Postprocessing: Map Missing Fields to `Psd` Struct

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** The TypeScript source exposes 7 fields at the top-level document that Rust either never promotes from `ImageResources` or never maps at all: `resolution` (resource 1005), `guides` (resource 1032), `alpha_channel_names` (resource 1045), `selected_layer_ids` (resource 1069), `icc_profile` (resource 1039), `path_selection_descriptor` (resource 3000), `slices` (resource 1050). Add these fields to `Psd` and populate them in `apply_document_postprocess`.

**Architecture:** Changes span `src/psd.rs` (add fields to `Psd`), `src/document_resource_postprocess.rs` (populate on read, write back on pre-write). No parser changes needed — the data is already in `ImageResources`; we just need to wire it up.

**Tech Stack:** Rust, `cargo test`

**TS reference:** `photoshop/psd/src/psd/document-postprocess.ts`, `resource-postprocess.ts`

---

### Task 1: Add missing top-level fields to `Psd`

**Files:**
- Modify: `src/psd.rs`

- [ ] **Step 1: Find the `Psd` struct**

```bash
grep -n "pub struct Psd\|pub resolution\|pub guides\|pub alpha_channel\|pub selected_layer\|pub icc_profile\|pub path_selection\|pub slices\|pub paths" /Users/jakubkolcar/projects/customs/ag-psd-rust/src/psd.rs | head -30
```

- [ ] **Step 2: Add the missing fields**

In `pub struct Psd`, add:

```rust
    /// Document resolution in pixels-per-inch (from resource 1005)
    pub resolution: Option<f64>,
    /// Guide lines (from resource 1032)
    pub guides: Option<Vec<crate::image_resources::GuideInfo>>,
    /// Alpha channel names (from resource 1045)
    pub alpha_channel_names: Option<Vec<String>>,
    /// Currently selected layer IDs (from resource 1069)
    pub selected_layer_ids: Option<Vec<u32>>,
    /// ICC color profile bytes (from resource 1039)
    pub icc_profile: Option<Vec<u8>>,
    /// Path selection descriptor (from resource 3000)
    pub path_selection_descriptor: Option<Vec<(String, crate::descriptor::DescriptorValue)>>,
    /// Document slices (from resource 1050)
    pub slices: Option<Vec<crate::image_resources::Slice>>,
```

- [ ] **Step 3: Run all tests to confirm compilation**

```bash
cargo test 2>&1
```
Expected: all pass (new `Option` fields default to `None`).

- [ ] **Step 4: Commit**

```bash
git add src/psd.rs
git commit -m "feat: add resolution/guides/alpha_channel_names/selected_layer_ids/icc_profile/slices to Psd"
```

---

### Task 2: Populate `resolution` from resource 1005

**Files:**
- Modify: `src/document_resource_postprocess.rs`

- [ ] **Step 1: Find `apply_document_postprocess`**

```bash
grep -n "fn apply_document_postprocess\|resolution_info\|resource_1005\|1005" /Users/jakubkolcar/projects/customs/ag-psd-rust/src/document_resource_postprocess.rs | head -20
```

- [ ] **Step 2: Add resolution mapping**

In `apply_document_postprocess`, after reading `image_resources`:

```rust
// Map resolution (resource 1005) → psd.resolution
if let Some(ref res_info) = psd.image_resources.resolution_info {
    // horizontal_res is stored as a 16.16 fixed-point × 65536; already decoded to f64
    psd.resolution = Some(res_info.horizontal_res);
}
```

- [ ] **Step 3: Add resolution to pre-write**

In the pre-write function (where `ImageResources` is rebuilt from `Psd` fields), add:

```rust
if let Some(dpi) = psd.resolution {
    // Build the 16-byte resolution info record with hardcoded units (matching TS)
    psd.image_resources.resolution_info = Some(ResolutionInfo {
        horizontal_res: dpi,
        h_res_unit: 1,
        width_unit: 2,
        vertical_res: dpi,
        v_res_unit: 1,
        height_unit: 2,
    });
}
```

- [ ] **Step 4: Write a test**

```rust
#[test]
fn psd_resolution_mapped_from_resource_1005() {
    // Build a minimal Psd with image_resources.resolution_info set
    let mut psd = Psd::default();
    psd.image_resources.resolution_info = Some(ResolutionInfo {
        horizontal_res: 300.0,
        h_res_unit: 1,
        width_unit: 2,
        vertical_res: 300.0,
        v_res_unit: 1,
        height_unit: 2,
    });
    apply_document_postprocess(&mut psd).unwrap();
    assert_eq!(psd.resolution, Some(300.0));
}
```

- [ ] **Step 5: Run all tests**

```bash
cargo test 2>&1
```

- [ ] **Step 6: Commit**

```bash
git add src/document_resource_postprocess.rs
git commit -m "feat: Psd.resolution populated from resource 1005 (horizontal DPI)"
```

---

### Task 3: Populate `guides` from resource 1032

**Files:**
- Modify: `src/document_resource_postprocess.rs`

- [ ] **Step 1: Add guides mapping**

```rust
// Map guides (resource 1032) → psd.guides
if let Some(ref grid_info) = psd.image_resources.grid_and_guides {
    if !grid_info.guides.is_empty() {
        psd.guides = Some(grid_info.guides.clone());
    }
}
```

- [ ] **Step 2: Add guides to pre-write**

```rust
if let Some(ref guides) = psd.guides {
    let existing = psd.image_resources.grid_and_guides.get_or_insert_with(Default::default);
    existing.guides = guides.clone();
}
```

- [ ] **Step 3: Run all tests**

```bash
cargo test 2>&1
```

- [ ] **Step 4: Commit**

```bash
git add src/document_resource_postprocess.rs
git commit -m "feat: Psd.guides populated from resource 1032 grid-and-guides"
```

---

### Task 4: Populate `alpha_channel_names` from resource 1045

**Files:**
- Modify: `src/document_resource_postprocess.rs`

- [ ] **Step 1: Add alpha channel names mapping**

```rust
// Map alpha channel names (resource 1045) → psd.alpha_channel_names
if let Some(ref names) = psd.image_resources.alpha_unicode_names {
    if !names.is_empty() {
        psd.alpha_channel_names = Some(names.clone());
    }
}
```

- [ ] **Step 2: Add to pre-write**

```rust
if let Some(ref names) = psd.alpha_channel_names {
    psd.image_resources.alpha_unicode_names = Some(names.clone());
}
```

- [ ] **Step 3: Run all tests**

```bash
cargo test 2>&1
```

- [ ] **Step 4: Commit**

```bash
git add src/document_resource_postprocess.rs
git commit -m "feat: Psd.alpha_channel_names populated from resource 1045"
```

---

### Task 5: Populate `selected_layer_ids`, `icc_profile`, `slices`

**Files:**
- Modify: `src/document_resource_postprocess.rs`

- [ ] **Step 1: Add the three remaining mappings**

```rust
// selected_layer_ids (resource 1069)
if let Some(ref ids) = psd.image_resources.layer_selection_ids {
    psd.selected_layer_ids = Some(ids.clone());
}

// icc_profile (resource 1039)
if let Some(ref profile) = psd.image_resources.icc_profile {
    psd.icc_profile = Some(profile.clone());
}

// slices (resource 1050)
if let Some(ref slices) = psd.image_resources.slices {
    psd.slices = Some(slices.clone());
}
```

- [ ] **Step 2: Add pre-write mappings**

```rust
if let Some(ref ids) = psd.selected_layer_ids {
    psd.image_resources.layer_selection_ids = Some(ids.clone());
}
if let Some(ref profile) = psd.icc_profile {
    psd.image_resources.icc_profile = Some(profile.clone());
}
if let Some(ref slices) = psd.slices {
    psd.image_resources.slices = Some(slices.clone());
}
```

- [ ] **Step 3: Run all tests**

```bash
cargo test 2>&1
```

- [ ] **Step 4: Commit**

```bash
git add src/document_resource_postprocess.rs
git commit -m "feat: Psd populates selected_layer_ids (1069), icc_profile (1039), slices (1050)"
```

---

### Task 6: Populate `path_selection_descriptor` from resource 3000

**Files:**
- Modify: `src/document_resource_postprocess.rs`
- Modify: `src/image_resources.rs` (resource 3000 read side, if not already storing the descriptor)

- [ ] **Step 1: Check how resource 3000 is stored**

```bash
grep -n "3000\|path_selection\|descriptor_resources\|origin_path" /Users/jakubkolcar/projects/customs/ag-psd-rust/src/image_resources.rs | head -20
grep -n "3000\|path_selection" /Users/jakubkolcar/projects/customs/ag-psd-rust/src/document_resource_postprocess.rs | head -10
```

- [ ] **Step 2: Add read-side mapping**

In `apply_document_postprocess`, find where `descriptor_resources` is populated and add:

```rust
// path_selection_descriptor (resource 3000)
if let Some(desc) = psd.image_resources.descriptor_resources.get(&3000) {
    psd.path_selection_descriptor = Some(desc.clone());
}
```

- [ ] **Step 3: Add pre-write mapping**

```rust
if let Some(ref desc) = psd.path_selection_descriptor {
    psd.image_resources.descriptor_resources.insert(3000, desc.clone());
}
```

- [ ] **Step 4: Run all tests**

```bash
cargo test 2>&1
```

- [ ] **Step 5: Commit**

```bash
git add src/document_resource_postprocess.rs
git commit -m "feat: Psd.path_selection_descriptor populated from resource 3000 on read"
```
