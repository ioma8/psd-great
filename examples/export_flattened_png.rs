use flate2::write::ZlibEncoder;
use flate2::Compression as ZlibCompression;
use psd_great::{read_psd, PixelData, PsdError, ReadOptions, Result};
use std::env;
use std::fs;
use std::io::Write;
use std::path::Path;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: cargo run --example export_flattened_png -- <input.psd> <output.png>");
        std::process::exit(2);
    }

    let input = &args[1];
    let output = &args[2];

    let file = fs::File::open(input).map_err(PsdError::Io)?;
    let psd = read_psd(
        file,
        ReadOptions {
            skip_layer_image_data: Some(true),
            skip_composite_image_data: Some(false),
            use_image_data: Some(true),
            use_raw_data: Some(false),
            ..Default::default()
        },
    )?;

    let image = psd
        .image_data
        .as_ref()
        .ok_or_else(|| PsdError::InvalidFormat("PSD has no composite image data".to_string()))?;

    write_rgba_png(Path::new(output), image)?;
    println!("{output}");
    Ok(())
}

fn write_rgba_png(path: &Path, image: &PixelData) -> Result<()> {
    let mut png = Vec::new();
    png.extend_from_slice(b"\x89PNG\r\n\x1a\n");

    let ihdr = build_ihdr(image.width as u32, image.height as u32);
    append_chunk(&mut png, b"IHDR", &ihdr);

    let idat = build_idat(image)?;
    append_chunk(&mut png, b"IDAT", &idat);
    append_chunk(&mut png, b"IEND", &[]);

    fs::write(path, png).map_err(PsdError::Io)
}

fn build_ihdr(width: u32, height: u32) -> Vec<u8> {
    let mut ihdr = Vec::with_capacity(13);
    ihdr.extend_from_slice(&width.to_be_bytes());
    ihdr.extend_from_slice(&height.to_be_bytes());
    ihdr.push(8); // bit depth
    ihdr.push(6); // color type RGBA
    ihdr.push(0); // compression method
    ihdr.push(0); // filter method
    ihdr.push(0); // interlace method
    ihdr
}

fn build_idat(image: &PixelData) -> Result<Vec<u8>> {
    let row_bytes = image.width * 4;
    if image.data.len() != row_bytes * image.height {
        return Err(PsdError::InvalidFormat(format!(
            "Unexpected composite image size: got {}, expected {}",
            image.data.len(),
            row_bytes * image.height
        )));
    }

    let mut filtered = Vec::with_capacity(image.data.len() + image.height);
    for row in image.data.chunks_exact(row_bytes) {
        filtered.push(0); // no filter
        filtered.extend_from_slice(row);
    }

    let mut encoder = ZlibEncoder::new(Vec::new(), ZlibCompression::default());
    encoder.write_all(&filtered).map_err(PsdError::Io)?;
    encoder.finish().map_err(PsdError::Io)
}

fn append_chunk(png: &mut Vec<u8>, chunk_type: &[u8; 4], data: &[u8]) {
    png.extend_from_slice(&(data.len() as u32).to_be_bytes());
    png.extend_from_slice(chunk_type);
    png.extend_from_slice(data);

    let mut crc_input = Vec::with_capacity(chunk_type.len() + data.len());
    crc_input.extend_from_slice(chunk_type);
    crc_input.extend_from_slice(data);
    png.extend_from_slice(&crc32(&crc_input).to_be_bytes());
}

fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = 0xffff_ffffu32;
    for &byte in bytes {
        crc ^= byte as u32;
        for _ in 0..8 {
            let mask = (crc & 1).wrapping_neg() & 0xedb8_8320;
            crc = (crc >> 1) ^ mask;
        }
    }
    !crc
}
