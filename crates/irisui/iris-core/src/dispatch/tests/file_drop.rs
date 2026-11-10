// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Unit tests for OS file drag-and-drop event dispatching and target boundary resolution.

use std::path::PathBuf;

use crate::event::UiEvent;
use crate::geometry::{Point, Rect};
use crate::tree::UiTree;

#[test]
fn test_dispatch_os_file_events() {
    let mut tree = UiTree::new();
    let root = tree.create_root().unwrap();
    {
        let node = tree.get_mut(root).unwrap();
        node.visible = true;
        node.interactive = true;
        node.computed_rect = Rect::new(0.0, 0.0, 500.0, 500.0);
    }

    let file_path = PathBuf::from("/project/assets/mesh.gltf");
    let hover_pt = Point::new(100.0, 100.0);

    // 1. FileHovered updates hover state to root widget
    let hover_res = tree.dispatch_event(&UiEvent::FileHovered {
        path: file_path.clone(),
        point: hover_pt,
    });
    assert_eq!(hover_res.target, Some(root));
    assert_eq!(tree.event_state.hovered, Some(root));
    assert_eq!(
        tree.event_state.hovered_file,
        Some((file_path.clone(), hover_pt))
    );

    // Consecutive FileHovered with same path updates point in-place without reallocating
    let hover_pt2 = Point::new(120.0, 140.0);
    tree.dispatch_event(&UiEvent::FileHovered {
        path: file_path.clone(),
        point: hover_pt2,
    });
    assert_eq!(
        tree.event_state.hovered_file,
        Some((file_path.clone(), hover_pt2))
    );

    // 2. FileDropped onto root widget is consumed
    let drop_res = tree.dispatch_event(&UiEvent::FileDropped {
        paths: vec![file_path.clone()],
        point: hover_pt,
    });
    assert!(drop_res.consumed);
    assert_eq!(drop_res.target, Some(root));

    // Dropped outside is unconsumed
    let outside_res = tree.dispatch_event(&UiEvent::FileDropped {
        paths: vec![file_path],
        point: Point::new(600.0, 600.0),
    });
    assert!(!outside_res.consumed);
    assert_eq!(outside_res.target, None);

    // 3. FileCancelled does not panic and returns unconsumed result
    let cancel_res = tree.dispatch_event(&UiEvent::FileCancelled);
    assert!(!cancel_res.consumed);
}

#[test]
fn test_file_drag_drop_coordinate_resolution_and_helpers() {
    let pt = Point::new(120.0, 80.0);
    let path = std::path::PathBuf::from("/assets/test.glb");

    // 1. Test UiEvent helper methods
    let move_ev = UiEvent::MouseMove { point: pt };
    assert_eq!(move_ev.pointer_position(), Some(pt));
    assert!(!move_ev.is_file_drag_event());

    let hover_ev = UiEvent::FileHovered {
        path: path.clone(),
        point: pt,
    };
    assert_eq!(hover_ev.pointer_position(), Some(pt));
    assert!(hover_ev.is_file_drag_event());
    assert_eq!(hover_ev.file_hover_payload(), Some((path.as_path(), pt)));
    assert_eq!(hover_ev.file_drop_payload(), None);

    let drop_ev = UiEvent::FileDropped {
        paths: vec![path.clone()],
        point: pt,
    };
    assert_eq!(drop_ev.pointer_position(), Some(pt));
    assert!(drop_ev.is_file_drag_event());
    assert_eq!(
        drop_ev.file_drop_payload(),
        Some((vec![path.clone()].as_slice(), pt))
    );

    let cancel_ev = UiEvent::FileCancelled;
    assert_eq!(cancel_ev.pointer_position(), None);
    assert!(cancel_ev.is_file_drag_event());

    // 2. Test UiTree file drop target resolution
    let mut tree = UiTree::new();
    let root = tree.create_root().unwrap();
    tree.get_mut(root).unwrap().computed_rect = Rect::new(0.0, 0.0, 500.0, 500.0);

    let panel = tree.create_node();
    let panel_node = tree.get_mut(panel).unwrap();
    panel_node.computed_rect = Rect::new(50.0, 50.0, 200.0, 200.0);
    panel_node.tag = 0xAA01;
    panel_node.is_boundary = true;
    tree.add_child(root, panel).unwrap();

    let target = tree.resolve_file_drop_target(pt);
    assert!(target.is_some());
    let t = target.unwrap();
    assert_eq!(t.widget_id, panel);
    assert_eq!(t.tag, 0xAA01);
    assert_eq!(t.boundary_id, Some(panel));

    // Dispatch hover event & query hovered_file
    tree.dispatch_event(&hover_ev);
    let (h_path, h_pt, h_target) = tree.hovered_file().unwrap();
    assert_eq!(h_path, path.as_path());
    assert_eq!(h_pt, pt);
    assert_eq!(h_target.unwrap().widget_id, panel);

    // Dispatch drop event & query frame_dropped_files
    tree.dispatch_event(&drop_ev);
    assert!(tree.hovered_file().is_none());
    let (d_paths, d_pt, d_target) = tree.frame_dropped_files().unwrap();
    assert_eq!(d_paths, &[path]);
    assert_eq!(d_pt, pt);
    assert_eq!(d_target.unwrap().widget_id, panel);
}