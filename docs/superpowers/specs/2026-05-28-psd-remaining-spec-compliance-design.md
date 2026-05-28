# PSD Remaining Spec Compliance Design

**Goal:** Eliminate the remaining audited PSD/PSB spec mismatches in the Rust reader-writer by correcting resource IDs, payload framing, header validation, color-structure parsing, and the public model where the current API encodes the wrong Photoshop concepts.

**Scope:** This pass covers the remaining mismatches in the File Header, Image Resources, and Additional Layer Information sections of the Adobe Photoshop file format specification. The reader should remain permissive where practical, but the writer must emit spec-correct bytes for the covered structures.

## Design

### 1. Public Model Correctness Over Compatibility Shims

The current public model still exposes a few spec-wrong concepts because earlier code mapped the wrong image resource IDs onto convenient Rust fields. This pass will prefer correct semantics over preserving those mistakes.

Concretely:

- resource `1026` will become layer group information, not layer clipping
- resource `1077` will become display information
- resource `1073` will become color samplers, not custom points
- resource `2999` will become clipping path name, not a descriptor

Where an existing public field is spec-wrong, it will either be removed, renamed, or replaced by a correctly typed field. Compatibility is secondary to making the API describe the actual PSD structure.

### 2. Image Resource Dispatch Must Match The Spec

`image_resources.rs` currently contains the largest remaining wire-format drift. The resource dispatcher and serializers will be corrected so each covered ID maps to the right payload shape:

- `1026`: big-endian layer group IDs
- `1036`: Photoshop 5+ thumbnail resource
- `1050`: version-aware slices resource
- `1073`: color samplers resource
- `1077`: display info resource
- `2999`: Pascal-string clipping path name

The reader may still preserve unknown or partially unsupported resource payloads, but it must stop interpreting these known IDs as the wrong structures.

### 3. Version-Aware Slices

Resource `1050` has materially different layouts across versions. Version `6` uses the legacy binary slice structure, while versions `7` and `8` use a descriptor-oriented form. The code will distinguish these forms explicitly rather than forcing all versions into one flat structure.

The public model can change as needed. The cleanest shape is a versioned enum so callers cannot accidentally construct an impossible combination of fields. The writer will then emit the correct format for each variant, and the reader will preserve enough information to round-trip supported versions faithfully.

### 4. Additional Layer Information Payload Fixes

The tagged-block container framing was already corrected in the earlier pass, but some block payloads are still wrong:

- `Txt2` must contain an inner 4-byte length followed by raw text-engine data
- `sn2P` must be a 4-byte non-zero integer, not a single byte

These are payload bugs rather than framing bugs, so the fixes belong in the per-key read/write handlers.

### 5. Additional Layer Information Coverage

Several documented keys are still missing or only partially modeled: `abdd`, `anFX`, `cinf`, `SoLE`, `LMsk`, `Mtrn`, `Mt16`, `Mt32`, and `FXid`. The goal of this pass is not to reverse-engineer every undocumented nuance, but it is to stop losing meaning or writing obviously wrong bytes for recognized keys.

The implementation strategy is:

- add typed support where the payload is simple and clearly specified
- improve existing typed support where the current layout is wrong
- preserve opaque payload bytes for recognized-but-not-fully-modeled keys rather than silently discarding them

This makes the crate more compliant without forcing a speculative full-model expansion for every obscure Adobe payload.

### 6. Linked Layer Fidelity

The linked-layer keys (`lnkD`, `lnk2`, `lnk3`) currently flatten a richer spec-defined structure. This pass will extend the linked-layer model so versioned fields, timestamps, file sizes, child document IDs, and the typed data variants (`liFD`, `liFE`, `liFA`) can round-trip correctly.

If a full semantic representation is still impractical for some variant, the public model should at least distinguish the version and raw typed payload enough to serialize back to the original bytes without corruption.

### 7. Header Validation And Reader Permissiveness

The file-header reader will become stricter on real spec invariants:

- reserved bytes must be zero in strict structural validation
- channel count, width, and height must be at least `1`

At the same time, the broader parser should remain permissive where the current implementation already accepts common real-world deviations that do not create ambiguity, such as extra alignment padding in some tagged blocks. The design principle is:

- reject structurally invalid headers
- accept benign compatibility quirks in deeper payloads when the bytes are still unambiguous

### 8. Photoshop Color Structure Parsing

`read_color()` currently only returns meaningful values for RGB and collapses other color spaces to black. That is not spec-compliant for color-bearing records that use Photoshop’s standard color structure.

This pass will parse the supported color spaces explicitly and preserve their channel values in the public model. If the existing `Color` type is too RGB-centric, it should be redesigned into a color-space-aware enum so callers can inspect the actual color-space data instead of receiving lossy conversion artifacts.

### 9. Reader/Writer Contract

The pass will enforce a clearer contract:

- the writer emits spec-correct bytes for all covered keys
- the reader accepts those bytes and round-trips them
- the reader remains tolerant of legacy malformed-but-unambiguous inputs where we already have compatibility behavior
- the public model reflects actual PSD concepts instead of misnamed convenience fields

## File Responsibilities

- `src/psd.rs`
  Update the public document model to use spec-correct resource concepts and any new enums/structs needed for slices, color samplers, display info, clipping path names, or color-space-aware values.

- `src/image_resources.rs`
  Correct resource-ID dispatch, parsing, serialization, and version-aware payload shapes for the covered image resources.

- `src/document_resource_postprocess.rs`
  Update mapping between raw image resources and the public `Psd` model so it no longer projects wrong resource semantics onto document fields.

- `src/additional_info.rs`
  Fix `Txt2` and `sn2P` payload formats, improve linked-layer fidelity, and add preservation or typed handling for the remaining documented keys.

- `src/reader.rs`
  Tighten header validation and expand Photoshop color-structure parsing.

- `src/writer.rs`
  Emit any newly typed color/resource structures in their spec-correct on-disk forms.

- `tests/ts_parity_test.rs`
  Update round-trip and parity tests that currently encode the old spec-wrong assumptions.

- `tests/integration_test.rs`
  Add end-to-end compliance regression tests where multiple sections interact.

## Error Handling

The code should reject or surface errors for structurally impossible states, but should not become fragile toward benign compatibility cases.

Reject:

- zero width, zero height, or zero channels in the header
- non-zero reserved header bytes
- impossible resource or tagged-block versions for known payloads
- malformed inner lengths or truncated payloads

Accept when unambiguous:

- extra compatibility padding beyond the minimum required by the spec
- known linked-layer variants whose semantic fields are only partially modeled, as long as their raw payload can still be preserved and rewritten correctly

## Testing Strategy

This pass will use TDD with small failing tests first.

Required coverage:

- resource `1026` parses and writes big-endian layer group IDs
- resources `1036`, `1073`, `1077`, and `2999` map to the correct public fields and emit the correct IDs
- resource `1050` round-trips both legacy and descriptor-based slice variants
- `Txt2` writes and reads the inner engine-data length correctly
- `sn2P` writes and reads a 4-byte value
- recognized additional-info keys that remain opaque are preserved byte-for-byte
- linked-layer variants round-trip with their versioned fields intact
- non-RGB Photoshop color structures parse without collapsing to black
- invalid headers fail for zero dimensions/channels and non-zero reserved bytes

Tests should be added before implementation and run at focused scope first, then with the full suite.

## Approaches Considered

### A. Minimal Wire Fixes With Compatibility Aliases

Keep the public model mostly unchanged and only patch the read/write dispatch tables. This reduces immediate churn, but it preserves spec-wrong field names and encourages more future bugs.

Rejected because the user explicitly allows public-model changes and the current API shape is part of the compliance problem.

### B. Spec-Correct Public Model Plus Targeted Wire Fixes

Correct the public semantics where they are wrong, but limit changes to the audited sections instead of redesigning unrelated APIs. Unknown or only partially modeled payloads are preserved rather than exhaustively typed.

Recommended because it aligns the API with the spec while keeping the change set bounded and testable.

### C. Full Exhaustive Modeling Of Every Documented Payload

Add first-class Rust types for every remaining image resource and tagged-block variant immediately.

Rejected for this pass because it is much larger than the audited gap and would mix compliance work with speculative product-surface expansion.

## Success Criteria

The work is complete when all of the following are true:

- the remaining audited mismatches are fixed
- the public API no longer assigns spec-wrong meanings to the covered resource IDs
- new tests prove the corrected behavior
- legacy compatibility that does not conflict with the spec remains intact on read
- the full test suite stays green

## Non-Goals

- implementing every Adobe file format outside native PSD/PSB
- exhaustive first-class typing for every obscure documented payload where opaque preservation is sufficient for this pass
- maintaining backward compatibility for public fields whose semantics are objectively wrong according to the spec
