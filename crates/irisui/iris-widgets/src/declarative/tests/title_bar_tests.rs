// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Unit Tests for Autonomous Frameless Window TitleBar Widget

use crate::declarative::UiScope;
use crate::declarative::widget::{
    TITLE_BAR_CONTROL_SEED, TitleBar, TitleBarAction, title_bar_close_tag, title_bar_maximize_tag,
    title_bar_minimize_tag,
};
use iris_core::{Color, UiTree};

#[test]
fn test_title_bar_structure_and_defaults() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root node must exist");
    let mut scope = UiScope::new(&mut tree, root);

    let resp = scope.add(TitleBar::new("Aeon Engine"));
    assert_eq!(resp.action(), None);
    assert!(!resp.is_minimize());
    assert!(!resp.is_maximize());
    assert!(!resp.is_close());
    assert!(!resp.drag_area_clicked());

    assert!(tree.has_node_named("TitleBar"));
    assert!(tree.has_node_named("TitleBarLeft"));
    assert!(tree.has_node_named("TitleBarTitle"));
    assert!(tree.has_node_named("TitleBarSpacer"));
    assert!(tree.has_node_named("TitleBarControls"));
    assert!(!tree.has_node_named("TitleBarIcon"));

    tree.assert_node_text("TitleBarTitle", "Aeon Engine");
    tree.assert_node_text("WindowMinimizeButton", "—");
    tree.assert_node_text("WindowMaximizeButton", "□");
    tree.assert_node_text("WindowCloseButton", "✕");
}

#[test]
fn test_title_bar_with_icon() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root node must exist");
    let mut scope = UiScope::new(&mut tree, root);

    let _ = scope.add(TitleBar::new("Aeon Engine").icon("⚙"));

    assert!(tree.has_node_named("TitleBarIcon"));
    tree.assert_node_text("TitleBarIcon", "⚙");
}

#[test]
fn test_title_bar_maximized_toggle_icon() {
    // Restored (not maximized) state
    let mut tree_normal = UiTree::new();
    let root_normal = tree_normal.create_root().expect("Root node");
    let mut scope_normal = UiScope::new(&mut tree_normal, root_normal);
    let _ = scope_normal.add(TitleBar::new("Title").maximized(false));

    tree_normal.assert_node_text("WindowMaximizeButton", "□");

    // Maximized state
    let mut tree_max = UiTree::new();
    let root_max = tree_max.create_root().expect("Root node");
    let mut scope_max = UiScope::new(&mut tree_max, root_max);
    let _ = scope_max.add(TitleBar::new("Title").maximized(true));

    tree_max.assert_node_text("WindowMaximizeButton", "❐");
}

#[test]
fn test_title_bar_hide_all_controls() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root node");
    let mut scope = UiScope::new(&mut tree, root);

    let _ = scope.add(TitleBar::new("Title").show_controls(false));

    assert!(!tree.has_node_named("TitleBarControls"));
    assert!(!tree.has_node_named("WindowMinimizeButton"));
    assert!(!tree.has_node_named("WindowMaximizeButton"));
    assert!(!tree.has_node_named("WindowCloseButton"));
}

#[test]
fn test_title_bar_granular_controls_visibility() {
    // 1. Hide minimize only
    let mut tree_no_min = UiTree::new();
    let root_no_min = tree_no_min.create_root().expect("Root node");
    let mut scope_no_min = UiScope::new(&mut tree_no_min, root_no_min);
    let _ = scope_no_min.add(TitleBar::new("Title").show_minimize(false));

    assert!(!tree_no_min.has_node_named("WindowMinimizeButton"));
    assert!(tree_no_min.has_node_named("WindowMaximizeButton"));
    assert!(tree_no_min.has_node_named("WindowCloseButton"));

    // 2. Hide maximize only (e.g. fixed-size dialogs)
    let mut tree_no_max = UiTree::new();
    let root_no_max = tree_no_max.create_root().expect("Root node");
    let mut scope_no_max = UiScope::new(&mut tree_no_max, root_no_max);
    let _ = scope_no_max.add(TitleBar::new("Title").show_maximize(false));

    assert!(tree_no_max.has_node_named("WindowMinimizeButton"));
    assert!(!tree_no_max.has_node_named("WindowMaximizeButton"));
    assert!(tree_no_max.has_node_named("WindowCloseButton"));

    // 3. Hide close only
    let mut tree_no_close = UiTree::new();
    let root_no_close = tree_no_close.create_root().expect("Root node");
    let mut scope_no_close = UiScope::new(&mut tree_no_close, root_no_close);
    let _ = scope_no_close.add(TitleBar::new("Title").show_close(false));

    assert!(tree_no_close.has_node_named("WindowMinimizeButton"));
    assert!(tree_no_close.has_node_named("WindowMaximizeButton"));
    assert!(!tree_no_close.has_node_named("WindowCloseButton"));
}

#[test]
fn test_title_bar_minimize_click_action() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root node");
    let base_tag = 0x1234_5678;
    let min_tag = title_bar_minimize_tag(base_tag);
    tree.event_state.frame_clicked_tag = Some(min_tag);

    let mut scope = UiScope::new(&mut tree, root);
    let mut callback_action = None;
    let resp = scope
        .title_bar("Aeon Engine")
        .tag(base_tag)
        .controls(|action| {
            callback_action = Some(action);
        });

    assert_eq!(resp.action(), Some(TitleBarAction::Minimize));
    assert!(resp.is_minimize());
    assert!(!resp.is_maximize());
    assert!(!resp.is_close());
    assert!(!resp.drag_area_clicked());
    assert_eq!(callback_action, Some(TitleBarAction::Minimize));
}

#[test]
fn test_title_bar_maximize_click_action() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root node");
    let base_tag = 0x1234_5678;
    let max_tag = title_bar_maximize_tag(base_tag);
    tree.event_state.frame_clicked_tag = Some(max_tag);

    let mut scope = UiScope::new(&mut tree, root);
    let mut callback_action = None;
    let resp = scope
        .title_bar("Aeon Engine")
        .tag(base_tag)
        .controls(|action| {
            callback_action = Some(action);
        });

    assert_eq!(resp.action(), Some(TitleBarAction::Maximize));
    assert!(!resp.is_minimize());
    assert!(resp.is_maximize());
    assert!(!resp.is_close());
    assert!(!resp.drag_area_clicked());
    assert_eq!(callback_action, Some(TitleBarAction::Maximize));
}

#[test]
fn test_title_bar_close_click_action() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root node");
    let base_tag = 0x1234_5678;
    let close_tag = title_bar_close_tag(base_tag);
    tree.event_state.frame_clicked_tag = Some(close_tag);

    let mut scope = UiScope::new(&mut tree, root);
    let mut callback_action = None;
    let resp = scope
        .title_bar("Aeon Engine")
        .tag(base_tag)
        .controls(|action| {
            callback_action = Some(action);
        });

    assert_eq!(resp.action(), Some(TitleBarAction::Close));
    assert!(!resp.is_minimize());
    assert!(!resp.is_maximize());
    assert!(resp.is_close());
    assert!(!resp.drag_area_clicked());
    assert_eq!(callback_action, Some(TitleBarAction::Close));
}

#[test]
fn test_title_bar_no_action_when_unclicked() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root node");
    let base_tag = 0x1234_5678;

    let mut scope = UiScope::new(&mut tree, root);
    let mut callback_called = false;
    let resp = scope.title_bar("Aeon Engine").tag(base_tag).controls(|_| {
        callback_called = true;
    });

    assert_eq!(resp.action(), None);
    assert!(!resp.is_minimize());
    assert!(!resp.is_maximize());
    assert!(!resp.is_close());
    assert!(!resp.drag_area_clicked());
    assert!(!callback_called);
}

#[test]
fn test_title_bar_drag_area_clicked() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root node");
    let base_tag = 0x1234_5678;
    // Simulate user clicking on the title bar body (outside the button tags)
    tree.event_state.frame_clicked_tag = Some(base_tag);

    let mut scope = UiScope::new(&mut tree, root);
    let resp = scope.title_bar("Aeon Engine").tag(base_tag).show();

    assert_eq!(resp.action(), None);
    assert!(resp.clicked());
    assert!(
        resp.drag_area_clicked(),
        "drag_area_clicked must return true when the title bar body is clicked"
    );
}

#[test]
fn test_title_bar_default_seed_derivation() {
    // When base_tag == 0, tags must derive deterministically from TITLE_BAR_CONTROL_SEED
    let min_tag = title_bar_minimize_tag(0);
    let max_tag = title_bar_maximize_tag(0);
    let close_tag = title_bar_close_tag(0);

    assert_ne!(min_tag, 0);
    assert_ne!(max_tag, 0);
    assert_ne!(close_tag, 0);
    assert_ne!(min_tag, max_tag);
    assert_ne!(min_tag, close_tag);
    assert_ne!(max_tag, close_tag);

    assert_eq!(min_tag, title_bar_minimize_tag(TITLE_BAR_CONTROL_SEED));
    assert_eq!(max_tag, title_bar_maximize_tag(TITLE_BAR_CONTROL_SEED));
    assert_eq!(close_tag, title_bar_close_tag(TITLE_BAR_CONTROL_SEED));
}

#[test]
fn test_title_bar_custom_dimensions_and_colors() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root node");
    let mut scope = UiScope::new(&mut tree, root);

    let custom_bg = Color::rgba(0.05, 0.05, 0.05, 1.0);
    let custom_border = Color::rgba(0.2, 0.2, 0.2, 1.0);
    let custom_text = Color::rgba(0.9, 0.9, 0.9, 1.0);

    let resp = scope.add(
        TitleBar::new("Custom Title")
            .height(40.0)
            .background(custom_bg)
            .border_color(custom_border)
            .text_color(custom_text),
    );

    let root_node = tree.get(resp.id).expect("Root titlebar node");
    assert_eq!(root_node.style.height, Some(40.0));
    assert_eq!(root_node.style.background_color, custom_bg);
    assert_eq!(root_node.style.border.color, custom_border);

    let title_node = tree
        .iter()
        .map(|(_, n)| n)
        .find(|n| n.name.as_deref() == Some("TitleBarTitle"))
        .expect("Title node");
    assert_eq!(title_node.text_color, custom_text);
}

#[test]
fn test_title_bar_implicit_tag_close_click() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root node");

    let mut scope = UiScope::new(&mut tree, root);
    let derived_tag = scope.tag_for("Aeon Engine");
    let close_tag = title_bar_close_tag(derived_tag);

    scope.tree_mut().event_state.frame_clicked_tag = Some(close_tag);

    let mut callback_action = None;
    let resp = scope.title_bar("Aeon Engine").controls(|action| {
        callback_action = Some(action);
    });

    assert_eq!(resp.action(), Some(TitleBarAction::Close));
    assert!(resp.is_close());
    assert!(!resp.drag_area_clicked());
    assert_eq!(callback_action, Some(TitleBarAction::Close));
}

#[test]
fn test_title_bar_title_text_click_triggers_drag_area() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root node");
    let mut scope = UiScope::new(&mut tree, root);
    let _ = scope.title_bar("Aeon Engine").show();
    let title_id = tree
        .iter()
        .find(|(_, n)| n.name.as_deref() == Some("TitleBarTitle"))
        .map(|(id, _)| id)
        .expect("TitleBarTitle node must exist");

    let mut tree2 = UiTree::new();
    let root2 = tree2.create_root().expect("Root node");
    tree2.event_state.frame_clicked_id = Some(title_id);
    let mut scope2 = UiScope::new(&mut tree2, root2);
    let resp = scope2.title_bar("Aeon Engine").show();

    assert_eq!(resp.action(), None);
    assert!(resp.clicked());
    assert!(
        resp.drag_area_clicked(),
        "Clicking title text must trigger drag area via interaction bubbling"
    );
}

#[test]
fn test_title_bar_spacer_click_triggers_drag_area() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root node");
    let mut scope = UiScope::new(&mut tree, root);
    let _ = scope.title_bar("Aeon Engine").show();
    let spacer_id = tree
        .iter()
        .find(|(_, n)| n.name.as_deref() == Some("TitleBarSpacer"))
        .map(|(id, _)| id)
        .expect("TitleBarSpacer node must exist");

    let mut tree2 = UiTree::new();
    let root2 = tree2.create_root().expect("Root node");
    tree2.event_state.frame_clicked_id = Some(spacer_id);
    let mut scope2 = UiScope::new(&mut tree2, root2);
    let resp = scope2.title_bar("Aeon Engine").show();

    assert_eq!(resp.action(), None);
    assert!(resp.clicked());
    assert!(
        resp.drag_area_clicked(),
        "Clicking empty spacer must trigger drag area via interaction bubbling"
    );
}

#[test]
fn test_title_bar_icon_click_triggers_drag_area() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root node");
    let mut scope = UiScope::new(&mut tree, root);
    let _ = scope.title_bar("Aeon Engine").icon("🚀").show();
    let icon_id = tree
        .iter()
        .find(|(_, n)| n.name.as_deref() == Some("TitleBarIcon"))
        .map(|(id, _)| id)
        .expect("TitleBarIcon node must exist");

    let mut tree2 = UiTree::new();
    let root2 = tree2.create_root().expect("Root node");
    tree2.event_state.frame_clicked_id = Some(icon_id);
    let mut scope2 = UiScope::new(&mut tree2, root2);
    let resp = scope2.title_bar("Aeon Engine").icon("🚀").show();

    assert_eq!(resp.action(), None);
    assert!(resp.clicked());
    assert!(
        resp.drag_area_clicked(),
        "Clicking icon must trigger drag area via interaction bubbling"
    );
}