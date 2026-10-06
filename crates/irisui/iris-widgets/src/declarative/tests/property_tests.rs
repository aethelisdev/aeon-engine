// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Scope Property & ID Isolation Tests
//!
//! Verifies two-way data-bound property primitives, hierarchical [`UiScope::with_id`]
//! tag isolation, and `##` label un-iquing without heap allocations.
//!

use crate::declarative::types::{hash_label, split_label_id};
use crate::declarative::{UiScope, hash_label_with_seed};
use iris_core::{InteractionEvent, MouseButton, Point, UiTree};

#[test]
fn test_property_primitives_two_way_binding() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root should be created");

    let tag_vsync = hash_label("VSync");
    let tag_shadow = hash_label("Shadow Distance");
    let tag_pos = hash_label("Position");
    let tag_pos_x = tag_pos.wrapping_add(1);

    // Simulate Click on VSync checkbox, Drag on Shadow Distance slider, Drag on Position X
    let events = [
        (
            tag_vsync,
            InteractionEvent::Click {
                button: MouseButton::Left,
            },
        ),
        (
            tag_shadow,
            InteractionEvent::Drag {
                delta: Point::new(10.0, 0.0),
            },
        ),
        (
            tag_pos_x,
            InteractionEvent::Drag {
                delta: Point::new(5.0, 0.0),
            },
        ),
    ];

    let mut vsync = false;
    let mut shadow_dist = 50.0_f32;
    let mut position = [0.0_f32, 1.0, 2.0];

    {
        let mut scope =
            UiScope::with_tagged_interactions(&mut tree, root, &events, Some(tag_vsync));

        let _ = scope.property_section("Graphics Settings");

        let res_check = scope.property_checkbox("VSync", &mut vsync);
        assert!(res_check.clicked());
        assert!(res_check.changed());
        assert!(vsync, "VSync boolean should be toggled to true in-place!");

        let res_slider =
            scope.property_slider("Shadow Distance", &mut shadow_dist, 0.0, 100.0, 1.0);
        assert!(res_slider.changed());
        assert_eq!(
            shadow_dist, 60.0,
            "Shadow distance should be increased by 10.0 * 1.0 = 60.0!"
        );

        let res_vec = scope.property_vec3("Position", &mut position, 0.5);
        assert!(res_vec.changed());
        assert_eq!(
            position[0], 2.5,
            "Position X should be increased by 5.0 * 0.5 = 2.5!"
        );
        assert_eq!(position[1], 1.0);
        assert_eq!(position[2], 2.0);

        let res_dropdown = scope.property_dropdown("Texture Quality", "High");
        assert!(!res_dropdown.clicked());
    }
}

#[test]
fn test_declarative_scope_with_id_tag_isolation() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root should be created");

    // Calculate isolated tags manually to verify against UiScope output
    let tag_t_offset = hash_label_with_seed(hash_label("Transform"), "Offset");
    let tag_c_offset = hash_label_with_seed(hash_label("BoxCollider"), "Offset");

    assert_ne!(
        tag_t_offset, tag_c_offset,
        "Isolated scope tags for identical labels must be strictly different!"
    );
    assert_ne!(
        tag_t_offset,
        hash_label("Offset"),
        "Seeded tag must not collide with raw global tag!"
    );

    // Frame 1: Send click only to the BoxCollider Offset tag
    let events = [(
        tag_c_offset,
        InteractionEvent::Click {
            button: MouseButton::Left,
        },
    )];

    let mut t_offset = false;
    let mut c_offset = false;

    {
        let mut scope = UiScope::with_tagged_interactions(&mut tree, root, &events, None);

        // Transform card scope
        scope.with_id("Transform", |s| {
            let res = s.property_checkbox("Offset", &mut t_offset);
            assert!(!res.clicked(), "Transform Offset must not receive click!");
            assert!(!res.changed());
        });

        // BoxCollider card scope
        scope.with_id("BoxCollider", |s| {
            let res = s.property_checkbox("Offset", &mut c_offset);
            assert!(
                res.clicked(),
                "BoxCollider Offset must receive the routed click!"
            );
            assert!(res.changed());
        });
    }

    assert!(
        !t_offset,
        "Transform Offset boolean must remain false (unmodified)!"
    );
    assert!(
        c_offset,
        "BoxCollider Offset boolean must be toggled to true!"
    );
}

#[test]
fn test_declarative_split_label_id_rendering() {
    let (vis1, full1) = split_label_id("Offset##collider");
    assert_eq!(vis1, "Offset");
    assert_eq!(full1, "Offset##collider");

    let (vis2, full2) = split_label_id("##hidden");
    assert_eq!(vis2, "");
    assert_eq!(full2, "##hidden");

    let (vis3, full3) = split_label_id("PlainLabel");
    assert_eq!(vis3, "PlainLabel");
    assert_eq!(full3, "PlainLabel");

    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root should be created");

    let tag_entity1 = hash_label("Scale##entity_1");
    let tag_entity2 = hash_label("Scale##entity_2");
    assert_ne!(
        tag_entity1, tag_entity2,
        "Labels with different ## suffixes must produce different tags!"
    );

    let mut scale1 = false;
    let mut scale2 = false;

    {
        let mut scope = UiScope::new(&mut tree, root);
        let res1 = scope.property_checkbox("Scale##entity_1", &mut scale1);
        let res2 = scope.property_checkbox("Scale##entity_2", &mut scale2);

        assert_ne!(res1.id, res2.id);
    }

    // Inspect the generated text node inside property rows
    // Find all nodes named "PropertyLabel" and verify their text is "Scale" (no ## suffix)
    let mut label_texts = Vec::new();
    for (_id, node) in tree.iter() {
        if node.name.as_deref() == Some("PropertyLabel") {
            label_texts.push(node.text.clone());
        }
    }

    assert_eq!(label_texts.len(), 2);
    assert_eq!(label_texts[0].as_deref(), Some("Scale"));
    assert_eq!(label_texts[1].as_deref(), Some("Scale"));
}

#[test]
fn test_nested_with_id_hierarchy() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root should be created");

    let mut tag_e1 = 0u64;
    let mut tag_e2 = 0u64;

    {
        let mut scope = UiScope::new(&mut tree, root);

        scope.with_id(1001u64, |s1| {
            s1.with_id("Transform", |s2| {
                tag_e1 = s2.tag_for("Position");
            });
        });

        scope.with_id(1002u64, |s1| {
            s1.with_id("Transform", |s2| {
                tag_e2 = s2.tag_for("Position");
            });
        });
    }

    assert_ne!(
        tag_e1, tag_e2,
        "Nested seeds across entities must produce completely distinct tags!"
    );
}

#[test]
fn test_property_slider_rich_track_and_options() {
    use crate::declarative::scope::properties::PropertySliderOptions;

    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root should be created");

    let tag_phys = hash_label("Fixed Update Frequency");
    let tag_num_box = crate::declarative::types::hash_label_with_seed(tag_phys, "##num_box");

    // Simulate TextInput on the number box tag directly
    let events = [(
        tag_num_box,
        InteractionEvent::TextInput {
            text: "144".to_string(),
        },
    )];

    let mut hz = 60.0_f32;

    {
        let mut scope = UiScope::with_tagged_interactions(&mut tree, root, &events, Some(tag_phys));

        let opts = PropertySliderOptions::new(30.0, 240.0, 1.0)
            .with_format("{:.0} Hz")
            .with_subtitle("Physics simulation frequency in Hertz.");

        let res = scope.property_slider_with_options("Fixed Update Frequency", &mut hz, opts);
        assert!(
            res.changed(),
            "Direct text input must mutate value in-place"
        );
        assert_eq!(hz, 144.0, "Frequency must be updated to 144.0 Hz");
    }

    // Verify visual tree structure: Track, Fill, and Subtitle exist
    let track_node = tree
        .iter()
        .find(|(_, n)| n.name.as_deref() == Some("SliderTrack"));
    assert!(track_node.is_some(), "Slider track must be present in tree");

    let fill_node = tree
        .iter()
        .find(|(_, n)| n.name.as_deref() == Some("SliderFill"));
    assert!(
        fill_node.is_some(),
        "Slider fill bar must be present in tree"
    );

    let sub_node = tree
        .iter()
        .find(|(_, n)| n.name.as_deref() == Some("SliderSubtitle"));
    assert!(
        sub_node.is_some(),
        "Slider subtitle must be present in tree"
    );
}