// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Unit tests for declarative scope data binding properties.

use crate::declarative::UiScope;
use crate::prelude::CardHeaderDescriptor;
use iris_core::UiTree;

#[test]
fn test_declarative_scope_prop_vec2_and_variants() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root node must exist");

    let mut uv = [1.0f32, 2.0f32];
    let mut offset = [10.0f32, -5.0f32];
    let mut pos = [0.0f32, 0.0f32];

    // 1. Standard prop_vec2 with custom channel labels ["U", "V"]
    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.prop_vec2("UV Tiling:", &mut uv, ["U", "V"], 0.05);
        assert!(!resp.changed());
        assert!(!resp.committed());
        assert_eq!(uv, [1.0, 2.0]);
    }

    // 2. Styled prop_vec2 with custom label width and range
    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp_styled = scope.prop_vec2_styled(
            "Offset:",
            &mut offset,
            ["X", "Y"],
            0.1,
            Some(90.0),
            Some(-100.0..=100.0),
        );
        assert!(!resp_styled.changed());
        assert!(!resp_styled.committed());
        assert_eq!(offset, [10.0, -5.0]);
    }

    // 3. Coordinate shorthand prop_vec2_xy
    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp_xy = scope.prop_vec2_xy("Position:", &mut pos, 0.1);
        assert!(!resp_xy.changed());
        assert!(!resp_xy.committed());
        assert_eq!(pos, [0.0, 0.0]);
    }

    // 4. Verify tree structure contains both channel labels
    let has_u = tree.iter().any(|(_, n)| {
        n.text
            .as_deref()
            .map(|t| t.starts_with("U:"))
            .unwrap_or(false)
    });
    let has_v = tree.iter().any(|(_, n)| {
        n.text
            .as_deref()
            .map(|t| t.starts_with("V:"))
            .unwrap_or(false)
    });
    assert!(has_u, "Must render U channel axis pill");
    assert!(has_v, "Must render V channel axis pill");
}

#[test]
fn test_declarative_scope_prop_vec3_and_variants() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root node must exist");

    let mut scale = [1.0f32, 1.0f32, 1.0f32];
    let mut translation = [10.0f32, 20.0f32, -5.0f32];

    // 1. Standard prop_vec3 with reset button support
    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.prop_vec3("Scale:", &mut scale, 0.05, Some([1.0, 1.0, 1.0]));
        assert!(!resp.changed());
        assert!(!resp.committed());
        assert!(
            resp.reset_btn.is_some(),
            "Reset button must be emitted when reset_value is provided"
        );
        assert_eq!(scale, [1.0, 1.0, 1.0]);
    }

    // 2. Styled prop_vec3 with custom label width and without reset button
    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp_styled =
            scope.prop_vec3_styled("Translation:", &mut translation, 0.1, None, Some(90.0));
        assert!(!resp_styled.changed());
        assert!(!resp_styled.committed());
        assert!(
            resp_styled.reset_btn.is_none(),
            "Reset button must be omitted when reset_value is None"
        );
        assert_eq!(translation, [10.0, 20.0, -5.0]);
    }

    // 3. Verify tree structure contains X, Y, Z labels and reset button
    let has_x = tree.iter().any(|(_, n)| {
        n.text
            .as_deref()
            .map(|t| t.starts_with("X:"))
            .unwrap_or(false)
    });
    let has_y = tree.iter().any(|(_, n)| {
        n.text
            .as_deref()
            .map(|t| t.starts_with("Y:"))
            .unwrap_or(false)
    });
    let has_z = tree.iter().any(|(_, n)| {
        n.text
            .as_deref()
            .map(|t| t.starts_with("Z:"))
            .unwrap_or(false)
    });
    let has_reset_icon = tree.iter().any(|(_, n)| n.text.as_deref() == Some("🔄"));

    assert!(has_x, "Must render X channel axis pill");
    assert!(has_y, "Must render Y channel axis pill");
    assert!(has_z, "Must render Z channel axis pill");
    assert!(has_reset_icon, "Must render reset button icon");
}

#[test]
fn test_declarative_scope_prop_color_and_variants() {
    use iris_core::Color;

    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root node must exist");

    let mut col = Color::rgba(1.0, 0.0, 0.0, 1.0); // #FF0000

    // 1. Standard prop_color
    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.prop_color("Tint:", &mut col);
        assert!(!resp.changed());
        assert!(!resp.committed());
        assert!(!resp.is_open());
        assert_eq!(resp.color(), Color::rgba(1.0, 0.0, 0.0, 1.0));
    }

    // 2. Styled prop_color
    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp_styled = scope.prop_color_styled("Emissive:", &mut col, Some(90.0), Some(80.0));
        assert!(!resp_styled.changed());
        assert!(!resp_styled.committed());
        assert!(!resp_styled.is_open());
    }

    // 3. Verify tree structure contains hex text and swatch pill
    let has_hex = tree
        .iter()
        .any(|(_, n)| n.text.as_deref() == Some("#FF0000"));
    assert!(
        has_hex,
        "Must render hex label #FF0000 inside color swatch pill"
    );
}

#[test]
fn test_declarative_scope_card_and_variants() {
    use iris_core::Color;

    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root node must exist");

    // 1. Standard card with title, icon, and accent color
    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.card("Transform", Some("📐"), Some(Color::CYAN), |card| {
            card.label("Position");
            42
        });
        assert_eq!(*resp.inner(), 42);
        assert_eq!(resp.into_inner(), 42);
    }

    // 2. Card with action button (e.g. "×" delete action) and .on_action fluent chaining
    let mut action_triggered = false;
    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope
            .card_with_action("Rigid Body", Some("📦"), None, "×", |card| {
                card.label("Mass");
                "body_content"
            })
            .on_action(|| {
                action_triggered = true;
            });
        assert_eq!(resp.inner, "body_content");
        assert!(!action_triggered, "Action was not clicked in this frame");
        assert!(!resp.action_clicked());
    }

    // 3. CardBuilder with fluent configuration
    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope
            .card_builder("Appearance")
            .icon("🎨")
            .accent(Color::YELLOW)
            .action("🗑")
            .border_radius(8.0)
            .show(|card| {
                card.label("Tint");
                999
            });
        assert_eq!(resp.inner, 999);
        assert!(!resp.action_clicked());
    }

    // 4. Standalone card headers
    {
        let mut scope = UiScope::new(&mut tree, root);
        let header_resp = scope.card_header("Simple Header", Some("💡"), None);
        assert!(!header_resp.clicked());

        let action_header_resp =
            scope.card_header_with_action("Action Header", Some("⚙"), None, "×");
        assert!(!action_header_resp.clicked());
        assert!(!action_header_resp.action_clicked());

        let desc = CardHeaderDescriptor::new("Advanced Header")
            .icon("📦")
            .atlas_icon([0.1, 0.2, 0.3, 0.0])
            .color(Color::rgba(0.2, 0.8, 0.4, 1.0))
            .action("×", 0xCAFE_BABE);
        let desc_resp = scope.card_header_desc(desc);
        assert!(!desc_resp.clicked());
        assert!(!desc_resp.action_clicked());
    }

    // 5. Verify tree structure contains expected semantic nodes
    let has_title = tree
        .iter()
        .any(|(_, n)| n.text.as_deref() == Some("Transform"));
    let has_icon = tree.iter().any(|(_, n)| n.text.as_deref() == Some("📐"));
    let has_action_glyph = tree.iter().any(|(_, n)| n.text.as_deref() == Some("×"));
    let has_trash_glyph = tree.iter().any(|(_, n)| n.text.as_deref() == Some("🗑"));
    let has_atlas_icon = tree
        .iter()
        .any(|(_, n)| n.name.as_deref() == Some("CardAtlasIcon"));

    assert!(has_title, "Must render CardTitle with 'Transform'");
    assert!(has_icon, "Must render CardTextIcon with '📐'");
    assert!(has_action_glyph, "Must render CardActionGlyph with '×'");
    assert!(has_trash_glyph, "Must render CardActionGlyph with '🗑'");
    assert!(has_atlas_icon, "Must render CardAtlasIcon with coordinates");
}

/// Comprehensive integration test sealing Section I (Phases 1-10): verifies all declarative
/// property methods (`prop_text`, `prop_f32`, `prop_i32`, `prop_bool`, `prop_combobox`,
/// `prop_vec2`, `prop_vec3`, `prop_color`) composed  inside `scope.card_with_action`.
///
/// Ensures 100% compliance with Rule 1.6 (zero intermediate `let` variable statements inside the view).
#[test]
fn test_declarative_scope_full_inspector_card_composition_sealing() {
    use iris_core::Color;

    struct MockComponent {
        name: String,
        mass: f32,
        layer: i32,
        is_sensor: bool,
        body_type: usize,
        uv_scale: [f32; 2],
        position: [f32; 3],
        tint: Color,
    }

    let mut comp = MockComponent {
        name: "PlayerRigidBody".to_string(),
        mass: 75.5,
        layer: 3,
        is_sensor: false,
        body_type: 1,
        uv_scale: [1.0, 1.0],
        position: [0.0, 1.8, -5.0],
        tint: Color::rgba(0.2, 0.7, 1.0, 1.0),
    };

    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root node must exist");

    let mut action_executed = false;

    // Execute declarative composition: notice ZERO intermediate `let` statements inside the card body!
    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope
            .card_with_action(
                "RigidBody Component",
                Some("📦"),
                Some(Color::CYAN),
                "×",
                |card| {
                    card.prop_text("Name:", &mut comp.name);
                    card.prop_f32("Mass:", &mut comp.mass, 0.01..=500.0, 0.1);
                    card.prop_i32("Layer:", &mut comp.layer, 0..=32);
                    card.prop_bool("Is Sensor:", &mut comp.is_sensor);
                    card.prop_combobox(
                        "Body Type:",
                        &mut comp.body_type,
                        &["Dynamic", "Kinematic", "Fixed"],
                    );
                    card.prop_vec2("UV Scale:", &mut comp.uv_scale, ["U", "V"], 0.05);
                    card.prop_vec3("Position:", &mut comp.position, 0.1, Some([0.0, 0.0, 0.0]));
                    card.prop_color("Tint:", &mut comp.tint);
                },
            )
            .on_action(|| {
                action_executed = true;
            });

        assert!(!resp.header_response.clicked());
        assert!(!resp.action_clicked());
        assert!(!action_executed);
    }

    // Verify all properties retain valid data
    assert_eq!(comp.name, "PlayerRigidBody");
    assert_eq!(comp.mass, 75.5);
    assert_eq!(comp.layer, 3);
    assert!(!comp.is_sensor);
    assert_eq!(comp.body_type, 1);
    assert_eq!(comp.uv_scale, [1.0, 1.0]);
    assert_eq!(comp.position, [0.0, 1.8, -5.0]);
    assert_eq!(comp.tint, Color::rgba(0.2, 0.7, 1.0, 1.0));

    // Verify semantic tree nodes are correctly generated and present
    let has_header = tree
        .iter()
        .any(|(_, n)| n.text.as_deref() == Some("RigidBody Component"));
    let has_box_icon = tree.iter().any(|(_, n)| n.text.as_deref() == Some("📦"));
    let has_action_del = tree.iter().any(|(_, n)| n.text.as_deref() == Some("×"));
    let has_name_label = tree
        .iter()
        .any(|(_, n)| n.text.as_deref().is_some_and(|t| t.starts_with("Name:")));
    let has_mass_label = tree
        .iter()
        .any(|(_, n)| n.text.as_deref().is_some_and(|t| t.starts_with("Mass:")));
    let has_pos_label = tree.iter().any(|(_, n)| {
        n.text
            .as_deref()
            .is_some_and(|t| t.starts_with("Position:"))
    });

    assert!(has_header, "Header title must exist in tree");
    assert!(has_box_icon, "Icon must exist in tree");
    assert!(has_action_del, "Action delete glyph must exist in tree");
    assert!(has_name_label, "Name label row must exist in tree");
    assert!(has_mass_label, "Mass label row must exist in tree");
    assert!(has_pos_label, "Position label row must exist in tree");
}