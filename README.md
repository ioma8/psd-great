# ag-psd - Rust

A Rust library for reading and writing Adobe Photoshop PSD and PSB files.

This crate is a parity-focused port of the TypeScript [ag-psd](https://github.com/Agamnentzar/ag-psd) parser/writer, with a typed Rust API and broad coverage of PSD document structure, layers, tagged blocks, resources, text data, effects, masks, paths, and smart-object metadata.

[![Crates.io](https://img.shields.io/crates/v/ag-psd.svg)](https://crates.io/crates/ag-psd)
[![Documentation](https://docs.rs/ag-psd/badge.svg)](https://docs.rs/ag-psd)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Installation

```toml
[dependencies]
ag-psd = "30.1.0"
```

## Quick Start

```rust
use ag_psd::*;
use std::fs::File;

fn main() -> Result<()> {
    let file = File::open("input.psd")?;
    let mut psd = read_psd(file, ReadOptions::default())?;

    println!("{}x{}", psd.width, psd.height);

    if let Some(ref mut layers) = psd.children {
        for layer in layers {
            layer.opacity = Some(200.0);
        }
    }

    let bytes = write_psd(&psd, &WriteOptions::default())?;
    std::fs::write("output.psd", bytes)?;
    Ok(())
}
```

## What It Supports

- PSD and PSB read/write
- Layer trees, groups, masks, blend modes, and layer effects
- Text layer structures, engine data, style runs, and paragraph runs
- Document resources, metadata, paths, slices, thumbnails, and XMP
- Smart-object related structures, linked files, and placed-layer metadata
- Pattern blocks, annotations, filter-effect blocks, and many Photoshop tagged blocks
- ABR, ASE, and CSH parsing helpers

## Current Status

| Area | Status | Notes |
|---|---|---|
| PSD/PSB structure | Broad support | Reader and writer cover the main parser/writer surface |
| Layers, masks, effects | Complete for targeted parity | Includes vector masks and pattern overlay structures |
| Text layers | Partial | Rich text structures are supported, but not every Photoshop text workflow is exhaustively validated |
| Color modes | Partial | Indexed palettes and generic color-mode data are supported; some composite-image paths still vary by mode |
| 16/32-bit depth | Partial | Structural support exists, but image interpretation and write behavior are not complete across all paths |
| Smart objects / smart filters | Partial | Typed structures exist, but not every workflow is fully validated end to end |

## Limitations

- This crate is a PSD/PSB parser and writer, not a Photoshop compositor or rasterizer.
- Editing text, vector data, masks, or effects does not automatically redraw layer pixels or the composite image.
- Some non-RGB modes such as `Multichannel`, `Duotone`, and `Lab` are preserved structurally, but composite-image decoding and write-side color handling are still incomplete in some paths.
- Higher bit depths are only partially supported.
- Newer Photoshop features, animation/timeline workflows, smart filters, and 3D-related data have significant typed coverage, but not every workflow has exhaustive interoperability validation.

## Examples

```bash
cargo run --example read_psd path/to/file.psd
cargo run --example modify_psd path/to/input.psd output.psd
cargo run --example extract_layers path/to/file.psd
cargo run --example create_psd
```

## API Notes

- Main document type: `Psd`
- Main layer type: `Layer`
- Core enums: `BlendMode`, `ColorMode`, `Compression`
- I/O options: `ReadOptions`, `WriteOptions`

Full API docs are on [docs.rs](https://docs.rs/ag-psd).
