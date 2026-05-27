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
        let mut summary = Vec::new();

        if let Some(ref shadows) = effects.drop_shadow {
            if !shadows.is_empty() && shadows[0].enabled.unwrap_or(false) {
                summary.push("Drop Shadow".to_string());
            }
        }

        if let Some(ref shadows) = effects.inner_shadow {
            if !shadows.is_empty() && shadows[0].enabled.unwrap_or(false) {
                summary.push("Inner Shadow".to_string());
            }
        }

        if let Some(ref glow) = effects.outer_glow {
            if glow.enabled.unwrap_or(false) {
                summary.push("Outer Glow".to_string());
            }
        }

        if let Some(ref glow) = effects.inner_glow {
            if glow.enabled.unwrap_or(false) {
                summary.push("Inner Glow".to_string());
            }
        }

        if let Some(ref bevel) = effects.bevel {
            if bevel.enabled.unwrap_or(false) {
                summary.push("Bevel & Emboss".to_string());
            }
        }

        if let Some(ref satin) = effects.satin {
            if satin.enabled.unwrap_or(false) {
                summary.push("Satin".to_string());
            }
        }

        if let Some(ref stroke) = effects.stroke {
            if !stroke.is_empty() && stroke[0].enabled.unwrap_or(false) {
                summary.push("Stroke".to_string());
            }
        }

        if let Some(ref overlay) = effects.solid_fill {
            if !overlay.is_empty() && overlay[0].enabled.unwrap_or(false) {
                summary.push("Color Overlay".to_string());
            }
        }

        if let Some(ref overlay) = effects.gradient_overlay {
            if !overlay.is_empty() && overlay[0].enabled.unwrap_or(false) {
                summary.push("Gradient Overlay".to_string());
            }
        }

        if let Some(ref overlay) = effects.pattern_overlay {
            if overlay.enabled.unwrap_or(false) {
                summary.push("Pattern Overlay".to_string());
            }
        }

        (!summary.is_empty(), summary)
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
