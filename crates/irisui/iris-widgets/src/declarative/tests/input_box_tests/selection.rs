// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Unit Tests for Input Box Mouse Drag-to-Select and Caret Positioning

use crate::declarative::UiScope;
use iris_core::{Rect, UiTree};

#[test]
fn test_input_box_mouse_click_caret_placement_and_double_click_selection() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root exists");
    let tag = 0x5020;
    let props = crate::declarative::scope::InputBoxProps::new(
        "Hello World",
        "Placeholder...",
        200.0,
        false,
        false,
    );

    // Initial pass to mount the node
    {
        let mut scope = UiScope::new(&mut tree, root);
        scope.input_box_tagged(&props, tag);
        let bounds = Rect::new(0.0, 0.0, 400.0, 100.0);
        scope.finish_layout(bounds);
    }

    let box_id = tree.find_node_by_tag(tag).expect("Box node exists");
    if let Some(node) = tree.get_mut(box_id) {
        node.computed_rect = Rect::new(0.0, 0.0, 200.0, 24.0);
    }

    // 1. Single click targeting character boundary after "Hel"
    let hel_width = iris_text::measure_text_advance("Hel", 11.0);
    tree.clear_frame_clicks();
    tree.event_state.frame_clicked_id = Some(box_id);
    tree.event_state.frame_clicked_tag = Some(tag);
    tree.event_state.frame_clicked_pos = Some(iris_core::Point::new(8.0 + hel_width, 12.0));

    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.input_box_tagged(&props, tag);
        assert!(resp.clicked());
    }

    // Caret index must resolve to index 3 ("Hel"), not full length 11
    assert_eq!(tree.event_state.text_edit_cursor(tag), Some(3));
    assert_eq!(tree.event_state.text_edit_all_selected(tag), Some(false));

    // 2. Double click selects the entire buffer
    tree.clear_frame_clicks();
    tree.event_state.frame_clicked_id = Some(box_id);
    tree.event_state.frame_clicked_tag = Some(tag);
    tree.event_state.frame_double_clicked_tag = Some(tag);

    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.input_box_tagged(&props, tag);
        assert!(resp.clicked());
    }

    assert_eq!(tree.event_state.text_edit_all_selected(tag), Some(true));

    // 3. Second click at another position repositions the caret (must not reset to 0)
    let wor_width = iris_text::measure_text_advance("Hello Wor", 11.0);
    tree.clear_frame_clicks();
    tree.event_state.frame_clicked_id = Some(box_id);
    tree.event_state.frame_clicked_tag = Some(tag);
    tree.event_state.frame_clicked_pos = Some(iris_core::Point::new(8.0 + wor_width, 12.0));

    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.input_box_tagged(&props, tag);
        assert!(resp.clicked());
    }

    assert_eq!(tree.event_state.text_edit_cursor(tag), Some(9));
    assert_eq!(tree.event_state.text_edit_selection(tag), None);
}

#[test]
fn test_input_box_mouse_drag_selection_flow() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root exists");
    let tag = 0x5021;
    let props = crate::declarative::scope::InputBoxProps::new(
        "Antigravity Engine",
        "Placeholder...",
        200.0,
        false,
        false,
    );

    // Initial pass to mount the node
    {
        let mut scope = UiScope::new(&mut tree, root);
        scope.input_box_tagged(&props, tag);
        let bounds = Rect::new(0.0, 0.0, 400.0, 100.0);
        scope.finish_layout(bounds);
    }

    let box_id = tree.find_node_by_tag(tag).expect("Box node exists");
    if let Some(node) = tree.get_mut(box_id) {
        node.computed_rect = Rect::new(0.0, 0.0, 200.0, 24.0);
    }

    // 1. Mouse down at offset "Anti" (length 4)
    let anti_width = iris_text::measure_text_advance("Anti", 11.0);
    tree.clear_frame_clicks();
    tree.event_state.pressed = Some(box_id);
    tree.event_state.pressed_tag = Some(tag);
    tree.event_state.frame_clicked_pos = Some(iris_core::Point::new(8.0 + anti_width, 12.0));
    tree.event_state.frame_clicked_id = Some(box_id);
    tree.event_state.frame_clicked_tag = Some(tag);

    {
        let mut scope = UiScope::new(&mut tree, root);
        let _ = scope.input_box_tagged(&props, tag);
    }

    assert_eq!(tree.event_state.text_edit_cursor(tag), Some(4));
    assert_eq!(tree.event_state.text_drag_anchor, Some(4));

    // 2. Dragging pointer to offset "Antigravity" (length 11)
    let gravity_width = iris_text::measure_text_advance("Antigravity", 11.0);
    tree.clear_frame_clicks();
    tree.event_state.is_pointer_drag_active = true;
    tree.event_state.pointer_capture = Some(box_id);
    tree.event_state.pointer_capture_tag = Some(tag);
    tree.event_state.last_pointer_pos = Some(iris_core::Point::new(8.0 + gravity_width, 12.0));

    {
        let mut scope = UiScope::new(&mut tree, root);
        let _ = scope.input_box_tagged(&props, tag);
    }

    assert_eq!(tree.event_state.text_edit_cursor(tag), Some(11));
    assert_eq!(tree.event_state.text_edit_selection(tag), Some((4, 11)));
}

#[test]
fn test_text_input_with_width_tagged_click_drag_and_double_click() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root exists");
    let tag = 0x5022;
    let mut text = "Dynamic Cube".to_string();

    // Initial pass to mount the node
    {
        let mut scope = UiScope::new(&mut tree, root);
        let _ = scope.text_input_with_width_tagged("Entity name...", tag, 200.0, &mut text);
        let bounds = Rect::new(0.0, 0.0, 400.0, 100.0);
        scope.finish_layout(bounds);
    }

    let input_id = tree.find_node_by_tag(tag).expect("Input node exists");
    if let Some(node) = tree.get_mut(input_id) {
        node.computed_rect = Rect::new(0.0, 0.0, 200.0, 24.0);
    }

    // 1. Single click targeting character boundary after "Dyn" (length 3)
    let dyn_width = iris_text::measure_text_advance("Dyn", 11.0);
    tree.clear_frame_clicks();
    tree.event_state.frame_clicked_id = Some(input_id);
    tree.event_state.frame_clicked_tag = Some(tag);
    tree.event_state.frame_clicked_pos = Some(iris_core::Point::new(8.0 + dyn_width, 12.0));

    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.text_input_with_width_tagged("Entity name...", tag, 200.0, &mut text);
        assert!(resp.clicked());
    }

    assert_eq!(tree.event_state.text_edit_cursor(tag), Some(3));
    assert_eq!(tree.event_state.text_edit_selection(tag), None);

    // 2. Second click targeting character boundary after "Dynamic Cu" (length 10) - must not reset to 0!
    let cu_width = iris_text::measure_text_advance("Dynamic Cu", 11.0);
    tree.clear_frame_clicks();
    tree.event_state.frame_clicked_id = Some(input_id);
    tree.event_state.frame_clicked_tag = Some(tag);
    tree.event_state.frame_clicked_pos = Some(iris_core::Point::new(8.0 + cu_width, 12.0));

    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.text_input_with_width_tagged("Entity name...", tag, 200.0, &mut text);
        assert!(resp.clicked());
    }

    assert_eq!(tree.event_state.text_edit_cursor(tag), Some(10));
    assert_eq!(tree.event_state.text_edit_selection(tag), None);

    // 3. Dragging pointer from 10 backwards to 7 ("Dynamic")
    let dynamic_width = iris_text::measure_text_advance("Dynamic", 11.0);
    tree.clear_frame_clicks();
    tree.event_state.is_pointer_drag_active = true;
    tree.event_state.pointer_capture = Some(input_id);
    tree.event_state.pointer_capture_tag = Some(tag);
    tree.event_state.last_pointer_pos = Some(iris_core::Point::new(8.0 + dynamic_width, 12.0));

    {
        let mut scope = UiScope::new(&mut tree, root);
        let _ = scope.text_input_with_width_tagged("Entity name...", tag, 200.0, &mut text);
    }

    assert_eq!(tree.event_state.text_edit_cursor(tag), Some(7));
    assert_eq!(tree.event_state.text_edit_selection(tag), Some((7, 10)));

    // 4. Double click selects all text
    tree.clear_frame_clicks();
    tree.event_state.is_pointer_drag_active = false;
    tree.event_state.pointer_capture = None;
    tree.event_state.last_pointer_pos = None;
    tree.event_state.frame_clicked_id = Some(input_id);
    tree.event_state.frame_clicked_tag = Some(tag);
    tree.event_state.frame_double_clicked_tag = Some(tag);

    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.text_input_with_width_tagged("Entity name...", tag, 200.0, &mut text);
        assert!(resp.clicked());
    }

    assert_eq!(tree.event_state.text_edit_all_selected(tag), Some(true));
}

#[test]
fn test_text_input_drag_to_select_from_unfocused_mouse_down_flow() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root");
    let mut text = "Dynamic Cube Mesh".to_string();
    let tag = 998877u64;

    // Build initial tree
    let input_id = {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.text_input_with_width_tagged("Entity name...", tag, 200.0, &mut text);
        resp.id
    };
    if let Some(n) = tree.get_mut(input_id) {
        n.computed_rect = Rect::new(0.0, 0.0, 200.0, 24.0);
    }

    // Initially unfocused
    assert!(!tree.is_text_input_focused());
    assert_eq!(tree.event_state.active_text_edit, None);

    // Step 1: MouseDown at character index 0 ("D")
    tree.event_state.pressed = Some(input_id);
    tree.event_state.pressed_tag = Some(tag);
    tree.event_state.pointer_capture = Some(input_id);
    tree.event_state.pointer_capture_tag = Some(tag);
    tree.event_state.frame_clicked_pos = Some(iris_core::Point::new(8.0, 12.0));
    tree.event_state.frame_clicked_rect = Some(Rect::new(0.0, 0.0, 200.0, 24.0));

    {
        let mut scope = UiScope::new(&mut tree, root);
        let _ = scope.text_input_with_width_tagged("Entity name...", tag, 200.0, &mut text);
    }

    // Must immediately focus and anchor at 0
    assert!(tree.is_text_input_focused());
    assert_eq!(tree.event_state.text_edit_cursor(tag), Some(0));
    assert_eq!(tree.event_state.text_drag_anchor, Some(0));
    assert_eq!(tree.event_state.text_edit_selection(tag), None);

    // Step 2: MouseMove past drag threshold to index 7 ("Dynamic")
    let dynamic_width = iris_text::measure_text_advance("Dynamic", 11.0);
    tree.event_state.is_pointer_drag_active = true;
    tree.event_state.last_pointer_pos = Some(iris_core::Point::new(8.0 + dynamic_width, 12.0));

    {
        let mut scope = UiScope::new(&mut tree, root);
        let _ = scope.text_input_with_width_tagged("Entity name...", tag, 200.0, &mut text);
    }

    // Must be selected from 0 to 7
    assert_eq!(tree.event_state.text_edit_cursor(tag), Some(7));
    assert_eq!(tree.event_state.text_edit_selection(tag), Some((0, 7)));

    // Verify SelectionCapsule was added to tree
    let capsule_found = tree
        .iter()
        .any(|(_, n)| n.name.as_deref() == Some("SelectionCapsule"));
    assert!(
        capsule_found,
        "SelectionCapsule must be rendered during drag"
    );

    // Step 3: MouseUp (release after drag)
    tree.event_state.is_pointer_drag_active = false;
    tree.event_state.pressed = None;
    tree.event_state.pressed_tag = None;
    tree.event_state.pointer_capture = None;
    tree.event_state.pointer_capture_tag = None;
    tree.event_state.text_drag_anchor = None;
    tree.event_state.frame_drag_released_id = Some(input_id);
    tree.event_state.frame_drag_released_tag = Some(tag);

    {
        let mut scope = UiScope::new(&mut tree, root);
        let _ = scope.text_input_with_width_tagged("Entity name...", tag, 200.0, &mut text);
    }

    // Selection must persist after mouse release
    assert_eq!(tree.event_state.text_edit_cursor(tag), Some(7));
    assert_eq!(tree.event_state.text_edit_selection(tag), Some((0, 7)));
}

#[test]
fn test_text_input_drag_to_select_full_dispatch_cycle() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root");
    let mut text = "Dynamic Cube Mesh".to_string();
    let tag = 0x8899u64;

    // Initial render & layout pass
    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.text_input_with_width_tagged("Entity name...", tag, 200.0, &mut text);
        let bounds = Rect::new(0.0, 0.0, 400.0, 100.0);
        scope.finish_layout(bounds);
        if let Some(n) = tree.get_mut(resp.id) {
            n.computed_rect = Rect::new(0.0, 0.0, 200.0, 24.0);
        }
    }

    let input_id = tree.find_node_by_tag(tag).expect("Found input");
    if let Some(n) = tree.get_mut(input_id) {
        n.computed_rect = Rect::new(0.0, 0.0, 200.0, 24.0);
    }

    // 1. MouseDown at 8.0, 12.0
    tree.dispatch_event(&iris_core::UiEvent::MouseDown {
        button: iris_core::MouseButton::Left,
        point: iris_core::Point::new(8.0, 12.0),
    });

    {
        let mut scope = UiScope::new(&mut tree, root);
        let _ = scope.text_input_with_width_tagged("Entity name...", tag, 200.0, &mut text);
    }

    assert_eq!(tree.event_state.text_edit_cursor(tag), Some(0));
    assert_eq!(tree.event_state.text_drag_anchor, Some(0));

    // 2. MouseMove to index 7 ("Dynamic")
    let dynamic_width = iris_text::measure_text_advance("Dynamic", 11.0);
    tree.dispatch_event(&iris_core::UiEvent::MouseMove {
        point: iris_core::Point::new(8.0 + dynamic_width, 12.0),
    });

    {
        let mut scope = UiScope::new(&mut tree, root);
        let _ = scope.text_input_with_width_tagged("Entity name...", tag, 200.0, &mut text);
    }

    assert_eq!(tree.event_state.text_edit_cursor(tag), Some(7));
    assert_eq!(tree.event_state.text_edit_selection(tag), Some((0, 7)));

    // 3. MouseUp at same position (release after drag)
    let up_res = tree.dispatch_event(&iris_core::UiEvent::MouseUp {
        button: iris_core::MouseButton::Left,
        point: iris_core::Point::new(8.0 + dynamic_width, 12.0),
    });
    assert!(up_res.is_consumed());

    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.text_input_with_width_tagged("Entity name...", tag, 200.0, &mut text);
        assert!(!resp.clicked());
    }

    // SELECTION MUST NOT DISAPPEAR ON MOUSE UP!
    assert_eq!(tree.event_state.text_edit_cursor(tag), Some(7));
    assert_eq!(tree.event_state.text_edit_selection(tag), Some((0, 7)));
}

#[test]
fn test_input_box_drag_to_select_full_dispatch_cycle() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root");
    let tag = 0x99AAu64;
    let props = crate::declarative::scope::InputBoxProps::new(
        "Dynamic Cube Mesh",
        "Search...",
        200.0,
        false,
        false,
    );

    // Initial render & layout pass
    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.input_box_tagged(&props, tag);
        let bounds = Rect::new(0.0, 0.0, 400.0, 100.0);
        scope.finish_layout(bounds);
        if let Some(n) = tree.get_mut(resp.id) {
            n.computed_rect = Rect::new(0.0, 0.0, 200.0, 24.0);
        }
    }

    let box_id = tree.find_node_by_tag(tag).expect("Found box");
    if let Some(n) = tree.get_mut(box_id) {
        n.computed_rect = Rect::new(0.0, 0.0, 200.0, 24.0);
    }

    // 1. MouseDown at 8.0, 12.0
    tree.dispatch_event(&iris_core::UiEvent::MouseDown {
        button: iris_core::MouseButton::Left,
        point: iris_core::Point::new(8.0, 12.0),
    });

    {
        let mut scope = UiScope::new(&mut tree, root);
        let _ = scope.input_box_tagged(&props, tag);
    }

    assert_eq!(tree.event_state.text_edit_cursor(tag), Some(0));
    assert_eq!(tree.event_state.text_drag_anchor, Some(0));

    // 2. MouseMove to index 7 ("Dynamic")
    let dynamic_width = iris_text::measure_text_advance("Dynamic", 11.0);
    tree.dispatch_event(&iris_core::UiEvent::MouseMove {
        point: iris_core::Point::new(8.0 + dynamic_width, 12.0),
    });

    {
        let mut scope = UiScope::new(&mut tree, root);
        let _ = scope.input_box_tagged(&props, tag);
    }

    assert_eq!(tree.event_state.text_edit_cursor(tag), Some(7));
    assert_eq!(tree.event_state.text_edit_selection(tag), Some((0, 7)));

    // 3. MouseUp at same position (release after drag)
    let up_res = tree.dispatch_event(&iris_core::UiEvent::MouseUp {
        button: iris_core::MouseButton::Left,
        point: iris_core::Point::new(8.0 + dynamic_width, 12.0),
    });
    assert!(up_res.is_consumed());

    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.input_box_tagged(&props, tag);
        assert!(!resp.clicked());
    }

    // Selection must persist after release
    assert_eq!(tree.event_state.text_edit_cursor(tag), Some(7));
    assert_eq!(tree.event_state.text_edit_selection(tag), Some((0, 7)));
}