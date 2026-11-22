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
    /// Optional explicit bounding rectangle of the visible grid viewport in logical pixels.
    /// When `None`, resolves autonomously from retained tag/parent layout geometry.
    pub viewport_rect: Option<Rect>,
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
    /// Creates a new virtualized grid configuration with an explicit viewport rectangle,
    /// default 115x125 cell dimensions, and 10px gap.
    #[inline]
    #[must_use]
    pub const fn new(name: &'static str, tag: u64, viewport_rect: Rect) -> Self {
        Self {
            name,
            tag,
            viewport_rect: Some(viewport_rect),
            item_width: 115.0,
            item_height: 125.0,
            spacing: (10.0, 10.0),
            padding: (10.0, 10.0),
            background: None,
            scrollbar_tags: None,
        }
    }

    /// Creates a virtualized grid configuration whose viewport bounds resolve automatically
    /// from retained layout geometry (`UiTree::tag_rect` / `UiScope::available_rect`) and
    /// expand via flexbox inside its parent container.
    #[inline]
    #[must_use]
    pub const fn auto(name: &'static str, tag: u64) -> Self {
        Self {
            name,
            tag,
            viewport_rect: None,
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
            let empty_style = if let Some(explicit) = cfg.viewport_rect {
                Style::new()
                    .flex_col()
                    .width(explicit.width)
                    .height(explicit.height)
            } else {
                Style::new()
                    .flex_col()
                    .flex_grow(1.0)
                    .min_width(0.0)
                    .min_height(0.0)
            };
            let empty_fn = self.empty_fn;
            let resp = self.scope.container_tagged_resp(
                cfg.name,
                empty_style,
                WidgetRole::ScrollAreaViewport,
                cfg.tag,
                |vp| {
                    if let Some(empty) = empty_fn {
                        empty(vp);
                    }
                },
            );
            if let Some(ctx_fn) = self.context_menu_fn {
                resp.context_menu(self.scope, ctx_fn);
            }
            return None;
        }

        let vp_rect = cfg.viewport_rect.unwrap_or_else(|| {
            let avail = self.scope.available_rect();
            let rem_w = self.scope.resolve_remaining_axis_from_ancestors(true);
            let rem_h = self.scope.resolve_remaining_axis_from_ancestors(false);
            let cached = self
                .scope
                .tree
                .tag_rect(cfg.tag)
                .filter(|r| r.width > 1.0 && r.height > 1.0)
                .or_else(|| {
                    self.scope
                        .tree
                        .viewport_size(cfg.tag)
                        .filter(|s| s.width > 1.0 && s.height > 1.0)
                        .map(|s| Rect::new(avail.x, avail.y, s.width, s.height))
                });
            match cached {
                Some(c) => Rect::new(
                    c.x,
                    c.y,
                    rem_w.unwrap_or(c.width),
                    rem_h.unwrap_or(c.height),
                ),
                None => Rect::new(
                    avail.x,
                    avail.y,
                    rem_w.unwrap_or(avail.width),
                    rem_h.unwrap_or(avail.height),
                ),
            }
        });

        let grid = ResponsiveGrid::new(cfg.item_width, cfg.item_height)
            .spacing(cfg.spacing.0, cfg.spacing.1)
            .padding(cfg.padding.0, cfg.padding.1);
        let content_h = grid.compute_content_height(total, vp_rect.width);
        let max_scroll = (content_h - vp_rect.height).max(0.0);
        let scroll_y = self.scope.scroll_offset(cfg.tag).clamp(0.0, max_scroll);
        if cfg.tag != 0 {
            self.scope.tree.set_scroll_offset(cfg.tag, scroll_y);
            self.scope.tree.set_max_scroll(cfg.tag, max_scroll);
        }

        let host_style = if let Some(explicit) = cfg.viewport_rect {
            Style::new()
                .flex_col()
                .width(explicit.width)
                .height(explicit.height)
        } else {
            Style::new()
                .flex_col()
                .flex_grow(1.0)
                .min_width(0.0)
                .min_height(0.0)
        };
        let mut vp_style = Style::new()
            .flex_col()
            .flex_grow(1.0)
            .min_width(0.0)
            .min_height(0.0)
            .clip_children(true)
            .scroll_offset_y(scroll_y);
        if let Some(bg) = cfg.background {
            vp_style = vp_style.background(bg);
        }

        let context_menu_fn = self.context_menu_fn;
        let host_id = self.scope.tree.create_node();
        if let Some(host) = self.scope.tree.get_mut(host_id) {
            host.set_name("VirtualGridHost");
            host.set_style(host_style);
        }
        let _ = self.scope.tree.add_child(self.scope.parent, host_id);

        let mut host_scope = self.scope.child_scope(host_id);
        let resp = host_scope
            .container_tagged_resp(
                cfg.name,
                vp_style,
                WidgetRole::ScrollAreaViewport,
                cfg.tag,
                |viewport| {
                    grid.for_each_visible_cell(
                        vp_rect,
                        total,
                        scroll_y,
                        |item_idx, local_pos, _world_rect| {
                            render_cell(viewport, item_idx, local_pos);
                        },
                    );
                },
            )
            .content_size(&mut host_scope, Size::new(vp_rect.width, content_h));

        if let Some(ctx_fn) = context_menu_fn {
            resp.context_menu(&mut host_scope, ctx_fn);
        }

        if let Some((track_tag, thumb_tag)) = cfg.scrollbar_tags
            && content_h > vp_rect.height
        {
            let scroll_style = ScrollAreaStyle::dark_default();
            if let Some(geom) =
                ScrollBarGeometry::compute_vertical(vp_rect, content_h, scroll_y, &scroll_style)
            {
                let is_dragging = host_scope.is_tag_captured(thumb_tag);
                let cursor_pos = host_scope.cursor_pos();
                host_scope.scrollbar_vertical(
                    geom,
                    vp_rect,
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

    #[test]
    fn test_virtual_grid_empty_context_menu_and_auto_dock_resize() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("root");
        let grid_tag = 7701;

        {
            let mut scope = UiScope::new(&mut tree, root);
            scope.container_named(
                "DockLeaf",
                Style::new().flex_col().width(500.0).height(300.0),
                |leaf| {
                    leaf.virtual_grid(0, VirtualGridConfig::auto("EmptyGrid", grid_tag))
                        .empty_state(|vp| {
                            vp.label("EmptyGridHere");
                        })
                        .context_menu(|menu| {
                            menu.label("CreateHere");
                        })
                        .render_item(|_, _, _| {});
                },
            );
            scope.finish_layout(Rect::new(0.0, 0.0, 500.0, 300.0));
        }

        assert!(
            tree.iter()
                .any(|(_, n)| n.text.as_deref() == Some("EmptyGridHere"))
        );
        let click_pt = Point::new(50.0, 50.0);
        assert!(
            tree.dispatch_event(&irisui_core::UiEvent::MouseDown {
                button: irisui_core::MouseButton::Right,
                point: click_pt,
            })
            .is_consumed()
        );
        let _ = tree.dispatch_event(&irisui_core::UiEvent::MouseUp {
            button: irisui_core::MouseButton::Right,
            point: click_pt,
        });
        tree.clear();
        let root_ctx = tree.create_root().expect("root_ctx");
        {
            let mut scope = UiScope::new(&mut tree, root_ctx);
            scope.container_named(
                "DockLeaf",
                Style::new().flex_col().width(500.0).height(300.0),
                |leaf| {
                    leaf.virtual_grid(0, VirtualGridConfig::auto("EmptyGrid", grid_tag))
                        .empty_state(|vp| {
                            vp.label("EmptyGridHere");
                        })
                        .context_menu(|menu| {
                            menu.label("CreateHere");
                        })
                        .render_item(|_, _, _| {});
                },
            );
            scope.finish_layout(Rect::new(0.0, 0.0, 500.0, 300.0));
        }
        assert!(
            tree.iter()
                .any(|(_, n)| n.text.as_deref() == Some("CreateHere"))
        );

        // Resize dock width from stale 900px down to 260px -> should fit 1-2 cols max
        tree.clear();
        tree.set_tag_rect(grid_tag, Rect::new(0.0, 0.0, 900.0, 300.0));
        let root2 = tree.create_root().expect("root2");
        let mut max_x = 0.0_f32;
        {
            let mut scope = UiScope::new(&mut tree, root2);
            scope.container_named(
                "DockLeaf",
                Style::new().flex_col().width(260.0).height(300.0),
                |leaf| {
                    leaf.virtual_grid(
                        12,
                        VirtualGridConfig::auto("AutoGrid", grid_tag)
                            .with_cell_size(110.0, 100.0, 8.0),
                    )
                    .render_item(|vp, idx, pos| {
                        max_x = max_x.max(pos.x + 110.0);
                        vp.empty_box(
                            Style::new()
                                .position_absolute()
                                .left(pos.x)
                                .top(pos.y)
                                .width(110.0)
                                .height(100.0),
                        );
                        let _ = idx;
                    });
                },
            );
            scope.finish_layout(Rect::new(0.0, 0.0, 260.0, 300.0));
        }
        assert!(
            max_x <= 260.0,
            "Grid must lay out within new 260px width rather than stale 900px, got max_x={max_x}"
        );
    }
}