# psd-great

A Rust library for reading and writing Adobe Photoshop PSD and PSB files.

This crate began as a port of the TypeScript [ag-psd](https://github.com/Agamnentzar/ag-psd) parser/writer, but has since been expanded substantially with additional PSD/PSB feature coverage and a typed Rust API for document structure, layers, tagged blocks, resources, text data, effects, masks, paths, and smart-object metadata.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Installation

This crate is not currently published on crates.io.

Use it as a Git dependency:

```toml
[dependencies]
psd-great = { git = "git@github.com:ioma8/psd-great.git" }
```

## Quick Start

```rust
use psd_great::*;
use std::fs::File;

fn main() -> Result<()> {
    let file = File::open("input.psd")?;
    let mut psd = read_psd(file, ReadOptions::default())?;

    println!("{}x{}", psd.width, psd.height);

    if let Some(ref mut layers) = psd.children {
        for layer in layers {
            layer.opacity = Some(128.0);
        }
    }

    let bytes = write_psd(&psd, &WriteOptions::default())?;
    std::fs::write("output.psd", bytes)?;
    Ok(())
}
```

## Crate Layout

The codebase is organized into a few top-level groups:

- `api` - public document model types such as `Psd`, `Layer`, `Color`, `DisplayInfo`, and related enums/structs
- `format` - PSD/PSB wire-format sections such as image resources, tagged blocks, and document postprocess logic
- `io` - high-level reader and writer entry points
- `support` - internal helpers such as descriptor parsing, compression, JPEG helpers, and binary record support
- `formats` - additional Adobe-related formats such as ABR, ASE, and CSH

The crate root still re-exports the main user-facing types and functions for ergonomic use:

- `read_psd`, `write_psd`
- `Psd`, `Layer`
- `ReadOptions`, `WriteOptions`
- core enums and data types such as `BlendMode`, `ColorMode`, `Compression`, and `Color`

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
| Layers, masks, effects | Broad support | Includes vector masks, many tagged blocks, and effect structures |
| Text layers | Partial | Rich text structures are supported, but not every Photoshop text workflow is exhaustively validated |
| Color modes | Partial | Indexed palettes and generic color-mode data are supported; some composite-image behavior still varies by mode |
| 16/32-bit depth | Partial | Structural support exists, but not every path is fully validated end to end |
| Smart objects / linked data | Partial | Typed structures exist, but not every Photoshop workflow is exhaustively covered |

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
- Additional file formats: `read_abr`, `read_ase`, `write_ase`, `read_csh`

Generate API docs locally with:

```bash
cargo doc --open
```

## Notes For Contributors

- The public model lives under `src/api/`
- PSD/PSB wire-format logic lives under `src/format/`
- Reader/writer entry points live under `src/io/`
- Internal helpers live under `src/support/`

This structure is intentional: public data model, wire format, I/O orchestration, and low-level support are kept distinct so the crate is easier to navigate and document.
