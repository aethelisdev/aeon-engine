// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Dropdown and Combobox Subsystem
//!
//! Provides standardized, hardware-accelerated GPU SDF combobox trigger buttons,
//! inspector property rows, and floating popup selection menus for Iris UI.
//!

pub mod button;
pub mod popup;
pub mod row;

pub use button::{ComboboxButtonBuilder, ComboboxButtonFrame, ComboboxButtonStyle};
pub use popup::{ComboboxPopupBuilder, ComboboxPopupFrame, ComboboxPopupStyle};
pub use row::{ComboboxRowBuilder, ComboboxRowFrame, ComboboxRowStyle};