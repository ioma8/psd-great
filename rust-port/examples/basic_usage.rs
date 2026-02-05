//! Basic usage example of the ag-psd library

use ag_psd::*;

fn main() {
    // Create a simple PSD document
    let psd = Psd {
        width: 800,
        height: 600,
        channels: Some(3),
        bits_per_channel: Some(8),
        color_mode: Some(ColorMode::RGB),
        children: Some(vec![
            Layer {
                top: Some(0),
                left: Some(0),
                bottom: Some(600),
                right: Some(800),
                blend_mode: Some(BlendMode::Normal),
                opacity: Some(100.0),
                additional_info: LayerAdditionalInfo {
                    name: Some("Layer 1".to_string()),
                    ..Default::default()
                },
                ..Default::default()
            }
        ]),
        additional_info: LayerAdditionalInfo {
            name: Some("My Document".to_string()),
            ..Default::default()
        },
        ..Default::default()
    };

    println!("✅ Created PSD: {}x{}", psd.width, psd.height);
    println!("   Color mode: {:?}", psd.color_mode);
    println!("   Layers: {}", psd.children.as_ref().map(|c| c.len()).unwrap_or(0));
    
    // Demonstrate serialization
    let json = serde_json::to_string_pretty(&psd).unwrap();
    println!("\n📄 JSON representation (first 500 chars):");
    println!("{}", &json[..json.len().min(500)]);
    if json.len() > 500 {
        println!("... (truncated)");
    }
}
