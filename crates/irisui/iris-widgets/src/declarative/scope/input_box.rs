// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Text Input Box Widget Module Root
//!
//! Provides text input rendering, caret positioning, visible text slicing, and selection highlight
//! primitives on [`crate::declarative::UiScope`] for search inputs, text fields, and inspector inline string editing.

pub mod box_widget;
pub mod interaction;
pub mod rendering;
pub mod types;

pub use interaction::*;
pub use iris_text::measure_text_advance;
pub use types::*;