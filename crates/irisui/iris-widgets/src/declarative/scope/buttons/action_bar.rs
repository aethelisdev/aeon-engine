// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Panel Action Bar & Dual Footer Button Primitives
//!
//! Provides the [`ActionBarButton`] descriptor, [`DualActionBarResponse`] composite interaction
//! response, and [`UiScope::dual_action_bar`] / [`UiScope::action_bar_button`] methods for
//! constructing equal-width bottom action bars in dockable tool panels without imperative
//! layout arithmetic or intermediate variable bindings.

use crate::declarative::scope::core::UiScope;
use crate::declarative::scope::display::WrappedLabelDescriptor;
use crate::declarative::types::WidgetResponse;
use iris_core::{
    AlignItems, Color, Insets, JustifyContent, Style, TextAlign, TextWrap, WidgetRole,
};

/// Composite interaction response returned by [`UiScope::dual_action_bar`].
///
/// Pairs the interaction state of the primary (left) and secondary (right) action buttons
/// and exposes fluent callback combinators for zero-`let` declarative event handling.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct DualActionBarResponse {
    /// Interaction response for the primary (left) action bar button.
    pub primary: WidgetResponse,
    /// Interaction response for the secondary (right) action bar button.
    pub secondary: WidgetResponse,
}

impl DualActionBarResponse {
    /// Creates a new composite response pairing primary and secondary button interactions.
    ///
    /// # Arguments
    /// * `primary` - Interaction response of the left button.
    /// * `secondary` - Interaction response of the right button.
    #[inline]
    #[must_use]
    pub const fn new(primary: WidgetResponse, secondary: WidgetResponse) -> Self {
        Self { primary, secondary }
    }

    /// Invokes `f` if the primary (left) action button was clicked in the current frame.
    ///
    /// Enables fluent method chaining without intermediate `let` bindings.
    ///
    /// # Arguments
    /// * `f` - Callback executed when `self.primary.clicked()` is `true`.
    #[inline]
    pub fn on_primary_click<F: FnOnce()>(self, f: F) -> Self {
        if self.primary.clicked() {
            f();
        }
        self
    }

    /// Invokes `f` if the secondary (right) action button was clicked in the current frame.
    ///
    /// Enables fluent method chaining without intermediate `let` bindings.
    ///
    /// # Arguments
    /// * `f` - Callback executed when `self.secondary.clicked()` is `true`.
    #[inline]
    pub fn on_secondary_click<F: FnOnce()>(self, f: F) -> Self {
        if self.secondary.clicked() {
            f();
        }
        self
    }

    /// Evaluates `f` and returns its `Option<T>` result if the secondary (right) button was clicked,
    /// or returns `None` otherwise.
    ///
    /// Allows a panel footer builder to return an optional domain action directly as the tail expression
    /// of a fluent chain without declaring any intermediate mutable variables.
    ///
    /// # Arguments
    /// * `f` - Producer closure invoked when `self.secondary.clicked()` is `true`.
    #[inline]
    #[must_use]
    pub fn then_secondary<T, F: FnOnce() -> Option<T>>(self, f: F) -> Option<T> {
        if self.secondary.clicked() { f() } else { None }
    }
}

/// Configuration descriptor for an individual button within a panel action bar.
///
/// Encapsulates node naming, caption text, optional leading atlas icon, active/open toggle
/// state, internal padding, and optional explicit semantic tag override.
#[derive(Debug, Clone, Copy)]
pub struct ActionBarButton<'a> {
    /// Debug node name assigned to the button container node.
    pub name: &'static str,
    /// Caption text rendered inside the button.
    pub label: &'a str,
    /// Debug node name assigned to the inner text label node.
    pub text_name: &'static str,
    /// Optional `(icon_node_name, icon_uv)` tuple for a leading 11px vector icon.
    pub icon: Option<(&'static str, [f32; 4])>,
    /// Optional explicit 64-bit semantic tag. When `None`, a deterministic tag is derived via [`UiScope::tag_for`].
    pub tag: Option<u64>,
    /// Whether the button is currently in an active/open toggle state (e.g. an open menu trigger).
    pub is_active: bool,
    /// Horizontal text alignment for the button caption.
    pub text_align: TextAlign,
    /// Internal horizontal/vertical padding insets applied to the button container.
    pub padding: Insets,
}

impl<'a> ActionBarButton<'a> {
    /// Constructs a new action bar button descriptor with default centered alignment and automatic tag derivation.
    ///
    /// # Arguments
    /// * `name` - Debug node name for the button container (also used for deterministic tag derivation when `tag` is `None`).
    /// * `label` - Visible caption text.
    #[inline]
    #[must_use]
    pub const fn new(name: &'static str, label: &'a str) -> Self {
        Self {
            name,
            label,
            text_name: name,
            icon: None,
            tag: None,
            is_active: false,
            text_align: TextAlign::Center,
            padding: Insets::ZERO,
        }
    }

    /// Attaches a leading 11×11 pixel icon from the UI icon atlas with a dedicated debug node name.
    ///
    /// # Arguments
    /// * `icon_name` - Debug node name assigned to the icon child node.
    /// * `icon_uv` - Normalized `[u_min, v_min, u_max, v_max]` atlas UV coordinates.
    #[inline]
    #[must_use]
    pub const fn with_icon(mut self, icon_name: &'static str, icon_uv: [f32; 4]) -> Self {
        self.icon = Some((icon_name, icon_uv));
        self
    }

    /// Sets the debug node name assigned to the inner text label child node.
    ///
    /// # Arguments
    /// * `text_name` - Static debug identifier for the label node.
    #[inline]
    #[must_use]
    pub const fn with_text_name(mut self, text_name: &'static str) -> Self {
        self.text_name = text_name;
        self
    }

    /// Overrides the deterministic tag with an explicit 64-bit semantic tag.
    ///
    /// # Arguments
    /// * `tag` - Explicit semantic tag assigned to the button node.
    #[inline]
    #[must_use]
    pub const fn with_tag(mut self, tag: u64) -> Self {
        self.tag = Some(tag);
        self
    }

    /// Configures whether the button renders in its active/open toggle state.
    ///
    /// # Arguments
    /// * `is_active` - `true` when the associated popup or mode is active.
    #[inline]
    #[must_use]
    pub const fn with_active(mut self, is_active: bool) -> Self {
        self.is_active = is_active;
        self
    }

    /// Sets the horizontal text alignment of the button caption.
    ///
    /// # Arguments
    /// * `text_align` - Alignment mode ([`TextAlign::Left`], [`TextAlign::Center`], [`TextAlign::Right`]).
    #[inline]
    #[must_use]
    pub const fn with_text_align(mut self, text_align: TextAlign) -> Self {
        self.text_align = text_align;
        self
    }

    /// Sets the internal padding insets of the button container.
    ///
    /// # Arguments
    /// * `padding` - Padding insets applied inside the button border.
    #[inline]
    #[must_use]
    pub const fn with_padding(mut self, padding: Insets) -> Self {
        self.padding = padding;
        self
    }
}

impl<'a> UiScope<'a> {
    /// Emits a single styled panel action bar button with optional leading icon, hover/pressed
    /// states, and equal-flex sizing constraints.
    ///
    /// # Arguments
    /// * `width` - Explicit base width in logical pixels for dual-engine flex invariance.
    /// * `height` - Explicit button height in logical pixels.
    /// * `desc` - Button configuration descriptor.
    pub fn action_bar_button(
        &mut self,
        width: f32,
        height: f32,
        desc: ActionBarButton<'_>,
    ) -> WidgetResponse {
        let resolved_tag = desc.tag.unwrap_or_else(|| self.tag_for(desc.name));
        let is_pressed = self.tree().event_state.pressed_tag == Some(resolved_tag)
            || self
                .tree()
                .event_state
                .pressed
                .and_then(|pid| self.tree().get(pid))
                .is_some_and(|n| n.tag == resolved_tag);

        let (bg, border, text_col) = if desc.is_active {
            (
                Color::rgba(0.118, 0.125, 0.145, 1.0),
                Color::rgba(0.353, 0.376, 0.439, 0.95),
                Color::WHITE,
            )
        } else {
            (
                Color::rgba(0.157, 0.165, 0.188, 0.98),
                Color::rgba(0.212, 0.220, 0.259, 0.85),
                Color::rgba(0.886, 0.894, 0.918, 1.0),
            )
        };

        let mut btn_style = Style::new()
            .flex_row()
            .align_items(AlignItems::Center)
            .justify_content(JustifyContent::Center)
            .gap(5.0)
            .width(width)
            .flex_grow(1.0)
            .height(height)
            .padding_insets(desc.padding)
            .background(bg)
            .border(1.0, border)
            .border_radius(5.0);

        if is_pressed {
            btn_style = btn_style
                .background(Color::rgba(0.090, 0.098, 0.118, 1.0))
                .border(1.0, Color::rgba(0.0, 0.85, 1.0, 0.85));
        } else if !desc.is_active {
            btn_style = btn_style
                .hover_background(Color::rgba(0.200, 0.208, 0.235, 1.0))
                .hover_border(1.0, Color::rgba(0.271, 0.282, 0.329, 0.95));
        }

        self.container_tagged_resp(
            desc.name,
            btn_style,
            WidgetRole::Button,
            resolved_tag,
            |btn| {
                if let Some((icon_name, icon_uv)) = desc.icon {
                    if desc.is_active {
                        btn.icon_named(icon_name, icon_uv, text_col, 11.0);
                    } else {
                        btn.icon_hover_named(icon_name, icon_uv, text_col, Color::WHITE, 11.0);
                    }
                }

                let mut label_desc =
                    WrappedLabelDescriptor::new(11.0, text_col, Style::new().height(height))
                        .align(desc.text_align)
                        .wrap(TextWrap::None);
                if !desc.is_active {
                    label_desc = label_desc.hover_color(Color::WHITE);
                }

                btn.label_styled_passive_wrapped(desc.text_name, desc.label, label_desc);
            },
        )
    }

    /// Emits a horizontal two-button bottom action bar row dividing available panel width equally
    /// between `primary` and `secondary` buttons without horizontal flex drift.
    ///
    /// # Arguments
    /// * `container_name` - Debug node name assigned to the outer footer row container.
    /// * `panel_width` - Total width of the parent panel in logical pixels.
    /// * `primary` - Descriptor for the left action button.
    /// * `secondary` - Descriptor for the right action button.
    pub fn dual_action_bar(
        &mut self,
        container_name: &'static str,
        panel_width: f32,
        primary: ActionBarButton<'_>,
        secondary: ActionBarButton<'_>,
    ) -> DualActionBarResponse {
        let padding_x = 8.0;
        let footer_h = 24.0;
        let btn_gap = 8.0;
        let available_w = (panel_width - padding_x * 2.0 - btn_gap).max(0.0);
        let btn_w = (available_w * 0.5).floor();

        let row_style = Style::new()
            .flex_row()
            .align_items(AlignItems::Center)
            .gap(btn_gap)
            .height(34.0)
            .flex_shrink(0.0)
            .margin_insets(Insets::new(4.0, padding_x, 6.0, padding_x));

        self.container_named(container_name, row_style, |row| {
            let primary_resp = row.action_bar_button(btn_w, footer_h, primary);
            let secondary_resp = row.action_bar_button(btn_w, footer_h, secondary);
            DualActionBarResponse::new(primary_resp, secondary_resp)
        })
        .inner
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use iris_core::{MouseButton, Point, Rect, UiEvent, UiTree};

    #[test]
    fn test_dual_action_bar_layout_and_fluent_callbacks() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root node must exist");

        let mut primary_clicked = false;
        let mut secondary_clicked = false;

        {
            let mut scope = UiScope::new(&mut tree, root);
            scope.configure_container("TestRoot", Style::new().flex_col(), false);
            let mapped = scope
                .dual_action_bar(
                    "TestFooter",
                    320.0,
                    ActionBarButton::new("PrimaryBtn", "Add Item")
                        .with_icon("PrimaryIcon", [0.0, 0.0, 0.25, 0.25])
                        .with_text_name("PrimaryBtnText")
                        .with_text_align(TextAlign::Left)
                        .with_padding(Insets::new(0.0, 6.0, 0.0, 6.0)),
                    ActionBarButton::new("SecondaryBtn", "Save Item")
                        .with_text_name("SecondaryBtnText")
                        .with_text_align(TextAlign::Center),
                )
                .on_primary_click(|| {
                    primary_clicked = true;
                })
                .on_secondary_click(|| {
                    secondary_clicked = true;
                })
                .then_secondary(|| Some(42));
            assert_eq!(mapped, None);
            scope.finish_layout(Rect::new(0.0, 0.0, 320.0, 40.0));
        }
        assert!(!primary_clicked);
        assert!(!secondary_clicked);

        let primary_rect = tree.assert_node_rect("PrimaryBtn");
        let secondary_rect = tree.assert_node_rect("SecondaryBtn");
        assert_eq!(primary_rect.width, 148.0);
        assert_eq!(primary_rect.height, 24.0);
        assert_eq!(secondary_rect.width, 148.0);
        assert_eq!(secondary_rect.height, 24.0);

        // Simulate clicking SecondaryBtn
        let click_pt = Point::new(secondary_rect.x + 10.0, secondary_rect.y + 10.0);
        let _ = tree.dispatch_event(&UiEvent::MouseDown {
            button: MouseButton::Left,
            point: click_pt,
        });
        let _ = tree.dispatch_event(&UiEvent::MouseUp {
            button: MouseButton::Left,
            point: click_pt,
        });

        let emitted = {
            let mut scope = UiScope::new(&mut tree, root);
            scope
                .dual_action_bar(
                    "TestFooter",
                    320.0,
                    ActionBarButton::new("PrimaryBtn", "Add Item"),
                    ActionBarButton::new("SecondaryBtn", "Save Item"),
                )
                .on_primary_click(|| {
                    primary_clicked = true;
                })
                .on_secondary_click(|| {
                    secondary_clicked = true;
                })
                .then_secondary(|| Some("saved"))
        };
        assert!(!primary_clicked);
        assert!(secondary_clicked);
        assert_eq!(emitted, Some("saved"));
    }
}