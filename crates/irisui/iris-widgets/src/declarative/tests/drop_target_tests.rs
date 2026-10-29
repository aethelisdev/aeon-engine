// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use iris_core::{Color, Point, Rect, Style, UiEvent, UiTree};

use crate::declarative::layout_subtree;
use crate::declarative::scope::core::UiScope;
use crate::declarative::widget::drop_target::{DROP_TARGET_SEED, DropTargetId};

#[test]
fn test_drop_target_id_resolution() {
    let name_id: DropTargetId = "AssetLanding".into();
    assert_eq!(name_id.name(), "AssetLanding");
    let tag1 = name_id.resolve_tag();
    assert_ne!(tag1, 0);

    let string_id: DropTargetId = String::from("AssetLanding").into();
    assert_eq!(string_id.resolve_tag(), tag1);

    let tag_id: DropTargetId = 0x1234_5678_u64.into();
    assert_eq!(tag_id.resolve_tag(), 0x1234_5678);
    assert_eq!(tag_id.name(), "DropTarget");

    let u32_id: DropTargetId = 42_u32.into();
    assert_eq!(u32_id.resolve_tag(), 42);
    assert_ne!(DROP_TARGET_SEED, 0);
}

#[test]
fn test_drop_target_hover_detection() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root created");

    // 1. Initial build and layout of the drop target
    {
        let mut scope = UiScope::new(&mut tree, root);
        scope.drop_target_styled(
            "LandingZone",
            Style::new().width(200.0).height(200.0),
            |zone, is_hovered| {
                assert!(!is_hovered, "Must not be hovered on initial render");
                zone.label("Drop assets here");
            },
        );
    }
    layout_subtree(&mut tree, root, Rect::new(0.0, 0.0, 800.0, 600.0));

    let landing_tag = DropTargetId::from("LandingZone").resolve_tag();
    let landing_id = tree
        .find_node_by_tag(landing_tag)
        .expect("Landing zone node exists");
    assert_eq!(
        tree.get(landing_id).unwrap().computed_rect,
        Rect::new(0.0, 0.0, 200.0, 200.0)
    );

    // 2. Dispatch FileHovered inside bounds (100.0, 100.0)
    let hover_inside = UiEvent::FileHovered {
        path: PathBuf::from("/models/character.gltf"),
        point: Point::new(100.0, 100.0),
    };
    tree.dispatch_event(&hover_inside);

    // Rebuild scope and verify is_hovered is true
    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.drop_target_styled(
            "LandingZone",
            Style::new().width(200.0).height(200.0),
            |_zone, is_hovered| {
                assert!(is_hovered, "Must be hovered when file is inside bounds");
            },
        );
        assert!(resp.is_hovered());
    }

    // 3. Dispatch FileHovered outside bounds (450.0, 450.0)
    let hover_outside = UiEvent::FileHovered {
        path: PathBuf::from("/models/character.gltf"),
        point: Point::new(450.0, 450.0),
    };
    tree.dispatch_event(&hover_outside);

    // Rebuild scope and verify is_hovered is false
    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.drop_target_styled(
            "LandingZone",
            Style::new().width(200.0).height(200.0),
            |_zone, is_hovered| {
                assert!(
                    !is_hovered,
                    "Must not be hovered when file is outside bounds"
                );
            },
        );
        assert!(!resp.is_hovered());
    }

    // 4. Dispatch FileCancelled
    tree.dispatch_event(&UiEvent::FileCancelled);
    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.drop_target_styled(
            "LandingZone",
            Style::new().width(200.0).height(200.0),
            |_zone, is_hovered| {
                assert!(!is_hovered, "Must not be hovered after FileCancelled");
            },
        );
        assert!(!resp.is_hovered());
    }
}

#[test]
fn test_drop_target_on_drop_inside_triggers_callback() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root created");

    // 1. Initial build and layout
    {
        let mut scope = UiScope::new(&mut tree, root);
        scope.drop_target_styled(
            "DropBox",
            Style::new().width(300.0).height(200.0),
            |box_scope, _is_hovered| {
                box_scope.label("Target Area");
            },
        );
    }
    layout_subtree(&mut tree, root, Rect::new(0.0, 0.0, 800.0, 600.0));

    // 2. Dispatch FileDropped event directly over the DropBox (150.0, 100.0)
    let test_paths = vec![
        PathBuf::from("/project/assets/wood.png"),
        PathBuf::from("/project/assets/metal.png"),
    ];
    let drop_event = UiEvent::FileDropped {
        paths: test_paths,
        point: Point::new(150.0, 100.0),
    };
    let dispatch_res = tree.dispatch_event(&drop_event);
    assert!(
        dispatch_res.consumed,
        "Drop event inside DropBox must be consumed"
    );

    // 3. Rebuild UI and evaluate on_drop callback
    static DROP_RECEIVED: AtomicBool = AtomicBool::new(false);
    static DROPPED_COUNT: AtomicUsize = AtomicUsize::new(0);

    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope
            .drop_target_styled(
                "DropBox",
                Style::new().width(300.0).height(200.0),
                |box_scope, _is_hovered| {
                    box_scope.label("Target Area");
                },
            )
            .on_drop(|paths| {
                DROP_RECEIVED.store(true, Ordering::SeqCst);
                DROPPED_COUNT.store(paths.len(), Ordering::SeqCst);
                assert_eq!(paths[0], PathBuf::from("/project/assets/wood.png"));
                assert_eq!(paths[1], PathBuf::from("/project/assets/metal.png"));
            });

        assert!(resp.response.file_dropped());
        assert_eq!(resp.dropped_paths().map(|p| p.len()), Some(2));
    }

    assert!(
        DROP_RECEIVED.load(Ordering::SeqCst),
        "on_drop closure must be executed"
    );
    assert_eq!(
        DROPPED_COUNT.load(Ordering::SeqCst),
        2,
        "All dropped paths must be received"
    );
}

#[test]
fn test_drop_target_on_drop_outside_does_not_trigger_callback() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root created");

    // 1. Initial build and layout
    {
        let mut scope = UiScope::new(&mut tree, root);
        scope.drop_target_styled(
            "DropBoxIsolated",
            Style::new().width(200.0).height(100.0),
            |box_scope, _is_hovered| {
                box_scope.label("Isolated Target");
            },
        );
    }
    layout_subtree(&mut tree, root, Rect::new(0.0, 0.0, 800.0, 600.0));

    // 2. Dispatch FileDropped event OUTSIDE DropBox (600.0, 400.0)
    let drop_event = UiEvent::FileDropped {
        paths: vec![PathBuf::from("/project/assets/sky.png")],
        point: Point::new(600.0, 400.0),
    };
    tree.dispatch_event(&drop_event);

    // 3. Rebuild UI and verify on_drop callback is NOT triggered
    static DROP_SHOULD_BE_FALSE: AtomicBool = AtomicBool::new(false);

    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope
            .drop_target_styled(
                "DropBoxIsolated",
                Style::new().width(200.0).height(100.0),
                |box_scope, _is_hovered| {
                    box_scope.label("Isolated Target");
                },
            )
            .on_drop(|_| {
                DROP_SHOULD_BE_FALSE.store(true, Ordering::SeqCst);
            });

        assert!(!resp.response.file_dropped());
        assert_eq!(resp.dropped_paths(), None);
    }

    assert!(
        !DROP_SHOULD_BE_FALSE.load(Ordering::SeqCst),
        "on_drop closure must NOT execute when dropped outside bounds"
    );
}

#[test]
fn test_drop_target_nested_child_hit_bubbles_to_target() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root created");

    // 1. Drop target with an interactive nested child button
    {
        let mut scope = UiScope::new(&mut tree, root);
        scope.drop_target_styled(
            "ParentDropZone",
            Style::new().width(400.0).height(300.0),
            |zone, is_hovered| {
                let border_col = if is_hovered {
                    Color::hex("#007ACC")
                } else {
                    Color::TRANSPARENT
                };
                zone.container(Style::new().border(2.0, border_col), |nested| {
                    let _ = nested.button("Upload");
                });
            },
        );
    }
    layout_subtree(&mut tree, root, Rect::new(0.0, 0.0, 800.0, 600.0));

    // 2. Drop file directly on the inner child
    let drop_event = UiEvent::FileDropped {
        paths: vec![PathBuf::from("/models/hero.fbx")],
        point: Point::new(50.0, 50.0),
    };
    tree.dispatch_event(&drop_event);

    // 3. Rebuild and verify the parent drop target caught the drop
    let mut caught_nested = false;
    {
        let mut scope = UiScope::new(&mut tree, root);
        let _ = scope
            .drop_target("ParentDropZone", |zone, _is_hovered| {
                zone.button("Upload");
            })
            .on_drop(|paths| {
                assert_eq!(paths.len(), 1);
                assert_eq!(paths[0], PathBuf::from("/models/hero.fbx"));
                caught_nested = true;
            });
    }

    assert!(
        caught_nested,
        "Parent drop target must catch drops delivered to child descendants"
    );
}