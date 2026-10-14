// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Container & Hierarchical Layout Primitives
//!
//! Provides structural layout primitives on [`UiScope`]: generic containers, flex rows,
//! flex columns, themed cards, separators, and horizontal dividers.
//!

use super::core::UiScope;
use iris_core::{
    Color, Insets, Point, Rect, Size, Style, WidgetCursor, WidgetId, WidgetResponse, WidgetRole,
};

impl<'a> UiScope<'a> {
    /// Emits a tagged dockable panel root container with absolute bounding geometry and vertical flex flow.
    ///
    /// # Arguments
    /// * `name` - Descriptive debug identifier assigned to the panel root node.
    /// * `rect` - Bounding rectangle in physical pixels allocated for the panel.
    /// * `tag` - Semantic tag for tracking interactions and event routing.
    /// * `bg` - Background fill color.
    /// * `f` - Closure executing within the panel root scope.
    pub fn panel_tagged<F>(
        &mut self,
        name: &'static str,
        rect: Rect,
        tag: u64,
        bg: Color,
        f: F,
    ) -> WidgetId
    where
        F: FnOnce(&mut UiScope<'_>),
    {
        let node_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(node_id) {
            node.set_name(name);
            node.tag = tag;
            node.computed_rect = rect;
            node.interactive = true;
            node.set_style(
                Style::new()
                    .position_absolute()
                    .left(rect.x)
                    .top(rect.y)
                    .width(rect.width)
                    .height(rect.height)
                    .flex_col()
                    .background(bg)
                    .clip_children(true),
            );
        }
        let _ = self.tree.add_child(self.parent, node_id);

        let mut child_scope = self.child_scope(node_id);
        f(&mut child_scope);
        crate::declarative::layout_subtree(self.tree, node_id, rect);
        node_id
    }

    /// Emits a horizontal toolbar container with a fixed height and centered flex alignment.
    ///
    /// # Arguments
    /// * `name` - Descriptive debug name for the toolbar node.
    /// * `height` - Fixed toolbar height in physical pixels.
    /// * `bg` - Background fill color.
    /// * `f` - Closure executing within the toolbar scope.
    pub fn toolbar<F>(&mut self, name: &'static str, height: f32, bg: Color, f: F) -> WidgetId
    where
        F: FnOnce(&mut UiScope<'_>),
    {
        let style = Style::new()
            .flex_row()
            .align_items(iris_core::AlignItems::Center)
            .height(height)
            .background(bg)
            .gap(6.0)
            .padding_insets(Insets::new(0.0, 8.0, 0.0, 8.0))
            .border(1.0, Color::rgba(1.0, 1.0, 1.0, 0.08));
        self.container_named(name, style, f)
    }

    /// Emits a tagged horizontal toolbar container with a fixed height and centered flex alignment.
    ///
    /// # Arguments
    /// * `name` - Descriptive debug name for the toolbar node.
    /// * `height` - Fixed toolbar height in physical pixels.
    /// * `tag` - Semantic tag for tracking interactions and event routing.
    /// * `bg` - Background fill color.
    /// * `f` - Closure executing within the toolbar scope.
    pub fn toolbar_tagged<F>(
        &mut self,
        name: &'static str,
        height: f32,
        tag: u64,
        bg: Color,
        f: F,
    ) -> WidgetId
    where
        F: FnOnce(&mut UiScope<'_>),
    {
        let style = Style::new()
            .flex_row()
            .align_items(iris_core::AlignItems::Center)
            .height(height)
            .background(bg)
            .gap(6.0)
            .padding_insets(Insets::new(0.0, 8.0, 0.0, 8.0))
            .border(1.0, Color::rgba(1.0, 1.0, 1.0, 0.08));
        self.container_tagged(name, style, WidgetRole::Default, tag, f)
    }

    /// Synchronizes a scrollable container's offset and scrollbar thumb in-place on the active tree without rebuilding.
    ///
    /// Delegates directly to [`crate::scroll_area::ScrollArea::sync_scroll_in_place`].
    ///
    /// # Arguments
    /// * `container_tag` - Semantic tag of the scroll container viewport.
    /// * `track_tag` - Optional semantic tag of the scrollbar track.
    /// * `thumb_tag` - Optional semantic tag of the draggable scrollbar thumb.
    /// * `scroll_y` - Current vertical scroll offset in physical pixels.
    /// * `max_scroll_y` - Maximum scroll limit in physical pixels.
    ///
    /// Returns `true` if any nodes were found and updated.
    #[inline]
    pub fn sync_scroll_in_place(
        &mut self,
        container_tag: u64,
        track_tag: Option<u64>,
        thumb_tag: Option<u64>,
        scroll_y: f32,
        max_scroll_y: f32,
    ) -> bool {
        crate::scroll_area::ScrollArea::sync_scroll_in_place(
            self.tree,
            container_tag,
            track_tag,
            thumb_tag,
            scroll_y,
            max_scroll_y,
        )
    }

    /// Updates a container node's scroll offset and re-layouts its subtree in-place on the active tree.
    ///
    /// Delegates directly to [`crate::scroll_area::ScrollArea::update_container_scroll_by_tag`].
    #[inline]
    pub fn update_container_scroll_by_tag(&mut self, container_tag: u64, scroll_y: f32) -> bool {
        crate::scroll_area::ScrollArea::update_container_scroll_by_tag(
            self.tree,
            container_tag,
            scroll_y,
        )
    }

    /// Emits a generic styled container and executes a nested child builder closure.
    ///
    /// # Arguments
    /// * `style` - Flexbox and visual styling properties applied to the container.
    /// * `f` - Closure executing within the newly formed child scope.
    pub fn container<F>(&mut self, style: Style, f: F) -> WidgetId
    where
        F: FnOnce(&mut UiScope<'_>),
    {
        let node_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(node_id) {
            node.set_style(style);
        }
        let _ = self.tree.add_child(self.parent, node_id);

        let mut child_scope = self.child_scope(node_id);
        f(&mut child_scope);
        node_id
    }

    /// Emits a generic styled container with a descriptive node name for tree inspection and debugging.
    ///
    /// # Arguments
    /// * `name` - Descriptive identifier assigned to the created UI node.
    /// * `style` - Flexbox and visual styling properties applied to the container.
    /// * `f` - Closure executing within the newly formed child scope.
    pub fn container_named<F>(&mut self, name: impl Into<String>, style: Style, f: F) -> WidgetId
    where
        F: FnOnce(&mut UiScope<'_>),
    {
        let node_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(node_id) {
            node.set_name(name);
            node.set_style(style);
        }
        let _ = self.tree.add_child(self.parent, node_id);

        let mut child_scope = self.child_scope(node_id);
        f(&mut child_scope);
        node_id
    }

    /// Emits a non-interactive (passive) container that allows clicks to pass through to underlying viewports or handlers.
    ///
    /// The container node is marked with `interactive = false`, ensuring hardware raycasts and UI hit-testing
    /// completely ignore this bounding box while still computing correct flexbox and scissor layout for its children.
    ///
    /// # Arguments
    /// * `style` - Flexbox and visual styling properties applied to the container.
    /// * `f` - Closure executing within the newly formed child scope.
    pub fn container_passive<F>(&mut self, style: Style, f: F) -> WidgetId
    where
        F: FnOnce(&mut UiScope<'_>),
    {
        let node_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(node_id) {
            node.interactive = false;
            node.set_style(style);
        }
        let _ = self.tree.add_child(self.parent, node_id);

        let mut child_scope = self.child_scope(node_id);
        f(&mut child_scope);
        node_id
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
    pub fn container_tagged<F>(
        &mut self,
        name: impl Into<String>,
        style: Style,
        role: WidgetRole,
        tag: u64,
        f: F,
    ) -> WidgetId
    where
        F: FnOnce(&mut UiScope<'_>),
    {
        let node_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(node_id) {
            node.role = role;
            node.tag = tag;
            node.interactive = true;
            node.set_name(name);
            node.set_style(style);
        }
        let _ = self.tree.add_child(self.parent, node_id);

        let mut child_scope = self.child_scope(node_id);
        f(&mut child_scope);
        node_id
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
        let id = self.container_tagged(name, style, role, tag, f);
        self.response(id)
    }

    /// Emits a horizontal flexbox container (`flex_direction: Row`) with standard item centering.
    ///
    /// # Arguments
    /// * `f` - Child scope closure emitting horizontal items.
    pub fn row<F>(&mut self, f: F) -> WidgetId
    where
        F: FnOnce(&mut UiScope<'_>),
    {
        let style = Style::new()
            .flex_row()
            .align_items(iris_core::AlignItems::Center)
            .gap(6.0);
        self.container(style, f)
    }

    /// Emits a vertical flexbox container (`flex_direction: Column`) with stretch alignment.
    ///
    /// # Arguments
    /// * `f` - Child scope closure emitting vertical items.
    pub fn column<F>(&mut self, f: F) -> WidgetId
    where
        F: FnOnce(&mut UiScope<'_>),
    {
        let style = Style::new()
            .flex_col()
            .align_items(iris_core::AlignItems::Stretch)
            .gap(4.0);
        self.container(style, f)
    }

    /// Emits an elevated, bordered card container featuring a prominent heading and divider.
    ///
    /// # Arguments
    /// Emits a named elevated, bordered card container conforming to flexbox layout.
    ///
    /// The card expands according to its parent container's layout constraints without requiring
    /// hardcoded physical coordinates or triggering premature layout subtree evaluations.
    ///
    /// # Arguments
    /// * `name` - Descriptive debug identifier assigned to the card container node.
    /// * `title` - Card heading string rendered in a styled header row.
    /// * `f` - Child scope closure emitting the body content.
    pub fn card_named<F>(&mut self, name: impl Into<String>, title: &str, f: F) -> WidgetId
    where
        F: FnOnce(&mut UiScope<'_>),
    {
        let card_style = Style::new()
            .flex_col()
            .background(Color::rgba(0.090, 0.094, 0.110, 0.98))
            .border(1.0, Color::rgba(0.133, 0.141, 0.165, 0.85))
            .border_radius(6.0)
            .padding_insets(Insets::new(6.0, 8.0, 6.0, 8.0))
            .gap(4.0);

        self.container_named(name, card_style, |card_scope| {
            // Header bar
            let header_style = Style::new()
                .flex_row()
                .align_items(iris_core::AlignItems::Center)
                .height(20.0)
                .gap(6.0);

            card_scope.container_named("CardHeader", header_style, |header_scope| {
                header_scope.label_styled_passive(
                    "CardHeaderTitle",
                    title,
                    11.5,
                    Color::rgba(0.886, 0.894, 0.918, 1.0),
                    iris_core::TextAlign::Left,
                    Style::new().height(20.0),
                );
            });

            // Card body column
            card_scope.column(f);
        })
    }

    /// Emits an elevated, bordered card container conforming to flexbox layout.
    ///
    /// Automatically applies standard dark styling, header label, and places body contents into
    /// a vertical flex column.
    ///
    /// # Arguments
    /// * `title` - Card heading string.
    /// * `f` - Child scope closure emitting the body content.
    pub fn card<F>(&mut self, title: &str, f: F) -> WidgetId
    where
        F: FnOnce(&mut UiScope<'_>),
    {
        self.card_named("Card", title, f)
    }

    /// Emits an elevated, bordered card container allowing custom header and body closures.
    ///
    /// # Arguments
    /// * `header_fn` - Closure rendering the custom card title row.
    /// * `body_fn` - Closure rendering the primary card contents.
    pub fn card_custom<H, B>(&mut self, header_fn: H, body_fn: B) -> WidgetId
    where
        H: FnOnce(&mut UiScope<'_>),
        B: FnOnce(&mut UiScope<'_>),
    {
        let card_style = Style::new()
            .flex_col()
            .background(Color::rgba(0.082, 0.086, 0.100, 0.95))
            .border(1.0, Color::rgba(0.15, 0.16, 0.20, 0.85))
            .border_radius(6.0)
            .padding(8.0)
            .gap(6.0);

        self.container(card_style, |card_scope| {
            let header_style = Style::new()
                .flex_row()
                .align_items(iris_core::AlignItems::Center)
                .justify_content(iris_core::JustifyContent::SpaceBetween)
                .padding_insets(Insets::new(0.0, 2.0, 2.0, 2.0));

            card_scope.container(header_style, header_fn);
            card_scope.column(body_fn);
        })
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
    pub fn container_measured<F>(
        &mut self,
        name: &'static str,
        style: Style,
        origin: Point,
        min_height: f32,
        f: F,
    ) -> WidgetId
    where
        F: FnOnce(&mut UiScope<'_>),
    {
        let node_id = self.container_named(name, style, f);
        self.layout_subtree_measured(node_id, origin, min_height);
        node_id
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
        let ring_id = self.empty_box_passive_named(
            "ViewportDropLandingRing",
            Style::new()
                .position_absolute()
                .left(center.x - ring_radius)
                .top(center.y - ring_radius)
                .width(ring_radius * 2.0)
                .height(ring_radius * 2.0)
                .border(2.0, color)
                .border_radius(ring_radius),
        );
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

        let dot_id = self.empty_box_passive_named(
            "ViewportDropLandingDot",
            Style::new()
                .position_absolute()
                .left(center.x - dot_radius)
                .top(center.y - dot_radius)
                .width(dot_radius * 2.0)
                .height(dot_radius * 2.0)
                .background(color)
                .border_radius(dot_radius),
        );
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