// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Text Input Control Subsystem (`controls::text_input`)
//!
//! Subsystem providing modular text editing infrastructure, including undo/redo history stacks,
//! input validation, and editing session lifecycle management.

pub mod undo;

pub use undo::{
    DEFAULT_MAX_UNDO_DEPTH, TEXT_UNDO_COALESCE_TIMEOUT_MS, TextSnapshot, TextUndoStack,
    UndoActionKind, is_word_boundary,
};