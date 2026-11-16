// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Scoped Popups & Theme-Driven Card Styling (`irisui-widgets::declarative::scope::scoped_popup`)
//!
//! Provides customizable [`PopupCardStyle`], [`DropdownCardConfig`], and [`AnchoredPopupConfig`]
//! descriptors resolved against [`irisui_core::Theme`] alongside declarative tag-scoped popup
//! methods on [`UiScope`], eliminating per-panel boolean popup flags.

use super::containers::InnerResponse;
use super::core::UiScope;
use super::menu::{CascadingPopupConfig, PopupPlacement};
use crate::declarative::types::WidgetResponse;
use crate::declarative::widget::Widget;
use irisui_core::{Color, Insets, Rect, Style, Theme, UiLayer, WidgetId, WidgetRole};

/// Customizable visual style overrides for floating popup card containers.
///
/// Any `None` field falls back cleanly to the active [`irisui_core::Theme`] tokens
/// (`modal_bg`, `modal_border`, `corner_radius_md`), preventing hardcoded color literals.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct PopupCardStyle {
    /// Optional background fill override; defaults to `theme.modal_bg`.
    pub background: Option<Color>,
    /// Optional outline border color override; defaults to `theme.modal_border`.
    pub border_color: Option<Color>,
    /// Optional border stroke width in logical pixels; defaults to `1.0`.
    pub border_width: Option<f32>,
    /// Optional corner rounding radius in logical pixels; defaults to `theme.corner_radius_md`.
    pub border_radius: Option<f32>,
    /// Optional inner padding insets; defaults to uniform `4.0` px.
    pub padding: Option<Insets>,
    /// Optional drop shadow `(offset_x, offset_y, blur_radius, color)`.
    pub shadow: Option<(f32, f32, f32, Color)>,
}

impl PopupCardStyle {
    /// Creates an empty style descriptor that inherits all tokens from the active [`Theme`].
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            background: None,
            border_color: None,
            border_width: None,
            border_radius: None,
            padding: None,
            shadow: None,
        }
    }

    /// Overrides the popup card background fill color.
    #[inline]
    #[must_use]
    pub const fn with_background(mut self, color: Color) -> Self {
        self.background = Some(color);
        self
    }

    /// Overrides the popup card border stroke width and color.
    #[inline]
    #[must_use]
    pub const fn with_border(mut self, width: f32, color: Color) -> Self {
        self.border_width = Some(width);
        self.border_color = Some(color);
        self
    }

    /// Overrides the popup card corner radius in pixels.
    #[inline]
    #[must_use]
    pub const fn with_border_radius(mut self, radius: f32) -> Self {
        self.border_radius = Some(radius);
        self
    }

    /// Overrides inner container padding insets.
    #[inline]
    #[must_use]
    pub const fn with_padding(mut self, padding: Insets) -> Self {
        self.padding = Some(padding);
        self
    }

    /// Overrides the drop shadow `(offset_x, offset_y, blur_radius, color)`.
    #[inline]
    #[must_use]
    pub const fn with_shadow(
        mut self,
        offset_x: f32,
        offset_y: f32,
        blur_radius: f32,
        color: Color,
    ) -> Self {
        self.shadow = Some((offset_x, offset_y, blur_radius, color));
        self
    }

    /// Builds a flexbox popup card [`Style`] positioned at `(x, y)` with `width`,
    /// resolving any unset tokens against `theme`.
    #[must_use]
    pub fn resolve_style(&self, theme: &Theme, x: f32, y: f32, width: f32) -> Style {
        let bg = self.background.unwrap_or(theme.modal_bg);
        let border_col = self.border_color.unwrap_or(theme.modal_border);
        let border_w = self.border_width.unwrap_or(1.0);
        let radius = self.border_radius.unwrap_or(theme.corner_radius_md);
        let padding = self.padding.unwrap_or_else(|| Insets::all(4.0));
        let (sx, sy, sblur, scol) =
            self.shadow
                .unwrap_or((0.0, 6.0, 16.0, Color::rgba(0.0, 0.0, 0.0, 0.75)));

        Style::new()
            .position_absolute()
            .left(x)
            .top(y)
            .flex_col()
            .width(width)
            .padding_insets(padding)
            .background(bg)
            .border(border_w, border_col)
            .border_radius(radius)
            .box_shadow(sx, sy, sblur, scol)
    }
}

/// Configuration descriptor for positioning and styling a floating dropdown card.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DropdownCardConfig {
    /// Left horizontal screen offset in logical pixels.
    pub x: f32,
    /// Top vertical screen offset in logical pixels.
    pub y: f32,
    /// Card width in logical pixels.
    pub width: f32,
    /// Optional visual style overrides resolved against the active [`Theme`].
    pub card_style: PopupCardStyle,
}

impl DropdownCardConfig {
    /// Creates a dropdown card descriptor positioned at `(x, y)` with fixed `width`.
    #[inline]
    #[must_use]
    pub const fn new(x: f32, y: f32, width: f32) -> Self {
        Self {
            x,
            y,
            width,
            card_style: PopupCardStyle::new(),
        }
    }

    /// Applies custom popup card styling overrides.
    #[inline]
    #[must_use]
    pub const fn with_card_style(mut self, style: PopupCardStyle) -> Self {
        self.card_style = style;
        self
    }
}

/// Configuration descriptor for anchored floating popups relative to a trigger widget.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AnchoredPopupConfig {
    /// Semantic 64-bit tag of the anchor widget (`0` skips tag lookup/activation).
    pub anchor_tag: u64,
    /// Fallback bounds used when `anchor_tag` is absent or not yet laid out in the tree.
    pub fallback_rect: Rect,
    /// Preferred popup card width in logical pixels.
    pub width: f32,
    /// Preferred vertical orientation relative to the anchor rect.
    pub placement: PopupPlacement,
    /// Optional pre-known height hint eliminating initial upward-flip estimation jumps.
    pub expected_height: Option<f32>,
    /// Optional visual style overrides resolved against the active [`Theme`].
    pub card_style: PopupCardStyle,
}

impl AnchoredPopupConfig {
    /// Creates an anchored popup descriptor with default styling and below-anchor placement.
    #[inline]
    #[must_use]
    pub const fn new(anchor_tag: u64, fallback_rect: Rect, width: f32) -> Self {
        Self {
            anchor_tag,
            fallback_rect,
            width,
            placement: PopupPlacement::Below,
            expected_height: None,
            card_style: PopupCardStyle::new(),
        }
    }

    /// Sets vertical placement preference relative to the anchor bounds.
    #[inline]
    #[must_use]
    pub const fn with_placement(mut self, placement: PopupPlacement) -> Self {
        self.placement = placement;
        self
    }

    /// Supplies an expected popup height hint for accurate pre-layout vertical flipping.
    #[inline]
    #[must_use]
    pub const fn with_expected_height(mut self, height: f32) -> Self {
        self.expected_height = Some(height);
        self
    }

    /// Applies custom popup card styling overrides.
    #[inline]
    #[must_use]
    pub const fn with_card_style(mut self, style: PopupCardStyle) -> Self {
        self.card_style = style;
        self
    }
}

/// Resolves screen viewport dimensions from the active UI tree root, defaulting to `(1920.0, 1080.0)`.
#[inline]
#[must_use]
pub(super) fn resolve_viewport_extent(scope: &UiScope<'_>) -> (f32, f32) {
    scope
        .tree()
        .root()
        .and_then(|r| scope.tree().get(r))
        .map(|n| (n.computed_rect.width, n.computed_rect.height))
        .filter(|&(w, h)| w > 10.0 && h > 10.0)
        .unwrap_or((1920.0, 1080.0))
}

/// Allocates and attaches an elevated popup card container node on [`UiLayer::Popup`].
pub(super) fn create_popup_card_node(
    scope: &mut UiScope<'_>,
    name: &'static str,
    x: f32,
    y: f32,
    width: f32,
    card_style: &PopupCardStyle,
) -> WidgetId {
    let theme = scope.theme();
    let style = card_style.resolve_style(&theme, x, y, width);
    let node_id = scope.tree.create_node();
    if let Some(node) = scope.tree.get_mut(node_id) {
        node.set_name(name);
        node.set_role(WidgetRole::DropdownPopup);
        node.set_layer(UiLayer::Popup);
        node.interactive = true;
        node.tag = crate::declarative::types::hash_label(name);
        node.set_style(style);
        node.computed_rect.x = x;
        node.computed_rect.y = y;
        node.computed_rect.width = width;
    }
    let root_anchor = scope.tree.root().unwrap_or(scope.parent);
    let _ = scope.tree.add_child(root_anchor, node_id);
    node_id
}

impl<'a> UiScope<'a> {
    /// Emits a styled floating dropdown popup card positioned per `config` on [`UiLayer::Popup`].
    ///
    /// Resolves unspecified style tokens against the active [`irisui_core::Theme`] and registers
    /// `name`'s semantic tag in `active_popup_tags` for outside-click and `close_all_popups` dismissal.
    pub fn dropdown_menu_card_styled<R, F>(
        &mut self,
        name: &'static str,
        config: DropdownCardConfig,
        f: F,
    ) -> InnerResponse<R>
    where
        F: FnOnce(&mut UiScope<'_>) -> R,
    {
        let node_id = create_popup_card_node(
            self,
            name,
            config.x,
            config.y,
            config.width,
            &config.card_style,
        );
        let name_tag = crate::declarative::types::hash_label(name);
        self.tree.set_popup_tag_active(name_tag, true);
        let child_seed = crate::declarative::types::combine_seeds(self.seed, name_tag);
        let mut child_scope = self.child_scope_seeded(node_id, child_seed);
        let inner = f(&mut child_scope);
        let height = crate::declarative::layout::measure_height(self.tree, node_id);
        let (_, window_h) = resolve_viewport_extent(self);
        let max_y = window_h - 28.0;
        let clamped_y = if config.y + height > max_y {
            (max_y - height).max(6.0)
        } else {
            config.y
        };
        let bounds = Rect::new(config.x, clamped_y, config.width, height);
        crate::declarative::layout_subtree(self.tree, node_id, bounds);
        if let Some(node) = self.tree.get_mut(node_id) {
            node.computed_rect = bounds;
            node.style.inset_left = Some(config.x);
            node.style.inset_top = Some(clamped_y);
            node.style.height = Some(height);
        }
        self.tree.set_tag_rect(name_tag, bounds);
        self.tree.refresh_hover_from_cursor();
        let response = self.response(node_id);
        InnerResponse::new(response, inner)
    }

    /// Emits a scoped floating dropdown menu card when `popup_tag` is currently active in [`irisui_core::UiTree`].
    ///
    /// Eliminates panel-owned visibility booleans: triggers use [`UiScope::open_popup`] or
    /// [`UiScope::toggle_popup`] / [`UiScope::toggle_exclusive_popup`], while dismissal flows
    /// autonomously through [`UiScope::close_all_popups`], outside-click, or Escape handling.
    pub fn scoped_dropdown_menu_card_named<R, F>(
        &mut self,
        name: &'static str,
        popup_tag: u64,
        x: f32,
        y: f32,
        width: f32,
        f: F,
    ) -> Option<InnerResponse<R>>
    where
        F: FnOnce(&mut UiScope<'_>) -> R,
    {
        self.scoped_dropdown_menu_card(name, popup_tag, DropdownCardConfig::new(x, y, width), f)
    }

    /// Emits a customizable scoped floating dropdown card when `popup_tag` is currently active.
    pub fn scoped_dropdown_menu_card<R, F>(
        &mut self,
        name: &'static str,
        popup_tag: u64,
        config: DropdownCardConfig,
        f: F,
    ) -> Option<InnerResponse<R>>
    where
        F: FnOnce(&mut UiScope<'_>) -> R,
    {
        if !self.tree.is_popup_tag_active(popup_tag) {
            return None;
        }
        self.tree.set_popup_tag_active(popup_tag, true);
        Some(self.dropdown_menu_card_styled(name, config, f))
    }

    /// Emits an anchored popup card with explicit placement, height hints, and theme-aware style overrides.
    pub fn popup_anchored_styled<R, F>(
        &mut self,
        name: &'static str,
        config: AnchoredPopupConfig,
        f: F,
    ) -> InnerResponse<R>
    where
        F: FnOnce(&mut UiScope<'_>) -> R,
    {
        if config.anchor_tag != 0 {
            self.tree.set_popup_tag_active(config.anchor_tag, true);
        }

        let anchor_rect = self
            .tree
            .resolve_anchor_rect(config.anchor_tag)
            .unwrap_or(config.fallback_rect);

        self.popup_anchored_internal(name, anchor_rect, &config, f)
    }

    /// Emits a scoped floating popup anchored to `anchor_tag` only when `anchor_tag` is active.
    pub fn scoped_popup_anchored_to_tag<R, F>(
        &mut self,
        name: &'static str,
        anchor_tag: u64,
        fallback_rect: Rect,
        width: f32,
        placement: PopupPlacement,
        f: F,
    ) -> Option<InnerResponse<R>>
    where
        F: FnOnce(&mut UiScope<'_>) -> R,
    {
        self.scoped_popup_anchored(
            name,
            AnchoredPopupConfig::new(anchor_tag, fallback_rect, width).with_placement(placement),
            f,
        )
    }

    /// Emits a customizable scoped anchored popup only when `config.anchor_tag` is active.
    pub fn scoped_popup_anchored<R, F>(
        &mut self,
        name: &'static str,
        config: AnchoredPopupConfig,
        f: F,
    ) -> Option<InnerResponse<R>>
    where
        F: FnOnce(&mut UiScope<'_>) -> R,
    {
        if !self.tree.is_popup_tag_active(config.anchor_tag) {
            return None;
        }
        Some(self.popup_anchored_styled(name, config, f))
    }

    /// Emits a scoped cascading popup card anchored to `anchor_tag` only when `anchor_tag` is active.
    ///
    /// Combines zero-boolean visibility ownership in [`irisui_core::UiTree`] with automatic submenu
    /// union bounds calculation and customizable [`CascadingPopupConfig::card_style`].
    pub fn scoped_cascading_popup_anchored_to_tag<R, F>(
        &mut self,
        name: &'static str,
        anchor_tag: u64,
        fallback_rect: Rect,
        config: CascadingPopupConfig,
        f: F,
    ) -> Option<InnerResponse<R>>
    where
        F: FnOnce(&mut UiScope<'_>) -> R,
    {
        if !self.tree.is_popup_tag_active(anchor_tag) {
            return None;
        }
        Some(self.cascading_popup_anchored_to_tag(name, anchor_tag, fallback_rect, config, f))
    }

    /// Emits a widget that exclusively toggles `popup_tag` in [`irisui_core::UiTree::active_popup_tags`]
    /// when clicked, returning the widget's [`WidgetResponse`].
    ///
    /// Eliminates panel boolean flags and imperative `if resp.clicked()` plumbing in consumer toolbars.
    #[inline]
    pub fn add_popup_trigger<W: Widget<Response = WidgetResponse>>(
        &mut self,
        widget: W,
        popup_tag: u64,
    ) -> WidgetResponse {
        let resp = self.add(widget);
        if resp.clicked() {
            self.tree.toggle_exclusive_popup(popup_tag);
        }
        resp
    }

    /// Emits a tagged interactive container that exclusively toggles `popup_tag` in
    /// [`irisui_core::UiTree::active_popup_tags`] when clicked.
    ///
    /// Passes `is_popup_active` into `f` so consumer button visuals (icons, borders, backgrounds)
    /// adapt reactively with zero intermediate `let` statements or boolean state fields.
    #[inline]
    pub fn container_tagged_popup_toggle<F>(
        &mut self,
        name: &'static str,
        style: Style,
        role: WidgetRole,
        tag: u64,
        popup_tag: u64,
        f: F,
    ) -> WidgetResponse
    where
        F: FnOnce(&mut UiScope<'_>, bool),
    {
        let is_active = self.tree.is_popup_tag_active(popup_tag);
        let resp = self.container_tagged_resp(name, style, role, tag, |inner| f(inner, is_active));
        if resp.clicked() {
            self.tree.toggle_exclusive_popup(popup_tag);
        }
        resp
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::declarative::scope::menu::CascadingPopupConfig;
    use irisui_core::{Color, MouseButton, Point, Rect, Theme, UiEvent, UiTree};

    #[test]
    fn test_scoped_dropdown_menu_card_lifecycle_and_theme_colors() {
        let mut tree = UiTree::new();
        tree.theme = Theme {
            modal_bg: Color::rgba(0.12, 0.14, 0.18, 0.99),
            modal_border: Color::rgba(0.35, 0.40, 0.50, 0.95),
            corner_radius_md: 9.0,
            ..Theme::dark_default()
        };
        let root = tree.create_root().expect("root node");
        const DROPDOWN_TRIGGER_TAG: u64 = 0xAA01;

        // Closed initially -> scoped dropdown emits None
        {
            let mut scope = UiScope::new(&mut tree, root);
            let res = scope.scoped_dropdown_menu_card_named(
                "AspectDropdown",
                DROPDOWN_TRIGGER_TAG,
                100.0,
                80.0,
                160.0,
                |card| {
                    card.dropdown_item(1, "", "16:9", Some("✓"), true);
                },
            );
            assert!(res.is_none());
            assert!(!scope.has_open_popup());
        }

        // Open declaratively via toggle_popup (no bool needed)
        {
            let mut scope = UiScope::new(&mut tree, root);
            assert!(scope.toggle_popup(DROPDOWN_TRIGGER_TAG));
            let res = scope.scoped_dropdown_menu_card_named(
                "AspectDropdown",
                DROPDOWN_TRIGGER_TAG,
                100.0,
                80.0,
                160.0,
                |card| {
                    card.dropdown_item(1, "", "16:9", Some("✓"), true);
                },
            );
            assert!(res.is_some());
            let expected_bg = scope.theme().modal_bg;
            let expected_border = scope.theme().modal_border;
            let card_id = res.expect("open card").response.id;
            let card_node = scope.tree().get(card_id).expect("card node");
            assert_eq!(card_node.style.background_color, expected_bg);
            assert_eq!(card_node.style.border.color, expected_border);
            assert_eq!(card_node.style.corner_radii.top_left, 9.0);
        }

        // close_all_popups clears both trigger tag and card tag
        assert!(tree.close_all_popups());
        assert!(!tree.has_open_popup());
        assert!(!tree.is_popup_tag_active(DROPDOWN_TRIGGER_TAG));
        assert!(!tree.is_popup_tag_active(crate::declarative::types::hash_label("AspectDropdown")));
    }

    #[test]
    fn test_scoped_cascading_popup_and_custom_card_style() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("root node");
        if let Some(r) = tree.get_mut(root) {
            r.computed_rect = Rect::new(0.0, 0.0, 1280.0, 720.0);
        }

        const ADD_BTN_TAG: u64 = 0xBB01;
        const OTHER_BTN_TAG: u64 = 0xBB02;
        let anchor = tree.create_node();
        if let Some(n) = tree.get_mut(anchor) {
            n.tag = ADD_BTN_TAG;
            n.computed_rect = Rect::new(40.0, 20.0, 28.0, 24.0);
        }
        let _ = tree.add_child(root, anchor);

        let custom_bg = Color::rgba(0.20, 0.15, 0.25, 0.95);
        let custom_border = Color::rgba(0.80, 0.40, 0.90, 1.0);
        let cfg = CascadingPopupConfig::new(190.0, PopupPlacement::Below).with_card_style(
            PopupCardStyle::new()
                .with_background(custom_bg)
                .with_border(2.0, custom_border)
                .with_border_radius(11.0),
        );

        // Toggle exclusive popup ON
        assert!(tree.toggle_exclusive_popup(ADD_BTN_TAG));
        {
            let mut scope = UiScope::new(&mut tree, root);
            let res = scope.scoped_cascading_popup_anchored_to_tag(
                "ScopedAddMenu",
                ADD_BTN_TAG,
                Rect::ZERO,
                cfg,
                |menu| {
                    menu.dropdown_item(101, "➕", "Empty", None, true);
                },
            );
            let card_id = res.expect("cascading popup must render").response.id;
            let node = scope.tree().get(card_id).expect("card node");
            assert_eq!(node.style.background_color, custom_bg);
            assert_eq!(node.style.border.color, custom_border);
            assert_eq!(node.style.border.width.top, 2.0);
            assert_eq!(node.style.corner_radii.top_left, 11.0);
        }

        // Switching to another exclusive popup clears ADD_BTN_TAG
        assert!(tree.toggle_exclusive_popup(OTHER_BTN_TAG));
        assert!(!tree.is_popup_tag_active(ADD_BTN_TAG));
        assert!(tree.is_popup_tag_active(OTHER_BTN_TAG));

        // Outside click closes everything cleanly
        let _ = tree.dispatch_event(&UiEvent::MouseDown {
            button: MouseButton::Left,
            point: Point::new(900.0, 600.0),
        });
        assert!(!tree.has_open_popup());
        assert!(!tree.is_popup_tag_active(OTHER_BTN_TAG));
    }

    #[test]
    fn test_add_popup_trigger_and_autoclose_dropdown_items() {
        use crate::declarative::widget::Button;

        let mut tree = UiTree::new();
        let root = tree.create_root().expect("root node");
        if let Some(r) = tree.get_mut(root) {
            r.computed_rect = Rect::new(0.0, 0.0, 1280.0, 720.0);
        }

        const TRIGGER_TAG: u64 = 0xCC01;
        const ITEM_TAG: u64 = 0xCC02;

        // 1. Clicking trigger opens exclusive popup tag
        tree.event_state.frame_clicked_tag = Some(TRIGGER_TAG);
        {
            let mut scope = UiScope::new(&mut tree, root);
            let resp = scope.add_popup_trigger(
                Button::new("Trigger").tag(TRIGGER_TAG).width(100.0),
                TRIGGER_TAG,
            );
            assert!(resp.clicked());
            assert!(scope.is_popup_tag_active(TRIGGER_TAG));
        }

        // 2. Clicking autoclose dropdown item emits clicked() == true and closes popup
        tree.event_state.clear_frame_clicks();
        tree.event_state.frame_clicked_tag = Some(ITEM_TAG);
        let mut action_triggered = false;
        {
            let mut scope = UiScope::new(&mut tree, root);
            scope.scoped_popup_anchored_to_tag(
                "TriggerPopup",
                TRIGGER_TAG,
                Rect::new(10.0, 10.0, 100.0, 24.0),
                160.0,
                PopupPlacement::Below,
                |menu| {
                    menu.dropdown_item_autoclose(ITEM_TAG, "", "Action", None, true)
                        .on_click(|| action_triggered = true);
                },
            );
        }
        assert!(action_triggered);
        assert!(!tree.is_popup_tag_active(TRIGGER_TAG));
        assert!(!tree.has_open_popup());
    }
}