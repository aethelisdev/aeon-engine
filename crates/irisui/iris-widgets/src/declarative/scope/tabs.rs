// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Sidebar Tabs & Navigation Strips
//!
//! Provides autonomous vertical sidebar and horizontal tab navigation primitives on [`UiScope`].
//! Automatically resolves active selection, evaluates click interactions, updates bound state,
//! resets linked viewport scroll offsets on tab switches, and emits strongly typed responses

use super::core::UiScope;
use iris_core::Color;
use iris_core::geometry::Insets;
use iris_core::node::WidgetRole;
use iris_core::style::{AlignItems, Style, TextAlign};

/// Outcome emitted when rendering a declarative tab strip.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TabBarResponse<T> {
    /// Active tab identifier after evaluating interactions.
    pub active_tab: T,
    /// Whether the active tab changed during the current frame.
    pub changed: bool,
    /// Explicit tab clicked during this frame, if any.
    pub clicked_tab: Option<T>,
}

impl<T: Copy + PartialEq> TabBarResponse<T> {
    /// Returns `true` if the selected tab changed during this interaction pass.
    #[inline]
    #[must_use]
    pub const fn changed(&self) -> bool {
        self.changed
    }

    /// Returns the currently active tab identifier.
    #[inline]
    #[must_use]
    pub const fn active(&self) -> T {
        self.active_tab
    }
}

impl<'a> UiScope<'a> {
    /// Renders an autonomous vertical sidebar tab strip with default hashed semantic tags.
    ///
    /// Evaluates click interactions against the active tree state, mutates `*active_tab` directly
    /// when an item is clicked, and resets the scroll offset of `scroll_reset_tag` to `0.0`
    /// when switching to a different tab.
    ///
    /// # Arguments
    /// * `tabs` - Slice of tab descriptor pairs: `(display_label, tab_id)`.
    /// * `active_tab` - Mutable reference to the bound active tab identifier.
    /// * `scroll_reset_tag` - Optional 64-bit semantic tag of a viewport whose scroll offset
    ///   is reset to `0.0` upon tab switching.
    pub fn sidebar_tabs<T: Copy + PartialEq + 'static>(
        &mut self,
        tabs: &[(&'static str, T)],
        active_tab: &mut T,
        scroll_reset_tag: Option<u64>,
    ) -> TabBarResponse<T> {
        let mut changed = false;
        let mut clicked_tab = None;
        let initial_active = *active_tab;

        let sidebar_bg = self.theme().panel_bg;
        let text_secondary = self.theme().text_secondary;

        self.container_named(
            "SidebarTabs",
            Style::new()
                .flex_col()
                .padding_insets(Insets::new(8.0, 6.0, 8.0, 6.0))
                .background(sidebar_bg),
            |col| {
                for &(label, tab_id) in tabs {
                    let is_active = initial_active == tab_id;
                    let (bg, hover_bg, text_col) = if is_active {
                        (
                            Color::rgba(0.0, 0.65, 0.85, 0.12),
                            Color::rgba(0.0, 0.65, 0.85, 0.18),
                            Color::rgba(0.0, 0.90, 1.0, 1.0),
                        )
                    } else {
                        (
                            Color::TRANSPARENT,
                            Color::rgba(0.18, 0.20, 0.28, 0.50),
                            text_secondary,
                        )
                    };

                    let tab_tag = col.tag_for(label);
                    let resp = col.container_tagged_resp(
                        "SidebarTabItem",
                        Style::new()
                            .flex_row()
                            .align_items(AlignItems::Center)
                            .height(30.0)
                            .padding_insets(Insets::new(0.0, 10.0, 0.0, 0.0))
                            .margin_insets(Insets::new(0.0, 0.0, 2.0, 0.0))
                            .border_radius(4.0)
                            .clip_children(true)
                            .background(bg)
                            .hover_background(hover_bg),
                        WidgetRole::Button,
                        tab_tag,
                        |btn| {
                            let bar_color = if is_active {
                                Color::rgba(0.0, 0.85, 1.0, 1.0)
                            } else {
                                Color::TRANSPARENT
                            };

                            btn.empty_box_passive_named(
                                "TabAccentBar",
                                Style::new().width(3.0).height(30.0).background(bar_color),
                            );

                            btn.empty_box_passive_named(
                                "TabSpacer",
                                Style::new().width(11.0).height(1.0),
                            );

                            btn.label_styled_hover_passive(
                                "TabLabel",
                                label,
                                12.0,
                                (text_col, Color::WHITE),
                                TextAlign::Left,
                                Style::new().flex_grow(1.0),
                            );
                        },
                    );

                    if resp.clicked() {
                        *active_tab = tab_id;
                        clicked_tab = Some(tab_id);
                        if tab_id != initial_active {
                            changed = true;
                            if let Some(tag) = scroll_reset_tag {
                                col.tree_mut().set_scroll_offset(tag, 0.0);
                            }
                        }
                    }
                }
            },
        );

        TabBarResponse {
            active_tab: *active_tab,
            changed,
            clicked_tab,
        }
    }

    /// Renders an autonomous vertical sidebar tab strip with explicitly provided semantic tags.
    ///
    /// Preserves exact 64-bit tag values for backward compatibility and specialized hit targets
    /// without any heap allocations.
    ///
    /// # Arguments
    /// * `tabs` - Slice of tab descriptor triplets: `(display_label, tab_id, semantic_tag)`.
    /// * `active_tab` - Mutable reference to the bound active tab identifier.
    /// * `scroll_reset_tag` - Optional 64-bit semantic tag of a viewport whose scroll offset
    ///   is reset to `0.0` upon tab switching.
    pub fn sidebar_tabs_tagged<T: Copy + PartialEq + 'static>(
        &mut self,
        tabs: &[(&'static str, T, u64)],
        active_tab: &mut T,
        scroll_reset_tag: Option<u64>,
    ) -> TabBarResponse<T> {
        let mut changed = false;
        let mut clicked_tab = None;
        let initial_active = *active_tab;

        let sidebar_bg = self.theme().panel_bg;
        let text_secondary = self.theme().text_secondary;

        self.container_named(
            "SidebarTabs",
            Style::new()
                .flex_col()
                .padding_insets(Insets::new(8.0, 6.0, 8.0, 6.0))
                .background(sidebar_bg),
            |col| {
                for &(label, tab_id, tab_tag) in tabs {
                    let is_active = initial_active == tab_id;
                    let (bg, hover_bg, text_col) = if is_active {
                        (
                            Color::rgba(0.0, 0.65, 0.85, 0.12),
                            Color::rgba(0.0, 0.65, 0.85, 0.18),
                            Color::rgba(0.0, 0.90, 1.0, 1.0),
                        )
                    } else {
                        (
                            Color::TRANSPARENT,
                            Color::rgba(0.18, 0.20, 0.28, 0.50),
                            text_secondary,
                        )
                    };

                    let resp = col.container_tagged_resp(
                        "SidebarTabItem",
                        Style::new()
                            .flex_row()
                            .align_items(AlignItems::Center)
                            .height(30.0)
                            .padding_insets(Insets::new(0.0, 10.0, 0.0, 0.0))
                            .margin_insets(Insets::new(0.0, 0.0, 2.0, 0.0))
                            .border_radius(4.0)
                            .clip_children(true)
                            .background(bg)
                            .hover_background(hover_bg),
                        WidgetRole::Button,
                        tab_tag,
                        |btn| {
                            let bar_color = if is_active {
                                Color::rgba(0.0, 0.85, 1.0, 1.0)
                            } else {
                                Color::TRANSPARENT
                            };

                            btn.empty_box_passive_named(
                                "TabAccentBar",
                                Style::new().width(3.0).height(30.0).background(bar_color),
                            );

                            btn.empty_box_passive_named(
                                "TabSpacer",
                                Style::new().width(11.0).height(1.0),
                            );

                            btn.label_styled_hover_passive(
                                "TabLabel",
                                label,
                                12.0,
                                (text_col, Color::WHITE),
                                TextAlign::Left,
                                Style::new().flex_grow(1.0),
                            );
                        },
                    );

                    if resp.clicked() {
                        *active_tab = tab_id;
                        clicked_tab = Some(tab_id);
                        if tab_id != initial_active {
                            changed = true;
                            if let Some(tag) = scroll_reset_tag {
                                col.tree_mut().set_scroll_offset(tag, 0.0);
                            }
                        }
                    }
                }
            },
        );

        TabBarResponse {
            active_tab: *active_tab,
            changed,
            clicked_tab,
        }
    }
}