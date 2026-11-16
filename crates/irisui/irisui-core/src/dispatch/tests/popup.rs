// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Unit tests for popup menus, outside-click detection, escape dismissal, and autonomous tooltips.

use crate::event::{KeyCode, MouseButton, UiEvent};
use crate::geometry::{Point, Rect};
use crate::node::{UiLayer, WidgetRole};
use crate::tree::UiTree;

#[test]
fn test_autonomous_outside_click_dismisses_popups() {
    let mut tree = UiTree::new();
    let root = tree.create_root().unwrap();
    let btn = tree.create_node();
    tree.add_child(root, btn).unwrap();
    {
        let b = tree.get_mut(btn).unwrap();
        b.visible = true;
        b.interactive = true;
        b.role = WidgetRole::Button;
        b.computed_rect = Rect::new(10.0, 10.0, 80.0, 30.0);
        b.tag = 100;
    }

    // 1. Open both menu bar and combobox popups
    tree.set_active_menu(Some(50));
    tree.set_active_combobox(Some(60));
    assert!(tree.has_open_popup());

    // 2. Click outside (on empty canvas area)
    let outside_click = UiEvent::MouseDown {
        button: MouseButton::Left,
        point: Point::new(400.0, 400.0),
    };
    let res = tree.dispatch_event(&outside_click);
    assert!(res.is_consumed());
    assert!(res.needs_redraw);
    assert!(!tree.has_open_popup());
    assert_eq!(tree.active_menu_bar_tag, None);
    assert_eq!(tree.active_combobox_tag, None);
}

#[test]
fn test_outside_click_detector_swallows_event_and_protects_background_widgets() {
    let mut tree = UiTree::new();
    let root = tree.create_root().unwrap();
    let btn = tree.create_node();
    tree.add_child(root, btn).unwrap();
    {
        let b = tree.get_mut(btn).unwrap();
        b.visible = true;
        b.interactive = true;
        b.role = WidgetRole::Button;
        b.computed_rect = Rect::new(10.0, 10.0, 80.0, 30.0);
        b.tag = 100;
    }

    let popup_node = tree.create_node();
    tree.add_child(root, popup_node).unwrap();
    {
        let p = tree.get_mut(popup_node).unwrap();
        p.visible = true;
        p.interactive = true;
        p.role = WidgetRole::DropdownPopup;
        p.layer = UiLayer::Popup;
        p.computed_rect = Rect::new(100.0, 100.0, 120.0, 100.0);
        p.tag = 200;
    }

    tree.set_active_combobox(Some(200));
    assert!(tree.has_open_popup());

    // 1. Point inside popup returns false for outside click detector
    assert!(tree.is_point_inside_popup(Point::new(150.0, 150.0)));

    // 2. Click directly on background button while popup is open:
    // Should dismiss popup and swallow the click (NOT press the button!)
    let click_on_btn = UiEvent::MouseDown {
        button: MouseButton::Left,
        point: Point::new(30.0, 20.0),
    };
    let res = tree.dispatch_event(&click_on_btn);

    assert!(res.is_consumed());
    assert!(res.needs_redraw);
    assert_eq!(res.target, None);
    assert!(!tree.has_open_popup());
    assert_eq!(tree.event_state.pressed, None);

    // 3. Releasing mouse should NOT trigger button click
    let release_event = UiEvent::MouseUp {
        button: MouseButton::Left,
        point: Point::new(30.0, 20.0),
    };
    let release_res = tree.dispatch_event(&release_event);
    assert!(!release_res.is_consumed());
    assert_eq!(tree.event_state.frame_clicked_tag, None);
    assert_eq!(tree.event_state.frame_clicked_id, None);
}

#[test]
fn test_autonomous_click_inside_popup_retains_open_state() {
    let mut tree = UiTree::new();
    let root = tree.create_root().unwrap();

    let popup_node = tree.create_node();
    tree.add_child(root, popup_node).unwrap();
    {
        let p = tree.get_mut(popup_node).unwrap();
        p.visible = true;
        p.interactive = true;
        p.role = WidgetRole::DropdownPopup;
        p.layer = UiLayer::Popup;
        p.computed_rect = Rect::new(100.0, 50.0, 150.0, 200.0);
        p.tag = 777;
    }

    let popup_item = tree.create_node();
    tree.add_child(popup_node, popup_item).unwrap();
    {
        let pi = tree.get_mut(popup_item).unwrap();
        pi.visible = true;
        pi.interactive = true;
        pi.role = WidgetRole::Button;
        pi.computed_rect = Rect::new(105.0, 55.0, 140.0, 24.0);
        pi.tag = 778;
    }

    tree.set_active_combobox(Some(777));
    assert!(tree.has_open_popup());

    // Click inside the popup item
    let inside_click = UiEvent::MouseDown {
        button: MouseButton::Left,
        point: Point::new(120.0, 65.0),
    };
    let res = tree.dispatch_event(&inside_click);
    assert!(res.is_consumed());
    // Combobox state must NOT be cleared by clicks inside
    assert!(tree.is_combobox_open(777));
}

#[test]
fn test_autonomous_escape_key_dismisses_all_popups() {
    let mut tree = UiTree::new();
    let _root = tree.create_root().unwrap();

    tree.set_active_menu(Some(42));
    tree.set_active_combobox(Some(84));
    assert!(tree.has_open_popup());

    let esc_ev = UiEvent::KeyboardInput {
        key: KeyCode::Escape,
        text: None,
        is_pressed: true,
        ctrl_held: false,
        shift_held: false,
    };
    let res = tree.dispatch_event(&esc_ev);

    assert!(
        res.is_consumed(),
        "Escape must consume the event when a popup is active"
    );
    assert!(res.needs_redraw, "Escape dismiss must request redraw");
    assert!(
        !tree.has_open_popup(),
        "All popups must be closed on Escape"
    );
    assert_eq!(tree.active_menu_bar_tag, None);
    assert_eq!(tree.active_combobox_tag, None);
}

#[test]
fn test_tooltip_hover_timer_and_activation() {
    let mut tree = UiTree::new();
    let root = tree.create_root().unwrap();
    let btn = tree.create_node();
    tree.add_child(root, btn).unwrap();

    let btn_node = tree.get_mut(btn).unwrap();
    btn_node.visible = true;
    btn_node.interactive = true;
    btn_node.computed_rect = Rect::new(50.0, 50.0, 100.0, 30.0);
    btn_node.tag = 101;

    // 1. Move cursor over button -> Hover initiated, tooltip pending
    let move_ev = UiEvent::MouseMove {
        point: Point::new(60.0, 60.0),
    };
    let res = tree.dispatch_event(&move_ev);
    assert!(!res.needs_redraw);
    assert_eq!(res.target, Some(btn));
    assert!(res.next_timeout.is_some());
    assert!(!tree.has_open_tooltip());

    // 2. Before delay expires: tooltip is not open
    assert!(!tree.is_tooltip_open_for(btn, 101));

    // 3. Time advances past 400ms delay threshold
    let future_time = std::time::Instant::now() + std::time::Duration::from_millis(450);
    let woke = tree.check_hover_timers(future_time);
    assert!(
        woke,
        "check_hover_timers must return true on transition to open"
    );
    assert!(tree.has_open_tooltip());
    assert!(tree.is_tooltip_open_for(btn, 101));
}

#[test]
fn test_tooltip_dismiss_on_mouse_down_and_escape() {
    let mut tree = UiTree::new();
    let root = tree.create_root().unwrap();
    let btn = tree.create_node();
    tree.add_child(root, btn).unwrap();

    let btn_node = tree.get_mut(btn).unwrap();
    btn_node.visible = true;
    btn_node.interactive = true;
    btn_node.computed_rect = Rect::new(10.0, 10.0, 80.0, 24.0);
    btn_node.tag = 202;

    // Move over button and force open tooltip
    let _ = tree.dispatch_event(&UiEvent::MouseMove {
        point: Point::new(20.0, 20.0),
    });
    let future_time = std::time::Instant::now() + std::time::Duration::from_millis(450);
    tree.check_hover_timers(future_time);
    assert!(tree.has_open_tooltip());

    // MouseDown immediately dismisses tooltip
    let down_ev = UiEvent::MouseDown {
        button: MouseButton::Left,
        point: Point::new(20.0, 20.0),
    };
    let _ = tree.dispatch_event(&down_ev);
    assert!(!tree.has_open_tooltip(), "MouseDown must dismiss tooltip");

    // MouseUp release
    let up_ev = UiEvent::MouseUp {
        button: MouseButton::Left,
        point: Point::new(20.0, 20.0),
    };
    let _ = tree.dispatch_event(&up_ev);

    // Re-hover and force open again
    let _ = tree.dispatch_event(&UiEvent::MouseMove {
        point: Point::new(25.0, 22.0),
    });
    tree.check_hover_timers(std::time::Instant::now() + std::time::Duration::from_millis(450));
    assert!(tree.has_open_tooltip());

    // Escape dismisses tooltip
    let esc_ev = UiEvent::KeyboardInput {
        key: KeyCode::Escape,
        text: None,
        is_pressed: true,
        ctrl_held: false,
        shift_held: false,
    };
    let res = tree.dispatch_event(&esc_ev);
    assert!(res.is_consumed());
    assert!(res.needs_redraw);
    assert!(!tree.has_open_tooltip(), "Escape must dismiss tooltip");
}

#[test]
fn test_tooltip_closes_when_mouse_leaves() {
    let mut tree = UiTree::new();
    let root = tree.create_root().unwrap();
    let btn = tree.create_node();
    tree.add_child(root, btn).unwrap();

    let btn_node = tree.get_mut(btn).unwrap();
    btn_node.visible = true;
    btn_node.interactive = true;
    btn_node.computed_rect = Rect::new(0.0, 0.0, 50.0, 50.0);

    // Hover button
    let _ = tree.dispatch_event(&UiEvent::MouseMove {
        point: Point::new(25.0, 25.0),
    });
    tree.check_hover_timers(std::time::Instant::now() + std::time::Duration::from_millis(450));
    assert!(tree.has_open_tooltip());

    // Move mouse out to empty space -> Tooltip closed and redraw requested
    let move_out = UiEvent::MouseMove {
        point: Point::new(200.0, 200.0),
    };
    let res = tree.dispatch_event(&move_out);
    assert!(
        res.needs_redraw,
        "Exiting hover target must trigger redraw to erase tooltip"
    );
    assert!(!tree.has_open_tooltip());
}

#[test]
fn test_dismiss_popups_on_outside_click_with_active_popup_tags() {
    let mut tree = UiTree::new();
    let root = tree.create_root().unwrap();
    let trigger_btn = tree.create_node();
    let child_text = tree.create_node();
    tree.add_child(root, trigger_btn).unwrap();
    tree.add_child(trigger_btn, child_text).unwrap();

    {
        let btn = tree.get_mut(trigger_btn).unwrap();
        btn.visible = true;
        btn.interactive = true;
        btn.role = WidgetRole::Button;
        btn.computed_rect = Rect::new(50.0, 50.0, 150.0, 36.0);
        btn.tag = 5005;
    }
    {
        let txt = tree.get_mut(child_text).unwrap();
        txt.visible = true;
        txt.tag = 0; // Untagged descendant
        txt.computed_rect = Rect::new(60.0, 55.0, 80.0, 20.0);
    }

    // Register active popup tag
    tree.set_popup_tag_active(5005, true);
    assert!(tree.is_popup_tag_active(5005));
    assert!(tree.has_open_popup());

    // 1. Click on child text inside trigger button resolves ancestor tag and does NOT dismiss
    let child_click = Point::new(70.0, 60.0);
    assert!(tree.is_point_inside_popup(child_click));
    assert!(!tree.dismiss_popups_on_outside_click(child_click));
    assert!(tree.is_popup_tag_active(5005));

    // 2. Click outside triggers dismissal and clears active popup tag
    let outside_click = Point::new(600.0, 600.0);
    assert!(!tree.is_point_inside_popup(outside_click));
    assert!(tree.dismiss_popups_on_outside_click(outside_click));
    assert!(!tree.is_popup_tag_active(5005));
    assert!(!tree.has_open_popup());
}

#[test]
fn test_close_all_popups_clears_every_popup_channel_and_portal_children() {
    use crate::dispatch::{PopupMenuState, TooltipState};

    let mut tree = UiTree::new();
    let root = tree.create_root().unwrap();

    // 1. Attach anonymous root popup portal + child card
    let portal = tree.attach_layer_portal(root, UiLayer::Popup).unwrap();
    let portal_card = tree.create_layer_node(UiLayer::Popup);
    tree.add_child(portal, portal_card).unwrap();
    if let Some(n) = tree.get_mut(portal_card) {
        n.computed_rect = Rect::new(50.0, 50.0, 120.0, 80.0);
    }

    // 2. Attach direct DropdownPopup node
    let dropdown_node = tree.create_node();
    tree.add_child(root, dropdown_node).unwrap();
    if let Some(n) = tree.get_mut(dropdown_node) {
        n.layer = UiLayer::Popup;
        n.role = WidgetRole::DropdownPopup;
        n.computed_rect = Rect::new(200.0, 50.0, 140.0, 90.0);
    }

    // 3. Populate every popup state channel
    tree.set_active_menu(Some(11));
    tree.set_active_combobox(Some(22));
    tree.open_popup(33);
    tree.open_popup(44);
    tree.set_active_submenu(33, 55);
    tree.event_state.active_context_menu = Some(PopupMenuState {
        owner_id: root,
        owner_tag: 66,
        anchor_pos: Point::new(10.0, 10.0),
        container_id: Some(dropdown_node),
    });
    tree.event_state.active_tooltip = Some(TooltipState {
        target_id: root,
        target_tag: 77,
        anchor_pos: Point::new(10.0, 20.0),
        hover_start: std::time::Instant::now(),
        delay: std::time::Duration::ZERO,
        is_open: true,
    });

    assert!(tree.has_open_popup());
    assert!(tree.has_open_tooltip());
    assert!(tree.has_active_popup_nodes());

    // 4. close_all_popups clears every popup channel atomically
    assert!(tree.close_all_popups());
    assert!(tree.popups_dismissed());
    assert!(!tree.has_open_popup());
    assert!(!tree.has_open_tooltip());
    assert!(!tree.has_active_popup_nodes());
    assert_eq!(tree.active_menu_bar_tag, None);
    assert_eq!(tree.active_combobox_tag, None);
    assert!(tree.active_popup_tags.is_empty());
    assert!(tree.active_submenus.is_empty());
    assert!(tree.event_state.active_context_menu.is_none());
    assert!(!tree.get(dropdown_node).unwrap().visible);
    assert!(!tree.get(portal_card).unwrap().visible);
    assert!(
        tree.get(portal).unwrap().visible,
        "Anonymous root portal anchor stays reusable while its children are hidden"
    );

    // 5. Idempotent second call returns false
    assert!(!tree.close_all_popups());
}

#[test]
fn test_popup_tag_lifecycle_and_exclusive_switching() {
    let mut tree = UiTree::new();
    let _root = tree.create_root().unwrap();

    // Zero tag is safely ignored
    tree.open_popup(0);
    assert!(!tree.toggle_popup(0));
    assert!(!tree.toggle_exclusive_popup(0));
    assert!(!tree.is_popup_tag_active(0));
    assert!(!tree.has_open_popup());

    // Toggle open / close
    assert!(tree.toggle_popup(1001));
    assert!(tree.is_popup_tag_active(1001));
    assert!(!tree.toggle_popup(1001));
    assert!(!tree.is_popup_tag_active(1001));

    // Exclusive switching closes previous popup tag and submenus
    tree.open_popup(1001);
    tree.set_active_submenu(1001, 2001);
    assert!(tree.toggle_exclusive_popup(1002));
    assert!(!tree.is_popup_tag_active(1001));
    assert!(tree.active_submenus.is_empty());
    assert!(tree.is_popup_tag_active(1002));

    // Exclusive re-toggle closes active popup
    assert!(!tree.toggle_exclusive_popup(1002));
    assert!(!tree.has_open_popup());
}