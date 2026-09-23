// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Interaction Types & Hashing Primitives (`iris-widgets::declarative::types`)
//!
//! Exposes widget interaction responses and persistent tag hashing utilities for declarative UI scopes.
//!

use iris_core::WidgetId;

/// Encapsulates the interaction result of an emitted widget, allowing immediate
/// consumption of user interactions in modern declarative style (`if ui.button("Save").clicked()`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct WidgetResponse {
    /// The unique allocated identifier of the emitted widget node in the UI arena.
    pub id: WidgetId,
    /// Whether the widget was clicked by the user during this interaction pass.
    pub clicked: bool,
    /// Whether the pointer cursor is currently hovering over this widget.
    pub hovered: bool,
    /// Whether the bound property value was modified during this interaction pass.
    pub changed: bool,
}

impl WidgetResponse {
    /// Constructs a new widget response with explicit state flags.
    #[inline]
    pub const fn new(id: WidgetId, clicked: bool, hovered: bool, changed: bool) -> Self {
        Self {
            id,
            clicked,
            hovered,
            changed,
        }
    }

    /// Returns `true` if the widget was clicked during this frame.
    #[inline]
    pub const fn clicked(&self) -> bool {
        self.clicked
    }

    /// Returns `true` if the widget is currently hovered.
    #[inline]
    pub const fn hovered(&self) -> bool {
        self.hovered
    }

    /// Returns `true` if the underlying value was changed during this frame.
    #[inline]
    pub const fn changed(&self) -> bool {
        self.changed
    }
}

/// Computes a deterministic 64-bit FNV-1a hash of a label string for persistent declarative node tagging.
///
/// Used to bind stable widget tags across frames without manually leaking raw identifiers.
#[inline]
pub fn hash_label(label: &str) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for &b in label.as_bytes() {
        hash ^= b as u64;
        hash = hash.wrapping_mul(0x0100_0000_01b3);
    }
    hash
}