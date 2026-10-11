// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Native Dock Event Handling & Interaction Engines
//!
//! Sub-system event dispatchers managing splitter dividers, tab headers,
//! drag-and-drop operations, and floating windows.
//!

pub mod splitter;
pub mod tab_strip;

pub use splitter::{SplitterInteractionResponse, handle_splitter_event};
pub use tab_strip::{TabStripInteractionResponse, handle_tab_strip_event};