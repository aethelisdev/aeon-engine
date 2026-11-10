// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Unit tests for declarative scope data binding properties.

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

    // 1. Initial render: unchecked
    let tag = {
        let mut scope = UiScope::new(&mut tree, root);
        let tag = scope.tag_for("Is Sensor:");
        let resp = scope.prop_bool("Is Sensor:", &mut is_sensor);
        assert!(!is_sensor);
        assert!(!resp.changed());
        assert!(!resp.clicked());
        tag
    };

    // 2. Simulated mouse click via event state tag matching
    tree.event_state.frame_clicked_tag = Some(tag);
    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.prop_checkbox("Is Sensor:", &mut is_sensor);
        assert!(
            is_sensor,
            "Checkbox click must toggle is_sensor in-place to true"
        );
        assert!(
            resp.clicked(),
            "WidgetResponse::clicked must be true on click"
        );
        assert!(
            resp.changed(),
            "WidgetResponse::changed must be true on click"
        );
    }

    // 3. Next frame (unclicked): must preserve true and render checkmark
    tree.event_state.frame_clicked_tag = None;
    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.prop_checkbox("Is Sensor:", &mut is_sensor);
        assert!(
            is_sensor,
            "is_sensor must stay true across subsequent frames"
        );
        assert!(!resp.clicked());
        assert!(!resp.changed());
    }

    // 4. Toggle back to false
    tree.event_state.frame_clicked_tag = Some(tag);
    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.prop_checkbox_styled("Is Sensor:", &mut is_sensor, Some(90.0));
        assert!(
            !is_sensor,
            "Second click must toggle is_sensor back to false"
        );
        assert!(resp.clicked());
        assert!(resp.changed());
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