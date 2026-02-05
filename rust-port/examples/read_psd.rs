//! Example: Read and print PSD information
//!
//! This example demonstrates how to read a PSD file and display its properties,
//! including document info, layers, effects, and resources.

use ag_psd::*;
use std::env;
use std::fs::File;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        println!("Usage: {} <path-to-psd-file>", args[0]);
        println!("\nExample:");
        println!("  cargo run --example read_psd ../test/test.psd");
        return Ok(());
    }

    let file_path = &args[1];
    println!("Reading PSD file: {}\n", file_path);

    // Open and read the PSD file
    let file = File::open(file_path)
        .map_err(|e| PsdError::Io(e))?;
    
    let options = ReadOptions {
        skip_layer_image_data: Some(false),
        skip_composite_image_data: Some(false),
        skip_thumbnail: Some(false),
        use_image_data: Some(true),
        use_raw_data: Some(false),
        ..Default::default()
    };

    let psd = read_psd(file, options)?;

    // Print document information
    print_document_info(&psd);
    
    // Print layer information
    if let Some(ref children) = psd.children {
        println!("\n📋 Layers ({} total):", count_layers(&psd.children));
        println!("{}", "=".repeat(60));
        print_layers(children, 0);
    } else {
        println!("\nNo layers found in document.");
    }

    // Print image resources
    if let Some(ref resources) = psd.image_resources {
        print_image_resources(resources);
    }

    println!("\n✅ Successfully read PSD file!");
    
    Ok(())
}

fn print_document_info(psd: &Psd) {
    println!("📄 Document Information:");
    println!("{}", "=".repeat(60));
    println!("  Dimensions: {}x{} pixels", psd.width, psd.height);
    println!("  Color mode: {:?}", psd.color_mode.unwrap_or(ColorMode::RGB));
    println!("  Channels: {}", psd.channels.unwrap_or(0));
    println!("  Bits per channel: {}", psd.bits_per_channel.unwrap_or(8));
    
    if let Some(ref name) = psd.additional_info.name {
        println!("  Name: {}", name);
    }
    
    if let Some(ref linked_files) = psd.linked_files {
        println!("  Linked files: {}", linked_files.len());
    }
    
    if let Some(ref artboards) = psd.artboards {
        println!("  Artboards: {}", artboards.count);
    }
}

fn print_layers(layers: &[Layer], indent: usize) {
    let prefix = "  ".repeat(indent);
    
    for (i, layer) in layers.iter().enumerate() {
        let name = layer.additional_info.name
            .as_ref()
            .map(|s| s.as_str())
            .unwrap_or("<unnamed>");
        
        let is_group = layer.additional_info.section_divider
            .as_ref()
            .map(|sd| matches!(sd.divider_type, SectionDividerType::OpenFolder | SectionDividerType::ClosedFolder))
            .unwrap_or(false);
        
        let layer_type = if is_group { "📁" } else { "📄" };
        
        println!("{}{}[{}] {}", prefix, layer_type, i + 1, name);
        
        // Print layer properties
        if let Some(blend_mode) = layer.blend_mode {
            println!("{}    Blend mode: {:?}", prefix, blend_mode);
        }
        
        if let Some(opacity) = layer.opacity {
            println!("{}    Opacity: {:.1}%", prefix, (opacity / 255.0) * 100.0);
        }
        
        if let Some(bounds) = get_layer_bounds(layer) {
            println!("{}    Bounds: {}", prefix, bounds);
        }
        
        if let Some(ref color) = layer.additional_info.layer_color {
            println!("{}    Color: {:?}", prefix, color);
        }
        
        if layer.hidden == Some(true) {
            println!("{}    Hidden: yes", prefix);
        }
        
        // Print effects
        if let Some(ref effects) = layer.additional_info.effects {
            print_layer_effects(effects, indent + 1);
        }
        
        // Print text data
        if let Some(ref text_data) = layer.additional_info.text {
            let text = &text_data.text;
            let preview = if text.len() > 50 {
                format!("{}...", &text[..50])
            } else {
                text.clone()
            };
            println!("{}    Text: \"{}\"", prefix, preview);
        }
        
        // Print child layers recursively
        if let Some(ref children) = layer.children {
            print_layers(children, indent + 1);
        }
    }
}

fn print_layer_effects(effects: &LayerEffectsInfo, indent: usize) {
    let prefix = "  ".repeat(indent);
    println!("{}✨ Effects:", prefix);
    
    if effects.disabled == Some(true) {
        println!("{}  (disabled)", prefix);
    }
    
    if let Some(ref shadows) = effects.drop_shadow {
        for shadow in shadows {
            if shadow.enabled == Some(true) {
                println!("{}  • Drop Shadow", prefix);
                if let Some(angle) = shadow.angle {
                    println!("{}    Angle: {}°", prefix, angle);
                }
                if let Some(ref distance) = shadow.distance {
                    println!("{}    Distance: {}{:?}", prefix, distance.value, distance.units);
                }
            }
        }
    }
    
    if let Some(ref glow) = effects.outer_glow {
        if glow.enabled == Some(true) {
            println!("{}  • Outer Glow", prefix);
        }
    }
    
    if let Some(ref glow) = effects.inner_glow {
        if glow.enabled == Some(true) {
            println!("{}  • Inner Glow", prefix);
        }
    }
    
    if effects.bevel.is_some() {
        println!("{}  • Bevel & Emboss", prefix);
    }
    
    if effects.stroke.is_some() {
        println!("{}  • Stroke", prefix);
    }
}

fn print_image_resources(resources: &ImageResources) {
    println!("\n🖼️  Image Resources:");
    println!("{}", "=".repeat(60));
    
    if let Some(ref version_info) = resources.version_info {
        println!("  Writer: {}", version_info.writer_name);
        println!("  Reader: {}", version_info.reader_name);
        println!("  File version: {}", version_info.file_version);
    }
    
    if let Some(ref xmp) = resources.xmp_metadata {
        println!("  XMP metadata: {} bytes", xmp.len());
    }
    
    if let Some(ref alpha_ids) = resources.alpha_identifiers {
        println!("  Alpha channels: {}", alpha_ids.len());
    }
}

fn get_layer_bounds(layer: &Layer) -> Option<String> {
    match (layer.top, layer.left, layer.bottom, layer.right) {
        (Some(t), Some(l), Some(b), Some(r)) => {
            Some(format!("({}, {}) to ({}, {}) [{}x{}]",
                l, t, r, b, r - l, b - t))
        }
        _ => None,
    }
}

fn count_layers(children: &Option<Vec<Layer>>) -> usize {
    match children {
        None => 0,
        Some(layers) => {
            let mut count = layers.len();
            for layer in layers {
                count += count_layers(&layer.children);
            }
            count
        }
    }
}
