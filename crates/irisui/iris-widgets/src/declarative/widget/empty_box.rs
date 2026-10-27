// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Empty Box & Swatch Builder
//!
//! Provides the configurable [`EmptyBox`] builder struct implementing [`Widget`].
//! Replaces legacy `empty_box_tagged` with a unified builder.

use super::Widget;
use crate::declarative::scope::core::UiScope;
use crate::declarative::types::WidgetResponse;
use iris_core::{Style, WidgetRole};

/// Configurable builder for styled empty boxes, canvas viewports, and color swatches.
#[derive(Debug, Clone)]
pub struct EmptyBox {
    name: String,
    style: Style,
    role: WidgetRole,
    tag: u64,
    interactive: bool,
}

impl EmptyBox {
    /// Creates a new empty box builder with descriptive name and layout style.
    #[inline]
    pub fn new(name: impl Into<String>, style: Style) -> Self {
        Self {
            name: name.into(),
            style,
            role: WidgetRole::Default,
            tag: 0,
            interactive: false,
        }
    }

    /// Sets the semantic widget role.
    #[inline]
    #[must_use]
    pub fn role(mut self, role: WidgetRole) -> Self {
        self.role = role;
        self
    }

    /// Sets a persistent 64-bit semantic tag for interaction tracking.
    #[inline]
    #[must_use]
    pub fn tag(mut self, tag: u64) -> Self {
        self.tag = tag;
        self
    }

    /// Explicitly enables or disables user interaction for this leaf node.
    #[inline]
    #[must_use]
    pub fn interactive(mut self, interactive: bool) -> Self {
        self.interactive = interactive;
        self
    }
}

impl Widget for EmptyBox {
    type Response = WidgetResponse;

    fn ui(self, scope: &mut UiScope<'_>) -> WidgetResponse {
        let node_id = scope.tree.create_node();
        let is_interactive = self.interactive || self.tag != 0 || self.role.is_interactive();

        if let Some(node) = scope.tree.get_mut(node_id) {
            node.set_name(self.name);
            node.tag = self.tag;
            node.role = self.role;
            node.interactive = is_interactive;
            node.cursor = self.role.default_cursor();
            node.set_style(self.style);
        }
        let _ = scope.tree.add_child(scope.parent, node_id);

        let (clicked, hovered, _) = if is_interactive {
            scope.check_interaction(node_id)
        } else {
            (false, false, None)
        };
        let pressed = is_interactive && scope.is_pressed(node_id);

        let mut resp = WidgetResponse::new(node_id, clicked, hovered, pressed);
        if let Some(node) = scope.tree.get(node_id) {
            resp.rect = node.computed_rect;
        }
        resp
    }
}