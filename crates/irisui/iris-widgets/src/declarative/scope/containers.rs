// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Container & Hierarchical Layout Primitives
//!
//! Provides structural layout primitives on [`UiScope`]: generic containers, flex rows,
//! flex columns, themed cards, separators, and horizontal dividers.
//!

use super::core::UiScope;
use iris_core::{Color, Insets, Rect, Style, WidgetCursor, WidgetId, WidgetRole};

impl<'a> UiScope<'a> {
    /// Emits a top-level panel container with an explicit bounding rectangle and debug name.
    ///
    /// Sets both style (absolute positioning, width, height, background, clip_children)
    /// and initializes `computed_rect = rect` so that child clipping and hit-testing immediately
    /// function across the panel bounds without requiring an external layout resolution pass.
    ///
    /// # Arguments
    /// * `name` - Static debug name assigned to the panel node.
    /// * `rect` - Bounding rectangle defining the position and physical dimensions of the panel.
    /// * `bg` - Background fill color for the panel.
    /// * `f` - Closure executing within the newly formed child scope.
    pub fn panel<F>(&mut self, name: &'static str, rect: Rect, bg: Color, f: F) -> WidgetId
    where
        F: FnOnce(&mut UiScope<'_>),
    {
        let node_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(node_id) {
            node.set_name(name);
            node.computed_rect = rect;
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

        let mut child_scope = UiScope {
            tree: self.tree,
            parent: node_id,
            events: self.events,
            hovered_id: self.hovered_id,
            tagged_events: self.tagged_events,
            hovered_tag: self.hovered_tag,
        };
        f(&mut child_scope);
        crate::declarative::layout_subtree(self.tree, node_id, rect);
        node_id
    }

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

        let mut child_scope = UiScope {
            tree: self.tree,
            parent: node_id,
            events: self.events,
            hovered_id: self.hovered_id,
            tagged_events: self.tagged_events,
            hovered_tag: self.hovered_tag,
        };
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

    /// Emits a scrollable container with flex-grow expansion and hardware child clipping.
    ///
    /// # Arguments
    /// * `name` - Descriptive identifier assigned to the scroll viewport node.
    /// * `bg` - Background fill color.
    /// * `f` - Closure executing within the scroll viewport scope.
    pub fn scroll_area<F>(&mut self, name: &'static str, bg: Color, f: F) -> WidgetId
    where
        F: FnOnce(&mut UiScope<'_>),
    {
        let style = Style::new()
            .flex_col()
            .flex_grow(1.0)
            .clip_children(true)
            .background(bg);
        self.container_named(name, style, f)
    }

    /// Emits a tagged scrollable container with flex-grow expansion and hardware child clipping.
    ///
    /// # Arguments
    /// * `name` - Descriptive identifier assigned to the scroll viewport node.
    /// * `tag` - Semantic tag for tracking interactions and event routing.
    /// * `bg` - Background fill color.
    /// * `f` - Closure executing within the scroll viewport scope.
    pub fn scroll_area_tagged<F>(
        &mut self,
        name: &'static str,
        tag: u64,
        bg: Color,
        f: F,
    ) -> WidgetId
    where
        F: FnOnce(&mut UiScope<'_>),
    {
        let style = Style::new()
            .flex_col()
            .flex_grow(1.0)
            .clip_children(true)
            .background(bg);
        self.container_tagged(name, style, WidgetRole::Default, tag, f)
    }

    /// Emits a high-performance virtualized scrollable container that renders only the visible slice of items.
    ///
    /// Automatically manages [`VirtualList`](crate::scroll_area::VirtualList) windowing, overscan buffer padding,
    /// sub-pixel scroll offset application, and child iteration via declarative closures.
    ///
    /// Returns the computed maximum vertical scroll limit in physical pixels.
    ///
    /// # Arguments
    /// * `name` - Descriptive identifier assigned to the scroll viewport node.
    /// * `tag` - Semantic tag for tracking interactions and event routing.
    /// * `viewport_height` - Available visible height in physical pixels.
    /// * `scroll_y` - Current vertical scroll offset in physical pixels.
    /// * `config` - Sizing, overscan, and container styling configuration descriptor.
    /// * `item_renderer` - Closure called for each visible index: `(scope, index)`.
    pub fn virtual_scroll_area<F>(
        &mut self,
        name: &'static str,
        tag: u64,
        viewport_height: f32,
        scroll_y: f32,
        config: crate::scroll_area::VirtualScrollConfig,
        mut item_renderer: F,
    ) -> f32
    where
        F: FnMut(&mut UiScope<'_>, usize),
    {
        let stride = config.stride();
        let vlist = crate::scroll_area::VirtualList::new(config.total_items, stride)
            .with_overscan(config.overscan);
        let max_scroll = vlist.max_scroll_y(viewport_height);
        let safe_scroll = scroll_y.clamp(0.0, max_scroll);
        let slice = vlist.compute_slice(viewport_height, safe_scroll);
        let scroll_offset = vlist.compute_scroll_offset(safe_scroll, slice.start_idx);

        self.scroll_area_tagged(name, tag, config.bg, |vp| {
            vp.set_scroll_offset_y(scroll_offset);
            for idx in slice.start_idx..slice.end_idx {
                item_renderer(vp, idx);
            }
        });

        max_scroll
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

        let mut child_scope = UiScope {
            tree: self.tree,
            parent: node_id,
            events: self.events,
            hovered_id: self.hovered_id,
            tagged_events: self.tagged_events,
            hovered_tag: self.hovered_tag,
        };
        f(&mut child_scope);
        node_id
    }

    /// Emits a generic styled container with a descriptive node name for tree inspection and debugging.
    ///
    /// # Arguments
    /// * `name` - Descriptive identifier assigned to the created UI node.
    /// * `style` - Flexbox and visual styling properties applied to the container.
    /// * `f` - Closure executing within the newly formed child scope.
    pub fn container_named<F>(&mut self, name: &'static str, style: Style, f: F) -> WidgetId
    where
        F: FnOnce(&mut UiScope<'_>),
    {
        let node_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(node_id) {
            node.set_name(name);
            node.set_style(style);
        }
        let _ = self.tree.add_child(self.parent, node_id);

        let mut child_scope = UiScope {
            tree: self.tree,
            parent: node_id,
            events: self.events,
            hovered_id: self.hovered_id,
            tagged_events: self.tagged_events,
            hovered_tag: self.hovered_tag,
        };
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

        let mut child_scope = UiScope {
            tree: self.tree,
            parent: node_id,
            events: self.events,
            hovered_id: self.hovered_id,
            tagged_events: self.tagged_events,
            hovered_tag: self.hovered_tag,
        };
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
        name: &'static str,
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

        let mut child_scope = UiScope {
            tree: self.tree,
            parent: node_id,
            events: self.events,
            hovered_id: self.hovered_id,
            tagged_events: self.tagged_events,
            hovered_tag: self.hovered_tag,
        };
        f(&mut child_scope);
        node_id
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
    /// * `title` - Card heading string.
    /// * `f` - Child scope closure emitting the body content.
    pub fn card<F>(&mut self, title: &str, f: F) -> WidgetId
    where
        F: FnOnce(&mut UiScope<'_>),
    {
        let card_style = Style::new()
            .flex_col()
            .background(Color::rgba(0.12, 0.12, 0.15, 0.95))
            .border(1.0, Color::rgba(0.22, 0.22, 0.28, 0.8))
            .border_radius(4.0)
            .padding(6.0)
            .gap(4.0);

        self.container(card_style, |card_scope| {
            // Header bar
            let header_style = Style::new()
                .flex_row()
                .align_items(iris_core::AlignItems::Center)
                .padding_insets(Insets::new(2.0, 4.0, 2.0, 4.0));

            card_scope.container(header_style, |header_scope| {
                header_scope.text_colored(title, Color::rgba(0.9, 0.92, 0.96, 1.0));
            });

            // Card body column
            card_scope.column(f);
        })
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

    /// Emits a horizontal divider separator line.
    pub fn separator(&mut self) -> WidgetId {
        let node_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(node_id) {
            node.role = WidgetRole::Separator;
            node.set_style(
                Style::new()
                    .height(1.0)
                    .background(Color::rgba(0.24, 0.25, 0.30, 0.6)),
            );
        }
        let _ = self.tree.add_child(self.parent, node_id);
        node_id
    }

    /// Emits a 1-pixel horizontal rule divider container with a custom color.
    ///
    /// # Arguments
    /// * `color` - RGBA color of the divider line.
    /// Emits a styled leaf element (empty container node without child scopes, e.g. solid box or backdrop).
    ///
    /// # Arguments
    /// * `style` - Visual bounds and styling properties applied to the leaf node.
    pub fn empty_box(&mut self, style: Style) -> WidgetId {
        let node_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(node_id) {
            node.set_style(style);
        }
        let _ = self.tree.add_child(self.parent, node_id);
        node_id
    }

    /// Emits a styled leaf element with `interactive = false` so it does not intercept hit-testing.
    ///
    /// Used for background grids, visual outlines, guide lines, and purely decorative graphics.
    ///
    /// # Arguments
    /// * `style` - Visual bounds and styling properties applied to the passive leaf node.
    pub fn empty_box_passive(&mut self, style: Style) -> WidgetId {
        let node_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(node_id) {
            node.interactive = false;
            node.set_style(style);
        }
        let _ = self.tree.add_child(self.parent, node_id);
        node_id
    }

    /// Emits a named, non-interactive leaf box node with custom styling.
    ///
    /// Ideal for decorative lines, pins, markers, and visual outlines that should not block
    /// mouse events or register in hit-testing.
    ///
    /// # Arguments
    /// * `name` - Descriptive name of the widget node for tree debugging.
    /// * `style` - Visual bounds and styling properties applied to the leaf node.
    pub fn empty_box_passive_named(&mut self, name: &'static str, style: Style) -> WidgetId {
        let node_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(node_id) {
            node.set_name(name);
            node.interactive = false;
            node.set_style(style);
        }
        let _ = self.tree.add_child(self.parent, node_id);
        node_id
    }

    /// Emits a non-container leaf node with custom styling, widget role, and 64-bit semantic tag.
    ///
    /// # Arguments
    /// * `name` - Descriptive name of the widget node for tree debugging.
    /// * `style` - Visual bounds and styling properties applied to the leaf node.
    /// * `role` - Accessibility and interaction role for the widget.
    /// * `tag` - 64-bit semantic identifier for hardware hit-testing and event routing.
    pub fn empty_box_tagged(
        &mut self,
        name: &'static str,
        style: Style,
        role: WidgetRole,
        tag: u64,
    ) -> WidgetId {
        let node_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(node_id) {
            node.set_name(name);
            node.tag = tag;
            node.role = role;
            node.interactive = true;
            node.cursor = role.default_cursor();
            node.set_style(style);
        }
        let _ = self.tree.add_child(self.parent, node_id);
        node_id
    }

    /// Emits a 1-pixel horizontal rule divider container with a custom color.
    ///
    /// # Arguments
    /// * `color` - RGBA color of the divider line.
    pub fn divider(&mut self, color: Color) -> WidgetId {
        let div_style = Style::new().background(color).height(1.0);
        self.empty_box(div_style)
    }

    /// Emits a 1-pixel vertical divider container with custom height and color.
    ///
    /// # Arguments
    /// * `height` - Explicit vertical height of the divider line.
    /// * `color` - RGBA color of the divider line.
    pub fn vertical_divider(&mut self, height: f32, color: Color) -> WidgetId {
        let div_style = Style::new().background(color).width(1.0).height(height);
        self.empty_box(div_style)
    }

    /// Emits an empty flex-grow spacer node to expand and push subsequent items.
    pub fn spacer(&mut self) -> WidgetId {
        self.empty_box(Style::new().flex_grow(1.0))
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
}