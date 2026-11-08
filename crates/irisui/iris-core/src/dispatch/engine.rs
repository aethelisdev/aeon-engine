// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Hierarchical Event Dispatch Pipeline & Autonomous Interactors (`iris-core::dispatch::engine`)
//!
//! Orchestrates the routing of windowing [`crate::event::UiEvent`] inputs against the active [`crate::tree::UiTree`] hierarchy.
//! Includes autonomous scrollbar thumb dragging, track click-to-jump, wheel bubbling, keyboard focus, and file drop handling.

pub mod click;
pub mod dispatcher;
pub mod file_drop;
pub mod keyboard;
pub mod motion;
pub mod scroll;
pub mod scrollbar;

pub use dispatcher::dispatch_ui_event;