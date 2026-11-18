// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

use super::*;
use irisui_core::{CanvasPrimitive, Color, Point, Rect, UiTree};

#[test]
fn test_collect_text_sections_basic() {
    let mut tree = UiTree::new();
    let root = tree.create_root().unwrap();
    if let Some(node) = tree.get_mut(root) {
        node.computed_rect = Rect::new(0.0, 0.0, 800.0, 600.0);
    }

    let label = tree.create_node();
    if let Some(node) = tree.get_mut(label) {
        node.computed_rect = Rect::new(10.0, 10.0, 100.0, 24.0);
        node.text = Some("Hello World".to_owned());
    }
    tree.add_child(root, label).unwrap();

    let sections = collect_text_sections(&tree);
    assert_eq!(sections.len(), 1);
    assert_eq!(sections[0].text, "Hello World");
}

#[test]
fn test_collect_text_occlusion_culling() {
    let mut tree = UiTree::new();
    let root = tree.create_root().unwrap();
    if let Some(node) = tree.get_mut(root) {
        node.computed_rect = Rect::new(0.0, 0.0, 800.0, 600.0);
    }

    // Background label at (50, 50, 100, 30) on Content layer
    let bg_label = tree.create_node();
    if let Some(node) = tree.get_mut(bg_label) {
        node.computed_rect = Rect::new(50.0, 50.0, 100.0, 30.0);
        node.text = Some("Occluded Text".to_owned());
        node.layer = UiLayer::Content;
    }
    tree.add_child(root, bg_label).unwrap();

    // Popup covering (40, 40, 150, 100) on Popup layer
    let popup = tree.create_node();
    if let Some(node) = tree.get_mut(popup) {
        node.computed_rect = Rect::new(40.0, 40.0, 150.0, 100.0);
        node.layer = UiLayer::Popup;
        node.style.background_color = Color::BLACK; // Opaque
    }
    tree.add_child(root, popup).unwrap();

    // Popup label inside the popup
    let popup_label = tree.create_node();
    if let Some(node) = tree.get_mut(popup_label) {
        node.computed_rect = Rect::new(50.0, 60.0, 100.0, 20.0);
        node.text = Some("Popup Text".to_owned());
        node.layer = UiLayer::Popup;
    }
    tree.add_child(popup, popup_label).unwrap();

    let sections = collect_text_sections(&tree);
    // The background label MUST be culled completely, only popup_label should survive!
    assert_eq!(sections.len(), 1);
    assert_eq!(sections[0].text, "Popup Text");
}

#[test]
fn test_collect_text_scissor_clipping_culling() {
    let mut tree = UiTree::new();
    let root = tree.create_root().unwrap();
    if let Some(node) = tree.get_mut(root) {
        node.computed_rect = Rect::new(0.0, 0.0, 800.0, 600.0);
        node.layer = UiLayer::Background;
    }

    // Panel with clip_children: true, bounded at (0, 0, 400, 200)
    let panel = tree.create_node();
    if let Some(node) = tree.get_mut(panel) {
        node.computed_rect = Rect::new(0.0, 0.0, 400.0, 200.0);
        node.style.clip_children = true;
        node.layer = UiLayer::Content;
    }
    tree.add_child(root, panel).unwrap();

    // Label 1 inside panel (y = 50)
    let label_inside = tree.create_node();
    if let Some(node) = tree.get_mut(label_inside) {
        node.computed_rect = Rect::new(10.0, 50.0, 100.0, 20.0);
        node.text = Some("Inside Panel".to_owned());
        node.layer = UiLayer::Content;
    }
    tree.add_child(panel, label_inside).unwrap();

    // Label 2 overflowing below panel (y = 250, outside panel height 200)
    let label_overflow = tree.create_node();
    if let Some(node) = tree.get_mut(label_overflow) {
        node.computed_rect = Rect::new(10.0, 250.0, 100.0, 20.0);
        node.text = Some("Bleeding Text Outside".to_owned());
        node.layer = UiLayer::Content;
    }
    tree.add_child(panel, label_overflow).unwrap();

    let sections = collect_text_sections(&tree);
    assert_eq!(sections.len(), 1);
    assert_eq!(sections[0].text, "Inside Panel");
}

#[test]
fn test_collect_text_hover_color_in_place() {
    let mut tree = UiTree::new();
    let root = tree.create_root().unwrap();
    let btn_tag = 12345u64;

    let btn = tree.create_node();
    if let Some(node) = tree.get_mut(btn) {
        node.computed_rect = Rect::new(0.0, 0.0, 100.0, 30.0);
        node.tag = btn_tag;
        node.layer = UiLayer::Content;
    }
    tree.add_child(root, btn).unwrap();

    let label = tree.create_node();
    if let Some(node) = tree.get_mut(label) {
        node.computed_rect = Rect::new(10.0, 5.0, 80.0, 20.0);
        node.text = Some("Click Me".to_owned());
        node.text_color = Color::BLACK;
        node.hover_text_color = Some(Color::WHITE);
        node.layer = UiLayer::Content;
    }
    tree.add_child(btn, label).unwrap();

    // 1. Unhovered collection
    let normal_sections = collect_text_sections(&tree);
    assert_eq!(normal_sections.len(), 1);
    assert_eq!(normal_sections[0].color, Color::BLACK);

    // 2. Hovered collection
    let hover_opts = TextCollectionOptions::default().with_hovered_tag(Some(btn_tag));
    let hover_sections = collect_text_sections_with_options(&tree, &hover_opts);
    assert_eq!(hover_sections.len(), 1);
    assert_eq!(hover_sections[0].color, Color::WHITE);
}

#[test]
fn test_collect_text_focus_and_caret_in_place() {
    let mut tree = UiTree::new();
    let root = tree.create_root().unwrap();
    let input_tag = 9999u64;

    let input_box = tree.create_node();
    if let Some(node) = tree.get_mut(input_box) {
        node.computed_rect = Rect::new(0.0, 0.0, 200.0, 30.0);
        node.tag = input_tag;
        node.role = WidgetRole::TextInput;
        node.layer = UiLayer::Content;
    }
    tree.add_child(root, input_box).unwrap();

    let text_node = tree.create_node();
    if let Some(node) = tree.get_mut(text_node) {
        node.computed_rect = Rect::new(10.0, 5.0, 100.0, 20.0);
        node.text = Some("Search query".to_owned());
        node.text_color = Color::rgb(0.5, 0.5, 0.5);
        node.focus_text_color = Some(Color::WHITE);
        node.layer = UiLayer::Content;
    }
    tree.add_child(input_box, text_node).unwrap();

    let caret_node = tree.create_node();
    if let Some(node) = tree.get_mut(caret_node) {
        node.computed_rect = Rect::new(115.0, 5.0, 6.0, 20.0);
        node.text = Some("|".to_owned());
        node.text_color = Color::CYAN;
        node.role = WidgetRole::TextInputCaret;
        node.layer = UiLayer::Content;
    }
    tree.add_child(input_box, caret_node).unwrap();

    // 1. Unfocused collection -> text is gray, caret is NOT collected
    let unfocused_sections = collect_text_sections(&tree);
    assert_eq!(unfocused_sections.len(), 1);
    assert_eq!(unfocused_sections[0].color, Color::rgb(0.5, 0.5, 0.5));
    assert_eq!(unfocused_sections[0].text, "Search query");

    // 2. Focused collection with blink on -> text is WHITE, caret IS collected
    let focused_opts = TextCollectionOptions::default()
        .with_focused_tag(Some(input_tag))
        .with_blink_caret(true);
    let focused_sections = collect_text_sections_with_options(&tree, &focused_opts);
    assert_eq!(focused_sections.len(), 2);
    assert_eq!(focused_sections[0].color, Color::WHITE);
    assert_eq!(focused_sections[1].text, "|");

    // 3. Focused collection with blink off -> caret is hidden
    let blink_off_opts = TextCollectionOptions::default()
        .with_focused_tag(Some(input_tag))
        .with_blink_caret(false);
    let blink_off_sections = collect_text_sections_with_options(&tree, &blink_off_opts);
    assert_eq!(blink_off_sections.len(), 1);
}

#[test]
fn test_collect_text_layer_sorting_order() {
    let mut tree = UiTree::new();
    let root = tree.create_root().unwrap();
    if let Some(node) = tree.get_mut(root) {
        node.computed_rect = Rect::new(0.0, 0.0, 800.0, 600.0);
        node.layer = UiLayer::Background;
    }

    // Add nodes in reverse/scrambled layer order: Tooltip, Modal, Content, Popup, Background
    let tooltip_node = tree.create_node();
    if let Some(node) = tree.get_mut(tooltip_node) {
        node.computed_rect = Rect::new(10.0, 10.0, 100.0, 20.0);
        node.text = Some("Tooltip Text".to_owned());
        node.layer = UiLayer::Tooltip;
    }
    tree.add_child(root, tooltip_node).unwrap();

    let modal_node = tree.create_node();
    if let Some(node) = tree.get_mut(modal_node) {
        node.computed_rect = Rect::new(10.0, 40.0, 100.0, 20.0);
        node.text = Some("Modal Text".to_owned());
        node.layer = UiLayer::Modal;
    }
    tree.add_child(root, modal_node).unwrap();

    let content_node = tree.create_node();
    if let Some(node) = tree.get_mut(content_node) {
        node.computed_rect = Rect::new(10.0, 70.0, 100.0, 20.0);
        node.text = Some("Content Text".to_owned());
        node.layer = UiLayer::Content;
    }
    tree.add_child(root, content_node).unwrap();

    let popup_node = tree.create_node();
    if let Some(node) = tree.get_mut(popup_node) {
        node.computed_rect = Rect::new(10.0, 100.0, 100.0, 20.0);
        node.text = Some("Popup Text".to_owned());
        node.layer = UiLayer::Popup;
    }
    tree.add_child(root, popup_node).unwrap();

    let bg_node = tree.create_node();
    if let Some(node) = tree.get_mut(bg_node) {
        node.computed_rect = Rect::new(10.0, 130.0, 100.0, 20.0);
        node.text = Some("Background Text".to_owned());
        node.layer = UiLayer::Background;
    }
    tree.add_child(root, bg_node).unwrap();

    let sections = collect_text_sections(&tree);
    assert_eq!(sections.len(), 5);
    // Strict layer elevation ordering: Background (0) -> Content (1) -> Modal (3) -> Popup (4) -> Tooltip (5)
    assert_eq!(sections[0].text, "Background Text");
    assert_eq!(sections[0].layer, UiLayer::Background);

    assert_eq!(sections[1].text, "Content Text");
    assert_eq!(sections[1].layer, UiLayer::Content);

    assert_eq!(sections[2].text, "Modal Text");
    assert_eq!(sections[2].layer, UiLayer::Modal);

    assert_eq!(sections[3].text, "Popup Text");
    assert_eq!(sections[3].layer, UiLayer::Popup);

    assert_eq!(sections[4].text, "Tooltip Text");
    assert_eq!(sections[4].layer, UiLayer::Tooltip);
}

#[test]
fn test_collect_text_scissor_bypass_for_elevated_layer() {
    let mut tree = UiTree::new();
    let root = tree.create_root().unwrap();
    if let Some(node) = tree.get_mut(root) {
        node.computed_rect = Rect::new(0.0, 0.0, 800.0, 600.0);
        node.layer = UiLayer::Background;
    }

    // Panel bounded at (0, 0, 300, 300) with clip_children = true
    let panel = tree.create_node();
    if let Some(node) = tree.get_mut(panel) {
        node.computed_rect = Rect::new(0.0, 0.0, 300.0, 300.0);
        node.style.clip_children = true;
        node.layer = UiLayer::Content;
    }
    tree.add_child(root, panel).unwrap();

    // Popup placed at (250, 250, 200, 150) - sticking out to (450, 400)
    let popup = tree.create_node();
    if let Some(node) = tree.get_mut(popup) {
        node.computed_rect = Rect::new(250.0, 250.0, 200.0, 150.0);
        node.role = WidgetRole::DropdownPopup;
        node.layer = UiLayer::Popup;
    }
    tree.add_child(panel, popup).unwrap();

    // Label inside popup placed at (320, 320, 100, 20) - entirely outside parent panel bounds!
    let popup_label = tree.create_node();
    if let Some(node) = tree.get_mut(popup_label) {
        node.computed_rect = Rect::new(320.0, 320.0, 100.0, 20.0);
        node.text = Some("Flyout Label".to_owned());
        node.layer = UiLayer::Popup;
    }
    tree.add_child(popup, popup_label).unwrap();

    let sections = collect_text_sections(&tree);
    assert_eq!(sections.len(), 1);
    assert_eq!(sections[0].text, "Flyout Label");
    // Scissor clip must be None (decoupled from panel), NOT clipped by parent panel (0, 0, 300, 300)
    assert_eq!(sections[0].clip_bounds, None);
}

#[test]
fn test_collect_split_and_volatility_filtered_text_sections() {
    let mut tree = UiTree::new();
    let root = tree.create_root().unwrap();
    if let Some(n) = tree.get_mut(root) {
        n.computed_rect = Rect::new(0.0, 0.0, 800.0, 600.0);
    }

    let static_label = tree.create_node();
    if let Some(n) = tree.get_mut(static_label) {
        n.computed_rect = Rect::new(10.0, 10.0, 120.0, 20.0);
        n.text = Some("Static Header".to_owned());
        n.layer = UiLayer::Content;
    }
    tree.add_child(root, static_label).unwrap();

    let live_label = tree.create_node();
    let live_tag = 0x7777;
    if let Some(n) = tree.get_mut(live_label) {
        n.computed_rect = Rect::new(10.0, 40.0, 120.0, 20.0);
        n.tag = live_tag;
        n.text = Some("120 FPS".to_owned());
        n.layer = UiLayer::Content;
    }
    tree.add_child(root, live_label).unwrap();
    tree.register_live_tag(live_tag);

    let canvas = tree.create_node();
    if let Some(n) = tree.get_mut(canvas) {
        n.computed_rect = Rect::new(10.0, 70.0, 200.0, 30.0);
        n.layer = UiLayer::Content;
    }
    tree.add_child(root, canvas).unwrap();
    tree.add_canvas_primitive(
        canvas,
        CanvasPrimitive::Text {
            pos: Point::new(12.0, 72.0),
            text: "Pos: 0.0".to_owned(),
            font_size: 12.0,
            color: Color::WHITE,
        },
    );

    let (static_only, volatile_only) =
        collect_split_text_sections_with_options(&tree, &TextCollectionOptions::default());
    assert_eq!(static_only.len(), 1);
    assert_eq!(static_only[0].text, "Static Header");
    assert_eq!(volatile_only.len(), 2);
    assert_eq!(volatile_only[0].text, "120 FPS");
    assert_eq!(volatile_only[1].text, "Pos: 0.0");

    let filter_static = collect_text_sections_with_options(
        &tree,
        &TextCollectionOptions::default().with_volatility(TextVolatilityFilter::StaticOnly),
    );
    assert_eq!(filter_static.len(), 1);
    assert_eq!(filter_static[0].text, "Static Header");

    let filter_volatile = collect_text_sections_with_options(
        &tree,
        &TextCollectionOptions::default().with_volatility(TextVolatilityFilter::VolatileOnly),
    );
    assert_eq!(filter_volatile.len(), 2);
}