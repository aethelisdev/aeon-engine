// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Color Picker Unit Tests
//!
//! Validates 2D HSV color picker layout tree emission, pointer-capture dragging across
//! Saturation-Value space and Rainbow Hue spectrum, and commit triggers.
//!

use iris_core::geometry::{Point, Rect};
use iris_core::node::WidgetRole;
use iris_core::tree::UiTree;

use crate::color_picker::HsvColorPickerState;
use crate::declarative::scope::UiScope;
use crate::declarative::types::combine_seeds;
use crate::declarative::widget::color_picker::ColorPicker;

#[test]
fn test_color_picker_initial_emission() {
    let mut tree = UiTree::new();
    let root = tree.create_node();
    let mut scope = UiScope::new(&mut tree, root);

    let mut state = HsvColorPickerState {
        hue: 200.0,
        saturation: 0.6,
        value: 0.8,
        alpha: 1.0,
    };

    let base_tag = 0xC010_0001;
    let resp = scope.add(ColorPicker::new(&mut state).tag(base_tag));

    assert!(!resp.changed());
    assert!(!resp.drag_released());
    assert_eq!(resp.state().hue, 200.0);

    // Verify SV and Hue child nodes are emitted with WidgetRole::ColorPickerArea
    let sv_tag = combine_seeds(base_tag, 1);
    let hue_tag = combine_seeds(base_tag, 2);

    let sv_id = tree.find_node_by_tag(sv_tag).expect("SV Box must exist");
    let hue_id = tree.find_node_by_tag(hue_tag).expect("Hue Bar must exist");

    assert_eq!(tree.get(sv_id).unwrap().role, WidgetRole::ColorPickerArea);
    assert_eq!(tree.get(hue_id).unwrap().role, WidgetRole::ColorPickerArea);
}

#[test]
fn test_color_picker_sv_drag_interaction() {
    let mut tree = UiTree::new();
    let root = tree.create_node();

    let mut state = HsvColorPickerState {
        hue: 120.0,
        saturation: 0.0,
        value: 0.0,
        alpha: 1.0,
    };

    let base_tag = 0xC010_0002;
    let sv_tag = combine_seeds(base_tag, 1);

    // Layout first pass
    {
        let mut scope = UiScope::new(&mut tree, root);
        let _ = scope.add(ColorPicker::new(&mut state).tag(base_tag));
    }

    let sv_id = tree.find_node_by_tag(sv_tag).expect("SV Box node exists");
    if let Some(node) = tree.get_mut(sv_id) {
        node.computed_rect = Rect::new(10.0, 10.0, 160.0, 130.0);
    }

    // Simulate drag interaction in the center of the SV box (rel_x = 80.0, rel_y = 65.0)
    tree.event_state.pointer_capture = Some(sv_id);
    tree.event_state.pointer_capture_tag = Some(sv_tag);
    tree.event_state.last_pointer_pos = Some(Point::new(80.0, 65.0));

    // Next frame declarative pass on retained tree
    {
        let mut scope2 = UiScope::new(&mut tree, root);
        let resp = scope2.add(ColorPicker::new(&mut state).tag(base_tag));

        // Center relative to (0..160, 0..130):
        // rel_x = 80 / 160 = 0.5 saturation
        // rel_y = 65 / 130 = 0.5 => value = 1.0 - 0.5 = 0.5
        assert!(resp.changed());
        assert!((state.saturation - 0.5).abs() < 0.05);
        assert!((state.value - 0.5).abs() < 0.05);
    }
}

#[test]
fn test_color_picker_hue_drag_interaction() {
    let mut tree = UiTree::new();
    let root = tree.create_node();

    let mut state = HsvColorPickerState {
        hue: 0.0,
        saturation: 1.0,
        value: 1.0,
        alpha: 1.0,
    };

    let base_tag = 0xC010_0003;
    let hue_tag = combine_seeds(base_tag, 2);

    // First pass to emit tree
    {
        let mut scope = UiScope::new(&mut tree, root);
        let _ = scope.add(ColorPicker::new(&mut state).tag(base_tag));
    }

    let hue_id = tree.find_node_by_tag(hue_tag).expect("Hue Bar exists");

    // Drag pointer at 3/4 along the hue bar height (y = 97.5 / 130.0 = 0.75 * 360.0 = 270.0)
    tree.event_state.pointer_capture = Some(hue_id);
    tree.event_state.pointer_capture_tag = Some(hue_tag);
    tree.event_state.last_pointer_pos = Some(Point::new(9.0, 97.5));

    {
        let mut scope2 = UiScope::new(&mut tree, root);
        let resp = scope2.add(ColorPicker::new(&mut state).tag(base_tag));

        // 97.5 / 130.0 = 0.75 * 360.0 = 270.0 degrees (Purple)
        assert!(resp.changed());
        assert!((state.hue - 270.0).abs() < 5.0);
    }
}

#[test]
fn test_color_picker_drag_released() {
    let mut tree = UiTree::new();
    let root = tree.create_node();

    let mut state = HsvColorPickerState::default();
    let base_tag = 0xC010_0004;
    let sv_tag = combine_seeds(base_tag, 1);

    tree.event_state.frame_drag_released_tag = Some(sv_tag);

    let mut scope = UiScope::new(&mut tree, root);
    let resp = scope.add(ColorPicker::new(&mut state).tag(base_tag));

    assert!(resp.drag_released());
}