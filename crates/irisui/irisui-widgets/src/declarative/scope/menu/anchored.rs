// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Anchored & Cascading Floating Popups (`irisui-widgets::declarative::scope::menu::anchored`)
//!
//! Coordinates viewport-clamped anchored popups, height caching for upward flips,
//! tag-based anchor resolution, and union bounding boxes for active cascading submenus.

use super::super::containers::InnerResponse;
use super::super::core::UiScope;
use super::super::scoped_popup::{
    AnchoredPopupConfig, create_popup_card_node, resolve_viewport_extent,
};
use super::types::{CascadingPopupConfig, PopupPlacement};
use irisui_core::{Rect, UiLayer, WidgetRole};

impl<'a> UiScope<'a> {
    /// Emits an elevated floating popup card container dynamically anchored to `anchor_rect`.
    pub fn popup_anchored<R, F>(&mut self, anchor_rect: Rect, width: f32, f: F) -> InnerResponse<R>
    where
        F: FnOnce(&mut UiScope<'_>) -> R,
    {
        self.popup_anchored_with_placement(
            "AnchoredPopup",
            anchor_rect,
            width,
            PopupPlacement::Below,
            f,
        )
    }

    /// Emits an elevated floating popup card container with an explicit debug name anchored to `anchor_rect`.
    pub fn popup_anchored_named<R, F>(
        &mut self,
        name: &'static str,
        anchor_rect: Rect,
        width: f32,
        f: F,
    ) -> InnerResponse<R>
    where
        F: FnOnce(&mut UiScope<'_>) -> R,
    {
        self.popup_anchored_with_placement(name, anchor_rect, width, PopupPlacement::Below, f)
    }

    /// Emits an elevated floating popup card container with explicit placement preference and boundary clamping.
    pub fn popup_anchored_with_placement<R, F>(
        &mut self,
        name: &'static str,
        anchor_rect: Rect,
        width: f32,
        placement: PopupPlacement,
        f: F,
    ) -> InnerResponse<R>
    where
        F: FnOnce(&mut UiScope<'_>) -> R,
    {
        self.popup_anchored_with_placement_and_height(name, anchor_rect, width, placement, None, f)
    }

    /// Emits an elevated floating popup card container with explicit placement preference,
    /// boundary clamping, and optional expected initial height.
    pub fn popup_anchored_with_placement_and_height<R, F>(
        &mut self,
        name: &'static str,
        anchor_rect: Rect,
        width: f32,
        placement: PopupPlacement,
        expected_height: Option<f32>,
        f: F,
    ) -> InnerResponse<R>
    where
        F: FnOnce(&mut UiScope<'_>) -> R,
    {
        let mut cfg = AnchoredPopupConfig::new(0, anchor_rect, width).with_placement(placement);
        cfg.expected_height = expected_height;
        self.popup_anchored_internal(name, anchor_rect, &cfg, f)
    }

    /// Shared internal builder resolving smart viewport clamping and theme-driven card styling.
    pub(in crate::declarative::scope) fn popup_anchored_internal<R, F>(
        &mut self,
        name: &'static str,
        anchor_rect: Rect,
        config: &AnchoredPopupConfig,
        f: F,
    ) -> InnerResponse<R>
    where
        F: FnOnce(&mut UiScope<'_>) -> R,
    {
        let width = config.width;
        let placement = config.placement;
        let expected_height = config.expected_height;
        let card_style = &config.card_style;
        let (window_w, window_h) = resolve_viewport_extent(self);

        let mut x = anchor_rect.x;
        if x + width > window_w - 6.0 {
            x = (anchor_rect.right() - width).clamp(6.0, window_w - width - 6.0);
        }

        let name_tag = crate::declarative::types::hash_label(name);
        let cached_h = self
            .tree
            .tag_rect(name_tag)
            .map(|r| r.height)
            .filter(|&h| h > 10.0);
        let initial_h = expected_height.or(cached_h).unwrap_or(200.0);

        let mut y = match placement {
            PopupPlacement::Below => anchor_rect.bottom() + 2.0,
            PopupPlacement::Above => {
                if anchor_rect.y - initial_h - 2.0 < 6.0 {
                    anchor_rect.bottom() + 2.0
                } else {
                    anchor_rect.y - initial_h - 2.0
                }
            }
        };
        let max_y = window_h - 28.0;
        if y + initial_h > max_y {
            y = (max_y - initial_h).max(6.0);
        }

        let node_id = create_popup_card_node(self, name, x, y, width, card_style);
        self.tree.set_popup_tag_active(name_tag, true);

        let child_seed = crate::declarative::types::combine_seeds(self.seed, name_tag);
        let mut child_scope = self.child_scope_seeded(node_id, child_seed);
        let inner = f(&mut child_scope);
        let height = crate::declarative::layout::measure_height(self.tree, node_id);

        y = match placement {
            PopupPlacement::Below => {
                if anchor_rect.bottom() + height + 4.0 > max_y {
                    (anchor_rect.y - height - 2.0).max(6.0)
                } else {
                    anchor_rect.bottom() + 2.0
                }
            }
            PopupPlacement::Above => {
                if anchor_rect.y - height - 2.0 < 6.0 {
                    anchor_rect.bottom() + 2.0
                } else {
                    anchor_rect.y - height - 2.0
                }
            }
        };
        if y + height > max_y {
            y = (max_y - height).max(6.0);
        }

        let bounds = Rect::new(x, y, width, height);
        crate::declarative::layout_subtree(self.tree, node_id, bounds);
        if let Some(node) = self.tree.get_mut(node_id) {
            node.computed_rect = bounds;
            node.style.inset_left = Some(x);
            node.style.inset_top = Some(y);
            node.style.height = Some(height);
        }
        self.tree.set_tag_rect(name_tag, bounds);
        self.tree.refresh_hover_from_cursor();
        let response = self.response(node_id);
        InnerResponse::new(response, inner)
    }

    /// Opens an anchored floating popup card positioned relative to a tagged anchor widget.
    ///
    /// If the tagged anchor is not found, falls back to `fallback_rect`.
    pub fn popup_anchored_to_tag<R, F>(
        &mut self,
        name: &'static str,
        anchor_tag: u64,
        fallback_rect: Rect,
        width: f32,
        placement: PopupPlacement,
        f: F,
    ) -> InnerResponse<R>
    where
        F: FnOnce(&mut UiScope<'_>) -> R,
    {
        if anchor_tag != 0 {
            self.tree.set_popup_tag_active(anchor_tag, true);
        }

        let anchor_rect = self
            .tree
            .resolve_anchor_rect(anchor_tag)
            .unwrap_or(fallback_rect);

        self.popup_anchored_with_placement(name, anchor_rect, width, placement, f)
    }

    /// Opens an elevated floating cascading popup card anchored to `anchor_tag`.
    ///
    /// Autonomously synchronizes `active_submenu_tag` to the underlying tree and
    /// calculates the union bounding box encompassing both the root card and any
    /// open submenu cards for accurate hit-testing.
    pub fn cascading_popup_anchored_to_tag<R, F>(
        &mut self,
        name: &'static str,
        anchor_tag: u64,
        fallback_rect: Rect,
        config: CascadingPopupConfig,
        f: F,
    ) -> InnerResponse<R>
    where
        F: FnOnce(&mut UiScope<'_>) -> R,
    {
        if anchor_tag != 0 {
            self.tree.set_popup_tag_active(anchor_tag, true);
        }

        if let Some(sub_tag) = config.active_submenu_tag {
            self.tree
                .set_active_submenu(crate::declarative::types::hash_label(name), sub_tag);
        }
        if let Some((p_tag, c_tag)) = config.active_nested_submenu {
            self.tree.set_active_submenu(p_tag, c_tag);
        }

        let anchor_rect = self
            .tree
            .resolve_anchor_rect(anchor_tag)
            .unwrap_or(fallback_rect);

        let mut cfg = AnchoredPopupConfig::new(anchor_tag, anchor_rect, config.width)
            .with_placement(config.placement)
            .with_card_style(config.card_style);
        cfg.expected_height = config.expected_height;

        let mut res = self.popup_anchored_internal(name, anchor_rect, &cfg, f);

        res.response.rect = self
            .tree
            .iter()
            .filter(|(_, n)| {
                n.layer == UiLayer::Popup
                    && n.role == WidgetRole::DropdownPopup
                    && n.id != res.response.id
                    && (n.name.as_deref().is_some_and(|nm| nm.ends_with("Submenu"))
                        || (n.tag != 0
                            && (config.active_submenu_tag == Some(n.tag)
                                || self.tree.active_submenus.values().any(|&v| v == n.tag))))
            })
            .fold(res.response.rect, |acc, (_, n)| acc.union(n.computed_rect));

        res
    }
}