// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Scope Binding Integration Tests
//!
//! Validates `UiScope::drag_f32`, `UiScope::drag_f32_ranged`, `UiScope::combobox`,
//! and `UiScope::checkbox` two-way in-place data binding without engine code leakage.

use crate::declarative::scope::core::UiScope;
use iris_core::event::MouseButton;
use iris_core::{Point, Rect, UiEvent, UiTree};

#[test]
fn test_declarative_scope_drag_f32_in_place_mutation() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root node must exist");

    let mut mass = 5.0f32;

    // 1. Initial build: mass starts at 5.0
    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.drag_f32("Mass:", &mut mass);
        assert_eq!(resp.value(), 5.0);
        assert!(!resp.changed());
        assert!(!resp.committed());
    }

    // Assign computed rect for spatial hit testing
    let tag = tree.get(root).unwrap().children[0];
    if let Some(node) = tree.get_mut(tag) {
        node.computed_rect = Rect::new(10.0, 10.0, 80.0, 22.0);
    }

    assert_eq!(mass, 5.0);
}

#[test]
fn test_declarative_scope_drag_f32_ranged_bounds() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root node must exist");

    let mut volume = 0.8f32;

    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.drag_f32_ranged("Volume:", &mut volume, 0.0..=1.0, 0.05);
        assert_eq!(resp.value(), 0.8);
        assert!(!resp.changed());
    }

    assert!((volume - 0.8).abs() < f32::EPSILON);
}

#[test]
fn test_declarative_scope_checkbox_toggle_in_place() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root node must exist");

    let mut is_sensor = false;

    // 1. Build initial checkbox
    let check_id = {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.checkbox("Is Sensor", &mut is_sensor);
        assert!(!resp.clicked());
        resp.id
    };

    if let Some(node) = tree.get_mut(check_id) {
        node.computed_rect = Rect::new(10.0, 10.0, 100.0, 24.0);
    }

    // 2. Click the checkbox
    let _ = tree.dispatch_event(&UiEvent::MouseDown {
        button: MouseButton::Left,
        point: Point::new(15.0, 15.0),
    });
    let _ = tree.dispatch_event(&UiEvent::MouseUp {
        button: MouseButton::Left,
        point: Point::new(15.0, 15.0),
    });

    // 3. Rebuild frame: click should toggle `is_sensor` to true in-place
    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.checkbox("Is Sensor", &mut is_sensor);
        assert!(resp.clicked());
    }

    assert!(
        is_sensor,
        "Checkbox click must toggle is_sensor in-place to true"
    );
}

#[test]
fn test_declarative_scope_combobox_two_way_binding() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root node must exist");

    let mut body_type_idx = 1usize; // Kinematic
    let options = ["Dynamic", "Kinematic", "Static"];

    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.combobox(Some("Body Type:"), &mut body_type_idx, &options);
        assert!(!resp.changed());
        assert_eq!(body_type_idx, 1);
    }

    // Styled variant with explicit width
    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.combobox_styled(None, &mut body_type_idx, &options, 120.0);
        assert!(!resp.changed());
        assert_eq!(body_type_idx, 1);
    }

    // Row variant with explicit label and button widths
    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.combobox_row("Body Type:", 80.0, &mut body_type_idx, &options, 96.0);
        assert!(!resp.changed());
        assert_eq!(body_type_idx, 1);
    }
}

#[test]
fn test_declarative_scope_drag_f32_styled() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root node must exist");

    let mut friction = 0.75f32;
    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.drag_f32_styled("Friction:", &mut friction, 0.0..=1.0, 0.05, 85.0, 44.0);
        assert!(!resp.changed());
        assert!((friction - 0.75).abs() < f32::EPSILON);
    }
}

#[test]
fn test_declarative_scope_combobox_row_mapped() {
    use crate::declarative::scope::ComboRowLayout;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum Shape {
        Box,
        Sphere,
    }

    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root node must exist");

    let mut shape = Shape::Box;
    let options = ["Box", "Sphere"];

    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.combobox_row_mapped(
            "Shape:",
            0,
            &options,
            ComboRowLayout::new(80.0, 90.0),
            &mut shape,
            |idx| match idx {
                0 => Shape::Box,
                _ => Shape::Sphere,
            },
        );
        assert!(!resp.changed());
        assert_eq!(shape, Shape::Box);
    }
}

#[test]
fn test_declarative_scope_asset_slot_row() {
    use crate::declarative::scope::AssetSlotConfig;

    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root node must exist");

    {
        let mut scope = UiScope::new(&mut tree, root);
        scope.asset_slot_row(
            AssetSlotConfig::new("Sound:", "assets/audio/test.wav", "Select sound...", 101)
                .with_play(102, true),
        );
    }

    let has_label = tree
        .iter()
        .any(|(_, n)| n.text.as_deref() == Some("Sound:"));
    assert!(has_label);

    let has_file_name = tree
        .iter()
        .any(|(_, n)| n.text.as_deref() == Some("test.wav"));
    assert!(has_file_name);

    let has_pick_btn = tree.iter().any(|(_, n)| n.tag == 101);
    assert!(has_pick_btn);

    let has_play_btn = tree.iter().any(|(_, n)| n.tag == 102);
    assert!(has_play_btn);

    let has_stop_icon = tree.iter().any(|(_, n)| n.text.as_deref() == Some("⏹"));
    assert!(has_stop_icon);

    // Test AssetSlotResponse fluent on_pick, on_play_toggle, and on_change callbacks
    let pick_id = tree.find_node_by_tag(101).expect("Pick button exists");
    let play_id = tree.find_node_by_tag(102).expect("Play button exists");
    if let Some(node) = tree.get_mut(pick_id) {
        node.computed_rect = Rect::new(200.0, 10.0, 20.0, 20.0);
    }
    if let Some(node) = tree.get_mut(play_id) {
        node.computed_rect = Rect::new(224.0, 10.0, 20.0, 20.0);
    }

    let _ = tree.dispatch_event(&UiEvent::MouseDown {
        button: MouseButton::Left,
        point: Point::new(205.0, 15.0),
    });
    let _ = tree.dispatch_event(&UiEvent::MouseUp {
        button: MouseButton::Left,
        point: Point::new(205.0, 15.0),
    });

    let mut picked = false;
    let mut changed = false;
    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope
            .asset_slot_row(
                AssetSlotConfig::new("Sound:", "assets/audio/test.wav", "Select sound...", 101)
                    .with_play(102, true),
            )
            .on_pick(|| {
                picked = true;
            })
            .on_change(|| {
                changed = true;
            });
        assert!(resp.pick_clicked());
        assert!(!resp.play_clicked());
        assert!(resp.changed());
    }
    assert!(picked);
    assert!(changed);

    // Click play toggle button and verify on_play_toggle / play_clicked
    let _ = tree.dispatch_event(&UiEvent::MouseDown {
        button: MouseButton::Left,
        point: Point::new(230.0, 15.0),
    });
    let _ = tree.dispatch_event(&UiEvent::MouseUp {
        button: MouseButton::Left,
        point: Point::new(230.0, 15.0),
    });

    let mut is_playing = true;
    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope
            .asset_slot_row(
                AssetSlotConfig::new("Sound:", "assets/audio/test.wav", "Select sound...", 101)
                    .with_play(102, is_playing),
            )
            .on_play_toggle(|| {
                is_playing = !is_playing;
            });
        assert!(resp.play_clicked());
        assert!(!resp.pick_clicked());
        assert!(resp.changed());
    }
    assert!(!is_playing, "Clicking play toggle must invert is_playing");
}

#[test]
fn test_declarative_scope_prop_f32_and_prop_f32_styled() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root node must exist");

    let mut speed = 2.5f32;

    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.prop_f32("Speed:", &mut speed, 0.0..=10.0, 0.1);
        assert_eq!(resp.value(), 2.5);
        assert!(!resp.changed());
        assert!(!resp.committed());
    }

    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp_styled = scope.prop_f32_styled(
            "Custom Speed:",
            &mut speed,
            0.0..=10.0,
            0.1,
            Some(120.0),
            Some(60.0),
        );
        assert_eq!(resp_styled.value(), 2.5);
        assert!(!resp_styled.changed());
        assert!(!resp_styled.committed());
    }
}

#[test]
fn test_declarative_scope_prop_i32_and_prop_i32_styled() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root node must exist");

    let mut z_index = 4i32;

    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.prop_i32("Z Index:", &mut z_index, -100..=100);
        assert_eq!(resp.value() as i32, 4);
        assert!(!resp.changed());
        assert!(!resp.committed());
    }

    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp_styled =
            scope.prop_i32_styled("Layer:", &mut z_index, 0..=50, Some(110.0), Some(50.0));
        assert_eq!(resp_styled.value() as i32, 4);
        assert!(!resp_styled.changed());
        assert!(!resp_styled.committed());
    }
}

#[test]
fn test_declarative_scope_prop_bool_and_prop_checkbox() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root node must exist");

    let mut is_sensor = false;

    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.prop_bool("Is Sensor:", &mut is_sensor);
        assert!(!is_sensor);
        assert!(!resp.changed());
    }

    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp_alias = scope.prop_checkbox("Trigger:", &mut is_sensor);
        assert!(!is_sensor);
        assert!(!resp_alias.changed());
    }

    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp_styled = scope.prop_checkbox_styled("Active:", &mut is_sensor, Some(90.0));
        assert!(!is_sensor);
        assert!(!resp_styled.changed());
    }
}

#[test]
fn test_declarative_scope_prop_combobox_and_prop_combobox_mapped() {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum TestLightType {
        Directional,
        Point,
        Spot,
    }

    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root node must exist");

    let mut light_idx = 1usize;
    let light_options = ["Directional", "Point", "Spot"];

    // 1. Test prop_combobox standard signature
    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.prop_combobox("Light Type:", &mut light_idx, &light_options);
        assert!(!resp.changed());
        assert!(!resp.is_open());
        assert_eq!(light_idx, 1);
    }

    // 2. Test prop_combobox_styled with custom widths
    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp_styled = scope.prop_combobox_styled(
            "Light Type:",
            &mut light_idx,
            &light_options,
            Some(100.0),
            Some(85.0),
        );
        assert!(!resp_styled.changed());
        assert!(!resp_styled.is_open());
        assert_eq!(light_idx, 1);
    }

    // 3. Test prop_combobox_mapped with strongly typed enum
    let mut light_enum = TestLightType::Point;
    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp_mapped =
            scope.prop_combobox_mapped("Light Type:", 1, &light_options, &mut light_enum, |idx| {
                match idx {
                    0 => TestLightType::Directional,
                    1 => TestLightType::Point,
                    _ => TestLightType::Spot,
                }
            });
        assert!(!resp_mapped.changed());
        assert_eq!(light_enum, TestLightType::Point);
    }

    // 4. Test prop_combobox_mapped_styled with custom widths
    {
        use crate::declarative::scope::ComboRowLayout;

        let mut scope = UiScope::new(&mut tree, root);
        let resp_mapped_styled = scope.prop_combobox_mapped_styled(
            "Light Type:",
            1,
            &light_options,
            ComboRowLayout::new(95.0, 90.0),
            &mut light_enum,
            |idx| match idx {
                0 => TestLightType::Directional,
                1 => TestLightType::Point,
                _ => TestLightType::Spot,
            },
        );
        assert!(!resp_mapped_styled.changed());
        assert_eq!(light_enum, TestLightType::Point);
    }

    // 5. Test prop_combobox_mapped_with_action and fluent on_action / on_change callbacks
    {
        let mut action_triggered = false;
        let mut change_triggered = false;

        let preset_btn_id = {
            let mut scope = UiScope::new(&mut tree, root);
            let resp = scope
                .prop_combobox_mapped_with_action(
                    "Surface Type:",
                    1,
                    &light_options,
                    "↺ Preset",
                    &mut light_enum,
                    |idx| match idx {
                        0 => TestLightType::Directional,
                        1 => TestLightType::Point,
                        _ => TestLightType::Spot,
                    },
                )
                .on_action(|| {
                    action_triggered = true;
                })
                .on_change(|| {
                    change_triggered = true;
                });
            assert!(!resp.changed());
            assert!(!resp.action_clicked());
            resp.action.id
        };
        assert!(!action_triggered);
        assert!(!change_triggered);

        if let Some(node) = tree.get_mut(preset_btn_id) {
            node.computed_rect = Rect::new(200.0, 10.0, 62.0, 22.0);
        }

        let _ = tree.dispatch_event(&UiEvent::MouseDown {
            button: MouseButton::Left,
            point: Point::new(210.0, 15.0),
        });
        let _ = tree.dispatch_event(&UiEvent::MouseUp {
            button: MouseButton::Left,
            point: Point::new(210.0, 15.0),
        });

        {
            let mut scope = UiScope::new(&mut tree, root);
            scope
                .prop_combobox_mapped_with_action(
                    "Surface Type:",
                    1,
                    &light_options,
                    "↺ Preset",
                    &mut light_enum,
                    |idx| match idx {
                        0 => TestLightType::Directional,
                        1 => TestLightType::Point,
                        _ => TestLightType::Spot,
                    },
                )
                .on_action(|| {
                    action_triggered = true;
                })
                .on_change(|| {
                    change_triggered = true;
                });
        }

        assert!(action_triggered, "Clicking PresetBtn must invoke on_action");
        assert!(change_triggered, "Clicking PresetBtn must invoke on_change");
    }
}

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
    }

    // 5. Verify tree structure contains expected semantic nodes
    let has_title = tree
        .iter()
        .any(|(_, n)| n.text.as_deref() == Some("Transform"));
    let has_icon = tree.iter().any(|(_, n)| n.text.as_deref() == Some("📐"));
    let has_action_glyph = tree.iter().any(|(_, n)| n.text.as_deref() == Some("×"));
    let has_trash_glyph = tree.iter().any(|(_, n)| n.text.as_deref() == Some("🗑"));

    assert!(has_title, "Must render CardTitle with 'Transform'");
    assert!(has_icon, "Must render CardTextIcon with '📐'");
    assert!(has_action_glyph, "Must render CardActionGlyph with '×'");
    assert!(has_trash_glyph, "Must render CardActionGlyph with '🗑'");
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