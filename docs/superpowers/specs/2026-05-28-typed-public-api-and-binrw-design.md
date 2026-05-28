# Typed Public API And Fixed-Layout Binrw Design

**Goal:** Remove magic strings and numbers from the public API where the PSD semantics are known, and expand `binrw` usage for fixed-layout binary records where it improves correctness and maintainability.

**Scope:** This pass covers two related cleanup areas:

- public API typing: replace stringly or generic wrapper-based public fields with domain enums and structs where the meaning is known and stable
- binary codec cleanup: convert fixed-layout binary records to `binrw` and route existing readers/writers through those records

This pass does **not** attempt a broad parser rewrite. Variable-layout payloads, descriptor-driven structures, and runtime-tagged heterogeneous blocks remain manual where that is the clearer implementation.

## Design

### 1. Public API Must Prefer Domain Types Over Raw Codes

The crate still exposes too many public fields that force callers to understand Photoshop wire codes directly:

- `PsdStringCode`
- `PsdIntCode`
- `PsdU16Code`
- `PsdU32Code`
- literal four-character identifiers passed into public model fields
- raw integers representing closed-choice concepts

Those generic wrappers are useful for passthrough or unknown values, but they should not be the first-class public representation when the semantic domain is already known.

This pass will make the public API follow a stricter rule:

- use a typed enum or struct when the public meaning is known and stable
- use raw wrapper/code types only for genuinely open-ended or passthrough cases

Initial conversion targets for this pass include:

- linked-file data kinds such as `liFD`, `liFE`, `liFA`
- public resource/unit choice fields currently wrapped in raw numeric code newtypes
- public fixed-choice string codes that already map to a small closed set
- slice or layer metadata fields where only a known set of values is intended for public use

### 2. Raw Wrapper Types Remain For Open-Ended Cases

This is not a blanket ban on `PsdStringCode` or numeric wrappers. Some PSD surfaces are intentionally open:

- unknown tagged-block keys
- descriptor keys and class IDs
- private Adobe extensions
- passthrough payload discriminants

For those cases, the raw wrapper types remain appropriate because a closed enum would either lie or create churn for little benefit.

The goal is to move raw wrappers out of the user-facing “known semantics” path, not to eliminate them from the implementation entirely.

### 3. Breaking Public API Cleanup Is Intentional

The user has explicitly approved a breaking public API change. This pass will take advantage of that and avoid leaving “legacy-but-wrong” parallel fields around just for compatibility.

Design constraints:

- prefer replacing wrong or weak public types instead of adding duplicate aliases
- if a public field becomes typed, downstream callers should use the typed value directly
- if a raw code must still exist for passthrough reasons, it should be visibly raw rather than pretending to be semantic

### 4. Fixed-Layout `binrw` Expansion Only

The repository already has `binrw_support.rs` with multiple fixed-layout records expressed cleanly via `BinRead` / `BinWrite`. This pass will extend that approach where it makes sense.

Good `binrw` candidates:

- fixed-size headers
- compact resource records
- fixed-layout tagged-block subrecords
- effect/common-state style records with static field ordering
- small POD-like binary records currently hand-read with repetitive `read_u16` / `read_u32` sequences

Non-candidates for this pass:

- descriptors
- runtime-tagged heterogeneous payloads
- versioned layouts whose structure changes deeply per tag
- linked-layer item bodies
- any parser where `binrw` would obscure branching or boundary management

### 5. Reader/Writer Boundary

The codec architecture after this pass should look like this:

- fixed-layout binary fragments are represented as dedicated `binrw` structs
- higher-level readers and writers keep ownership of section framing, version branching, and semantic mapping
- public API typing happens at the semantic layer, not at the raw binary record layer

That means:

- `binrw` record types can remain internal when they only exist to decode bytes
- public enums/structs should describe meaning, not byte layout
- mapping code between binary record and public semantic type should be explicit

### 6. Cleanup Targets

This pass will audit and clean public API magic-value usage in:

- `src/psd.rs`
- `src/layer.rs`
- `src/image_resources.rs`
- `src/additional_info.rs`
- `src/types.rs`
- crate-root exports in `src/lib.rs`

This pass will audit and convert fixed-layout binary records in:

- `src/additional_info.rs`
- `src/image_resources.rs`
- `src/effects_helpers.rs`
- remaining fixed-layout records in `src/reader.rs` / `src/writer.rs`
- additional definitions in `src/binrw_support.rs`

This pass should stay specific rather than trying to centralize every binary struct in one go.

### 7. Public API Categories

To keep the cleanup consistent, public values should fall into one of these categories:

#### Semantic Enum

Use when the field represents a closed, documented set of choices.

Examples:

- linked-file kind
- slice origin/type when the public domain is closed
- resource/unit choice values

#### Semantic Struct

Use when multiple raw fields together form one coherent concept.

Examples:

- grouped unit metadata
- strongly typed fixed-layout subrecords exposed publicly

#### Raw Wrapper

Use when the value is intentionally extensible or unknown.

Examples:

- descriptor keys
- opaque tagged-block identifiers
- unknown future/private discriminants

### 8. Error Handling

For public typing:

- reject impossible numeric/string codes when reading into a closed semantic enum
- preserve unknown values only when the public type is intentionally open-ended

For `binrw` conversion:

- fixed-layout records should fail at decode time with clear context if the binary shape is invalid
- callers should continue adding higher-level section/path context around low-level decode errors

### 9. Testing Strategy

This pass remains TDD-first.

Required coverage:

- public API regression tests proving callers can construct typed values without magic literals
- roundtrip tests showing typed public values serialize to the correct codes
- decode tests for converted `binrw` records
- smoke coverage proving manual variable-layout parsers remain unaffected by the targeted refactors
- full `cargo test --quiet` remains green

### 10. File Responsibilities

- `src/types.rs`
  Promote shared semantic enums/structs out of raw wrapper use where appropriate.

- `src/psd.rs`
  Replace public raw-code fields with typed semantic values where the document-facing meaning is known.

- `src/layer.rs`
  Replace public magic string/number fields in layer-facing models with typed values where the domain is closed.

- `src/image_resources.rs`
  Use typed public values for known resource semantics and route fixed-layout binary records through `binrw` types when the record layout is static and reused.

- `src/additional_info.rs`
  Replace public known-code fields with typed values and migrate remaining fixed-layout subrecords to `binrw` where helpful.

- `src/effects_helpers.rs`
  Convert remaining fixed-layout effect records to `binrw` if they are still manual and static.

- `src/binrw_support.rs`
  Add new fixed-layout binary record definitions that are reused cleanly across readers/writers.

- `src/lib.rs`
  Re-export the new public semantic enums/structs and stop foregrounding raw wrappers where they are no longer the intended API.

## Approaches Considered

### A. Public API Typing Only

Clean up the public surface but leave most binary record code manual.

Rejected because it misses half the goal and keeps repetitive fixed-layout binary logic scattered across readers and writers.

### B. Full Public Typing Plus Targeted Fixed-Layout `binrw`

Replace known public magic values with semantic types and convert only the fixed-layout binary records that benefit clearly from `binrw`.

Recommended because it delivers the requested cleanup without forcing a risky parser rewrite.

### C. Push `binrw` Everywhere Possible

Try to move most parsing and writing into `binrw`, including heavily variable or descriptor-driven structures.

Rejected because it would obscure complex PSD control flow, create excessive churn, and go beyond the user’s requested boundary.

## Success Criteria

The work is complete when all of the following are true:

- the public API no longer exposes avoidable magic strings or numbers for known semantics
- raw wrapper types remain only where the domain is intentionally open-ended or passthrough
- fixed-layout binary records that clearly benefit from `binrw` are converted
- variable-layout parsing remains manual where that is the clearer design
- tests demonstrate typed public construction and correct binary roundtrips
- the full test suite passes

## Non-Goals

- eliminating every raw wrapper type from the crate
- rewriting descriptor handling around `binrw`
- forcing all binary parsing into one abstraction layer
- broad unrelated refactors outside public typing and fixed-layout record cleanup
