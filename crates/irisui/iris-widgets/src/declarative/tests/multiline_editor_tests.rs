// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

use crate::controls::multiline::MultilineTextState;
use crate::declarative::UiScope;
use iris_core::event::{KeyCode, UiEvent};
use iris_core::{Point, UiTree, WidgetRole};

#[test]
fn test_multiline_editor_declarative_construction_and_hierarchy() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root node created");
    tree.get_mut(root)
        .unwrap()
        .set_style(iris_core::Style::new().width(300.0).height(200.0));

    let tag = 0x9001;
    let mut state = MultilineTextState::new("fn main() {\n    println!(\"Hello\");\n}");

    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.multiline_editor(tag, 200.0, &mut state);
        assert_eq!(resp.id, tree.find_node_by_tag(tag).expect("Found node"));
    }

    let editor_id = tree.find_node_by_tag(tag).expect("Editor node exists");
    let editor_node = tree.get(editor_id).expect("Editor node");
    assert_eq!(editor_node.role, WidgetRole::TextInput);
    assert_eq!(editor_node.tag, tag);

    // 3 lines -> 3 row children
    assert_eq!(editor_node.children.len(), 3);

    for (line_idx, &row_id) in editor_node.children.iter().enumerate() {
        let row_node = tree.get(row_id).expect("Row node exists");
        // Each row has 3 children: gutter cell + vertical divider + text cell
        assert_eq!(row_node.children.len(), 3);

        let gutter_id = row_node.children[0];
        let gutter_node = tree.get(gutter_id).expect("Gutter cell");
        assert_eq!(
            gutter_node.text.as_deref(),
            Some(format!("{}", line_idx + 1).as_str())
        );
        assert_eq!(gutter_node.text_align, iris_core::TextAlign::Center);

        let text_id = row_node.children[2];
        let text_node = tree.get(text_id).expect("Text cell");
        assert_eq!(text_node.text.as_deref(), state.line_str(line_idx));
        assert_eq!(text_node.style.margin.left, 8.0);
    }
}

#[test]
fn test_multiline_editor_caret_exact_alignment() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root node created");
    tree.get_mut(root)
        .unwrap()
        .set_style(iris_core::Style::new().width(300.0).height(200.0));

    let tag = 0x9004;
    let mut state = MultilineTextState::new("let speed = 9.81;");
    state.set_cursor(0, 17); // End of line

    tree.event_state.focused_tag = Some(tag);

    {
        let mut scope = UiScope::new(&mut tree, root);
        let _ = scope.multiline_editor(tag, 200.0, &mut state);
    }

    let editor_id = tree.find_node_by_tag(tag).expect("Editor node");
    let row_id = tree.get(editor_id).unwrap().children[0];
    let row_node = tree.get(row_id).unwrap();

    let text_cell_id = row_node.children[2];
    let text_node = tree.get(text_cell_id).unwrap();
    assert_eq!(text_node.style.margin.left, 8.0);

    let caret_id = text_node.children[0];
    let caret_node = tree.get(caret_id).unwrap();
    assert_eq!(caret_node.role, WidgetRole::TextInputCaret);

    let expected_advance = iris_text::measure_text_advance("let speed = 9.81;", 12.0);
    assert_eq!(caret_node.style.inset_left, Some(expected_advance));
}

#[test]
fn test_multiline_editor_click_caret_placement() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root node created");
    tree.get_mut(root)
        .unwrap()
        .set_style(iris_core::Style::new().width(300.0).height(200.0));

    let tag = 0x9002;
    let mut state = MultilineTextState::new("Line 0\nLine 1\nLine 2");

    // Simulate click on line 1
    tree.event_state.frame_clicked_tag = Some(tag);
    tree.event_state.frame_clicked_pos = Some(Point::new(50.0, 30.0)); // Y=30 -> Line 1 (height=20)

    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.multiline_editor(tag, 200.0, &mut state);
        assert!(resp.clicked());
    }

    // Focused
    assert_eq!(tree.event_state.focused_tag, Some(tag));
    // Line 1 is selected
    assert_eq!(state.cursor.0, 1);
}

#[test]
fn test_multiline_editor_keyboard_typing_and_navigation() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root node created");
    tree.get_mut(root)
        .unwrap()
        .set_style(iris_core::Style::new().width(300.0).height(200.0));

    let tag = 0x9003;
    let mut state = MultilineTextState::new("abc\ndef");

    // Focus widget
    tree.event_state.focused_tag = Some(tag);

    // 1. Simulate down arrow
    tree.event_state.frame_key = Some(KeyCode::ArrowDown);
    tree.event_state.frame_key_pressed = true;

    {
        let mut scope = UiScope::new(&mut tree, root);
        let _ = scope.multiline_editor(tag, 200.0, &mut state);
    }

    assert_eq!(state.cursor, (1, 0));

    // 2. Simulate text input "x"
    tree.event_state.frame_key = None;
    tree.event_state.frame_key_pressed = false;
    tree.event_state.frame_text_input = Some("x".to_string());

    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.multiline_editor(tag, 200.0, &mut state);
        assert!(resp.changed());
    }

    assert_eq!(state.text, "abc\nxdef");
    assert_eq!(state.cursor, (1, 1));
}

#[test]
fn test_multiline_editor_scrollbar_emission_when_overflowing() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root node created");
    tree.get_mut(root)
        .unwrap()
        .set_style(iris_core::Style::new().width(300.0).height(200.0));

    let tag = 0x9005;
    // 50 lines -> content height = 50 * 20.0 = 1000.0px > 200.0px viewport
    let content = (0..50)
        .map(|i| format!("Line {}", i))
        .collect::<Vec<_>>()
        .join("\n");
    let mut state = MultilineTextState::new(&content);

    {
        let mut scope = UiScope::new(&mut tree, root);
        let _ = scope.multiline_editor(tag, 200.0, &mut state);
    }

    let track_tag = tag.wrapping_add(0x1000);
    let thumb_tag = tag.wrapping_add(0x2000);

    let track_id = tree
        .find_node_by_tag(track_tag)
        .expect("ScrollBarTrack node must be emitted");
    let thumb_id = tree
        .find_node_by_tag(thumb_tag)
        .expect("ScrollBarThumb node must be emitted");

    let track_node = tree.get(track_id).unwrap();
    assert_eq!(track_node.role, WidgetRole::ScrollBarTrack);

    let thumb_node = tree.get(thumb_id).unwrap();
    assert_eq!(thumb_node.role, WidgetRole::ScrollBarThumb);
}

#[test]
fn test_multiline_editor_mouse_wheel_autonomous_scroll() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root node created");
    tree.get_mut(root)
        .unwrap()
        .set_style(iris_core::Style::new().width(300.0).height(200.0));

    let tag = 0x9006;
    let content = (0..50)
        .map(|i| format!("Line {}", i))
        .collect::<Vec<_>>()
        .join("\n");
    let mut state = MultilineTextState::new(&content);
    tree.set_tag_rect(tag, iris_core::geometry::Rect::new(0.0, 0.0, 300.0, 200.0));

    // Initial render
    {
        let mut scope = UiScope::new(&mut tree, root);
        let _ = scope.multiline_editor(tag, 200.0, &mut state);
    }
    let mut layout_engine = iris_layout::LayoutEngine::new();
    let _ = layout_engine.compute_layout(&mut tree, iris_core::Size::new(800.0, 600.0));

    // Simulate mouse wheel scroll down (delta_y = -40.0)
    let scroll_event = UiEvent::MouseScroll {
        delta_x: 0.0,
        delta_y: -40.0,
        point: Point::new(100.0, 100.0),
    };
    let dispatch = tree.dispatch_event(&scroll_event);
    assert!(dispatch.consumed);
    assert_eq!(tree.scroll_offset(tag), 40.0);

    // Re-render: multiline_editor must synchronize state.scroll_y from tree
    {
        let mut scope = UiScope::new(&mut tree, root);
        let _ = scope.multiline_editor(tag, 200.0, &mut state);
    }
    assert_eq!(state.scroll_y, 40.0);
}

#[test]
fn test_multiline_editor_cursor_auto_scroll_keeps_caret_visible() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root node created");
    tree.get_mut(root)
        .unwrap()
        .set_style(iris_core::Style::new().width(300.0).height(200.0));

    let tag = 0x9007;
    let content = (0..30)
        .map(|i| format!("Line {}", i))
        .collect::<Vec<_>>()
        .join("\n");
    let mut state = MultilineTextState::new(&content);
    tree.set_tag_rect(tag, iris_core::geometry::Rect::new(0.0, 0.0, 300.0, 200.0));

    // Move cursor to line 20: Y = 20 * 20.0 = 400.0px (exceeds 200px viewport)
    state.set_cursor(20, 0);

    // Focus and simulate keyboard navigation (ArrowDown)
    tree.event_state.focused_tag = Some(tag);
    tree.event_state.frame_key = Some(KeyCode::ArrowDown);
    tree.event_state.frame_key_pressed = true;

    {
        let mut scope = UiScope::new(&mut tree, root);
        let _ = scope.multiline_editor(tag, 200.0, &mut state);
    }

    // Auto-scroll must position viewport so cursor line is within visible view
    assert!(state.scroll_y >= 220.0);
    assert_eq!(tree.scroll_offset(tag), state.scroll_y);
}

#[test]
fn test_multiline_editor_thumb_drag_scroll() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root node created");
    tree.get_mut(root)
        .unwrap()
        .set_style(iris_core::Style::new().width(300.0).height(200.0));

    let tag = 0x9008;
    let content = (0..30)
        .map(|i| format!("Line {}", i))
        .collect::<Vec<_>>()
        .join("\n");
    let mut state = MultilineTextState::new(&content);
    tree.set_tag_rect(tag, iris_core::geometry::Rect::new(0.0, 0.0, 300.0, 200.0));

    // Initial render
    {
        let mut scope = UiScope::new(&mut tree, root);
        let _ = scope.multiline_editor(tag, 200.0, &mut state);
    }
    // Compute layout for the tree
    let mut layout_engine = iris_layout::LayoutEngine::new();
    let _ = layout_engine.compute_layout(&mut tree, iris_core::Size::new(800.0, 600.0));

    let thumb_node = tree
        .iter()
        .find(|(_, n)| n.role == WidgetRole::ScrollBarThumb)
        .expect("Thumb emitted");
    let thumb_rect = thumb_node.1.computed_rect;
    let thumb_center = Point::new(
        thumb_rect.x + thumb_rect.width * 0.5,
        thumb_rect.y + thumb_rect.height * 0.5,
    );

    // 1. MouseDown on thumb to initiate drag capture
    let down_res = tree.dispatch_event(&UiEvent::MouseDown {
        button: iris_core::MouseButton::Left,
        point: thumb_center,
    });
    assert!(down_res.consumed);

    // 2. MouseMove dragging down by 50px
    let move_res = tree.dispatch_event(&UiEvent::MouseMove {
        point: Point::new(thumb_center.x, thumb_center.y + 50.0),
    });
    assert!(move_res.consumed);
    assert!(tree.scroll_offset(tag) > 0.0);

    // Re-render next frame with new scroll position!
    let _ = tree.clear_children(root);
    {
        let mut scope = UiScope::new(&mut tree, root);
        let _ = scope.multiline_editor(tag, 200.0, &mut state);
    }
    let container_id = tree.find_node_by_tag(tag).unwrap();
    tree.get_mut(container_id).unwrap().computed_rect =
        iris_core::geometry::Rect::new(0.0, 0.0, 300.0, 200.0);
    let _ = layout_engine.compute_layout(&mut tree, iris_core::Size::new(800.0, 600.0));

    let thumb_id2 = tree
        .iter()
        .find(|(_, n)| n.role == WidgetRole::ScrollBarThumb)
        .map(|(id, _)| id)
        .unwrap();
    let thumb_rect2 = tree.get(thumb_id2).unwrap().computed_rect;
    assert_eq!(thumb_rect2.x, 293.0);
    assert!(thumb_rect2.y > thumb_rect.y);
    assert_eq!(thumb_rect2.y, 53.0);
}