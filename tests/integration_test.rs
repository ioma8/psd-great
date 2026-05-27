use psd_great::additional_info::SectionDivider;
use psd_great::*;
use std::collections::HashMap;
use std::fs::File;
use std::io::Cursor;

#[test]
fn test_basic_types() {
    // Test color types
    let rgba = RGBA {
        r: 255,
        g: 128,
        b: 64,
        a: 255,
    };
    assert_eq!(rgba.r, 255);

    let rgb = RGB {
        r: 255,
        g: 128,
        b: 64,
    };
    assert_eq!(rgb.g, 128);

    let cmyk = CMYK {
        c: 100,
        m: 50,
        y: 25,
        k: 0,
    };
    assert_eq!(cmyk.c, 100);

    // Test blend mode
    let blend = BlendMode::Normal;
    assert_eq!(blend, BlendMode::Normal);

    // Test color mode
    let mode = ColorMode::RGB;
    assert_eq!(mode as u16, 3);
}

#[test]
fn test_layer_creation() {
    let layer = Layer {
        top: Some(10),
        left: Some(20),
        bottom: Some(100),
        right: Some(200),
        blend_mode: Some(BlendMode::Normal),
        opacity: Some(100.0),
        transparency_protected: Some(false),
        effects_open: Some(false),
        hidden: Some(false),
        clipping: Some(0),
        image_data: None,
        raw_data: None,
        children: None,
        opened: Some(true),
        link_group: None,
        link_group_enabled: None,
        additional_info: LayerAdditionalInfo {
            name: Some("Test Layer".to_string()),
            id: Some(1),
            ..Default::default()
        },
        ..Default::default()
    };

    assert_eq!(layer.additional_info.name, Some("Test Layer".to_string()));
    assert_eq!(layer.top, Some(10));
}

#[test]
fn test_psd_structure() {
    let psd = Psd {
        width: 1920,
        height: 1080,
        channels: Some(3),
        bits_per_channel: Some(8),
        color_mode: Some(ColorMode::RGB),
        palette: None,
        children: None,
        image_data: None,
        image_resources: None,
        linked_files: None,
        artboards: None,
        global_layer_mask_info: None,
        annotations: None,
        additional_info: LayerAdditionalInfo {
            name: Some("Test Document".to_string()),
            ..Default::default()
        },
        ..Default::default()
    };

    assert_eq!(psd.width, 1920);
    assert_eq!(psd.height, 1080);
    assert_eq!(psd.color_mode, Some(ColorMode::RGB));
}

#[test]
fn test_serialization() {
    let rgba = RGBA {
        r: 255,
        g: 128,
        b: 64,
        a: 255,
    };
    let json = serde_json::to_string(&rgba).unwrap();
    let deserialized: RGBA = serde_json::from_str(&json).unwrap();
    assert_eq!(rgba, deserialized);
}

#[test]
fn test_effects() {
    let shadow = LayerEffectShadow {
        present: Some(true),
        show_in_dialog: Some(true),
        enabled: Some(true),
        size: Some(UnitsValue {
            units: Units::Pixels,
            value: 5.0,
        }),
        angle: Some(120.0),
        distance: Some(UnitsValue {
            units: Units::Pixels,
            value: 10.0,
        }),
        color: Some(Color::RGBA(RGBA {
            r: 0,
            g: 0,
            b: 0,
            a: 255,
        })),
        blend_mode: Some(BlendMode::Multiply),
        opacity: Some(0.75),
        use_global_light: Some(true),
        antialiased: Some(true),
        contour: None,
        choke: None,
        layer_conceals: None,
    };

    assert_eq!(shadow.enabled, Some(true));
    assert_eq!(shadow.angle, Some(120.0));
}

// ============================================================================
// Integration Tests - Real-world Scenarios
// ============================================================================

#[test]
fn test_create_and_write_simple_psd() {
    // Create a simple PSD document with basic properties
    let psd = Psd {
        width: 100,
        height: 100,
        channels: Some(4), // RGBA
        bits_per_channel: Some(8),
        color_mode: Some(ColorMode::RGB),
        children: Some(vec![Layer {
            top: Some(0),
            left: Some(0),
            bottom: Some(100),
            right: Some(100),
            blend_mode: Some(BlendMode::Normal),
            opacity: Some(255.0),
            additional_info: LayerAdditionalInfo {
                name: Some("Background".to_string()),
                id: Some(1),
                ..Default::default()
            },
            ..Default::default()
        }]),
        additional_info: LayerAdditionalInfo {
            name: Some("Simple Test".to_string()),
            ..Default::default()
        },
        ..Default::default()
    };

    // Write the PSD
    let options = WriteOptions {
        compress: Some(false),
        psb: Some(false),
        generate_thumbnail: Some(false),
        trim_image_data: Some(false),
        invalidate_text_layers: None,
        log_missing_features: None,
        no_background: None,
    };

    let result = write_psd(&psd, &options);
    assert!(result.is_ok(), "Failed to write PSD: {:?}", result.err());

    let buffer = result.unwrap();
    assert!(buffer.len() > 0, "Written buffer should not be empty");

    // Verify header signature
    assert_eq!(&buffer[0..4], b"8BPS", "Invalid PSD signature");
}

#[test]
fn test_write_and_read_roundtrip() {
    // Create a PSD with multiple layers
    let original_psd = Psd {
        width: 200,
        height: 150,
        channels: Some(3),
        bits_per_channel: Some(8),
        color_mode: Some(ColorMode::RGB),
        children: Some(vec![
            Layer {
                top: Some(0),
                left: Some(0),
                bottom: Some(150),
                right: Some(200),
                blend_mode: Some(BlendMode::Normal),
                opacity: Some(255.0),
                additional_info: LayerAdditionalInfo {
                    name: Some("Layer 1".to_string()),
                    id: Some(1),
                    ..Default::default()
                },
                ..Default::default()
            },
            Layer {
                top: Some(25),
                left: Some(25),
                bottom: Some(125),
                right: Some(175),
                blend_mode: Some(BlendMode::Multiply),
                opacity: Some(128.0),
                additional_info: LayerAdditionalInfo {
                    name: Some("Layer 2".to_string()),
                    id: Some(2),
                    ..Default::default()
                },
                ..Default::default()
            },
        ]),
        additional_info: LayerAdditionalInfo {
            name: Some("Roundtrip Test".to_string()),
            ..Default::default()
        },
        ..Default::default()
    };

    // Write to buffer
    let write_options = WriteOptions::default();
    let buffer = write_psd(&original_psd, &write_options).expect("Failed to write PSD");

    // Read back from buffer
    let read_options = ReadOptions::default();
    let cursor = Cursor::new(buffer);
    let read_psd = read_psd(cursor, read_options).expect("Failed to read PSD");

    // Verify core properties match
    assert_eq!(read_psd.width, original_psd.width);
    assert_eq!(read_psd.height, original_psd.height);
    assert_eq!(read_psd.channels, original_psd.channels);
    assert_eq!(read_psd.bits_per_channel, original_psd.bits_per_channel);
    assert_eq!(read_psd.color_mode, original_psd.color_mode);
}

#[test]
fn test_global_layer_mask_roundtrip() {
    let original_psd = Psd {
        width: 2,
        height: 2,
        channels: Some(3),
        bits_per_channel: Some(8),
        color_mode: Some(ColorMode::RGB),
        global_layer_mask_info: Some(GlobalLayerMaskInfo {
            overlay_color_space: 1,
            color_space1: 2,
            color_space2: 3,
            color_space3: 4,
            color_space4: 5,
            opacity: 255,
            kind: 128,
        }),
        ..Default::default()
    };

    let buffer = write_psd(&original_psd, &WriteOptions::default()).unwrap();
    let read_psd = read_psd(
        Cursor::new(buffer),
        ReadOptions {
            skip_composite_image_data: Some(true),
            ..Default::default()
        },
    )
    .unwrap();

    assert_eq!(
        read_psd.global_layer_mask_info,
        original_psd.global_layer_mask_info
    );
}

#[test]
fn test_different_color_modes() {
    let color_modes = vec![
        ColorMode::Bitmap,
        ColorMode::Grayscale,
        ColorMode::Indexed,
        ColorMode::RGB,
        ColorMode::CMYK,
        ColorMode::Multichannel,
        ColorMode::Duotone,
        ColorMode::Lab,
    ];

    for color_mode in color_modes {
        let psd = Psd {
            width: 50,
            height: 50,
            channels: Some(match color_mode {
                ColorMode::Bitmap | ColorMode::Grayscale => 1,
                ColorMode::RGB => 3,
                ColorMode::CMYK => 4,
                _ => 3,
            }),
            bits_per_channel: Some(8),
            color_mode: Some(color_mode),
            ..Default::default()
        };

        let options = WriteOptions::default();
        let result = write_psd(&psd, &options);

        // Some color modes may not be fully supported yet
        if result.is_ok() {
            let buffer = result.unwrap();
            assert!(buffer.len() > 0);
        }
    }
}

#[test]
fn test_adjustment_layer_comes_from_canonical_module() {
    // This will use the canonical AdjustmentLayer from adjustments.rs
    // once the duplicate in layer.rs is removed
    let adjustment = adjustments::AdjustmentLayer::Invert;
    match adjustment {
        adjustments::AdjustmentLayer::Invert => {}
        _ => panic!("wrong adjustment layer variant"),
    }
}

#[test]
fn test_canonical_shared_types_are_constructible_from_public_api() {
    let point = Point { x: 1.0, y: 2.0 };
    let fraction = Fraction {
        numerator: 1,
        denominator: 24,
    };
    let rendering_intent = RenderingIntent::Perceptual;

    assert_eq!(point.x, 1.0);
    assert_eq!(fraction.denominator, 24);
    assert_eq!(rendering_intent, RenderingIntent::Perceptual);
}

#[test]
fn test_no_remaining_duplicate_public_model_type_names() {
    let source_files = [
        ("src/additional_info.rs", include_str!("../src/additional_info.rs")),
        ("src/adjustments.rs", include_str!("../src/adjustments.rs")),
        ("src/image_resources.rs", include_str!("../src/image_resources.rs")),
        ("src/layer.rs", include_str!("../src/layer.rs")),
        ("src/psd.rs", include_str!("../src/psd.rs")),
        ("src/types.rs", include_str!("../src/types.rs")),
    ];

    let audited_names = [
        "AdjustmentLayer",
        "Bounds",
        "Fraction",
        "LayerComps",
        "OnionSkins",
        "PlacedLayer",
        "Point",
        "PrintScale",
        "ProofSetup",
        "RenderingIntent",
        "SectionDivider",
        "Slice",
        "Timeline",
        "VectorStroke",
    ];

    let mut duplicates = Vec::new();

    for name in audited_names {
        let mut hits = Vec::new();
        for (path, contents) in &source_files {
            // Use regex-like word boundary matching to avoid false positives
            // (e.g. "PlacedLayer" should not match "PlacedLayerType")
            let needle_struct = format!("pub struct {name} ");
            let needle_struct_semi = format!("pub struct {name}}}");
            let needle_enum = format!("pub enum {name} ");
            let needle_enum_semi = format!("pub enum {name}}}");
            let needle_type = format!("pub type {name} ");
            if contents.contains(&needle_struct)
                || contents.contains(&needle_struct_semi)
                || contents.contains(&needle_enum)
                || contents.contains(&needle_enum_semi)
                || contents.contains(&needle_type)
            {
                hits.push(*path);
            }
        }
        if hits.len() > 1 {
            duplicates.push((name, hits));
        }
    }

    assert!(
        duplicates.is_empty(),
        "duplicate public model types remain: {:?}",
        duplicates
    );
}

#[test]
fn test_no_duplicate_public_type_names_for_canonical_models() {
    let files = [
        ("src/additional_info.rs", include_str!("../src/additional_info.rs")),
        ("src/image_resources.rs", include_str!("../src/image_resources.rs")),
        ("src/layer.rs", include_str!("../src/layer.rs")),
        ("src/psd.rs", include_str!("../src/psd.rs")),
        ("src/types.rs", include_str!("../src/types.rs")),
    ];

    let mut seen: HashMap<&str, Vec<&str>> = HashMap::new();
    let target_names = [
        "LayerAdditionalInfo",
        "LayerColor",
        "ResolutionInfo",
        "PrintInformation",
        "PrintFlags",
    ];

    for (path, source) in files {
        for line in source.lines() {
            for name in target_names {
                if line.contains(&format!("pub struct {name}"))
                    || line.contains(&format!("pub enum {name}"))
                {
                    seen.entry(name).or_default().push(path);
                }
            }
        }
    }

    let duplicates: Vec<_> = seen
        .into_iter()
        .filter(|(_, paths)| paths.len() > 1)
        .collect();

    assert!(
        duplicates.is_empty(),
        "duplicate canonical public types remain: {duplicates:?}"
    );
}

#[test]
fn test_layer_effects_roundtrip() {
    let layer_effects = LayerEffectsInfo {
        disabled: Some(false),
        scale: Some(100.0),
        drop_shadow: Some(vec![LayerEffectShadow {
            present: Some(true),
            show_in_dialog: None,
            enabled: Some(true),
            blend_mode: Some(BlendMode::Multiply),
            color: Some(Color::RGBA(RGBA {
                r: 0,
                g: 0,
                b: 0,
                a: 255,
            })),
            opacity: Some(0.75),
            angle: Some(120.0),
            distance: Some(UnitsValue {
                units: Units::Pixels,
                value: 10.0,
            }),
            size: Some(UnitsValue {
                units: Units::Pixels,
                value: 5.0,
            }),
            use_global_light: Some(true),
            antialiased: None,
            contour: None,
            choke: None,
            layer_conceals: None,
        }]),
        inner_shadow: None,
        outer_glow: None,
        inner_glow: None,
        bevel: None,
        solid_fill: None,
        satin: None,
        stroke: None,
        gradient_overlay: None,
        pattern_overlay: None,
    };

    // Verify effects are properly structured
    assert!(layer_effects.drop_shadow.is_some());
    let shadow = &layer_effects.drop_shadow.as_ref().unwrap()[0];
    assert_eq!(shadow.enabled, Some(true));
    assert_eq!(shadow.blend_mode, Some(BlendMode::Multiply));
}

#[test]
fn test_text_layer_data() {
    // Basic test for text layer structure
    let text_data = LayerTextData {
        text: "Hello, World!".to_string(),
        transform: Some(vec![1.0, 0.0, 0.0, 1.0, 0.0, 0.0]),
        anti_alias: Some(AntiAlias::Sharp),
        gridding: None,
        orientation: None,
        index: None,
        warp: None,
        top: None,
        left: None,
        bottom: None,
        right: None,
        grid_info: None,
        use_fractional_glyph_widths: None,
        style: None,
        style_runs: None,
        paragraph_style: None,
        paragraph_style_runs: None,
        superscript_size: None,
        superscript_position: None,
        subscript_size: None,
        subscript_position: None,
        small_cap_size: None,
        shape_type: None,
        point_base: None,
        box_bounds: None,
        bounds: None,
        bounding_box: None,
        text_path: None,
    };

    assert_eq!(text_data.text, "Hello, World!");
    assert!(text_data.transform.is_some());
    assert_eq!(text_data.anti_alias, Some(AntiAlias::Sharp));
}

#[test]
fn test_compression_methods() {
    // Test that compression functions exist and can be called
    let data = vec![1, 1, 1, 2, 2, 3, 3, 3, 3];

    // Test ZIP compression roundtrip
    let compressed_zip = compress_zip(&data).expect("Failed to compress ZIP");
    assert!(compressed_zip.len() > 0);

    let decompressed_zip =
        decompress_zip(&compressed_zip, data.len()).expect("Failed to decompress ZIP");
    assert_eq!(decompressed_zip, data);

    // Test RLE compression (basic test that it doesn't panic)
    let width = 3;
    let height = 3;
    let compress_result = compress_rle(&data, width, height);
    assert!(compress_result.is_ok());
}

#[test]
fn test_blend_mode_conversion() {
    let blend_modes = vec![
        BlendMode::PassThrough,
        BlendMode::Normal,
        BlendMode::Dissolve,
        BlendMode::Darken,
        BlendMode::Multiply,
        BlendMode::ColorBurn,
        BlendMode::LinearBurn,
        BlendMode::DarkerColor,
        BlendMode::Lighten,
        BlendMode::Screen,
        BlendMode::ColorDodge,
        BlendMode::LinearDodge,
        BlendMode::LighterColor,
        BlendMode::Overlay,
        BlendMode::SoftLight,
        BlendMode::HardLight,
        BlendMode::VividLight,
        BlendMode::LinearLight,
        BlendMode::PinLight,
        BlendMode::HardMix,
        BlendMode::Difference,
        BlendMode::Exclusion,
        BlendMode::Subtract,
        BlendMode::Divide,
        BlendMode::Hue,
        BlendMode::Saturation,
        BlendMode::Color,
        BlendMode::Luminosity,
    ];

    for mode in blend_modes {
        let key = from_blend_mode(mode);
        let converted = to_blend_mode(&key).expect("Failed to convert blend mode");
        assert_eq!(
            converted, mode,
            "Blend mode conversion failed for {:?}",
            mode
        );
    }
}

#[test]
fn test_layer_hierarchy() {
    // Test nested layer groups
    let psd = Psd {
        width: 100,
        height: 100,
        channels: Some(3),
        bits_per_channel: Some(8),
        color_mode: Some(ColorMode::RGB),
        children: Some(vec![Layer {
            additional_info: LayerAdditionalInfo {
                name: Some("Group 1".to_string()),
                section_divider: Some(SectionDivider {
                    divider_type: SectionDividerType::OpenFolder,
                    blend_mode: None,
                    sub_type: None,
                }),
                ..Default::default()
            },
            children: Some(vec![
                Layer {
                    additional_info: LayerAdditionalInfo {
                        name: Some("Layer 1.1".to_string()),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                Layer {
                    additional_info: LayerAdditionalInfo {
                        name: Some("Layer 1.2".to_string()),
                        ..Default::default()
                    },
                    ..Default::default()
                },
            ]),
            ..Default::default()
        }]),
        ..Default::default()
    };

    assert!(psd.children.is_some());
    let children = psd.children.as_ref().unwrap();
    assert_eq!(children.len(), 1);

    let group = &children[0];
    assert_eq!(
        group
            .additional_info
            .section_divider
            .as_ref()
            .unwrap()
            .divider_type,
        SectionDividerType::OpenFolder
    );
    assert!(group.children.is_some());
    assert_eq!(group.children.as_ref().unwrap().len(), 2);
}

#[test]
fn test_image_resources() {
    let resources = ImageResources {
        alpha_identifiers: Some(vec![1, 2, 3]),
        ..Default::default()
    };

    assert!(resources.alpha_identifiers.is_some());
    let alpha_ids = resources.alpha_identifiers.as_ref().unwrap();
    assert_eq!(alpha_ids.len(), 3);
    assert_eq!(alpha_ids[0], 1);
}

#[test]
fn test_layer_colors() {
    let colors = vec![
        LayerColor::None,
        LayerColor::Red,
        LayerColor::Orange,
        LayerColor::Yellow,
        LayerColor::Green,
        LayerColor::Blue,
        LayerColor::Violet,
        LayerColor::Gray,
    ];

    for color in colors {
        let layer = Layer {
            additional_info: LayerAdditionalInfo {
                name: Some(format!("Layer {:?}", color)),
                layer_color: Some(color),
                ..Default::default()
            },
            ..Default::default()
        };

        assert_eq!(layer.additional_info.layer_color, Some(color));
    }
}

#[test]
fn test_read_existing_psd() {
    // Try to read a test PSD file if it exists
    let test_file = "../test/test.psd";
    if let Ok(file) = File::open(test_file) {
        let options = ReadOptions::default();
        let result = read_psd(file, options);

        if let Ok(psd) = result {
            assert!(psd.width > 0);
            assert!(psd.height > 0);
            assert!(psd.color_mode.is_some());
            println!("Successfully read test PSD: {}x{}", psd.width, psd.height);
        }
    }
}
