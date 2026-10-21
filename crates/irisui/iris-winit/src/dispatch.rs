// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Stateful Winit Event Dispatch (`iris-winit::dispatch`)
//!
//! Provides the primary unified entry point for routing platform [`WindowEvent`]s into
//! retained-mode [`UiTree`] hierarchies.
//!
//! Coordinates HighDPI scale normalization, cursor position tracking, keyboard modifier
//! updates, and hierarchical interaction bubbling with zero manual host orchestration.
//!

use crate::state::WinitState;
use iris_core::dispatch::DispatchResult;
use iris_core::tree::UiTree;
use winit::event::WindowEvent;

/// Dispatches a platform window event into the retained [`UiTree`].
///
/// Normalizes physical coordinates to logical UI space, updates [`WinitState`]
/// (modifiers, cursor tracking, scale factor), translates the event into a hardware-independent
/// [`UiEvent`](iris_core::event::UiEvent), and executes hierarchical tree dispatch.
///
/// # Arguments
///
/// * `tree` - Target retained-mode UI tree receiving the interaction.
/// * `state` - Stateful window adapter tracking platform cursors, HighDPI, and modifiers.
/// * `event` - Raw platform window event delivered by the windowing event loop.
///
/// # Returns
///
/// A [`DispatchResult`] indicating whether the event was consumed, which widget node was
/// targeted, any requested cursor overrides, and emitted interaction payloads.
pub fn dispatch_winit_event(
    tree: &mut UiTree,
    state: &mut WinitState,
    event: &WindowEvent,
) -> DispatchResult {
    let mut res = if let Some(ui_event) = state.on_window_event(event) {
        tree.dispatch_event(&ui_event)
    } else {
        DispatchResult::ignored()
    };

    if tree.check_hover_timers(std::time::Instant::now()) {
        res.needs_redraw = true;
    }

    res
}

/// Polls active UI timers (such as hover delays for tooltips).
///
/// Intended to be called on timer wakeups or event loop idle passes (e.g. `AboutToWait` in Winit)
/// to detect hover timer expirations without requiring pointer movement.
///
/// Returns a [`DispatchResult`] with `needs_redraw: true` if an autonomous tooltip or UI transition
/// was activated, prompting the host window to request a redraw.
pub fn poll_timers(tree: &mut UiTree, _state: &mut WinitState) -> DispatchResult {
    let now = std::time::Instant::now();
    let woke = tree.check_hover_timers(now);
    if woke {
        DispatchResult {
            consumed: false,
            needs_redraw: true,
            ..DispatchResult::ignored()
        }
    } else {
        DispatchResult::ignored()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use iris_core::geometry::{Point, Rect};
    use iris_core::node::WidgetRole;
    use winit::dpi::PhysicalPosition;
    use winit::event::{DeviceId, ElementState, MouseButton, WindowEvent};

    #[test]
    fn test_dispatch_winit_modifiers_change() {
        let mut tree = UiTree::new();
        let mut state = WinitState::new(1.0);

        let event = WindowEvent::ModifiersChanged(winit::event::Modifiers::default());
        let res = dispatch_winit_event(&mut tree, &mut state, &event);
        assert!(!res.is_consumed());
        assert!(!state.modifiers().shift());
    }

    #[test]
    fn test_dispatch_winit_cursor_moved_and_click() {
        let mut tree = UiTree::new();
        let mut state = WinitState::new(1.0);

        // Add root node
        let root = tree.create_root().expect("Root node created");
        if let Some(r) = tree.get_mut(root) {
            r.computed_rect = Rect::new(0.0, 0.0, 500.0, 500.0);
        }

        // Add an interactive button node at (10, 10, 100, 40)
        let button = tree.create_node();
        if let Some(b) = tree.get_mut(button) {
            b.computed_rect = Rect::new(10.0, 10.0, 100.0, 40.0);
            b.interactive = true;
            b.role = WidgetRole::Button;
            b.tag = 42;
        }
        tree.add_child(root, button).expect("Child added");

        // Move cursor over button
        let move_ev = WindowEvent::CursorMoved {
            device_id: DeviceId::dummy(),
            position: PhysicalPosition::new(50.0, 20.0),
        };
        let _res_move = dispatch_winit_event(&mut tree, &mut state, &move_ev);
        assert_eq!(state.cursor_pos(), Some(Point::new(50.0, 20.0)));

        // Click down on button
        let down_ev = WindowEvent::MouseInput {
            device_id: DeviceId::dummy(),
            state: ElementState::Pressed,
            button: MouseButton::Left,
        };
        let res_down = dispatch_winit_event(&mut tree, &mut state, &down_ev);
        assert!(res_down.is_consumed());
        assert_eq!(res_down.target, Some(button));
        assert_eq!(res_down.target_tag, 42);

        // Mouse up on button
        let up_ev = WindowEvent::MouseInput {
            device_id: DeviceId::dummy(),
            state: ElementState::Released,
            button: MouseButton::Left,
        };
        let res_up = dispatch_winit_event(&mut tree, &mut state, &up_ev);
        assert!(res_up.is_consumed());
    }

    #[test]
    fn test_dispatch_winit_escape_dismisses_popup() {
        use crate::adapter::winit_key_to_iris;
        use iris_core::event::UiEvent;
        use winit::keyboard::{Key, NamedKey};

        let mut tree = UiTree::new();
        let _root = tree.create_root().expect("Root node created");

        tree.set_active_menu(Some(99));
        assert!(tree.has_open_popup());

        // Verify logical Escape key translates accurately to iris KeyCode::Escape
        let (key_code, text) = winit_key_to_iris(&Key::Named(NamedKey::Escape));
        assert_eq!(key_code, iris_core::event::KeyCode::Escape);
        assert_eq!(text, None);

        // Dispatch translated event into the tree
        let ui_event = UiEvent::KeyboardInput {
            key: key_code,
            text,
            is_pressed: true,
        };
        let res = tree.dispatch_event(&ui_event);
        assert!(
            res.is_consumed(),
            "Escape must consume event when popup is open"
        );
        assert!(res.needs_redraw, "Escape dismiss must trigger redraw");
        assert!(!tree.has_open_popup(), "Menu must be closed after Escape");
    }

    #[test]
    fn test_dispatch_winit_outside_click_dismisses_popup() {
        let mut tree = UiTree::new();
        let mut state = WinitState::new(1.0);
        let _root = tree.create_root().expect("Root node created");

        tree.set_active_combobox(Some(101));
        assert!(tree.has_open_popup());

        let outside_click = WindowEvent::MouseInput {
            device_id: DeviceId::dummy(),
            state: ElementState::Pressed,
            button: MouseButton::Left,
        };

        let res = dispatch_winit_event(&mut tree, &mut state, &outside_click);
        assert!(res.is_consumed());
        assert!(res.needs_redraw);
        assert!(
            !tree.has_open_popup(),
            "Combobox must be closed after outside click"
        );
    }

    #[test]
    fn test_dispatch_winit_poll_timers_and_hover() {
        let mut tree = UiTree::new();
        let mut state = WinitState::new(1.0);
        let root = tree.create_root().expect("Root node created");
        let btn = tree.create_node();
        tree.add_child(root, btn).expect("Child added");

        let node = tree.get_mut(btn).expect("Node exists");
        node.visible = true;
        node.interactive = true;
        node.computed_rect = Rect::new(50.0, 50.0, 100.0, 30.0);

        // Move over button
        let move_ev = WindowEvent::CursorMoved {
            device_id: DeviceId::dummy(),
            position: PhysicalPosition::new(60.0, 60.0),
        };
        let res = dispatch_winit_event(&mut tree, &mut state, &move_ev);
        assert!(!res.needs_redraw);

        // Fast forward time past 400ms delay threshold and poll timers
        let target_time = std::time::Instant::now() + std::time::Duration::from_millis(450);
        tree.check_hover_timers(target_time);

        let poll_res = poll_timers(&mut tree, &mut state);
        assert!(tree.has_open_tooltip());
        assert!(!poll_res.is_consumed());
    }
}