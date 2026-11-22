// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Virtualized Data Table (`UiScope::data_table`)
//!
//! Provides a reusable, responsive multi-column table header and $O(1)$ virtualized row
//! viewport on [`UiScope`]. Eliminates manual column width subtraction arithmetic
//! (`vp_rect.width - 16.0`, proportional multiplication, remainder clamping) from consumers.

use super::context_menu::ContextMenuExt;
use super::core::UiScope;
use crate::declarative::widget::VirtualListResponse;
use irisui_core::{AlignItems, Color, Insets, Rect, Style, TextAlign, WidgetRole};

/// Column sizing rule for [`TableColumnSpec`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TableColumnWidth {
    /// Fixed width in logical pixels.
    Fixed(f32),
    /// Proportional share of total table width (`weight` in `0.0..=1.0`), clamped to at least `min_width`.
    Proportional {
        /// Fractional width weight relative to available horizontal space.
        weight: f32,
        /// Minimum column width in logical pixels.
        min_width: f32,
    },
    /// Consumes the remaining horizontal width after `Fixed` and `Proportional` columns, clamped to `min_width`.
    Remainder {
        /// Minimum column width in logical pixels.
        min_width: f32,
    },
}

/// Declarative column specification for [`DataTableConfig`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TableColumnSpec {
    /// Debug node name assigned to the column header label.
    pub node_name: &'static str,
    /// Display title rendered inside the column header.
    pub title: &'static str,
    /// Sizing policy for the column.
    pub width: TableColumnWidth,
    /// Horizontal text alignment inside the column header.
    pub align: TextAlign,
}

impl TableColumnSpec {
    /// Defines a fixed-width table column.
    #[inline]
    #[must_use]
    pub const fn fixed(
        node_name: &'static str,
        title: &'static str,
        width: f32,
        align: TextAlign,
    ) -> Self {
        Self {
            node_name,
            title,
            width: TableColumnWidth::Fixed(width),
            align,
        }
    }

    /// Defines a proportional-width table column with a minimum width clamp.
    #[inline]
    #[must_use]
    pub const fn proportional(
        node_name: &'static str,
        title: &'static str,
        weight: f32,
        min_width: f32,
        align: TextAlign,
    ) -> Self {
        Self {
            node_name,
            title,
            width: TableColumnWidth::Proportional { weight, min_width },
            align,
        }
    }

    /// Defines a remainder-width table column consuming leftover horizontal space.
    #[inline]
    #[must_use]
    pub const fn remainder(
        node_name: &'static str,
        title: &'static str,
        min_width: f32,
        align: TextAlign,
    ) -> Self {
        Self {
            node_name,
            title,
            width: TableColumnWidth::Remainder { min_width },
            align,
        }
    }
}

/// Configuration descriptor for [`UiScope::data_table`].
#[derive(Debug, Clone, Copy)]
pub struct DataTableConfig<'a> {
    /// Debug node name for the outer table viewport container.
    pub viewport_name: &'static str,
    /// Debug node name for the column header bar container.
    pub header_name: &'static str,
    /// Debug node name for the virtualized rows container.
    pub rows_name: &'static str,
    /// Semantic tag shared by the table viewport and virtualized list scroll state.
    pub tag: u64,
    /// Optional explicit bounding rectangle of the table viewport in logical pixels.
    /// When `None`, resolves autonomously from retained tag/parent layout geometry.
    pub viewport_rect: Option<Rect>,
    /// Height of the sticky column header bar in logical pixels.
    pub header_height: f32,
    /// Uniform height of each virtualized row in logical pixels.
    pub row_height: f32,
    /// Vertical gap between rows in logical pixels.
    pub row_gap: f32,
    /// Horizontal inset padding applied to both the header bar and rows container.
    pub horizontal_padding: f32,
    /// Column specifications defining the header layout.
    pub columns: &'a [TableColumnSpec],
    /// Optional `(track_tag, thumb_tag)` pair for rendering an interactive vertical scrollbar.
    pub scrollbar_tags: Option<(u64, u64)>,
}

impl<'a> DataTableConfig<'a> {
    /// Creates a new data table configuration with an explicit viewport rectangle and
    /// standard IDE defaults (26px header, 28px rows, 8px horizontal padding).
    #[inline]
    #[must_use]
    pub const fn new(
        viewport_name: &'static str,
        tag: u64,
        viewport_rect: Rect,
        columns: &'a [TableColumnSpec],
    ) -> Self {
        Self {
            viewport_name,
            header_name: "ListTableHeader",
            rows_name: "DataTableRowsContainer",
            tag,
            viewport_rect: Some(viewport_rect),
            header_height: 26.0,
            row_height: 28.0,
            row_gap: 1.0,
            horizontal_padding: 8.0,
            columns,
            scrollbar_tags: None,
        }
    }

    /// Creates a data table configuration whose viewport dimensions resolve automatically
    /// from retained layout geometry (`UiTree::tag_rect` / `UiScope::available_rect`)
    /// while expanding responsively inside its parent flexbox container.
    #[inline]
    #[must_use]
    pub const fn auto(
        viewport_name: &'static str,
        tag: u64,
        columns: &'a [TableColumnSpec],
    ) -> Self {
        Self {
            viewport_name,
            header_name: "ListTableHeader",
            rows_name: "DataTableRowsContainer",
            tag,
            viewport_rect: None,
            header_height: 26.0,
            row_height: 28.0,
            row_gap: 1.0,
            horizontal_padding: 8.0,
            columns,
            scrollbar_tags: None,
        }
    }

    /// Overrides the debug node names for the header bar and virtualized rows container.
    #[inline]
    #[must_use]
    pub const fn with_section_names(
        mut self,
        header_name: &'static str,
        rows_name: &'static str,
    ) -> Self {
        self.header_name = header_name;
        self.rows_name = rows_name;
        self
    }

    /// Overrides the header bar height, row height, and vertical row gap.
    #[inline]
    #[must_use]
    pub const fn with_row_metrics(
        mut self,
        header_height: f32,
        row_height: f32,
        row_gap: f32,
    ) -> Self {
        self.header_height = header_height;
        self.row_height = row_height;
        self.row_gap = row_gap;
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

type DefaultTableEmptyFn = fn(&mut UiScope<'_>);
type DefaultTableContextMenuFn = fn(&mut UiScope<'_>);

/// Default builder alias returned by [`UiScope::data_table`].
pub type DefaultDataTableBuilder<'s, 'a, 'c> =
    DataTableBuilder<'s, 'a, 'c, DefaultTableEmptyFn, DefaultTableContextMenuFn>;

/// Fluent builder for a virtualized multi-column data table.
pub struct DataTableBuilder<'s, 'a, 'c, EmptyFn, CtxFn> {
    scope: &'s mut UiScope<'a>,
    total_rows: usize,
    config: DataTableConfig<'c>,
    empty_fn: Option<EmptyFn>,
    context_menu_fn: Option<CtxFn>,
}

impl<'s, 'a, 'c, EmptyFn, CtxFn> DataTableBuilder<'s, 'a, 'c, EmptyFn, CtxFn>
where
    EmptyFn: FnOnce(&mut UiScope<'_>),
    CtxFn: FnOnce(&mut UiScope<'_>),
{
    /// Registers an empty-state builder closure invoked when `total_rows == 0`.
    #[inline]
    #[must_use]
    pub fn empty_state<NewEmpty>(self, f: NewEmpty) -> DataTableBuilder<'s, 'a, 'c, NewEmpty, CtxFn>
    where
        NewEmpty: FnOnce(&mut UiScope<'_>),
    {
        DataTableBuilder {
            scope: self.scope,
            total_rows: self.total_rows,
            config: self.config,
            empty_fn: Some(f),
            context_menu_fn: self.context_menu_fn,
        }
    }

    /// Registers a background right-click context menu closure on the table viewport container.
    #[inline]
    #[must_use]
    pub fn context_menu<NewCtx>(self, f: NewCtx) -> DataTableBuilder<'s, 'a, 'c, EmptyFn, NewCtx>
    where
        NewCtx: FnOnce(&mut UiScope<'_>),
    {
        DataTableBuilder {
            scope: self.scope,
            total_rows: self.total_rows,
            config: self.config,
            empty_fn: self.empty_fn,
            context_menu_fn: Some(f),
        }
    }

    /// Renders the column header bar and virtualized rows via `render_row(row_scope, row_index)`.
    pub fn render_row<F>(self, mut render_row: F) -> Option<VirtualListResponse>
    where
        F: FnMut(&mut UiScope<'_>, usize),
    {
        let cfg = self.config;
        let total = self.total_rows;
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
                cfg.rows_name,
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

        let total_w = (vp_rect.width - cfg.horizontal_padding * 2.0).max(0.0);
        let th_bg = self.scope.theme().card_header_bg;
        let th_border = self.scope.theme().divider;
        let scroll_y = self.scope.scroll_offset(cfg.tag);
        let available_table_h = (vp_rect.height - (cfg.header_height + 8.0)).max(10.0);

        let mut consumed_w = 0.0_f32;
        let mut remainder_count = 0_usize;
        for col in cfg.columns {
            match col.width {
                TableColumnWidth::Fixed(w) => consumed_w += w,
                TableColumnWidth::Proportional { weight, min_width } => {
                    consumed_w += (total_w * weight).max(min_width);
                }
                TableColumnWidth::Remainder { .. } => remainder_count += 1,
            }
        }
        let remainder_share = if remainder_count > 0 {
            ((total_w - consumed_w).max(0.0)) / (remainder_count as f32)
        } else {
            0.0
        };

        // 1. Multi-column header bar
        let mut header_style = Style::new()
            .flex_row()
            .align_items(AlignItems::Center)
            .height(cfg.header_height)
            .padding_insets(Insets::new(
                0.0,
                cfg.horizontal_padding,
                0.0,
                cfg.horizontal_padding,
            ))
            .margin_insets(Insets::new(0.0, 0.0, 4.0, 0.0))
            .background(th_bg)
            .border(1.0, th_border);
        if let Some(explicit) = cfg.viewport_rect {
            header_style = header_style.width(explicit.width);
        }
        self.scope
            .container_named(cfg.header_name, header_style, |th| {
                for col in cfg.columns {
                    let col_style = match col.width {
                        TableColumnWidth::Fixed(w) => Style::new().width(w),
                        TableColumnWidth::Proportional { weight, min_width } => {
                            Style::new().width((total_w * weight).max(min_width))
                        }
                        TableColumnWidth::Remainder { min_width } => {
                            if cfg.viewport_rect.is_none() {
                                Style::new().flex_grow(1.0).min_width(min_width)
                            } else {
                                Style::new().width(remainder_share.max(min_width))
                            }
                        }
                    };
                    th.label_styled_passive(
                        col.node_name,
                        col.title,
                        11.0,
                        Color::rgba(0.60, 0.65, 0.75, 1.0),
                        col.align,
                        col_style,
                    );
                }
            });

        // 2. Virtualized rows container
        let mut vlist = self
            .scope
            .virtual_list(total, cfg.row_height)
            .name(cfg.rows_name)
            .tag(cfg.tag)
            .margin_insets(Insets::new(
                0.0,
                cfg.horizontal_padding,
                0.0,
                cfg.horizontal_padding,
            ))
            .gap(cfg.row_gap)
            .scroll_y(scroll_y);
        if cfg.viewport_rect.is_some() {
            vlist = vlist.width(total_w).visible_height(available_table_h);
        }
        if let Some((track_tag, thumb_tag)) = cfg.scrollbar_tags {
            vlist = vlist.scrollbar_tags(track_tag, thumb_tag);
        }
        let resp = vlist.render_item(|rows_scope, idx| {
            render_row(rows_scope, idx);
        });

        if let Some(ctx_fn) = self.context_menu_fn {
            resp.context_menu(self.scope, ctx_fn);
        }

        Some(resp)
    }
}

impl<'a> UiScope<'a> {
    /// Creates a declarative virtualized multi-column data table builder.
    ///
    /// Solves proportional, fixed, and remainder column header widths automatically and binds
    /// a virtualized row list underneath.
    #[inline]
    #[must_use]
    pub fn data_table<'s, 'c>(
        &'s mut self,
        total_rows: usize,
        config: DataTableConfig<'c>,
    ) -> DefaultDataTableBuilder<'s, 'a, 'c> {
        DataTableBuilder {
            scope: self,
            total_rows,
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

    const TEST_COLUMNS: &[TableColumnSpec] = &[
        TableColumnSpec::proportional("ColNameHeader", "NAME", 0.40, 140.0, TextAlign::Left),
        TableColumnSpec::fixed("ColTypeHeader", "TYPE", 64.0, TextAlign::Center),
        TableColumnSpec::fixed("ColSizeHeader", "SIZE", 80.0, TextAlign::Right),
        TableColumnSpec::fixed("ColStatusHeader", "STATUS", 72.0, TextAlign::Center),
        TableColumnSpec::remainder("ColActionsHeader", "QUICK ACTIONS", 80.0, TextAlign::Right),
    ];

    #[test]
    fn test_data_table_solves_columns_and_virtualizes_rows() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("root");
        let vp_rect = Rect::new(0.0, 0.0, 600.0, 300.0);
        let cfg = DataTableConfig::new("AssetsContentViewport", 88, vp_rect, TEST_COLUMNS)
            .with_section_names("ListTableHeader", "AssetListRowsContainer");

        {
            let mut scope = UiScope::new(&mut tree, root);
            scope.data_table(200, cfg).render_row(|row, idx| {
                row.label_styled_passive(
                    "TestTableRow",
                    format!("Row {idx}"),
                    11.0,
                    Color::WHITE,
                    TextAlign::Left,
                    Style::new(),
                );
            });
        }

        assert!(
            tree.iter()
                .any(|(_, n)| n.name.as_deref() == Some("ListTableHeader"))
        );
        let name_col = tree
            .iter()
            .find(|(_, n)| n.name.as_deref() == Some("ColNameHeader"))
            .map(|(_, n)| n)
            .expect("ColNameHeader must exist");
        // total_w = 600 - 16 = 584; 584 * 0.40 = 233.6
        assert!((name_col.style.width.unwrap_or(0.0) - 233.6).abs() < 0.01);

        let row_count = tree
            .iter()
            .filter(|(_, n)| n.name.as_deref() == Some("TestTableRow"))
            .count();
        assert!(row_count > 0 && row_count < 25);
    }

    #[test]
    fn test_data_table_empty_context_menu_and_auto_dock_resize() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("root");
        let table_tag = 8801;

        // Right-click inside an empty DataTable must still open context menu
        {
            let mut scope = UiScope::new(&mut tree, root);
            scope.container_named(
                "DockLeaf",
                Style::new().flex_col().width(500.0).height(300.0),
                |leaf| {
                    leaf.data_table(
                        0,
                        DataTableConfig::auto("EmptyTable", table_tag, TEST_COLUMNS),
                    )
                    .empty_state(|vp| {
                        vp.label("NoRows");
                    })
                    .context_menu(|menu| {
                        menu.label("NewRowAction");
                    })
                    .render_row(|_, _| {});
                },
            );
            scope.finish_layout(Rect::new(0.0, 0.0, 500.0, 300.0));
        }

        assert!(
            tree.iter()
                .any(|(_, n)| n.text.as_deref() == Some("NoRows"))
        );
        let click_pt = irisui_core::Point::new(40.0, 40.0);
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
                    leaf.data_table(
                        0,
                        DataTableConfig::auto("EmptyTable", table_tag, TEST_COLUMNS),
                    )
                    .empty_state(|vp| {
                        vp.label("NoRows");
                    })
                    .context_menu(|menu| {
                        menu.label("NewRowAction");
                    })
                    .render_row(|_, _| {});
                },
            );
            scope.finish_layout(Rect::new(0.0, 0.0, 500.0, 300.0));
        }
        assert!(
            tree.iter()
                .any(|(_, n)| n.text.as_deref() == Some("NewRowAction"))
        );

        // Rebuild with a resized dock leaf and stale tag_rect from prior frame
        tree.clear();
        tree.set_tag_rect(table_tag, Rect::new(0.0, 0.0, 900.0, 300.0));
        let root2 = tree.create_root().expect("root2");
        {
            let mut scope = UiScope::new(&mut tree, root2);
            scope.container_named(
                "DockLeaf",
                Style::new().flex_col().width(420.0).height(300.0),
                |leaf| {
                    leaf.data_table(
                        5,
                        DataTableConfig::auto("AutoTable", table_tag, TEST_COLUMNS),
                    )
                    .render_row(|row, idx| {
                        row.label_styled_passive(
                            "TestTableRow",
                            format!("Row {idx}"),
                            11.0,
                            Color::WHITE,
                            TextAlign::Left,
                            Style::new(),
                        );
                    });
                },
            );
            scope.finish_layout(Rect::new(0.0, 0.0, 420.0, 300.0));
        }

        // total_w = 420 - 16 = 404; 404 * 0.40 = 161.6 (NOT stale 900)
        let name_col = tree
            .iter()
            .find(|(_, n)| n.name.as_deref() == Some("ColNameHeader"))
            .map(|(_, n)| n)
            .expect("ColNameHeader");
        assert!((name_col.style.width.unwrap_or(0.0) - 161.6).abs() < 0.05);

        let rem_col = tree
            .iter()
            .find(|(_, n)| n.name.as_deref() == Some("ColActionsHeader"))
            .map(|(_, n)| n)
            .expect("ColActionsHeader");
        assert_eq!(rem_col.style.flex_grow, 1.0);
        assert_eq!(rem_col.style.width, None);
    }
}