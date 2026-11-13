// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Scope Core Lifecycle & Subtree Layout
//!
//! Provides the core [`UiScope`] struct definition, lifecycle initialization,
//! interaction query evaluation, and recursive layout completion routines.

pub mod context;
pub mod interaction;
pub mod layout;
pub mod tagged_state;

pub use context::UiScope;