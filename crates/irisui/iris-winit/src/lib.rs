// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Iris Winit Adapter (`iris-winit`)
//!
//! Bridges native platform windowing ([`winit`]) with the Iris UI retained-mode
//! framework ([`iris-core`]).
//!
//! Provides hardware-independent event translation, HighDPI scaling, cursor icon mapping,
//! keyboard modifier tracking, and stateful window event handling.
//!
//! Adheres strictly to a zero-unsafe policy (`#![forbid(unsafe_code)]`).

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod adapter;
pub mod cursor;
pub mod dispatch;
pub mod modifiers;
pub mod state;
pub mod text_input;

#[cfg(test)]
mod tests;

pub use adapter::{
    winit_ime_to_iris, winit_key_to_iris, winit_mouse_button_to_iris, winit_to_ui_event,
};
pub use cursor::{widget_cursor_to_winit, winit_to_widget_cursor};
pub use dispatch::{dispatch_winit_event, poll_timers};
pub use modifiers::ModifiersState;
pub use state::WinitState;
pub use text_input::{
    TextInputActionResult, handle_focused_text_input, handle_key_stroke, handle_text_input_event,
};