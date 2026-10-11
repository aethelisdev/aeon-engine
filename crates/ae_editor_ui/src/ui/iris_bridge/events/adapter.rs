// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Winit WindowEvent to Iris UI Event Adapter
//!
//! Converts raw platform window events ([`winit::event::WindowEvent`]) into
//! hardware-independent [`irisui::prelude::UiEvent`] primitives.
//!

use irisui::prelude::{MouseButton as IrisMouseButton, Point, UiEvent};
use winit::event::{ElementState, MouseButton as WinitMouseButton, MouseScrollDelta, WindowEvent};

/// Translates a Winit window event into an Iris UI framework event.
///
/// Returns `Some(UiEvent)` if the platform event corresponds to an active UI gesture
/// (mouse movement, button press/release, or mouse wheel scroll).
#[inline]
pub fn winit_to_ui_event(event: &WindowEvent, cursor_pos: Point) -> Option<UiEvent> {
    match event {
        WindowEvent::CursorMoved { .. } => Some(UiEvent::MouseMove { point: cursor_pos }),
        WindowEvent::MouseInput { state, button, .. } => {
            let btn = match button {
                WinitMouseButton::Left => IrisMouseButton::Left,
                WinitMouseButton::Right => IrisMouseButton::Right,
                WinitMouseButton::Middle => IrisMouseButton::Middle,
                WinitMouseButton::Other(b) => IrisMouseButton::Other(*b),
                _ => IrisMouseButton::Left,
            };
            match state {
                ElementState::Pressed => Some(UiEvent::MouseDown {
                    button: btn,
                    point: cursor_pos,
                }),
                ElementState::Released => Some(UiEvent::MouseUp {
                    button: btn,
                    point: cursor_pos,
                }),
            }
        }
        WindowEvent::MouseWheel { delta, .. } => {
            let (dx, dy) = match delta {
                MouseScrollDelta::LineDelta(x, y) => (*x, *y),
                MouseScrollDelta::PixelDelta(pos) => (pos.x as f32 / 24.0, pos.y as f32 / 24.0),
            };
            Some(UiEvent::MouseScroll {
                delta_x: dx,
                delta_y: dy,
                point: cursor_pos,
            })
        }
        _ => None,
    }
}