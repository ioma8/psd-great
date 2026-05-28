# Color Sampler Depth Invariant Design

## Goal

Encode the color sampler depth invariant directly in the public model so callers cannot construct contradictory known-version states.

## Approved Public Shape

- `ColorSamplerPosition::V1 { horizontal, vertical }`
- `ColorSamplerPosition::V2 { horizontal, vertical, depth }`
- `ColorSamplerPosition::Unsupported { version, horizontal, vertical, depth: Option<u16> }`
- `ColorSampler` keeps `position` and `color_space`
- Remove standalone `ColorSampler.depth`

## Rationale

The current public model separates `depth` from the version-shaped enum, which allows impossible states such as `V1 + depth` and `V2 + no depth`. Serialization rejects those states later, but the API should prevent them for known versions.

Moving `depth` into `ColorSamplerPosition` makes the version-depth relationship explicit at construction time while preserving unsupported-version handling.

## Implementation Scope

- Update public model in `src/psd.rs`
- Update color sampler parsing and writing in `src/image_resources.rs`
- Update document resource postprocess/prewrite paths in `src/document_resource_postprocess.rs`
- Update integration and TS parity tests
- Preserve mixed-version rejection during `write_psd`

## Serialization Rules

- Version 1 records serialize without depth
- Version 2 records serialize with required depth
- Unsupported versions keep explicit `version` and `depth: Option<u16>`
- Mixed-version sampler lists remain rejected
- Resource version mismatches remain rejected

## Test Plan

- Add public-model tests showing V1 and V2 encode the invariant structurally
- Add roundtrip tests for V1, V2, and unsupported resources with the new shape
- Keep `write_psd` rejection coverage for mixed-version sampler lists
- Run targeted `cargo test` commands for integration, parity, and image resource tests
