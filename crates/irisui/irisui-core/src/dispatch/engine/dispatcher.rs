// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Root UI Event Dispatch Highway
//!
//! Evaluates incoming [`crate::event::UiEvent`] windowing inputs and coordinates routing to specialized sub-handlers.

use super::click::{handle_mouse_down, handle_mouse_up};
use super::file_drop::{handle_file_cancelled, handle_file_dropped, handle_file_hovered};
use super::keyboard::{handle_ime, handle_keyboard};
use super::motion::handle_mouse_move;
use super::scroll::handle_mouse_scroll;
use crate::dispatch::types::DispatchResult;
use crate::event::UiEvent;
use crate::tree::UiTree;

/// Dispatches an incoming [`UiEvent`] windowing input through the [`UiTree`] hierarchy.
///
/// # Execution Pipeline
/// 1. **Pointer Capture Evaluation**: If pointer capture is active, all pointer
///    events are routed exclusively to the capturing widget, bypassing spatial hit-testing.
/// 2. **Layered Spatial Hit-Testing**: Evaluates pointer coordinates across layer priority
///    stacking contexts with automatic modal window occlusion filtering.
/// 3. **Autonomous Scroll Interactivity**: Automatically handles scrollbar thumb drags,
///    track click-to-jump gestures, and mouse wheel bubbling.
/// 4. **State Machine Transitions**: Synchronizes hover, pressed, click, double-click,
///    and keyboard focus transitions on [`crate::dispatch::types::EventState`].
pub fn dispatch_ui_event(tree: &mut UiTree, event: &UiEvent) -> DispatchResult {
    // Clear transient frame click markers when starting new gestures or non-pointer events,
    // preserving clicks across immediate micro-motion until the frame finishes or a new press occurs
    if matches!(
        event,
        UiEvent::MouseDown { .. } | UiEvent::MouseScroll { .. }
    ) {
        tree.event_state.frame_clicked_id = None;
        tree.event_state.frame_clicked_tag = None;
        tree.event_state.frame_double_clicked_id = None;
        tree.event_state.frame_double_clicked_tag = None;
        tree.event_state.frame_secondary_clicked_id = None;
        tree.event_state.frame_secondary_clicked_tag = None;
        tree.event_state.frame_drag_released_id = None;
        tree.event_state.frame_drag_released_tag = None;
        tree.event_state.is_pointer_drag_active = false;
        tree.event_state.pointer_capture = None;
        tree.event_state.pointer_capture_tag = None;
        tree.event_state.pressed_tag = None;
        tree.event_state.frame_clicked_pos = None;
        tree.event_state.frame_clicked_rect = None;
    }

    match event {
        UiEvent::MouseMove { point } => handle_mouse_move(tree, *point),
        UiEvent::MouseDown { button, point } => handle_mouse_down(tree, *button, *point),
        UiEvent::MouseUp { button, point } => handle_mouse_up(tree, *button, *point),
        UiEvent::MouseScroll {
            delta_x,
            delta_y,
            point,
        } => handle_mouse_scroll(tree, *delta_x, *delta_y, *point),
        UiEvent::KeyboardInput {
            key,
            text,
            is_pressed,
            ctrl_held,
            shift_held,
        } => handle_keyboard(
            tree,
            *key,
            text.clone(),
            *is_pressed,
            *ctrl_held,
            *shift_held,
        ),
        UiEvent::Ime(ime_event) => handle_ime(tree, ime_event.clone()),
        UiEvent::FileHovered { point, path } => handle_file_hovered(tree, *point, path),
        UiEvent::FileDropped { point, paths } => handle_file_dropped(tree, *point, paths),
        UiEvent::FileCancelled => handle_file_cancelled(tree),
    }
}