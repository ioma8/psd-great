# Public API No-Magic-Values Plan

Goal: remove magic numbers and magic strings from the public Rust API for PSD concepts, while keeping legitimately free-form text and binary payload content as-is.

Scope rules:
- Convert PSD-coded concepts into enums/newtypes/wrappers.
- Keep human text as `String`:
  - names
  - comments
  - author fields
  - URLs / paths
  - free-form metadata text
- Keep binary payload fields as typed byte content where that is the real semantic model.

Target categories:

1. Layer/additional-info public API
- Replace integer/string-coded PSD concepts with typed enums/newtypes:
  - `additional_info::SectionDivider.divider_type`
  - `additional_info::SectionDivider.blend_mode`
  - `additional_info::SectionDivider.sub_type`
  - `additional_info::VectorFill.fill_type`
  - `additional_info::PlacedLayer.anti_alias_policy`
  - `additional_info::PlacedLayer.placed_layer_type`
  - `additional_info::ArtboardData.background_type`
  - `additional_info::PatternBlock.key`
  - `additional_info::HighDepthLayerInfo.key`
  - `additional_info::LinkedFilesBlock.key`
- Review remaining numeric flags:
  - `lmgm`
  - `vmgm`
  - `fcmy`
  - `shape_pattern.version`
  - `shape_pattern.present_count`
  and replace with typed wrappers where the PSD meaning is known.

2. `layer.rs` public API
- Replace string/int-coded concepts:
  - `PatternInfo.id`
  - `BezierPath.fill_rule`
  - every `adjustment_type` string
  - LUT / lookup kind strings
  - `SelectiveColorAdjustment.mode`
  - `LinkedFile.file_type`
  - `LinkedFile.creator`
  - `LinkedFile.child_document_id`
  - `LinkedFile.asset_locked_state`
  - `PlacedLayer.placed`
  - `PlacedLayer.comp`
  - `PlacedLayerFilter` / `Filter` coded string fields
  - `KeyDescriptorItem.key_origin_type`
  - `SectionDivider.key`
  - `SectionDivider.sub_type`
  - compositor capability strings
  - filter-effect mask compression ints
  - `PixelSource.source_type`
  - `Interpretation.interpret_alpha`
  - frame reader/link coded strings
  - `LayerAdditionalInfo.id`
  - `LayerAdditionalInfo.version`
  - `VectorOrigination.vowv`
  - `Layer.link_group`
- Reuse existing enums from `types.rs` where possible instead of inventing duplicates.

3. `psd.rs` public API
- Replace coded strings/ints with typed values:
  - guide direction strings
  - resolution unit strings
  - print-scale style string
  - proof-setup builtin/profile discriminants if needed
  - audio/timeline reader types
  - slice origin/slice type/alignment/background type strings
  - layer comp flags / captured info
  - global-layer-mask color-space integers
  - annotation type/date wrappers
  - artboard background-type integers
  - variable-set placement/alignment/clip strings
  - display-info units

4. `image_resources.rs` public API
- Replace coded values:
  - `PathResourceRecord.record_type`
  - `DisplayInfoResource` units
  - slice origin/slice type/alignment/html/source-type fields
  - layer comp captured-info integer
  - onion-skins blend-mode string

5. `descriptor.rs` public API
- Narrow loose string identifiers where the PSD spec has a coded domain:
  - descriptor class IDs / key IDs should become explicit newtypes instead of plain `String` in the public API if we want to ban magic strings consistently.
- This is the highest-churn area and should come after the model enums above.

Execution order:
1. Add audit tests that list the exact public fields still using magic numbers/strings.
2. Introduce shared enums/newtypes in `types.rs`.
3. Refactor `additional_info.rs`.
4. Refactor `layer.rs`.
5. Refactor `psd.rs` and `image_resources.rs`.
6. Decide whether `descriptor.rs` should use `DescriptorId` / `DescriptorClassId` newtypes.
7. Re-run the full suite after each batch.

Success criteria:
- No public API field remains numeric/stringly-typed when it really represents a closed PSD-coded domain.
- Full existing test suite stays green.
- TS parity remains green.
