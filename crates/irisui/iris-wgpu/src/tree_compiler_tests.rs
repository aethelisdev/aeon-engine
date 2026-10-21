// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

use super::*;
use iris_core::style::Style;

#[test]
fn test_tree_compiler_layer_sorting() {
    let mut tree = UiTree::new();
    let root = tree.create_node();
    if let Some(node) = tree.get_mut(root) {
        node.computed_rect = Rect::new(0.0, 0.0, 800.0, 600.0);
        node.style = Style::new().background(Color::BLACK);
        node.layer = UiLayer::Background;
    }

    let content = tree.create_node();
    if let Some(node) = tree.get_mut(content) {
        node.computed_rect = Rect::new(50.0, 50.0, 200.0, 100.0);
        node.style = Style::new().background(Color::WHITE);
        node.layer = UiLayer::Content;
    }
    let _ = tree.add_child(root, content);

    let modal = tree.create_node();
    if let Some(node) = tree.get_mut(modal) {
        node.computed_rect = Rect::new(100.0, 100.0, 300.0, 200.0);
        node.style = Style::new().background(Color::RED);
        node.layer = UiLayer::Modal;
    }
    let _ = tree.add_child(content, modal);

    let mut options = TreeCompilerOptions::new();
    let cmd_list = compile_tree_draw_commands(&tree, root, &mut options);

    // 3 quads should be present in order: Background, Content, Modal
    assert_eq!(cmd_list.quads.len(), 3);
    assert_eq!(cmd_list.quads[0].rect[2], 800.0); // Background
    assert_eq!(cmd_list.quads[1].rect[2], 200.0); // Content
    assert_eq!(cmd_list.quads[2].rect[2], 300.0); // Modal
}

#[test]
fn test_tree_compiler_clip_decoupling_for_elevated_layer() {
    let mut tree = UiTree::new();
    let root = tree.create_node();
    if let Some(node) = tree.get_mut(root) {
        node.computed_rect = Rect::new(0.0, 0.0, 400.0, 400.0);
        node.style = Style::new().clip_children(true);
        node.layer = UiLayer::Content;
    }

    // Popup node child inside clipped container
    let popup = tree.create_node();
    if let Some(node) = tree.get_mut(popup) {
        node.computed_rect = Rect::new(300.0, 300.0, 200.0, 200.0);
        node.style = Style::new().background(Color::BLUE);
        node.layer = UiLayer::Popup;
    }
    let _ = tree.add_child(root, popup);

    let mut options = TreeCompilerOptions::new();
    let cmd_list = compile_tree_draw_commands(&tree, root, &mut options);

    assert_eq!(cmd_list.quads.len(), 1);
    // Scissor clip must be decoupled (0.0, 0.0, 0.0, 0.0) for elevated layer
    assert_eq!(cmd_list.quads[0].clip_rect, [0.0, 0.0, 0.0, 0.0]);
}

#[test]
fn test_tree_compiler_custom_hook() {
    let mut tree = UiTree::new();
    let root = tree.create_node();
    if let Some(node) = tree.get_mut(root) {
        node.computed_rect = Rect::new(0.0, 0.0, 100.0, 100.0);
    }

    let mut custom_called = false;
    let mut custom_drawer =
        |_list: &mut DrawCommandList, id: WidgetId, _node: &WidgetNode, _clip: Option<Rect>| {
            if id == root {
                custom_called = true;
            }
        };

    let mut options = TreeCompilerOptions::new().with_custom_drawer(&mut custom_drawer);
    let _ = compile_tree_draw_commands(&tree, root, &mut options);

    assert!(custom_called);
}

#[test]
fn test_tree_compiler_hover_style_in_place() {
    let mut tree = UiTree::new();
    let root = tree.create_node();
    let btn_tag = 999u64;
    if let Some(node) = tree.get_mut(root) {
        node.computed_rect = Rect::new(0.0, 0.0, 100.0, 30.0);
        node.tag = btn_tag;
        node.style = Style::new()
            .background(Color::BLACK)
            .hover_background(Color::RED);
    }

    // 1. Unhovered compilation
    let mut normal_options = TreeCompilerOptions::new();
    let normal_cmds = compile_tree_draw_commands(&tree, root, &mut normal_options);
    assert_eq!(normal_cmds.quads.len(), 1);
    assert_eq!(
        normal_cmds.quads[0].color,
        Color::BLACK.to_linear().to_array()
    );

    // 2. Hovered compilation with exact tag
    let mut hover_options = TreeCompilerOptions::new().with_hovered_tag(Some(btn_tag));
    let hover_cmds = compile_tree_draw_commands(&tree, root, &mut hover_options);
    assert_eq!(hover_cmds.quads.len(), 1);
    assert_eq!(hover_cmds.quads[0].color, Color::RED.to_linear().to_array());

    // Tree node in memory was never mutated
    assert_eq!(tree.get(root).unwrap().style.background_color, Color::BLACK);
}

#[test]
fn test_tree_compiler_focus_style_in_place() {
    let mut tree = UiTree::new();
    let root = tree.create_node();
    let input_tag = 1234u64;
    if let Some(node) = tree.get_mut(root) {
        node.computed_rect = Rect::new(0.0, 0.0, 150.0, 30.0);
        node.tag = input_tag;
        node.role = WidgetRole::TextInput;
        node.style = Style::new()
            .background(Color::BLACK)
            .border(1.0, Color::rgb(0.5, 0.5, 0.5))
            .focus_border(2.0, Color::CYAN);
    }

    let caret = tree.create_node();
    if let Some(node) = tree.get_mut(caret) {
        node.computed_rect = Rect::new(10.0, 5.0, 1.5, 20.0);
        node.role = WidgetRole::TextInputCaret;
        node.style = Style::new().background(Color::WHITE);
    }
    let _ = tree.add_child(root, caret);

    // 1. Unfocused compilation -> caret not rendered
    let mut unfocused_options = TreeCompilerOptions::new();
    let unfocused_cmds = compile_tree_draw_commands(&tree, root, &mut unfocused_options);
    // Only the input box quad, caret skipped
    assert_eq!(unfocused_cmds.quads.len(), 1);
    assert_eq!(
        unfocused_cmds.quads[0].border_color,
        Color::rgb(0.5, 0.5, 0.5).to_linear().to_array()
    );

    // 2. Focused compilation with blink visible -> caret rendered, focus border applied
    let mut focused_options = TreeCompilerOptions::new()
        .with_focused_tag(Some(input_tag))
        .with_blink_caret(true);
    let focused_cmds = compile_tree_draw_commands(&tree, root, &mut focused_options);
    assert_eq!(focused_cmds.quads.len(), 2);
    assert_eq!(
        focused_cmds.quads[0].border_color,
        Color::CYAN.to_linear().to_array()
    );

    // 3. Focused compilation with blink off -> caret hidden
    let mut blink_off_options = TreeCompilerOptions::new()
        .with_focused_tag(Some(input_tag))
        .with_blink_caret(false);
    let blink_off_cmds = compile_tree_draw_commands(&tree, root, &mut blink_off_options);
    assert_eq!(blink_off_cmds.quads.len(), 1);
}

#[test]
fn test_tree_compiler_all_layers_strict_order() {
    let mut tree = UiTree::new();
    let root = tree.create_node();
    if let Some(node) = tree.get_mut(root) {
        node.computed_rect = Rect::new(0.0, 0.0, 1000.0, 1000.0);
        node.style = Style::new().background(Color::BLACK);
        node.layer = UiLayer::Background;
    }

    // Add nodes in reverse order: Tooltip (5), Popup (4), Modal (3), Floating (2), Content (1)
    let tooltip = tree.create_node();
    if let Some(node) = tree.get_mut(tooltip) {
        node.computed_rect = Rect::new(0.0, 0.0, 50.0, 50.0);
        node.style = Style::new().background(Color::rgb(0.9, 0.9, 0.9));
        node.layer = UiLayer::Tooltip;
    }
    let _ = tree.add_child(root, tooltip);

    let popup = tree.create_node();
    if let Some(node) = tree.get_mut(popup) {
        node.computed_rect = Rect::new(0.0, 0.0, 150.0, 150.0);
        node.style = Style::new().background(Color::rgb(0.8, 0.8, 0.8));
        node.layer = UiLayer::Popup;
    }
    let _ = tree.add_child(root, popup);

    let modal = tree.create_node();
    if let Some(node) = tree.get_mut(modal) {
        node.computed_rect = Rect::new(0.0, 0.0, 300.0, 300.0);
        node.style = Style::new().background(Color::rgb(0.7, 0.7, 0.7));
        node.layer = UiLayer::Modal;
    }
    let _ = tree.add_child(root, modal);

    let floating = tree.create_node();
    if let Some(node) = tree.get_mut(floating) {
        node.computed_rect = Rect::new(0.0, 0.0, 400.0, 400.0);
        node.style = Style::new().background(Color::rgb(0.6, 0.6, 0.6));
        node.layer = UiLayer::Floating;
    }
    let _ = tree.add_child(root, floating);

    let content = tree.create_node();
    if let Some(node) = tree.get_mut(content) {
        node.computed_rect = Rect::new(0.0, 0.0, 500.0, 500.0);
        node.style = Style::new().background(Color::rgb(0.5, 0.5, 0.5));
        node.layer = UiLayer::Content;
    }
    let _ = tree.add_child(root, content);

    let mut options = TreeCompilerOptions::new();
    let cmd_list = compile_tree_draw_commands(&tree, root, &mut options);

    // All 6 quads must be present in exact ascending layer order
    assert_eq!(cmd_list.quads.len(), 6);
    assert_eq!(cmd_list.quads[0].rect[2], 1000.0); // Background (0)
    assert_eq!(cmd_list.quads[1].rect[2], 500.0); // Content (1)
    assert_eq!(cmd_list.quads[2].rect[2], 400.0); // Floating (2)
    assert_eq!(cmd_list.quads[3].rect[2], 300.0); // Modal (3)
    assert_eq!(cmd_list.quads[4].rect[2], 150.0); // Popup (4)
    assert_eq!(cmd_list.quads[5].rect[2], 50.0); // Tooltip (5)
}

#[test]
fn test_tree_compiler_nested_submenu_clip_decoupling() {
    let mut tree = UiTree::new();
    let root = tree.create_node();
    if let Some(node) = tree.get_mut(root) {
        node.computed_rect = Rect::new(0.0, 0.0, 800.0, 600.0);
        node.style = Style::new().background(Color::BLACK);
        node.layer = UiLayer::Background;
    }

    // Panel with clip_children = true
    let panel = tree.create_node();
    if let Some(node) = tree.get_mut(panel) {
        node.computed_rect = Rect::new(0.0, 0.0, 300.0, 300.0);
        node.style = Style::new().clip_children(true);
        node.layer = UiLayer::Content;
    }
    let _ = tree.add_child(root, panel);

    // Parent popup with clip_children = true
    let parent_popup = tree.create_node();
    if let Some(node) = tree.get_mut(parent_popup) {
        node.computed_rect = Rect::new(200.0, 200.0, 150.0, 150.0);
        node.role = WidgetRole::DropdownPopup;
        node.style = Style::new().background(Color::BLUE).clip_children(true);
        node.layer = UiLayer::Popup;
    }
    let _ = tree.add_child(panel, parent_popup);

    // Cascading child submenu sticking out to (340, 220, 120, 100)
    let submenu = tree.create_node();
    if let Some(node) = tree.get_mut(submenu) {
        node.computed_rect = Rect::new(340.0, 220.0, 120.0, 100.0);
        node.role = WidgetRole::DropdownPopup;
        node.style = Style::new().background(Color::RED);
        node.layer = UiLayer::Popup;
    }
    let _ = tree.add_child(parent_popup, submenu);

    let mut options = TreeCompilerOptions::new();
    let cmd_list = compile_tree_draw_commands(&tree, root, &mut options);

    // Both popups must be decoupled from the panel's clip (0, 0, 300, 300)
    // Parent popup has clip_rect = [0.0, 0.0, 0.0, 0.0]
    // Submenu has clip_rect = [0.0, 0.0, 0.0, 0.0] because it is an elevated root (DropdownPopup)
    let parent_quad = cmd_list.quads.iter().find(|q| q.rect[2] == 150.0).unwrap();
    assert_eq!(parent_quad.clip_rect, [0.0, 0.0, 0.0, 0.0]);

    let submenu_quad = cmd_list.quads.iter().find(|q| q.rect[2] == 120.0).unwrap();
    assert_eq!(submenu_quad.clip_rect, [0.0, 0.0, 0.0, 0.0]);
}

#[test]
fn test_tree_compiler_hardware_scissor_reset_on_elevated_layer() {
    let mut tree = UiTree::new();
    let root = tree.create_node();
    if let Some(node) = tree.get_mut(root) {
        node.computed_rect = Rect::new(0.0, 0.0, 800.0, 600.0);
        node.style = Style::new().background(Color::BLACK);
        node.layer = UiLayer::Content;
    }

    let popup = tree.create_node();
    if let Some(node) = tree.get_mut(popup) {
        node.computed_rect = Rect::new(100.0, 100.0, 200.0, 200.0);
        node.style = Style::new().background(Color::BLUE);
        node.layer = UiLayer::Popup;
    }
    let _ = tree.add_child(root, popup);

    // Pass an initial clip (simulating host panel scissor)
    let mut options = TreeCompilerOptions::new().with_clip(Some(Rect::new(0.0, 0.0, 300.0, 300.0)));
    let cmd_list = compile_tree_draw_commands(&tree, root, &mut options);

    // The command stream must contain a ResetScissor command before or when drawing elevated layers
    assert!(
        cmd_list
            .commands
            .iter()
            .any(|c| matches!(c, crate::command::DrawCommand::ResetScissor))
    );
}

#[test]
fn test_tree_compiler_immediate_canvas_primitives() {
    let mut tree = UiTree::new();
    let root = tree.create_node();
    if let Some(node) = tree.get_mut(root) {
        node.computed_rect = Rect::new(0.0, 0.0, 400.0, 300.0);
        node.role = WidgetRole::ImmediateCanvas;
    }

    tree.add_canvas_primitive(
        root,
        iris_core::CanvasPrimitive::Rect {
            rect: Rect::new(10.0, 10.0, 100.0, 50.0),
            color: Color::WHITE,
            corner_radius: 2.0,
        },
    );
    tree.add_canvas_primitive(
        root,
        iris_core::CanvasPrimitive::Border {
            rect: Rect::new(20.0, 20.0, 80.0, 40.0),
            border_width: 1.0,
            color: Color::RED,
            corner_radius: 0.0,
        },
    );
    tree.add_canvas_primitive(
        root,
        iris_core::CanvasPrimitive::Line {
            start: iris_core::Point::new(0.0, 0.0),
            end: iris_core::Point::new(50.0, 50.0),
            thickness: 2.0,
            color: Color::GREEN,
        },
    );

    let mut options = TreeCompilerOptions::new();
    let cmd_list = compile_tree_draw_commands(&tree, root, &mut options);

    // 3 quads should be generated from immediate canvas primitives
    assert_eq!(cmd_list.quads.len(), 3);
}

#[test]
fn test_tree_compiler_role_based_layer_elevation() {
    let mut tree = UiTree::new();
    let root = tree.create_node();
    if let Some(node) = tree.get_mut(root) {
        node.computed_rect = Rect::new(0.0, 0.0, 500.0, 500.0);
        node.style = Style::new().background(Color::BLACK);
        // Explicitly leaves layer as Content (default)
    }

    // Role-inferred elevated popup without explicit set_layer call
    let popup = tree.create_node();
    if let Some(node) = tree.get_mut(popup) {
        node.computed_rect = Rect::new(10.0, 10.0, 100.0, 100.0);
        node.style = Style::new().background(Color::RED);
        node.role = WidgetRole::DropdownPopup;
        // layer is left as default Content
    }
    let _ = tree.add_child(root, popup);

    let mut options = TreeCompilerOptions::new();
    let cmd_list = compile_tree_draw_commands(&tree, root, &mut options);

    // Two quads compiled in strict Z-order: Content (root) then Popup (role-inferred)
    assert_eq!(cmd_list.quads.len(), 2);
    assert_eq!(cmd_list.quads[0].rect[2], 500.0);
    assert_eq!(cmd_list.quads[1].rect[2], 100.0);
}

#[test]
fn test_tree_compiler_layer_range_filtering_with_scissor() {
    let mut tree = UiTree::new();
    let root = tree.create_node();
    if let Some(node) = tree.get_mut(root) {
        node.computed_rect = Rect::new(0.0, 0.0, 600.0, 600.0);
        node.style = Style::new().background(Color::BLACK);
        node.layer = UiLayer::Content;
    }

    let modal = tree.create_node();
    if let Some(node) = tree.get_mut(modal) {
        node.computed_rect = Rect::new(50.0, 50.0, 200.0, 200.0);
        node.style = Style::new().background(Color::GREEN);
        node.layer = UiLayer::Modal;
    }
    let _ = tree.add_child(root, modal);

    // Initial clip provided, but only elevated overlays (Modal..Tooltip) requested
    let mut options = TreeCompilerOptions::new()
        .with_clip(Some(Rect::new(0.0, 0.0, 300.0, 300.0)))
        .with_layer_range(UiLayer::Modal, UiLayer::Tooltip);
    let cmd_list = compile_tree_draw_commands(&tree, root, &mut options);

    // Only modal should be compiled
    assert_eq!(cmd_list.quads.len(), 1);
    assert_eq!(cmd_list.quads[0].rect[2], 200.0);

    // Initial SetScissor should NOT be emitted because base layers were excluded
    let has_set_scissor = cmd_list
        .commands
        .iter()
        .any(|c| matches!(c, crate::command::DrawCommand::SetScissor { .. }));
    assert!(!has_set_scissor);
}

#[test]
fn test_tree_compiler_scissor_cleanliness_at_end() {
    let mut tree = UiTree::new();
    let root = tree.create_node();
    if let Some(node) = tree.get_mut(root) {
        node.computed_rect = Rect::new(0.0, 0.0, 400.0, 400.0);
        node.style = Style::new().background(Color::BLACK);
        node.layer = UiLayer::Content;
    }

    // Clip provided for content-only tree
    let mut options = TreeCompilerOptions::new().with_clip(Some(Rect::new(0.0, 0.0, 200.0, 200.0)));
    let cmd_list = compile_tree_draw_commands(&tree, root, &mut options);

    // Must start with SetScissor and end with ResetScissor
    assert!(matches!(
        cmd_list.commands.first(),
        Some(crate::command::DrawCommand::SetScissor { .. })
    ));
    assert!(matches!(
        cmd_list.commands.last(),
        Some(crate::command::DrawCommand::ResetScissor)
    ));
}

#[test]
fn test_tree_compiler_boundary_slices_attribution() {
    let mut tree = UiTree::new();
    let root = tree.create_node();
    if let Some(node) = tree.get_mut(root) {
        node.computed_rect = Rect::new(0.0, 0.0, 1000.0, 800.0);
        node.style = Style::new().background(Color::BLACK);
    }

    // Boundary Panel A
    let panel_a = tree.create_node();
    if let Some(node) = tree.get_mut(panel_a) {
        node.computed_rect = Rect::new(0.0, 0.0, 300.0, 800.0);
        node.style = Style::new().background(Color::rgba(0.1, 0.1, 0.1, 1.0));
    }
    tree.set_boundary(panel_a, true);
    let _ = tree.add_child(root, panel_a);

    let child_a1 = tree.create_node();
    if let Some(node) = tree.get_mut(child_a1) {
        node.computed_rect = Rect::new(10.0, 10.0, 100.0, 30.0);
        node.style = Style::new().background(Color::WHITE);
    }
    let _ = tree.add_child(panel_a, child_a1);

    let child_a2 = tree.create_node();
    if let Some(node) = tree.get_mut(child_a2) {
        node.computed_rect = Rect::new(10.0, 50.0, 100.0, 30.0);
        node.style = Style::new().background(Color::WHITE);
    }
    let _ = tree.add_child(panel_a, child_a2);

    // Boundary Panel B
    let panel_b = tree.create_node();
    if let Some(node) = tree.get_mut(panel_b) {
        node.computed_rect = Rect::new(300.0, 0.0, 700.0, 800.0);
        node.style = Style::new().background(Color::rgba(0.2, 0.2, 0.2, 1.0));
    }
    tree.set_boundary(panel_b, true);
    let _ = tree.add_child(root, panel_b);

    let child_b1 = tree.create_node();
    if let Some(node) = tree.get_mut(child_b1) {
        node.computed_rect = Rect::new(310.0, 10.0, 200.0, 40.0);
        node.style = Style::new().background(Color::BLUE);
    }
    let _ = tree.add_child(panel_b, child_b1);

    let mut options = TreeCompilerOptions::new();
    let cmd_list = compile_tree_draw_commands(&tree, root, &mut options);

    // Total quads: 1 (root) + 3 (panel_a) + 2 (panel_b) = 6 quads
    assert_eq!(cmd_list.quads.len(), 6);

    // Boundary slices must identify panel_a (3 quads) and panel_b (2 quads)
    let slice_a = cmd_list
        .boundary_slices
        .iter()
        .find(|(id, _)| *id == panel_a)
        .map(|(_, s)| *s)
        .expect("panel_a boundary slice must exist");
    assert_eq!(slice_a.count, 3);
    assert_eq!(slice_a.start, 1);

    let slice_b = cmd_list
        .boundary_slices
        .iter()
        .find(|(id, _)| *id == panel_b)
        .map(|(_, s)| *s)
        .expect("panel_b boundary slice must exist");
    assert_eq!(slice_b.count, 2);
    assert_eq!(slice_b.start, 4);
}