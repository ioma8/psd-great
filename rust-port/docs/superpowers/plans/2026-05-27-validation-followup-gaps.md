# Validation Follow-Up Gaps

This file records the concrete parity gaps found during validation after the "remaining TS parity gaps" implementation pass.

## 1. `lnkD__` is still not written

**Status**
- Reader supports `lnkD__`
- Writer does not

**Rust references**
- Read path includes `lnkD__`:
  - `src/additional_info.rs:720`
- Write path excludes `lnkD__`:
  - `src/additional_info.rs:2114`
- Section list excludes `lnkD__`:
  - `src/additional_info.rs:2332`

**Test gap**
- Current parity test only covers `lnkD` and `lnk3`:
  - `tests/ts_parity_test.rs:1258`

**Required fix**
- Add `lnkD__` to the write match arm alongside `lnk2 | lnkD | lnk3`
- Add `lnkD__` to the tagged-block section ordering list
- Extend the parity test to include `lnkD__`

## 2. `FEid` is only a minimal shell, not TS parity

**Status**
- Rust preserves only `id` and optional `version`
- Rust drops the structured payload

**Rust references**
- Typed model is too shallow:
  - `src/additional_info.rs:122`
- Reader only captures header-level fields:
  - `src/additional_info.rs:854`
- Writer only emits header shell:
  - `src/additional_info.rs:2239`

**TS references**
- Full read logic:
  - `/Users/jakubkolcar/projects/customs/photoshop/psd/src/psd/tagged-block-reader.ts:771`
- Full write logic:
  - `/Users/jakubkolcar/projects/customs/photoshop/psd/src/psd/tagged-block-writer.ts:751`

**Missing behaviors**
- effect item rect
- effect depth
- channel count
- slot payloads
- preview payloads
- typed reconstruction from parsed data

**Required fix**
- Expand the Rust `FilterEffectsBlock` model to match the TS structured payload
- Port the remaining reader logic for slots and preview blocks
- Port the remaining writer logic for slot encoding and preview encoding
- Add parity tests that exercise non-empty slots and preview data

## 3. `PxSD` is only a minimal shell, not TS parity

**Status**
- Rust preserves only `key`
- Rust drops nested image payload structure

**Rust references**
- Typed model is too shallow:
  - `src/additional_info.rs:154`
- Reader only captures `key`:
  - `src/additional_info.rs:890`
- Writer only emits a minimal chunk:
  - `src/additional_info.rs:2258`

**TS references**
- Full read logic:
  - `/Users/jakubkolcar/projects/customs/photoshop/psd/src/psd/tagged-block-reader.ts:950`
- Full write logic:
  - `/Users/jakubkolcar/projects/customs/photoshop/psd/src/psd/tagged-block-writer.ts:880`

**Missing behaviors**
- nested image list
- per-image rect
- per-image RGBA-derived channel payloads
- nested length fixup
- typed round-trip for parsed images

**Required fix**
- Expand `PixelSourceDataBlock` and `PixelSourceDataItem` to include the TS image payload structure
- Port nested image parsing
- Port nested image writing, including payload padding and nested-length fixup
- Add parity tests with actual image payloads, not only key-only items

## 4. `Txt2` synthesis is weaker than TS

**Status**
- Rust synthesizes a minimal `Txt2`
- TS synthesizes richer `_DocumentObjects` and carries document resources forward

**Rust references**
- Current prewrite:
  - `src/writer.rs:1002`

**TS references**
- Synthesis logic:
  - `/Users/jakubkolcar/projects/customs/photoshop/psd/src/psd/psd-writer.ts:12`

**Missing behaviors**
- `_StyleRun` extraction
- `_ParagraphRun` extraction
- copying `DocumentResources` / `ResourceDict` into synthesized `Txt2`
- closer structural parity for synthesized text objects

**Required fix**
- Extend `apply_text_prewrite()` to mirror the TS structure
- Extract style and paragraph run information from `TySh` / engine data where available
- Carry document resources into synthesized `Txt2` if present
- Strengthen the parity test so it validates `Txt2` structure, not only existence

## 5. Test coverage currently allows false confidence

**Status**
- Tests are green, but several new tests only prove presence, not parity depth

**Examples**
- `Txt2` test only checks that `text_engine` exists
- `FEid` test only checks minimal-item round-trip
- `PxSD` test only checks minimal-item round-trip
- linked-file variant test misses `lnkD__`

**Required fix**
- Tighten tests to assert semantic fields, not just `Some(...)`
- Add at least one structured sample per missing family
- Prefer tests modeled directly after TS fixtures/behaviors

## Recommended execution order

1. Fix `lnkD__` write support and extend the parity test
2. Strengthen `Txt2` synthesis and its parity assertions
3. Port full `FEid`
4. Port full `PxSD`
5. Re-run full `cargo test -- --nocapture`

