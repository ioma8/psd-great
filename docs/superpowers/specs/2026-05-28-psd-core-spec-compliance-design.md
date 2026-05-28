# PSD Core Spec Compliance Design

**Goal:** Bring the Rust PSD/PSB reader-writer into compliance with the Adobe Photoshop File Format Specification for the audited core PSD/PSB sections: file header, color-mode data, image resources/path resources, layer and mask information, additional layer information, and image data.

**Scope:** This work covers the native PSD/PSB format only. It does not expand support for the separate "Additional File Formats" chapter such as Actions, CMYK Setup files, or other standalone file containers beyond the current repo surface.

## Design

### 1. Header And Color Mode Codes

The crate must encode and decode the Adobe-defined color mode integers exactly as specified in the file header. The `ColorMode` enum values and `from_u16` conversion logic will be corrected so `Multichannel`, `Duotone`, and `Lab` use the spec values `7`, `8`, and `9`.

This change is isolated to type definitions plus tests that assert the on-disk numeric mapping. Any downstream code matching on `ColorMode` can remain semantic rather than numeric.

### 2. PackBits / RLE Compliance

The RLE decoder must implement PackBits semantics exactly. In particular, control byte `128` must be treated as a no-op rather than a repeat run. The decoder will be updated so valid PSD/TIFF-compatible PackBits streams decode correctly, while malformed streams still error cleanly.

The existing encoder can remain structurally similar if its output remains valid PackBits. The key requirement is spec-correct decoding and round-trip safety for PSD RLE channel data and composite image data.

### 3. Additional Layer Information Block Framing

Layer tagged blocks must follow the spec’s framing rules:

- `8BIM` or `8B64` signatures as appropriate
- 32-bit lengths for normal blocks
- 64-bit lengths for PSB-only keys that require them
- even-byte payload padding rather than unconditional 4-byte padding

This will be handled centrally in `additional_info.rs` so individual tagged-block parsers and writers do not need to know about transport framing. The reader and writer must agree on the same framing logic.

### 4. PSB Length Handling

PSB-specific size handling must be consistent in all container sections that use 64-bit lengths. The current code already has partial PSB support in some top-level sections; this work finishes the missing tagged-block-specific PSB framing and keeps the implementation limited to sizes below 4GB when the current in-memory structures require that.

Where the spec allows 64-bit lengths but the crate still cannot represent payloads above 4GB safely, the code should reject them explicitly rather than silently misparse them.

### 5. Composite Image Depth Handling

Composite image decoding and encoding must honor the header depth for 8-, 16-, and 32-bit data. The current composite pipeline collapses data into 8-bit-sized planes; this will be replaced with depth-aware channel sizing and decode paths.

The crate’s public `PixelData` remains RGBA `u8`, so the compliance goal here is format-correct parsing and writing, not a public API redesign for high-bit-depth pixel preservation. The implementation should:

- read 16/32-bit composite planes with correct byte counts
- correctly apply ZIP prediction per depth
- downsample decoded composite data to the existing `u8` RGBA surface for API exposure
- write 16/32-bit composite data in a spec-correct way from the existing RGBA source, using deterministic expansion rules already present in the writer

### 6. Merged Alpha Layer Count Signaling

When the merged result contains a first alpha channel for transparency, the layer info section must use a negative layer count as required by the spec. The writer will derive this from the same condition that drives composite alpha channel output, so the layer count and composite image data remain internally consistent.

The reader’s existing negative-count handling will stay as the source of truth for round-trip behavior.

### 7. Path Resource Fixed-Point Format

Saved path resources in image resources must use 8.24 fixed-point coordinates, not 16.16. The path resource read/write helpers in `image_resources.rs` will be corrected to use the same path-point math already used elsewhere in the codebase for vector path data.

This change is limited to the image-resource path record format and should not alter unrelated vector mask parsing logic.

## File Responsibilities

- `src/types.rs`
  Defines spec-correct color mode constants and conversions.

- `src/compression.rs`
  Implements PackBits-compliant RLE decode behavior and any supporting tests.

- `src/reader.rs`
  Handles depth-aware composite image parsing and keeps merged-alpha semantics aligned with the spec.

- `src/writer.rs`
  Handles depth-aware composite image writing and negative layer count signaling.

- `src/additional_info.rs`
  Centralizes spec-correct additional-info block framing, padding, and PSB-specific 64-bit length handling.

- `src/image_resources.rs`
  Fixes path resource coordinate encoding/decoding to use 8.24 fixed-point values.

- `tests/integration_test.rs`
  Holds end-to-end PSD/PSB compliance tests where cross-module coverage is more important than unit isolation.

- `tests/ts_parity_test.rs`
  Holds behavior-oriented regression tests that are already close to format round-trip semantics.

## Error Handling

The parser should fail loudly on unsupported-but-recognized structural states rather than guessing:

- invalid or out-of-range color mode codes
- impossible channel byte counts for a declared depth
- PSB 64-bit payload sizes whose high 32 bits are non-zero when the crate still cannot safely represent them
- malformed PackBits row streams

This keeps the library spec-faithful and prevents silent corruption.

## Testing Strategy

The implementation will use TDD with narrow failing tests for each audited violation:

- color mode header constants and round-trip parsing
- PackBits `128` no-op decoding
- additional-info padding and PSB framing
- negative layer count for merged alpha
- 16-bit and 32-bit composite image round-trip behavior
- path resource 8.24 encode/decode correctness

Tests will be added before production edits and run at the smallest useful scope first, then with the full suite.

## Approaches Considered

### A. Full API Redesign For High-Bit-Depth Pixels

This would introduce a depth-aware public pixel model instead of exposing only RGBA `u8`. It is the most complete design in principle, but it is much larger than the audited compliance gap and would ripple through examples, tests, and user-facing APIs.

Rejected for this pass because it expands product scope beyond spec-accurate file framing and parsing.

### B. Core Compliance Fixes In Place

This keeps the existing public API and fixes the file-format rules where they are currently wrong. It is the smallest change set that closes the audited spec gaps while preserving the current crate shape.

Recommended because it directly addresses the real failures without speculative redesign.

### C. Strict Read Compliance Only

This would make the parser compliant but leave writer gaps such as path resource output, tagged-block framing, and negative layer count semantics unresolved.

Rejected because the user asked for full compliance work, and the crate explicitly positions itself as both reader and writer.

## Success Criteria

The work is complete when all of the following are true:

- the six audited PSD/PSB spec violations are fixed
- new tests prove the corrected behavior
- existing tests remain green
- PSD and PSB framing rules are internally consistent across reader and writer
- no known core PSD/PSB spec mismatch from the audit remains open in the addressed sections

## Non-Goals

- implementing the entire "Additional File Formats" chapter from the Adobe document
- redesigning the public pixel API for native 16/32-bit sample exposure
- implementing support for payloads larger than 4GB in memory
