# PSD Lossless Final Compliance Design

**Goal:** Eliminate the last known PSD/PSB spec mismatches in the Rust reader/writer by making the remaining lossy public types spec-shaped and by finishing the wire-format handling for linked-layer blocks, color samplers, Photoshop color structures, and legacy slice descriptors.

**Scope:** This pass only covers the four remaining audited gaps:

- linked-layer blocks `lnkD` / `lnk2` / `lnk3`
- color samplers resource `1073`
- Photoshop color structures used by PSD payloads
- slices resource `1050` version `6` descriptor-tail framing

Everything else previously marked fixed stays out of scope unless one of these changes requires a narrow compatibility adjustment.

## Design

### 1. Public Model Becomes Lossless Where The Spec Requires It

The remaining issues are no longer mostly dispatch bugs. They are caused by public data shapes that cannot carry the documented PSD values exactly. This pass will correct that instead of layering more heuristics on top of lossy models.

Required model changes:

- Photoshop color-structure variants become raw/spec-shaped rather than normalized convenience values
- color samplers expose version-specific coordinate data explicitly
- linked-layer items expose the actual versioned metadata fields that are still being dropped or synthesized
- legacy slices keep their descriptor-tail state in a stricter, explicitly modeled form

This is an intentional API correction. Where the current API is lossy, exactness takes priority.

### 2. Photoshop Color Structures Must Preserve Raw Spec Semantics

The PSD color structure is:

- `u16` color-space ID
- four `u16` component values

The current `Color` variants still lose information for RGB, HSB, and Lab by converting to normalized or reduced forms. That prevents exact round-tripping and makes some records materially different after write.

This pass will redesign the PSD-facing color structure model as a color-space-aware enum with raw component fields for:

- RGB
- HSB
- CMYK
- Lab
- Grayscale
- opaque/custom spaces

Convenience conversions may be added as helper APIs, but they must not be the serialization model.

Reader/writer contract:

- `read_color()` returns exact raw values for supported spaces
- `write_color()` emits those exact values back
- unsupported but representable spaces remain preserved through raw color-space ID plus components

### 3. Color Samplers `1073` Must Model Versioned Coordinate Semantics

The current sampler support now preserves `color_space` and `depth`, but it still flattens positions as plain big-endian `i32` values for every version. The spec distinguishes coordinate behavior by version, so the model must stop pretending all versions are identical.

This pass will make sampler version semantics explicit. The public model will store:

- sampler version
- version-shaped coordinate representation
- color space
- optional depth for version `2`

The Rust shape will be a struct with:

- `version`
- `position`, represented by a version-aware enum
- `color_space`
- `depth`

The key requirement is that write logic no longer reconstructs version-specific bytes from a flattened generic pair of integers.

### 4. Linked-Layer Blocks Must Serialize Real Metadata

The linked-layer work is close, but the remaining fields still matter:

- real child-document ID instead of hardcoded `"chid"`
- asset modification time
- asset locked state
- linked-file info
- remaining versioned metadata that the current parser either skips or stores only indirectly

This pass will make linked-layer items fully version-aware at the typed level. The parser must read the actual fields from the block structure, and the writer must serialize those same fields back without inventing placeholders.

Design requirements:

- preserve the block key variant (`lnkD`, `lnk2`, `lnkD__`, `lnk3`) already handled today
- preserve item version
- preserve payload kind (`liFD`, `liFE`, `liFA`, or another raw 4-byte code)
- preserve open-descriptor data when present
- parse and write child-document ID, asset metadata, and linked-file metadata when the item version includes them
- reject malformed item layouts rather than silently truncating them

### 5. Legacy Slices `1050` Need Stricter V6 Descriptor-Tail Framing

Version `7/8` slices are already represented as descriptor-backed document slices. The remaining issue is legacy version `6`, where per-slice descriptor tails are still detected heuristically from “bytes remaining”.

This pass will tighten the low-level version `6` slice parser/writer so descriptor tails are treated as part of a stricter slice-record interpretation rather than guessed from leftover data. The design goal is not to redesign the version-aware public slice surface again; it is to make the existing legacy representation provably framed.

Required behavior:

- parse version `6` slice records deterministically
- preserve per-slice descriptor tails where present
- write version `6` slices using the same stricter framing rules
- continue preserving top-level descriptor-backed version `7/8` slices unchanged

### 6. Reader And Writer Contract

After this pass, the contract for the covered structures is:

- every modeled field is parsed from the actual PSD/PSB bytes rather than synthesized
- every modeled field is written back with spec-correct framing and values
- recognized but still not semantically expanded discriminants keep their raw identifiers instead of being collapsed
- compatibility tolerance remains only for unambiguous outer framing cases, not for the audited inner payload semantics

### 7. File Responsibilities

- `src/types.rs`
  Replace the remaining lossy PSD color-structure variants with raw/spec-shaped variants and keep shared type definitions coherent.

- `src/reader.rs`
  Implement exact Photoshop color-structure parsing using the new raw model.

- `src/writer.rs`
  Implement exact Photoshop color-structure serialization from the new raw model.

- `src/psd.rs`
  Expand public sampler and document-facing types so the richer low-level models are exposed without loss at the document API boundary.

- `src/image_resources.rs`
  Finish `1073` version-sensitive sampler handling and tighten `1050` v6 descriptor-tail parsing/writing.

- `src/document_resource_postprocess.rs`
  Preserve the richer sampler and slice representations when mapping between resources and `Psd`.

- `src/layer.rs`
  Expand linked-layer item metadata so all remaining modeled fields have first-class storage.

- `src/additional_info.rs`
  Parse and write the remaining linked-layer metadata fields using the richer linked-layer model.

- `tests/integration_test.rs`
  Add end-to-end regressions for color structures, linked-layer metadata, sampler versions, and v6 slice descriptor tails.

- `tests/ts_parity_test.rs`
  Update public-API parity tests that still assume the older lossy color or linked-layer shapes.

### 8. Error Handling

Reject:

- malformed linked-layer item structures
- impossible or unsupported sampler versions for known `1073` payload layouts
- truncated color structures or slice records
- internally inconsistent version `6` slice descriptor framing

Accept when unambiguous:

- outer compatibility padding already tolerated elsewhere
- unknown raw discriminants inside otherwise valid linked-layer records, as long as their typed identity can still be preserved

### 9. Testing Strategy

This pass remains TDD-first.

Required tests:

- raw Photoshop RGB, HSB, Lab, CMYK, and grayscale color structures round-trip exactly
- opaque/custom color spaces round-trip without collapsing to black or normalized values
- `1073` sampler records round-trip with version-sensitive coordinate semantics intact
- linked-layer items round-trip child-document ID, asset metadata, linked-file info, payload kind, and open descriptor
- version `6` slices round-trip descriptor tails using the stricter framing path
- full `cargo test --quiet` remains green
- a final source/spec audit of the covered sections finds no remaining mismatch in these four areas

## Approaches Considered

### A. Finish With More Internal Heuristics

Keep the public API mostly stable and infer the missing bytes during write.

Rejected because the remaining failures are specifically caused by missing public fidelity. More inference would keep the crate lossy and fragile.

### B. Lossless Spec-First Expansion

Promote the remaining spec-required data to first-class public types and make the reader/writer exact for those types.

Recommended because it closes the last audited gaps directly and leaves the code easier to validate in future audits.

### C. Raw-Bytes Preservation Instead Of Typed Fixes

Preserve more payloads opaquely and stop decoding the hard structures.

Rejected because the user asked to finish the remaining compliance work, not just avoid further corruption.

## Success Criteria

The work is complete when all of the following are true:

- the four remaining audit findings are closed
- the public API can represent the exact data needed for the covered structures
- targeted tests prove the corrected behavior before implementation and after
- the full test suite passes
- a fresh section-by-section audit of these areas finds no remaining mismatch

## Non-Goals

- redesigning unrelated PSD model surfaces
- exhaustively decoding every private Adobe payload not implicated by the remaining audit
- preserving lossy convenience representations when they conflict with the PSD wire format
