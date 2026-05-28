# PSD Final Spec Compliance Design

**Goal:** Eliminate the remaining PSD/PSB spec mismatches in the Rust reader-writer by fixing thumbnail resource format semantics, fully modeling color samplers and slices, improving linked-layer fidelity, and making Photoshop color-structure handling match the Adobe specification.

**Scope:** This pass covers the remaining audited gaps in Image Resources, Additional Layer Information linked-layer blocks, and the Photoshop color structure used by several PSD payloads. The reader should remain permissive for unambiguous compatibility cases, but for the covered structures both reader and writer must become spec-correct.

## Design

### 1. Public Model Must Carry The Missing Spec Data

The remaining compliance issues are now mostly caused by lossy public types rather than only bad byte-order or resource-ID dispatch. This pass will expand the public model where needed so the crate can represent the documented payloads directly.

Required model changes:

- thumbnail format must map directly to the spec-defined values for resource `1036`
- color samplers must carry version-sensitive position data plus color-space and depth fields
- slices must be version-aware so legacy binary slices and descriptor-based slices are both representable
- linked-layer items must retain the metadata that the current model still drops

These changes are intentional API corrections rather than compatibility regressions.

### 2. Thumbnail Resource `1036`

The Photoshop 5+ thumbnail resource uses a 28-byte header followed by thumbnail bytes, with format values:

- `1 = kJpegRGB`
- `0 = kRawRGB`

The current implementation has those values reversed. This pass will align the type names, parsing, and writing with the spec so the public model describes the actual on-disk meaning instead of an inverted local convention.

### 3. Color Samplers Resource `1073`

Resource `1073` is still only partially modeled. Each sampler record includes:

- position
- color space
- depth for version `2`
- version-sensitive position encoding behavior

The public model will be expanded so a `ColorSampler` can round-trip the documented fields without inventing defaults on write or discarding data on read. If version-specific position semantics differ, the model must preserve the original version so reserialization is exact.

### 4. Slices Resource `1050`

Slices need a proper versioned model:

- version `6`: legacy binary slice records, optional trailing per-slice descriptor data
- version `7/8`: descriptor-based top-level slice payload

The current low-level resource struct partially distinguishes these forms, but the public `Psd` model still collapses them into `Vec<Slice>` and the prewrite path always emits version `6`. This pass will replace that lossy surface with a version-aware slices enum, and the read/write mapping will stop silently dropping descriptor-backed forms.

### 5. Linked Layer Blocks `lnkD` / `lnk2` / `lnk3`

Linked-layer items still lose important spec-defined fields. The remaining work is to model and preserve:

- item version
- payload kind (`liFD`, `liFE`, `liFA`, or other raw kind)
- open-descriptor data when present
- child document IDs and related metadata
- file size and linked-file info where available
- other versioned fields currently discarded during parse or hardcoded during write

The design goal is exact round-tripping for the documented structure, not only “some payload bytes plus an ID”.

### 6. Photoshop Color Structure Semantics

The PSD color structure stores 2-byte color-space ID plus four unsigned 16-bit values. The current code still applies lossy or spec-wrong conversions:

- CMYK semantics are not modeled as documented
- grayscale is still compressed to a byte-style representation
- unsupported spaces collapse to black instead of preserving meaningful structure

This pass will make the `Color` model and the `read_color()` / `write_color()` code reflect the documented semantics for the supported spaces. For custom or opaque color spaces, the API should preserve the raw components instead of throwing away information.

### 7. Reader/Writer Contract

After this pass, the contract for the covered structures will be:

- reader parses the documented fields into the public model without inventing placeholder defaults
- writer emits spec-correct bytes for every modeled field
- unmodeled but recognized variants preserve their raw discriminants and payloads where needed to avoid corruption
- compatibility acceptance remains only where it does not contradict the actual spec framing

## File Responsibilities

- `src/psd.rs`
  Expand the public model for slices, color samplers, thumbnail format, and any linked-layer metadata promoted to the document-facing API.

- `src/image_resources.rs`
  Fix thumbnail format mapping, fully model color samplers, complete version-aware slices parsing/writing, and keep descriptor-backed slice forms intact.

- `src/document_resource_postprocess.rs`
  Map the new version-aware slices and richer color sampler types between low-level resources and `Psd`.

- `src/layer.rs`
  Expand linked-layer item types so the missing metadata is representable rather than silently discarded.

- `src/additional_info.rs`
  Parse and write the fuller linked-layer structure, including versioned item metadata and open-descriptor data.

- `src/types.rs`
  Adjust the color model if needed so Photoshop color structures stop collapsing to lossy byte-oriented approximations.

- `src/reader.rs`
  Implement spec-correct Photoshop color parsing.

- `src/writer.rs`
  Implement spec-correct Photoshop color writing.

- `tests/ts_parity_test.rs`
  Update or replace parity tests that still reflect the old lossy public model.

- `tests/integration_test.rs`
  Add end-to-end resource and linked-layer regression tests.

## Error Handling

Reject:

- impossible sampler or slice versions for known payloads
- malformed linked-layer item framing
- truncated thumbnail, sampler, slice, or linked-layer payloads
- impossible color-structure sizes for the modeled spaces

Accept when unambiguous:

- extra compatibility padding outside the defined payload data
- raw preservation for recognized-but-not-fully-semantic linked-layer variants

## Testing Strategy

This pass will use TDD again with focused failing tests first.

Required coverage:

- thumbnail resource `1036` uses the correct format codes on read and write
- color sampler records preserve version, position, color space, and depth
- slices version `6` round-trips legacy data including descriptor tails when present
- slices version `7/8` round-trip through the public model without collapsing to flat slices
- linked-layer items preserve metadata fields that were previously dropped
- Photoshop color structures for CMYK and grayscale round-trip with spec-correct numeric semantics
- custom or unsupported color-space records no longer collapse to black when the data can be preserved

## Approaches Considered

### A. Minimal Wire Patching

Patch only the visible byte mismatches and keep the current public types mostly unchanged.

Rejected because the remaining bugs are primarily caused by insufficient model surface. More byte patches without model changes would just encode more hidden loss.

### B. Spec-Correct Model Expansion With Focused Scope

Expand only the public types required for the remaining audited structures, and keep the rest of the crate stable.

Recommended because it closes the actual compliance gaps without turning this into a general redesign of the whole library.

### C. Raw-Preservation Fallback For Everything Hard

Move the remaining complex payloads into opaque byte buckets and stop trying to type them.

Rejected because the user asked to fix the rest, and the remaining issues are documented structures that should be modeled well enough to round-trip correctly.

## Success Criteria

The work is complete when all of the following are true:

- the remaining post-fix audit findings are closed
- the public API can represent the documented data required for `1036`, `1073`, `1050`, linked-layer blocks, and Photoshop color structures
- new tests prove the corrected wire behavior
- the full test suite remains green

## Non-Goals

- a total redesign of unrelated PSD model types
- exhaustive semantic decoding of every private Adobe payload outside the audited structures
- preserving older lossy public field meanings when they conflict with the specification
