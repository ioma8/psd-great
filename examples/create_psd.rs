//! Example: Create a simple PSD from scratch
//!
//! This example demonstrates how to create a new PSD document programmatically
//! with layers and write it to a file.

use psd_great::additional_info::SectionDivider;
use psd_great::*;
use std::env;
use std::fs::File;
use std::io::Write;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let output_path = if args.len() >= 2 {
        args[1].as_str()
    } else {
        "created_example.psd"
    };

    println!("Creating a new PSD document...\n");

    // Create a PSD with RGB color mode
    let mut psd = Psd {
        width: 800,
        height: 600,
        channels: Some(4), // RGBA
        bits_per_channel: Some(8),
        color_mode: Some(ColorMode::RGB),
        additional_info: LayerAdditionalInfo {
            name: Some("My Created Document".to_string()),
            ..Default::default()
        },
        ..Default::default()
    };

    // Create layers
    let mut layers = Vec::new();

    // Background layer
    layers.push(Layer {
        top: Some(0),
        left: Some(0),
        bottom: Some(600),
        right: Some(800),
        blend_mode: Some(BlendMode::Normal),
        opacity: Some(255.0),
        transparency_protected: Some(false),
        hidden: Some(false),
        clipping: Some(0),
        additional_info: LayerAdditionalInfo {
            name: Some("Background".to_string()),
            id: Some(1),
            layer_color: Some(LayerColor::None),
            ..Default::default()
        },
        ..Default::default()
    });

    layers.push(Layer {
        top: Some(100),
        left: Some(100),
        bottom: Some(400),
        right: Some(500),
        blend_mode: Some(BlendMode::Normal),
        opacity: Some(255.0),
        transparency_protected: Some(false),
        hidden: Some(false),
        clipping: Some(0),
        additional_info: LayerAdditionalInfo {
            name: Some("Foreground Layer".to_string()),
            id: Some(2),
            layer_color: Some(LayerColor::Blue),
            ..Default::default()
        },
        ..Default::default()
    });

    // Create a layer group
    let mut group_layer = Layer {
        blend_mode: Some(BlendMode::PassThrough),
        opacity: Some(255.0),
        additional_info: LayerAdditionalInfo {
            name: Some("Group 1".to_string()),
            id: Some(3),
            section_divider: Some(SectionDivider {
                divider_type: SectionDividerType::OpenFolder,
                blend_mode: None,
                sub_type: None,
            }),
            layer_color: Some(LayerColor::Green),
            ..Default::default()
        },
        ..Default::default()
    };

    // Add child layers to the group
    let group_children = vec![
        Layer {
            top: Some(50),
            left: Some(50),
            bottom: Some(150),
            right: Some(250),
            blend_mode: Some(BlendMode::Normal),
            opacity: Some(200.0),
            additional_info: LayerAdditionalInfo {
                name: Some("Child Layer 1".to_string()),
                id: Some(4),
                ..Default::default()
            },
            ..Default::default()
        },
        Layer {
            top: Some(200),
            left: Some(200),
            bottom: Some(400),
            right: Some(600),
            blend_mode: Some(BlendMode::Multiply),
            opacity: Some(180.0),
            additional_info: LayerAdditionalInfo {
                name: Some("Child Layer 2".to_string()),
                id: Some(5),
                layer_color: Some(LayerColor::Red),
                ..Default::default()
            },
            ..Default::default()
        },
    ];

    group_layer.children = Some(group_children);
    layers.push(group_layer);

    psd.children = Some(layers);

    // Print document info
    println!("Document created:");
    println!("  Size: {}x{}", psd.width, psd.height);
    println!("  Color mode: {:?}", psd.color_mode);
    println!("  Bits per channel: {:?}", psd.bits_per_channel);
    println!("  Total layers: {}", count_layers(&psd.children));
    println!();

    // Write the PSD to a buffer
    let options = WriteOptions {
        compress: Some(false),
        psb: Some(false),
        generate_thumbnail: Some(false),
        trim_image_data: Some(false),
        ..Default::default()
    };

    println!("Writing PSD to buffer...");
    let buffer = write_psd(&psd, &options)?;
    println!("  Buffer size: {} bytes", buffer.len());

    let mut file = File::create(output_path)?;
    file.write_all(&buffer)?;
    println!("  Saved to: {}", output_path);

    println!("\n✅ PSD created successfully!");

    Ok(())
}

/// Count total number of layers including nested layers
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
