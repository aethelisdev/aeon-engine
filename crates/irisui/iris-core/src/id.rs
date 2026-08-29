// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Type-safe widget identifier definitions based on generational indexing.
//!
//! Provides generationally tracked unique keys for UI tree nodes, eliminating
//! stale reference bugs and raw pointer lifecycle issues without unsafe code.

use slotmap::new_key_type;

new_key_type! {
    /// A unique, generational identifier for an individual UI node in the widget tree.
    /// `WidgetId` is used to index into the central `UiTree` arena without relying
    /// on raw pointers, preventing dangling references and ensuring memory safety.
    pub struct WidgetId;
}