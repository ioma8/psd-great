use ag_psd::*;

#[test]
fn test_basic_types() {
    // Test color types
    let rgba = RGBA { r: 255, g: 128, b: 64, a: 255 };
    assert_eq!(rgba.r, 255);
    
    let rgb = RGB { r: 255, g: 128, b: 64 };
    assert_eq!(rgb.g, 128);
    
    let cmyk = CMYK { c: 100, m: 50, y: 25, k: 0 };
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
        clipping: Some(false),
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
    };
    
    assert_eq!(psd.width, 1920);
    assert_eq!(psd.height, 1080);
    assert_eq!(psd.color_mode, Some(ColorMode::RGB));
}

#[test]
fn test_serialization() {
    let rgba = RGBA { r: 255, g: 128, b: 64, a: 255 };
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
        color: Some(Color::RGBA(RGBA { r: 0, g: 0, b: 0, a: 255 })),
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
