// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Generic Declarative Container Primitives & Response Extension
//!
//! Provides fundamental building block containers, passive viewports, tagged hit-testing
//! boundaries, custom-rendered canvas nodes, and inline measured layout containers on [`UiScope`].
//!

use super::super::core::UiScope;
use iris_core::{
    Color, Point, Rect, Size, Style, WidgetCursor, WidgetId, WidgetResponse, WidgetRole,
};

/// Response returned by container closures, pairing the container widget response with the inner closure output.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InnerResponse<R> {
    /// The container widget interaction response.
    pub response: WidgetResponse,
    /// The inner value returned by the user closure.
    pub inner: R,
}

impl<R> InnerResponse<R> {
    /// Creates a new paired inner response.
    #[inline]
    pub fn new(response: WidgetResponse, inner: R) -> Self {
        Self { response, inner }
    }
}

impl<R> std::ops::Deref for InnerResponse<R> {
    type Target = R;
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<'a> UiScope<'a> {
    /// Emits a generic styled container and executes a nested child builder closure, returning [`InnerResponse`].
    ///
    /// # Arguments
    /// * `style` - Flexbox and visual styling properties applied to the container.
    /// * `f` - Closure executing within the newly formed child scope.
    pub fn container<R, F>(&mut self, style: Style, f: F) -> InnerResponse<R>
    where
        F: FnOnce(&mut UiScope<'_>) -> R,
    {
        let node_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(node_id) {
            node.set_style(style);
        }
        let _ = self.tree.add_child(self.parent, node_id);

        let mut child_scope = self.child_scope(node_id);
        let inner = f(&mut child_scope);
        let response = self.response(node_id);
        InnerResponse::new(response, inner)
    }

    /// Emits a generic styled container with a descriptive node name for tree inspection and debugging.
    ///
    /// # Arguments
    /// * `name` - Descriptive identifier assigned to the created UI node.
    /// * `style` - Flexbox and visual styling properties applied to the container.
    /// * `f` - Closure executing within the newly formed child scope.
    pub fn container_named<R, F>(
        &mut self,
        name: impl Into<String>,
        style: Style,
        f: F,
    ) -> InnerResponse<R>
    where
        F: FnOnce(&mut UiScope<'_>) -> R,
    {
        let name_str = name.into();
        let node_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(node_id) {
            node.set_name(name_str);
            node.set_style(style);
        }
        let _ = self.tree.add_child(self.parent, node_id);

        let mut child_scope = self.child_scope(node_id);
        let inner = f(&mut child_scope);
        let response = self.response(node_id);
        InnerResponse::new(response, inner)
    }

    /// Emits a non-interactive (passive) container that allows clicks to pass through to underlying viewports or handlers.
    ///
    /// The container node is marked with `interactive = false`, ensuring hardware raycasts and UI hit-testing
    /// completely ignore this bounding box while still computing correct flexbox and scissor layout for its children.
    ///
    /// # Arguments
    /// * `style` - Flexbox and visual styling properties applied to the container.
    /// * `f` - Closure executing within the newly formed child scope.
    pub fn container_passive<R, F>(&mut self, style: Style, f: F) -> InnerResponse<R>
    where
        F: FnOnce(&mut UiScope<'_>) -> R,
    {
        let node_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(node_id) {
            node.interactive = false;
            node.set_style(style);
        }
        let _ = self.tree.add_child(self.parent, node_id);

        let mut child_scope = self.child_scope(node_id);
        let inner = f(&mut child_scope);
        let response = self.response(node_id);
        InnerResponse::new(response, inner)
    }

    /// Emits an interactive container configured with a semantic tag, role, and debug name, executing a child builder closure.
    ///
    /// Used for interactive canvas panels, custom gizmo root containers, or complex compound controls that require
    /// semantic identification during hit-testing without manual node attribute manipulation.
    ///
    /// # Arguments
    /// * `name` - Static debug name assigned to the container node.
    /// * `style` - Flexbox and visual styling properties applied to the container.
    /// * `role` - Semantic accessibility and widget role (e.g. [`WidgetRole::OscilloscopeCanvas`]).
    /// * `tag` - Persistent 64-bit semantic tag inspected during interaction dispatching.
    /// * `f` - Closure executing within the newly formed child scope.
    pub fn container_tagged<R, F>(
        &mut self,
        name: impl Into<String>,
        style: Style,
        role: WidgetRole,
        tag: u64,
        f: F,
    ) -> InnerResponse<R>
    where
        F: FnOnce(&mut UiScope<'_>) -> R,
    {
        let name_str = name.into();
        let node_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(node_id) {
            node.role = role;
            node.tag = tag;
            node.interactive = true;
            node.set_name(name_str);
            node.set_style(style);
        }
        let _ = self.tree.add_child(self.parent, node_id);

        let mut child_scope = self.child_scope(node_id);
        let inner = f(&mut child_scope);
        let response = self.response(node_id);
        InnerResponse::new(response, inner)
    }

    /// Emits a tagged container widget, returning [`WidgetResponse`] directly for fluent method chaining.
    ///
    /// Eliminates the two-step `let id = ...; scope.response(id)...` pattern, enabling immediate
    /// `.on_click(...)`, `.on_drag(...)`, or `.context_menu(...)` chaining on container widgets.
    ///
    /// # Arguments
    /// * `name` - Debug node name.
    /// * `style` - Visual presentation and layout metrics.
    /// * `role` - Semantic accessibility and widget role.
    /// * `tag` - Persistent 64-bit semantic tag.
    /// * `f` - Child scope closure emitting contained items.
    pub fn container_tagged_resp<F>(
        &mut self,
        name: impl Into<String>,
        style: Style,
        role: WidgetRole,
        tag: u64,
        f: F,
    ) -> WidgetResponse
    where
        F: FnOnce(&mut UiScope<'_>),
    {
        let resp = self.container_tagged(name, style, role, tag, f);
        resp.response
    }

    /// Emits an interactive or custom-rendered canvas container node with specified role, tag, and cursor.
    ///
    /// # Arguments
    /// * `style` - Visual bounds and styling properties.
    /// * `role` - Semantic accessibility / widget role (e.g. [`WidgetRole::OscilloscopeCanvas`]).
    /// * `tag` - Semantic tag for event dispatching or custom rendering hooks.
    /// * `cursor` - Optional cursor override (e.g. `Some(WidgetCursor::Grab)`).
    pub fn canvas(
        &mut self,
        style: Style,
        role: WidgetRole,
        tag: u64,
        cursor: Option<WidgetCursor>,
    ) -> WidgetId {
        let node_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(node_id) {
            node.set_style(style);
            node.role = role;
            node.tag = tag;
            node.interactive = true;
            node.cursor = cursor;
        }
        let _ = self.tree.add_child(self.parent, node_id);
        node_id
    }

    /// Emits a named custom-rendered canvas container node with specified name, style, role, tag, and cursor.
    ///
    /// # Arguments
    /// * `name` - Descriptive identifier assigned to the UI node for debugging and tree inspection.
    /// * `style` - Visual bounds and styling properties.
    /// * `role` - Semantic accessibility / widget role (e.g. [`WidgetRole::OscilloscopeCanvas`]).
    /// * `tag` - Semantic tag for event dispatching or custom rendering hooks.
    /// * `cursor` - Optional cursor override (e.g. `Some(WidgetCursor::Grab)`).
    pub fn canvas_named(
        &mut self,
        name: &'static str,
        style: Style,
        role: WidgetRole,
        tag: u64,
        cursor: Option<WidgetCursor>,
    ) -> WidgetId {
        let node_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(node_id) {
            node.set_name(name);
            node.set_style(style);
            node.role = role;
            node.tag = tag;
            node.interactive = true;
            node.cursor = cursor;
        }
        let _ = self.tree.add_child(self.parent, node_id);
        node_id
    }

    /// Measures the content bounding extents of a widget subtree and executes layout positioning in-place.
    ///
    /// Eliminates intermediate borrow-checker `let` measurements and layout boilerplate outside of the scope.
    ///
    /// # Arguments
    /// * `node_id` - Target widget node to layout.
    /// * `origin` - Top-left origin coordinates `(x, y)` for the positioned subtree.
    /// * `min_height` - Minimum height clamping applied to the measured container.
    pub fn layout_subtree_measured(
        &mut self,
        node_id: WidgetId,
        origin: Point,
        min_height: f32,
    ) -> Rect {
        let w = crate::declarative::measure_width(self.tree, node_id);
        let h = crate::declarative::measure_height(self.tree, node_id).max(min_height);
        let rect = Rect::new(origin.x, origin.y, w, h);
        crate::declarative::layout_subtree(self.tree, node_id, rect);
        rect
    }

    /// Emits a positioned container and performs immediate layout measurement on its subtree.
    ///
    /// # Arguments
    /// * `name` - Descriptive debug identifier.
    /// * `style` - Base layout styling applied to the container.
    /// * `origin` - Top-left origin coordinates `(x, y)` for positioning.
    /// * `min_height` - Minimum height clamping applied to the measured container.
    /// * `f` - Children construction closure.
    pub fn container_measured<R, F>(
        &mut self,
        name: &'static str,
        style: Style,
        origin: Point,
        min_height: f32,
        f: F,
    ) -> InnerResponse<R>
    where
        F: FnOnce(&mut UiScope<'_>) -> R,
    {
        let node_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(node_id) {
            node.set_name(name);
            node.set_style(style);
        }
        let _ = self.tree.add_child(self.parent, node_id);

        let mut child_scope = self.child_scope(node_id);
        let inner = f(&mut child_scope);
        self.layout_subtree_measured(node_id, origin, min_height);
        let response = self.response(node_id);
        InnerResponse::new(response, inner)
    }

    /// Emits a container positioned at exact bounds and performs autonomous subtree layout and hover styling.
    ///
    /// # Arguments
    /// * `name` - Debug node name.
    /// * `style` - Base layout styling applied to the container.
    /// * `bounds` - Exact layout bounding rectangle.
    /// * `f` - Children construction closure.
    pub fn container_rect<R, F>(
        &mut self,
        name: &'static str,
        style: Style,
        bounds: Rect,
        f: F,
    ) -> InnerResponse<R>
    where
        F: FnOnce(&mut UiScope<'_>) -> R,
    {
        self.container_rect_with_role(name, style, bounds, WidgetRole::Default, f)
    }

    /// Emits a container with a custom role positioned at exact bounds and performs autonomous subtree layout.
    ///
    /// # Arguments
    /// * `name` - Debug node name.
    /// * `style` - Base layout styling applied to the container.
    /// * `bounds` - Exact layout bounding rectangle.
    /// * `role` - Semantic accessibility / layer widget role.
    /// * `f` - Children construction closure.
    pub fn container_rect_with_role<R, F>(
        &mut self,
        name: &'static str,
        style: Style,
        bounds: Rect,
        role: WidgetRole,
        f: F,
    ) -> InnerResponse<R>
    where
        F: FnOnce(&mut UiScope<'_>) -> R,
    {
        let node_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(node_id) {
            node.set_name(name);
            node.set_style(style);
            node.set_role(role);
        }
        let _ = self.tree.add_child(self.parent, node_id);

        let mut child_scope = self.child_scope(node_id);
        let inner = f(&mut child_scope);
        crate::declarative::layout_subtree(self.tree, node_id, bounds);
        if let Some(cursor) = self.tree.cursor_position() {
            super::super::hover::update_hover_styles_recursive(self.tree, node_id, cursor);
        }
        let response = self.response(node_id);
        InnerResponse::new(response, inner)
    }

    /// Emits a concentric circular viewport landing drop target with an outer glowing ring and inner dot.
    ///
    /// # Arguments
    /// * `center` - Center coordinate on the viewport canvas.
    /// * `ring_radius` - Radius of the outer boundary ring.
    /// * `dot_radius` - Radius of the inner targeting center dot.
    /// * `color` - Color applied to both outer ring border and inner dot fill.
    pub fn landing_indicator(
        &mut self,
        center: Point,
        ring_radius: f32,
        dot_radius: f32,
        color: Color,
    ) {
        let ring_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(ring_id) {
            node.set_name("ViewportDropLandingRing");
            node.set_style(
                Style::new()
                    .position_absolute()
                    .left(center.x - ring_radius)
                    .top(center.y - ring_radius)
                    .width(ring_radius * 2.0)
                    .height(ring_radius * 2.0)
                    .border(2.0, color)
                    .border_radius(ring_radius),
            );
        }
        let _ = self.tree.add_child(self.parent, ring_id);
        crate::declarative::layout_subtree(
            self.tree,
            ring_id,
            Rect::new(
                center.x - ring_radius,
                center.y - ring_radius,
                ring_radius * 2.0,
                ring_radius * 2.0,
            ),
        );

        let dot_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(dot_id) {
            node.set_name("ViewportDropLandingDot");
            node.set_style(
                Style::new()
                    .position_absolute()
                    .left(center.x - dot_radius)
                    .top(center.y - dot_radius)
                    .width(dot_radius * 2.0)
                    .height(dot_radius * 2.0)
                    .background(color)
                    .border_radius(dot_radius),
            );
        }
        let _ = self.tree.add_child(self.parent, dot_id);
        crate::declarative::layout_subtree(
            self.tree,
            dot_id,
            Rect::new(
                center.x - dot_radius,
                center.y - dot_radius,
                dot_radius * 2.0,
                dot_radius * 2.0,
            ),
        );
    }
}

/// Extension trait adding fluent layout and interaction chaining to container responses.
///
/// Enables configuring virtual scroll sizes directly on container [`WidgetResponse`]
/// instances without declaring imperative intermediate `let` variables.
pub trait ContainerResponseExt {
    /// Configures the virtual scrollable content extent size of this container node.
    ///
    /// Enables  method chaining directly on container responses without
    /// declaring intermediate `let` variables.
    ///
    /// # Arguments
    /// * `scope` - Active UI construction scope owning the node tree.
    /// * `size` - Total virtual content size in physical pixels.
    fn content_size(self, scope: &mut UiScope<'_>, size: Size) -> Self;

    /// Configures the virtual scrollable content extent size using discrete width and height.
    ///
    /// # Arguments
    /// * `scope` - Active UI construction scope owning the node tree.
    /// * `width` - Virtual content width in physical pixels.
    /// * `height` - Virtual content height in physical pixels.
    fn content_size_wh(self, scope: &mut UiScope<'_>, width: f32, height: f32) -> Self;
}

impl ContainerResponseExt for WidgetResponse {
    #[inline]
    fn content_size(self, scope: &mut UiScope<'_>, size: Size) -> Self {
        scope.set_content_size(self.id, size);
        self
    }

    #[inline]
    fn content_size_wh(self, scope: &mut UiScope<'_>, width: f32, height: f32) -> Self {
        scope.set_content_size(self.id, Size::new(width, height));
        self
    }
}