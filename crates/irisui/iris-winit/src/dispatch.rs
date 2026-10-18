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
    if let Some(ui_event) = state.on_window_event(event) {
        tree.dispatch_event(&ui_event)
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
}