//! Example: Read, modify, and write a PSD
//!
//! This example demonstrates how to:
//! 1. Read an existing PSD file
//! 2. Modify its properties and layers
//! 3. Write it back to a new file

use ag_psd::*;
use std::env;
use std::fs::File;
use std::io::Write;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        println!("Usage: {} <input-psd> [output-psd]", args[0]);
        println!("\nExample:");
        println!("  cargo run --example modify_psd ../test/test.psd modified.psd");
        return Ok(());
    }

    let input_path = &args[1];
    let output_path = if args.len() >= 3 {
        args[2].clone()
    } else {
        "modified_output.psd".to_string()
    };

    println!("Reading PSD: {}\n", input_path);

    // Read the PSD file
    let file = File::open(input_path)
        .map_err(|e| PsdError::Io(e))?;
    
    let options = ReadOptions::default();
    let mut psd = read_psd(file, options)?;

    println!("Original document:");
    println!("  Size: {}x{}", psd.width, psd.height);
    println!("  Layers: {}", count_layers(&psd.children));

    // Modify the document
    println!("\nApplying modifications...");
    
    // 1. Update document name
    psd.additional_info.name = Some("Modified Document".to_string());
    println!("  ✓ Updated document name");

    // 2. Modify layers
    if let Some(ref mut children) = psd.children {
        modify_layers(children);
    }

    // 3. Add a new layer
    if psd.children.is_none() {
        psd.children = Some(Vec::new());
    }
    
    if let Some(ref mut children) = psd.children {
        let new_layer = Layer {
            top: Some(10),
            left: Some(10),
            bottom: Some(110),
            right: Some(210),
            blend_mode: Some(BlendMode::Normal),
            opacity: Some(255.0),
            additional_info: LayerAdditionalInfo {
                name: Some("New Added Layer".to_string()),
                id: Some(999),
                layer_color: Some(LayerColor::Violet),
                ..Default::default()
            },
            ..Default::default()
        };
        
        children.push(new_layer);
        println!("  ✓ Added new layer");
    }

    println!("\nModified document:");
    println!("  Layers: {}", count_layers(&psd.children));

    // Write the modified PSD
    let write_options = WriteOptions {
        compress: Some(false),
        psb: Some(false),
        generate_thumbnail: Some(false),
        ..Default::default()
    };

    println!("\nWriting to: {}", output_path);
    let buffer = write_psd(&psd, &write_options)?;
    
    let mut file = File::create(&output_path)?;
    file.write_all(&buffer)?;
    
    println!("  Buffer size: {} bytes", buffer.len());
    println!("  Saved successfully!");

    println!("\n✅ PSD modification complete!");
    
    Ok(())
}

fn modify_layers(layers: &mut [Layer]) {
    for layer in layers.iter_mut() {
        // Modify layer properties
        
        // 1. Adjust opacity - reduce by 10%
        if let Some(opacity) = layer.opacity {
            layer.opacity = Some((opacity * 0.9).max(0.0).min(255.0));
        }
        
        // 2. Add prefix to layer names
        if let Some(ref name) = layer.additional_info.name {
            let new_name = format!("MOD_{}", name);
            layer.additional_info.name = Some(new_name);
        }
        
        // 3. Change blend mode if it's Normal
        if layer.blend_mode == Some(BlendMode::Normal) {
            // Keep it normal, just as an example of conditional modification
        }
        
        // 4. Add a color tag if it doesn't have one
        if layer.additional_info.layer_color.is_none() {
            layer.additional_info.layer_color = Some(LayerColor::Orange);
        }
        
        // Recursively modify child layers
        if let Some(ref mut children) = layer.children {
            modify_layers(children);
        }
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
