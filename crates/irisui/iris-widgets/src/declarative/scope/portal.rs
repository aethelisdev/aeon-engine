// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Elevated Layer Portals (`iris-widgets::declarative::scope::portal`)
//!
//! Provides declarative portal abstractions on [`UiScope`] that elevate child widget hierarchies
//! directly to specific [`UiLayer`] stacking contexts (e.g. [`UiLayer::Popup`], [`UiLayer::Modal`]).
//!
//! Portals decouple floating menus, tooltips, dialogs, and overlays from parent container
//! layout constraints, flexbox distribution, and GPU scissor clipping (`clip_children: true`).

use super::containers::InnerResponse;
use super::core::UiScope;
use iris_core::geometry::Point;
use iris_core::{Color, Insets, Style, UiLayer, WidgetId};

impl<'a> UiScope<'a> {
    /// Obtains or attaches an elevated portal node targeting the specified stacking [`UiLayer`].
    ///
    /// Locates an existing portal node attached to the tree root, or allocates and attaches a new
    /// container on `layer` under the root node if none exists.
    ///
    /// # Arguments
    /// * `layer` - Target stacking context layer.
    #[must_use]
    pub fn get_or_attach_portal(&mut self, layer: UiLayer) -> WidgetId {
        let root_id = self.tree.root().unwrap_or(self.parent);
        self.tree
            .find_layer_portal(root_id, layer)
            .unwrap_or_else(|| {
                self.tree
                    .attach_layer_portal(root_id, layer)
                    .unwrap_or(root_id)
            })
    }

    /// Evaluates child widgets within an elevated [`UiLayer::Popup`] portal container.
    ///
    /// Bypasses parent flex flow and scissor clipping by rendering child nodes inside the root's
    /// elevated popup layer, guaranteeing highest-priority hit-testing and autonomous outside-click dismissal.
    ///
    /// # Arguments
    /// * `f` - Child builder closure receiving a scoped context targeting the elevated popup layer portal.
    pub fn popup_portal<R, F>(&mut self, f: F) -> InnerResponse<R>
    where
        F: FnOnce(&mut UiScope<'_>) -> R,
    {
        let portal_id = self.get_or_attach_portal(UiLayer::Popup);
        let mut portal_scope = self.child_scope(portal_id);
        let inner = f(&mut portal_scope);
        let _ = self.tree.set_layer_recursive(portal_id, UiLayer::Popup);
        let response = self.response(portal_id);
        InnerResponse::new(response, inner)
    }

    /// Evaluates child widgets within an elevated [`UiLayer::Popup`] portal container positioned at `point`.
    ///
    /// Coordinates absolute screen coordinates on the popup portal layer, allowing dropdowns,
    /// tooltips, and floating pickers to appear precisely under the cursor or target control.
    ///
    /// # Arguments
    /// * `point` - Screen coordinate in logical pixels.
    /// * `f` - Child builder closure receiving the positioned popup container scope.
    pub fn popup_portal_at<R, F>(&mut self, point: Point, f: F) -> InnerResponse<R>
    where
        F: FnOnce(&mut UiScope<'_>) -> R,
    {
        let resp = self.popup_portal(|portal| {
            portal.container(
                Style::new().position_absolute().left(point.x).top(point.y),
                f,
            )
        });
        resp.map(|c| c.inner)
    }

    /// Evaluates child widgets within a styled floating card on [`UiLayer::Popup`] positioned at `point`.
    ///
    /// Automatically applies standard dark IDE theme styling, subtle border, rounded corners,
    /// and drop shadow elevation.
    ///
    /// # Arguments
    /// * `point` - Screen coordinate in logical pixels.
    /// * `width` - Card width constraint in logical pixels.
    /// * `f` - Child builder closure receiving the styled popup card scope.
    pub fn popup_portal_card<R, F>(&mut self, point: Point, width: f32, f: F) -> InnerResponse<R>
    where
        F: FnOnce(&mut UiScope<'_>) -> R,
    {
        let resp = self.popup_portal(|portal| {
            portal.container(
                Style::new()
                    .position_absolute()
                    .left(point.x)
                    .top(point.y)
                    .width(width)
                    .flex_col()
                    .background(Color::rgba(0.09, 0.10, 0.13, 0.98))
                    .border(1.0, Color::rgba(0.22, 0.25, 0.32, 1.0))
                    .border_radius(6.0)
                    .box_shadow(0.0, 6.0, 18.0, Color::rgba(0.0, 0.0, 0.0, 0.65))
                    .padding_insets(Insets::all(4.0)),
                f,
            )
        });
        resp.map(|c| c.inner)
    }

    /// Evaluates child widgets within an elevated [`UiLayer::Modal`] portal container.
    ///
    /// # Arguments
    /// * `f` - Child builder closure receiving a scoped context targeting the elevated modal layer portal.
    pub fn modal_portal<R, F>(&mut self, f: F) -> InnerResponse<R>
    where
        F: FnOnce(&mut UiScope<'_>) -> R,
    {
        let portal_id = self.get_or_attach_portal(UiLayer::Modal);
        let mut portal_scope = self.child_scope(portal_id);
        let inner = f(&mut portal_scope);
        let _ = self.tree.set_layer_recursive(portal_id, UiLayer::Modal);
        let response = self.response(portal_id);
        InnerResponse::new(response, inner)
    }
}