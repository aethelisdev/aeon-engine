// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Aeon Engine Editor UI
//!
//! Declarative Iris UI editor layer.
//!

pub mod assets;
pub mod processor;
pub mod ui;

pub use processor as ui_processor;
pub use processor::UiContext;