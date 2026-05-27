//! Example: Extract layer information
//!
//! This example shows how to extract detailed information about layers,
//! including their properties, effects, masks, and hierarchy.

use psd_great::*;
use std::env;
use std::fs::File;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: {} <path-to-psd-file>", args[0]);
        println!("\nExample:");
        println!("  cargo run --example extract_layers ../test/test.psd");
        return Ok(());
    }

    let file_path = &args[1];
    println!("Extracting layer information from: {}\n", file_path);

    // Read the PSD file
    let file = File::open(file_path).map_err(|e| PsdError::Io(e))?;

    let options = ReadOptions {
        skip_layer_image_data: Some(false),
        ..Default::default()
    };

    let psd = read_psd(file, options)?;

    println!("Document: {}x{}", psd.width, psd.height);
    println!("{}", "=".repeat(70));
    println!();

    // Extract and display layer information
    if let Some(ref children) = psd.children {
        let layers_info = extract_all_layers(children, 0);

        println!("Total layers found: {}", layers_info.len());
        println!();

        for info in layers_info {
            print_layer_info(&info);
            println!("{}", "-".repeat(70));
        }
    } else {
        println!("No layers found in document.");
    }

    Ok(())
}

#[derive(Debug)]
struct LayerInfo {
    depth: usize,
    name: String,
    id: Option<i32>,
    layer_type: String,
    bounds: Option<(i32, i32, i32, i32)>,
    dimensions: Option<(i32, i32)>,
    blend_mode: Option<BlendMode>,
    opacity: Option<f64>,
    visible: bool,
    locked: bool,
    has_mask: bool,
    has_vector_mask: bool,
    has_effects: bool,
    has_text: bool,
    has_image_data: bool,
    layer_color: Option<LayerColor>,
    effects_summary: Vec<String>,
}

fn extract_all_layers(layers: &[Layer], depth: usize) -> Vec<LayerInfo> {
    let mut result = Vec::new();

    for layer in layers {
        result.push(extract_layer_info(layer, depth));

        // Recursively extract child layers
        if let Some(ref children) = layer.children {
            result.extend(extract_all_layers(children, depth + 1));
        }
    }

    result
}

fn extract_layer_info(layer: &Layer, depth: usize) -> LayerInfo {
    let name = layer
        .additional_info
        .name
        .clone()
        .unwrap_or_else(|| "<unnamed>".to_string());

    let is_group = layer
        .additional_info
        .section_divider
        .as_ref()
        .map(|sd| {
            matches!(
                sd.divider_type,
                SectionDividerType::OpenFolder | SectionDividerType::ClosedFolder
            )
        })
        .unwrap_or(false);

    let layer_type = if is_group {
        "Group".to_string()
    } else if layer.additional_info.text.is_some() {
        "Text Layer".to_string()
    } else if layer.additional_info.vector_origination.is_some() {
        "Shape Layer".to_string()
    } else {
        "Pixel Layer".to_string()
    };

    let bounds = match (layer.top, layer.left, layer.bottom, layer.right) {
        (Some(t), Some(l), Some(b), Some(r)) => Some((t, l, b, r)),
        _ => None,
    };

    let dimensions = bounds.map(|(t, l, b, r)| (r - l, b - t));

    let visible = !layer.hidden.unwrap_or(false);
    let locked = layer.transparency_protected.unwrap_or(false);

    let has_mask = layer.additional_info.mask.is_some();
    let has_vector_mask = layer.additional_info.vector_mask.is_some();
    let has_text = layer.additional_info.text.is_some();
    let has_image_data = layer.image_data.is_some();

    let (has_effects, effects_summary) = if let Some(ref effects) = layer.additional_info.effects {
        let mut summary = vec![format!("Version {}", effects.version)];
        if effects.descriptor.is_some() {
            summary.push("Descriptor".to_string());
        }
        (true, summary)
    } else {
        (false, Vec::new())
    };

    LayerInfo {
        depth,
        name,
        id: layer.additional_info.id,
        layer_type,
        bounds,
        dimensions,
        blend_mode: layer.blend_mode,
        opacity: layer.opacity,
        visible,
        locked,
        has_mask,
        has_vector_mask,
        has_effects,
        has_text,
        has_image_data,
        layer_color: layer.additional_info.layer_color,
        effects_summary,
    }
}

fn print_layer_info(info: &LayerInfo) {
    let indent = "  ".repeat(info.depth);

    println!("{}📄 {}", indent, info.name);
    println!("{}   Type: {}", indent, info.layer_type);

    if let Some(id) = info.id {
        println!("{}   ID: {}", indent, id);
    }

    if let Some(bounds) = info.bounds {
        println!(
            "{}   Position: ({}, {}) to ({}, {})",
            indent, bounds.1, bounds.0, bounds.3, bounds.2
        );
    }

    if let Some(dims) = info.dimensions {
        println!("{}   Size: {}x{}", indent, dims.0, dims.1);
    }

    if let Some(blend_mode) = info.blend_mode {
        println!("{}   Blend: {:?}", indent, blend_mode);
    }

    if let Some(opacity) = info.opacity {
        let opacity_pct = (opacity / 255.0 * 100.0).round();
        println!("{}   Opacity: {}%", indent, opacity_pct);
    }

    println!(
        "{}   Visible: {}",
        indent,
        if info.visible { "✓" } else { "✗" }
    );

    if info.locked {
        println!("{}   Locked: ✓", indent);
    }

    if let Some(color) = info.layer_color {
        if !matches!(color, LayerColor::None) {
            println!("{}   Color: {:?}", indent, color);
        }
    }

    if info.has_mask {
        println!("{}   Has mask: ✓", indent);
    }

    if info.has_vector_mask {
        println!("{}   Has vector mask: ✓", indent);
    }

    if info.has_text {
        println!("{}   Has text: ✓", indent);
    }

    if info.has_image_data {
        println!("{}   Has image data: ✓", indent);
    }

    if info.has_effects {
        println!("{}   Effects: {}", indent, info.effects_summary.join(", "));
    }
}
