// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Hierarchical UI Event Dispatcher & Interaction State (`iris-core::dispatch`)
//!
//! Provides the central [`dispatch_ui_event`] engine and [`DispatchResult`] interaction models.
//!
//! Evaluates incoming [`crate::event::UiEvent`] windowing inputs against the active [`crate::tree::UiTree`] arena,
//! coordinating z-order layer priority ([`crate::node::UiLayer`]), GPU scissor clipping boundaries,
//! pointer capture for uninterrupted drags, keyboard focus routing, and autonomous scrollbar/wheel bubbling.
//!

pub mod engine;
pub mod types;

#[cfg(test)]
mod tests;

pub use engine::dispatch_ui_event;
pub use types::{
    DOUBLE_CLICK_MAX_DELAY_MS, DOUBLE_CLICK_MAX_DISTANCE_SQ, DispatchResult, EventState,
    PopupMenuState,
};