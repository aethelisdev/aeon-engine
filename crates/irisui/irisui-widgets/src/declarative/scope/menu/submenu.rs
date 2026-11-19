// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Cascading Submenu Primitives (`irisui-widgets::declarative::scope::menu::submenu`)
//!
//! Renders hover-activated cascading submenu rows and auto-positioned child popup cards
//! with smart horizontal left/right flipping on [`UiScope`].

use super::super::containers::InnerResponse;
use super::super::core::UiScope;
use super::super::scoped_popup::resolve_viewport_extent;
use super::types::DropdownIcon;
use crate::declarative::types::WidgetResponse;
use irisui_core::Rect;

impl<'a> UiScope<'a> {
    /// Emits an autonomous cascading submenu row and executes a child builder closure if active.
    ///
    /// Returns an [`InnerResponse`] containing the item row's [`WidgetResponse`] and the optional
    /// result returned by `f`.
    pub fn submenu_named_with_inner<R, F>(
        &mut self,
        popup_name: &'static str,
        label: impl Into<String>,
        icon: Option<&str>,
        tag: u64,
        f: F,
    ) -> InnerResponse<Option<R>>
    where
        F: FnOnce(&mut UiScope<'_>) -> R,
    {
        let icon_val = match icon {
            Some(s) if !s.is_empty() => DropdownIcon::Text(s),
            _ => DropdownIcon::None,
        };
        self.render_submenu_impl(popup_name, label, icon_val, tag, f)
    }

    /// Emits an elevated, cascading branching submenu item row supporting optional text or texture atlas icons.
    pub fn submenu_named_with_icon_inner<R, F>(
        &mut self,
        popup_name: &'static str,
        label: impl Into<String>,
        icon: Option<crate::cascading_menu::CascadingMenuIcon>,
        tag: u64,
        f: F,
    ) -> InnerResponse<Option<R>>
    where
        F: FnOnce(&mut UiScope<'_>) -> R,
    {
        self.render_submenu_impl(popup_name, label, icon.into(), tag, f)
    }

    /// Emits an elevated, cascading branching submenu item row using unified [`irisui_core::UiIcon`] and [`irisui_core::UiTag`].
    ///
    /// Hovering opens the nested submenu automatically using `UiTree` SSOT state without caller-side submenu booleans.
    pub fn submenu_named_with_ui_icon_inner<R, F>(
        &mut self,
        popup_name: &'static str,
        label: impl Into<String>,
        icon: Option<irisui_core::UiIcon>,
        tag: irisui_core::UiTag,
        f: F,
    ) -> InnerResponse<Option<R>>
    where
        F: FnOnce(&mut UiScope<'_>) -> R,
    {
        self.render_submenu_impl(popup_name, label, icon.into(), tag.raw(), f)
    }

    /// Internal rendering implementation for cascading submenu rows and floating popup cards.
    fn render_submenu_impl<R, F>(
        &mut self,
        popup_name: &'static str,
        label: impl Into<String>,
        icon: DropdownIcon<'_>,
        tag: u64,
        f: F,
    ) -> InnerResponse<Option<R>>
    where
        F: FnOnce(&mut UiScope<'_>) -> R,
    {
        let label_str = label.into();
        let parent_card_id = self.parent;
        let parent_card_rect = self
            .tree
            .get(parent_card_id)
            .map(|n| n.computed_rect)
            .unwrap_or(Rect::ZERO);

        let parent_tag = if let Some(n) = self.tree.get(parent_card_id) {
            if n.tag != 0 {
                n.tag
            } else {
                let name = n.name.as_deref().unwrap_or("MenuCard");
                crate::declarative::types::hash_label(name)
            }
        } else {
            0
        };

        let resp = self.render_dropdown_item_impl(tag, icon, &label_str, Some("▶"), true);

        let is_hovered = resp.hovered()
            || self.tree.cursor_position().is_some_and(|c| {
                self.tree
                    .get(resp.id)
                    .is_some_and(|n| n.computed_rect.contains_point(c))
            });

        if is_hovered {
            self.tree.set_active_submenu(parent_tag, tag);
        }

        let inner = if self.tree.is_submenu_open(parent_tag, tag) {
            let item_rect = self
                .tree
                .get(resp.id)
                .map(|n| n.computed_rect)
                .unwrap_or(Rect::ZERO);

            let (window_w, _) = resolve_viewport_extent(self);
            let sub_w = if parent_card_rect.width > 0.0 {
                parent_card_rect.width
            } else {
                185.0
            };

            let mut sub_x = parent_card_rect.right() + 2.0;
            if sub_x + sub_w > window_w - 6.0 {
                let left_candidate = parent_card_rect.x - sub_w - 2.0;
                if left_candidate >= 6.0 {
                    sub_x = left_candidate;
                } else {
                    sub_x = (window_w - sub_w - 6.0).max(6.0);
                }
            }

            let sub_y = (item_rect.y - 4.0).max(6.0);
            let root_anchor = self.tree.root().unwrap_or(parent_card_id);
            let child_seed = crate::declarative::types::combine_seeds(
                crate::declarative::types::combine_seeds(self.seed, tag),
                crate::declarative::types::hash_label(popup_name),
            );
            let mut sub_scope = self.child_scope_seeded(root_anchor, child_seed);
            let inner_res =
                sub_scope.dropdown_menu_card_named(popup_name, sub_x, sub_y, sub_w, |card| {
                    if let Some(n) = card.tree.get_mut(card.parent) {
                        n.tag = tag;
                    }
                    f(card)
                });
            Some(inner_res.inner)
        } else {
            None
        };

        InnerResponse::new(resp, inner)
    }

    /// Emits a cascading submenu row and executes a child builder closure if the submenu is active.
    pub fn submenu_named<R, F>(
        &mut self,
        popup_name: &'static str,
        label: impl Into<String>,
        icon: Option<&str>,
        tag: u64,
        f: F,
    ) -> WidgetResponse
    where
        F: FnOnce(&mut UiScope<'_>) -> R,
    {
        self.submenu_named_with_inner(popup_name, label, icon, tag, f)
            .response
    }

    /// Emits an elevated, cascading branching submenu item row with default popup container naming.
    pub fn submenu<R, F>(
        &mut self,
        label: impl Into<String>,
        icon: Option<&str>,
        tag: u64,
        f: F,
    ) -> WidgetResponse
    where
        F: FnOnce(&mut UiScope<'_>) -> R,
    {
        self.submenu_named("CascadingSubmenuPopup", label, icon, tag, f)
    }
}