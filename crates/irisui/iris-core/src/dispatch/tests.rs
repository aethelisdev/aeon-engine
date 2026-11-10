// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Unit Tests for UI Event Dispatch and Autonomous Interactivity (`iris-core::dispatch::tests`)
//!
//! Submodule layout:
//! - [`click`]: Click dispatching, hit-testing, cross-frame tag persistence, and child bubbling.
//! - [`gesture`]: Double click, right/middle click isolation, and drag capture.
//! - [`scroll`]: Wheel scrolling, thumb dragging, track jump, and scroll bubbling.
//! - [`keyboard`]: Keyboard navigation, IME events, and text/numeric input focus isolation.
//! - [`popup`]: Popup menus, outside-click detection, escape dismissal, and autonomous tooltips.
//! - [`file_drop`]: OS file drag-and-drop event dispatching and target boundary resolution.

mod click;
mod file_drop;
mod gesture;
mod keyboard;
mod popup;
mod scroll;