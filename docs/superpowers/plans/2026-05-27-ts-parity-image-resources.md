# TS Parity Image Resources Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Match TypeScript image-resource preservation semantics for malformed-or-opaque resource payloads, then continue parity work for depth-aware writing.

**Architecture:** Keep Rusts structured `ImageResources` API, but change the read path to preserve raw bytes first and only populate typed fields when payloads are structurally valid. Reuse existing writer behavior so raw resources round-trip without loss.

**Tech Stack:** Rust, cargo test, PSD parser/writer modules

---

### Task 1: Raw resource preservation parity

**Files:**
- Modify: `src/image_resources.rs`
- Test: `tests/ts_parity_test.rs` or `src/image_resources.rs` unit tests

- [ ] Add a failing test for a short raw `1005` image resource payload.
- [ ] Run the focused test and confirm it fails with `UnexpectedEof`.
- [ ] Patch resource parsing to preserve bytes instead of eagerly failing typed decode.
- [ ] Re-run the focused test and confirm it passes.

### Task 2: Full parity verification

**Files:**
- Modify: `src/writer.rs` if depth-aware write changes are needed next
- Test: `tests/ts_parity_test.rs`

- [ ] Run `cargo test --test ts_parity_test -- --nocapture`.
- [ ] Fix remaining parity gaps, starting with depth-aware 16/32-bit writing if those tests are enabled next.
