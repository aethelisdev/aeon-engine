// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Virtualized Responsive Grid (`UiScope::virtual_grid`)
//!
//! Combines [`ResponsiveGrid`] column calculation and viewport row culling with an autonomous
//! [`WidgetRole::ScrollAreaViewport`] container, content size synchronization, empty state
//! fallback, and optional context menu / scrollbar emission.

use super::containers::ContainerResponseExt;
use super::context_menu::ContextMenuExt;
use super::core::UiScope;
use crate::declarative::types::WidgetResponse;
use crate::grid_view::ResponsiveGrid;
use crate::scroll_area::{ScrollAreaStyle, ScrollBarGeometry};
use irisui_core::{Color, Point, Rect, Size, Style, WidgetRole};

/// Configuration parameters for [`UiScope::virtual_grid`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VirtualGridConfig {
    /// Debug node name assigned to the scrollable grid viewport container.
    pub name: &'static str,
    /// 64-bit semantic tag identifying the scrollable grid viewport container.
    pub tag: u64,
    /// Bounding rectangle of the visible grid viewport in logical pixels.
    pub viewport_rect: Rect,
    /// Width of each individual grid cell in logical pixels.
    pub item_width: f32,
    /// Height of each individual grid cell in logical pixels.
    pub item_height: f32,
    /// Horizontal and vertical spacing between grid cells in logical pixels.
    pub spacing: (f32, f32),
    /// Outer horizontal and vertical padding inside the grid viewport in logical pixels.
    pub padding: (f32, f32),
    /// Optional explicit background fill color for the grid viewport.
    pub background: Option<Color>,
    /// Optional `(track_tag, thumb_tag)` pair for rendering an interactive vertical scrollbar.
    pub scrollbar_tags: Option<(u64, u64)>,
}

impl VirtualGridConfig {
    /// Creates a new virtualized grid configuration with default 115x125 cell dimensions and 10px gap.
    #[inline]
    #[must_use]
    pub const fn new(name: &'static str, tag: u64, viewport_rect: Rect) -> Self {
        Self {
            name,
            tag,
            viewport_rect,
            item_width: 115.0,
            item_height: 125.0,
            spacing: (10.0, 10.0),
            padding: (10.0, 10.0),
            background: None,
            scrollbar_tags: None,
        }
    }

    /// Configures uniform cell width, height, and gap/padding spacing.
    #[inline]
    #[must_use]
    pub const fn with_cell_size(mut self, width: f32, height: f32, gap: f32) -> Self {
        self.item_width = width;
        self.item_height = height;
        self.spacing = (gap, gap);
        self.padding = (gap, gap);
        self
    }

    /// Overrides the horizontal and vertical spacing between grid cells.
    #[inline]
    #[must_use]
    pub const fn with_spacing(mut self, spacing_x: f32, spacing_y: f32) -> Self {
        self.spacing = (spacing_x, spacing_y);
        self
    }

    /// Overrides the outer horizontal and vertical padding around the grid content.
    #[inline]
    #[must_use]
    pub const fn with_padding(mut self, padding_x: f32, padding_y: f32) -> Self {
        self.padding = (padding_x, padding_y);
        self
    }

    /// Sets an explicit background color on the grid viewport container.
    #[inline]
    #[must_use]
    pub const fn with_background(mut self, color: Color) -> Self {
        self.background = Some(color);
        self
    }

    /// Attaches semantic tags `(track_tag, thumb_tag)` for vertical scrollbar rendering.
    #[inline]
    #[must_use]
    pub const fn with_scrollbar_tags(mut self, track_tag: u64, thumb_tag: u64) -> Self {
        self.scrollbar_tags = Some((track_tag, thumb_tag));
        self
    }
}

type DefaultEmptyFn = fn(&mut UiScope<'_>);
type DefaultGridContextMenuFn = fn(&mut UiScope<'_>);

/// Default builder alias returned by [`UiScope::virtual_grid`].
pub type DefaultVirtualGridBuilder<'s, 'a> =
    VirtualGridBuilder<'s, 'a, DefaultEmptyFn, DefaultGridContextMenuFn>;

/// Fluent builder for a virtualized responsive grid container.
pub struct VirtualGridBuilder<'s, 'a, EmptyFn, CtxFn> {
    scope: &'s mut UiScope<'a>,
    total_items: usize,
    config: VirtualGridConfig,
    empty_fn: Option<EmptyFn>,
    context_menu_fn: Option<CtxFn>,
}

impl<'s, 'a, EmptyFn, CtxFn> VirtualGridBuilder<'s, 'a, EmptyFn, CtxFn>
where
    EmptyFn: FnOnce(&mut UiScope<'_>),
    CtxFn: FnOnce(&mut UiScope<'_>),
{
    /// Registers an empty-state builder closure invoked when `total_items == 0`.
    #[inline]
    #[must_use]
    pub fn empty_state<NewEmpty>(self, f: NewEmpty) -> VirtualGridBuilder<'s, 'a, NewEmpty, CtxFn>
    where
        NewEmpty: FnOnce(&mut UiScope<'_>),
    {
        VirtualGridBuilder {
            scope: self.scope,
            total_items: self.total_items,
            config: self.config,
            empty_fn: Some(f),
            context_menu_fn: self.context_menu_fn,
        }
    }

    /// Registers a background right-click context menu closure on the grid viewport container.
    #[inline]
    #[must_use]
    pub fn context_menu<NewCtx>(self, f: NewCtx) -> VirtualGridBuilder<'s, 'a, EmptyFn, NewCtx>
    where
        NewCtx: FnOnce(&mut UiScope<'_>),
    {
        VirtualGridBuilder {
            scope: self.scope,
            total_items: self.total_items,
            config: self.config,
            empty_fn: self.empty_fn,
            context_menu_fn: Some(f),
        }
    }

    /// Evaluates visible grid cells and invokes `render_cell(viewport_scope, item_index, local_pos)`
    /// for each visible cell without intermediate vector allocation.
    ///
    /// Automatically synchronizes [`WidgetRole::ScrollAreaViewport`] scroll offsets and total virtual
    /// content height on the viewport node.
    pub fn render_item<F>(self, mut render_cell: F) -> Option<WidgetResponse>
    where
        F: FnMut(&mut UiScope<'_>, usize, Point),
    {
        let cfg = self.config;
        let total = self.total_items;
        if total == 0 {
            if let Some(empty) = self.empty_fn {
                empty(self.scope);
            }
            return None;
        }

        let grid = ResponsiveGrid::new(cfg.item_width, cfg.item_height)
            .spacing(cfg.spacing.0, cfg.spacing.1)
            .padding(cfg.padding.0, cfg.padding.1);
        let content_h = grid.compute_content_height(total, cfg.viewport_rect.width);
        let scroll_y = self.scope.scroll_offset(cfg.tag);

        let mut style = Style::new()
            .flex_col()
            .clip_children(true)
            .scroll_offset_y(scroll_y)
            .width(cfg.viewport_rect.width)
            .height(cfg.viewport_rect.height);
        if let Some(bg) = cfg.background {
            style = style.background(bg);
        }

        let resp = self
            .scope
            .container_tagged_resp(
                cfg.name,
                style,
                WidgetRole::ScrollAreaViewport,
                cfg.tag,
                |viewport| {
                    grid.for_each_visible_cell(
                        cfg.viewport_rect,
                        total,
                        scroll_y,
                        |item_idx, local_pos, _world_rect| {
                            render_cell(viewport, item_idx, local_pos);
                        },
                    );
                },
            )
            .content_size(self.scope, Size::new(cfg.viewport_rect.width, content_h));

        if let Some(ctx_fn) = self.context_menu_fn {
            resp.context_menu(self.scope, ctx_fn);
        }

        if let Some((track_tag, thumb_tag)) = cfg.scrollbar_tags
            && content_h > cfg.viewport_rect.height
        {
            let scroll_style = ScrollAreaStyle::dark_default();
            if let Some(geom) = ScrollBarGeometry::compute_vertical(
                cfg.viewport_rect,
                content_h,
                scroll_y,
                &scroll_style,
            ) {
                let is_dragging = self.scope.is_tag_captured(thumb_tag);
                let cursor_pos = self.scope.cursor_pos();
                self.scope.scrollbar_vertical(
                    geom,
                    cfg.viewport_rect,
                    is_dragging,
                    cursor_pos,
                    track_tag,
                    thumb_tag,
                );
            }
        }

        Some(resp)
    }
}

impl<'a> UiScope<'a> {
    /// Creates a declarative virtualized responsive grid builder.
    ///
    /// Automatically computes responsive column counts, culls off-screen rows in $O(1)$ time,
    /// manages `scroll_offset_y` and `content_size`, and invokes the cell renderer with
    /// viewport-relative coordinates.
    #[inline]
    #[must_use]
    pub fn virtual_grid<'s>(
        &'s mut self,
        total_items: usize,
        config: VirtualGridConfig,
    ) -> DefaultVirtualGridBuilder<'s, 'a> {
        VirtualGridBuilder {
            scope: self,
            total_items,
            config,
            empty_fn: None,
            context_menu_fn: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use irisui_core::UiTree;

    #[test]
    fn test_virtual_grid_culls_offscreen_cells_and_renders_empty_state() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("root");
        let vp_rect = Rect::new(0.0, 0.0, 500.0, 280.0);
        let cfg = VirtualGridConfig::new("TestGridViewport", 77, vp_rect)
            .with_cell_size(115.0, 125.0, 10.0);

        // 1. Empty state
        {
            let mut scope = UiScope::new(&mut tree, root);
            scope
                .virtual_grid(0, cfg)
                .empty_state(|vp| {
                    vp.label("EmptyGridNotice");
                })
                .render_item(|_vp, _idx, _pos| {
                    panic!("Must not render items when total_items == 0");
                });
        }
        assert!(
            tree.iter()
                .any(|(_, n)| n.text.as_deref() == Some("EmptyGridNotice"))
        );

        // 2. 100 items in 500x280 viewport -> 3 columns, max 3 visible rows (<= 9 cells emitted)
        let mut tree2 = UiTree::new();
        let root2 = tree2.create_root().expect("root2");
        let mut rendered_indices = Vec::new();
        {
            let mut scope2 = UiScope::new(&mut tree2, root2);
            scope2.virtual_grid(100, cfg).render_item(|vp, idx, pos| {
                rendered_indices.push((idx, pos));
                vp.empty_box(
                    Style::new()
                        .position_absolute()
                        .left(pos.x)
                        .top(pos.y)
                        .width(115.0)
                        .height(125.0),
                );
            });
        }
        assert!(!rendered_indices.is_empty());
        assert!(
            rendered_indices.len() <= 9,
            "Expected at most 9 visible cells for 100 items, got {}",
            rendered_indices.len()
        );
        assert!(rendered_indices[1].1.x > rendered_indices[0].1.x);
    }
}