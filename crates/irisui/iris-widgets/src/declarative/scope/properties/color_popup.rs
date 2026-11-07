// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Autonomous Floating 2D HSV Color Picker Popup (`iris-widgets::declarative::scope::properties::color_popup`)
//!
//! Provides a reusable, retained-state floating 2D HSV Color Picker popup card on [`iris_core::UiLayer::Popup`]
//! with automatic pre-drag color snapshotting (`color_edit_origin`), continuous HSV hue retention
//! (`cached_color_hsv`), live two-way color mutation, and fluent commit/close callbacks.

use iris_core::geometry::{Point, Rect};
use iris_core::{AlignItems, Color, JustifyContent, Style, TextAlign, WidgetRole};

use crate::color_picker::HsvColorPickerState;
use crate::declarative::scope::core::UiScope;
use crate::declarative::widget::color_picker::ColorPicker;

/// Resolves a continuous [`HsvColorPickerState`] from an RGB [`Color`], preserving
/// cached hue (and saturation when value is near zero) if the cached HSV state is
/// colorimetrically equivalent to `color` (`max_channel_diff < 1e-2`) or when `color`
/// is grayscale (`saturation < 1e-3`) or black (`value < 1e-3`).
#[inline]
#[must_use]
pub fn resolve_hsv_from_color(color: Color, cached_hsv: Option<[f32; 3]>) -> HsvColorPickerState {
    let (rgb_h, rgb_s, rgb_v) = HsvColorPickerState::rgb_to_hsv(color.r, color.g, color.b);
    if let Some([ch, cs, cv]) = cached_hsv {
        let reconstructed = HsvColorPickerState::hsv_to_rgb(ch, cs, cv);
        let diff = (reconstructed.r - color.r)
            .abs()
            .max((reconstructed.g - color.g).abs())
            .max((reconstructed.b - color.b).abs());
        if diff < 1e-2 {
            return HsvColorPickerState {
                hue: ch,
                saturation: cs,
                value: cv,
                alpha: color.a,
            };
        }
        let preserved_hue = if rgb_s < 1e-3 || rgb_v < 1e-3 {
            ch
        } else {
            rgb_h
        };
        let preserved_sat = if rgb_v < 1e-3 { cs } else { rgb_s };
        return HsvColorPickerState {
            hue: preserved_hue,
            saturation: preserved_sat,
            value: rgb_v,
            alpha: color.a,
        };
    }
    HsvColorPickerState {
        hue: rgb_h,
        saturation: rgb_s,
        value: rgb_v,
        alpha: color.a,
    }
}

/// Semantic 64-bit tag configuration for the interactive sub-elements of a floating Color Picker popup.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColorPickerPopupTags {
    /// Tag of the anchor swatch node to position the popup relative to.
    pub anchor_tag: u64,
    /// Tag assigned to the popup card content container.
    pub card_tag: u64,
    /// Tag assigned to the header `✖` close button.
    pub close_tag: u64,
    /// Tag assigned to the interactive 2D Saturation-Value box.
    pub sv_tag: u64,
    /// Tag assigned to the interactive vertical Rainbow Hue bar.
    pub hue_tag: u64,
}

impl ColorPickerPopupTags {
    /// Constructs a new [`ColorPickerPopupTags`] bundle.
    #[inline]
    #[must_use]
    pub const fn new(
        anchor_tag: u64,
        card_tag: u64,
        close_tag: u64,
        sv_tag: u64,
        hue_tag: u64,
    ) -> Self {
        Self {
            anchor_tag,
            card_tag,
            close_tag,
            sv_tag,
            hue_tag,
        }
    }
}

/// Configuration descriptor for rendering a floating 2D HSV Color Picker popup card.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ColorPickerPopupConfig<'a> {
    /// Header title text displayed at the top of the popup card.
    pub title: &'a str,
    /// Bounding rectangle of the parent panel used to clamp popup screen coordinates.
    pub bounds: Rect,
    /// Optional fallback HSV `[hue, saturation, value]` used before an active session is cached.
    pub initial_hsv: Option<[f32; 3]>,
    /// Optional hex string override displayed when idle.
    pub hex_override: Option<&'a str>,
    /// Semantic tags for anchor lookup and hit-testing.
    pub tags: ColorPickerPopupTags,
}

impl<'a> ColorPickerPopupConfig<'a> {
    /// Constructs a new [`ColorPickerPopupConfig`] with the given title, clamping bounds, and tag set.
    #[inline]
    #[must_use]
    pub const fn new(title: &'a str, bounds: Rect, tags: ColorPickerPopupTags) -> Self {
        Self {
            title,
            bounds,
            initial_hsv: None,
            hex_override: None,
            tags,
        }
    }

    /// Supplies an initial HSV `[hue, saturation, value]` fallback.
    #[inline]
    #[must_use]
    pub const fn with_initial_hsv(mut self, hsv: [f32; 3]) -> Self {
        self.initial_hsv = Some(hsv);
        self
    }

    /// Supplies a hexadecimal display string override when the picker is idle.
    #[inline]
    #[must_use]
    pub const fn with_hex_override(mut self, hex: &'a str) -> Self {
        self.hex_override = Some(hex);
        self
    }
}

/// Interaction outcome returned by [`UiScope::color_picker_popup`] and its variants.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ColorPickerPopupResponse {
    /// Computed screen-space bounding rectangle of the floating popup card.
    pub rect: Rect,
    /// Whether the color was modified during the current frame.
    pub changed: bool,
    /// Atomic `(start_color, final_color)` pair emitted when a drag or click-edit gesture finishes
    /// and the resulting color differs from the pre-edit starting color.
    pub committed: Option<(Color, Color)>,
    /// Whether the `✖` close button in the popup header was clicked this frame.
    pub close_clicked: bool,
    /// Active [`Color`] value after evaluating the current frame.
    pub color: Color,
    /// Active `[hue, saturation, value]` state after evaluating the current frame.
    pub hsv: [f32; 3],
}

impl ColorPickerPopupResponse {
    /// Invokes `f` if the popup header close button (`✖`) was clicked this frame.
    #[inline]
    pub fn on_close<F: FnOnce()>(self, f: F) -> Self {
        if self.close_clicked {
            f();
        }
        self
    }

    /// Invokes `f(new_color)` if the color value changed live during this frame.
    #[inline]
    pub fn on_change<F: FnOnce(Color)>(self, f: F) -> Self {
        if self.changed {
            f(self.color);
        }
        self
    }

    /// Invokes `f(old_color, new_color)` if an edit gesture was committed on this frame.
    #[inline]
    pub fn on_commit<F: FnOnce(Color, Color)>(self, f: F) -> Self {
        if let Some((old_col, new_col)) = self.committed {
            f(old_col, new_col);
        }
        self
    }

    /// Maps a committed `(old_color, new_color)` transition into an optional action value `T`.
    #[inline]
    #[must_use]
    pub fn map_commit<T, F: FnOnce(Color, Color) -> T>(self, f: F) -> Option<T> {
        self.committed.map(|(old_col, new_col)| f(old_col, new_col))
    }

    /// Converts the response into `(rect, Option<T>)` by mapping any committed `(old_color, new_color)` transition.
    #[inline]
    #[must_use]
    pub fn into_rect_and_commit<T, F: FnOnce(Color, Color) -> T>(self, f: F) -> (Rect, Option<T>) {
        (
            self.rect,
            self.committed.map(|(old_col, new_col)| f(old_col, new_col)),
        )
    }

    /// Converts the response into `(rect, Option<T>)` using a fallible mapping closure for committed transitions.
    #[inline]
    #[must_use]
    pub fn into_rect_and_opt_commit<T, F: FnOnce(Color, Color) -> Option<T>>(
        self,
        f: F,
    ) -> (Rect, Option<T>) {
        (
            self.rect,
            self.committed
                .and_then(|(old_col, new_col)| f(old_col, new_col)),
        )
    }
}

impl<'a> UiScope<'a> {
    /// Renders an autonomous floating 2D HSV Color Picker popup bound two-way to `color`.
    ///
    /// Automatically retains continuous HSV angles and pre-drag origin color inside
    /// [`iris_core::dispatch::types::EventState`], mutating `*color` live during drag gestures
    /// and emitting `committed = Some((old_color, new_color))` upon pointer release.
    pub fn color_picker_popup(
        &mut self,
        color: &mut Color,
        config: ColorPickerPopupConfig<'_>,
    ) -> ColorPickerPopupResponse {
        self.render_color_picker_popup_internal(color, config, false)
    }

    /// Renders an autonomous floating 2D HSV Color Picker popup bound two-way to split RGBA float channels.
    ///
    /// Mutates `(*rgba.0, *rgba.1, *rgba.2, *rgba.3)` in-place during live dragging and returns
    /// a [`ColorPickerPopupResponse`] with atomic commit tracking.
    pub fn color_picker_popup_rgba(
        &mut self,
        rgba: (&mut f32, &mut f32, &mut f32, &mut f32),
        config: ColorPickerPopupConfig<'_>,
    ) -> ColorPickerPopupResponse {
        let mut col = Color::rgba(*rgba.0, *rgba.1, *rgba.2, *rgba.3);
        let resp = self.render_color_picker_popup_internal(&mut col, config, false);
        if resp.changed {
            *rgba.0 = col.r;
            *rgba.1 = col.g;
            *rgba.2 = col.b;
            *rgba.3 = col.a;
        }
        resp
    }

    /// Renders an autonomous floating 2D HSV Color Picker popup when no mutable color component is attached yet.
    ///
    /// Initializes from `fallback` (or `config.initial_hsv` if provided) and tracks drag sessions in
    /// [`iris_core::dispatch::types::EventState`].
    pub fn color_picker_popup_detached(
        &mut self,
        fallback: Color,
        config: ColorPickerPopupConfig<'_>,
    ) -> ColorPickerPopupResponse {
        let mut col = fallback;
        self.render_color_picker_popup_internal(&mut col, config, true)
    }

    fn render_color_picker_popup_internal(
        &mut self,
        color: &mut Color,
        config: ColorPickerPopupConfig<'_>,
        prefer_initial_hsv: bool,
    ) -> ColorPickerPopupResponse {
        let anchor_rect = self
            .tree
            .node_rect_tagged(config.tags.anchor_tag, Point::ZERO)
            .or_else(|| self.tree.tag_rect(config.tags.anchor_tag))
            .unwrap_or(Rect::new(
                config.bounds.x + 20.0,
                config.bounds.y + 120.0,
                40.0,
                20.0,
            ));

        let popup_w = 206.0;
        let popup_h = 224.0;
        let popup_x = anchor_rect
            .x
            .min(config.bounds.right() - popup_w - 6.0)
            .max(config.bounds.x + 6.0);
        let popup_y = if anchor_rect.bottom() + popup_h > config.bounds.bottom() - 30.0 {
            (anchor_rect.y - popup_h - 4.0).max(30.0)
        } else {
            anchor_rect.bottom() + 4.0
        };

        let session_tag = config.tags.card_tag;
        let initial_color = *color;
        let cached_hsv = self
            .tree
            .event_state
            .cached_color_hsv_for(session_tag)
            .or(config.initial_hsv);

        let mut hsv_state = if prefer_initial_hsv
            && self
                .tree
                .event_state
                .cached_color_hsv_for(session_tag)
                .is_none()
            && let Some([h, s, v]) = config.initial_hsv
        {
            HsvColorPickerState {
                hue: h,
                saturation: s,
                value: v,
                alpha: color.a,
            }
        } else {
            resolve_hsv_from_color(initial_color, cached_hsv)
        };

        let mut close_clicked = false;
        let mut changed = false;
        let mut committed = None;

        self.dropdown_menu_card_named(
            "ColorPickerPopupCard",
            popup_x,
            popup_y,
            popup_w,
            |dropdown_card| {
                dropdown_card.container_tagged(
                    "ColorPickerContent",
                    Style::new().flex_col().gap(6.0),
                    WidgetRole::DropdownPopup,
                    config.tags.card_tag,
                    |card| {
                        let header_style = Style::new()
                            .flex_row()
                            .align_items(AlignItems::Center)
                            .justify_content(JustifyContent::SpaceBetween)
                            .height(20.0);

                        card.container_named("ColorPickerHeaderRow", header_style, |header| {
                            header.label_styled_passive(
                                "ColorPickerHeaderLabel",
                                config.title,
                                11.5,
                                Color::rgba(0.886, 0.894, 0.918, 1.0),
                                TextAlign::Left,
                                Style::new().height(18.0),
                            );

                            let close_size = 16.0;
                            let close_style = Style::new()
                                .width(close_size)
                                .height(close_size)
                                .background(Color::rgba(0.157, 0.165, 0.188, 0.98))
                                .hover_background(Color::rgba(0.35, 0.12, 0.12, 0.95))
                                .border_radius(3.0)
                                .flex_row()
                                .align_items(AlignItems::Center)
                                .justify_content(JustifyContent::Center);

                            let close_resp = header.container_tagged(
                                "ColorPickerCloseButton",
                                close_style,
                                WidgetRole::Button,
                                config.tags.close_tag,
                                |btn| {
                                    btn.label_styled_hover_passive(
                                        "CloseGlyph",
                                        "✖",
                                        9.0,
                                        (Color::rgba(0.70, 0.73, 0.80, 0.90), Color::WHITE),
                                        TextAlign::Center,
                                        Style::new().height(close_size),
                                    );
                                },
                            );
                            if close_resp.response.clicked() {
                                close_clicked = true;
                            }
                        });

                        let picker_resp = card.add(
                            ColorPicker::new(&mut hsv_state)
                                .sv_size(160.0, 130.0)
                                .hue_width(18.0)
                                .gap(8.0)
                                .tags(config.tags.sv_tag, config.tags.hue_tag),
                        );

                        if picker_resp.changed() {
                            let _ = card
                                .tree
                                .event_state
                                .begin_or_get_color_edit_origin(session_tag, initial_color);
                            *color = picker_resp.color();
                            changed = true;
                        }

                        card.tree.event_state.set_cached_color_hsv(
                            session_tag,
                            [hsv_state.hue, hsv_state.saturation, hsv_state.value],
                        );

                        if picker_resp.drag_released() {
                            let start_col = card
                                .tree
                                .event_state
                                .take_color_edit_origin(session_tag)
                                .unwrap_or(initial_color);
                            if start_col != *color {
                                committed = Some((start_col, *color));
                            }
                        }

                        let preview_style = Style::new()
                            .flex_row()
                            .align_items(AlignItems::Center)
                            .gap(8.0)
                            .height(24.0);

                        card.container_named("ColorPickerPreviewRow", preview_style, |bottom| {
                            let current_col = if changed { *color } else { picker_resp.color() };
                            let swatch_style = Style::new()
                                .width(42.0)
                                .height(20.0)
                                .background(current_col)
                                .border(1.0, Color::rgba(0.40, 0.44, 0.55, 0.90))
                                .border_radius(4.0);

                            bottom.empty_box(swatch_style);

                            let hex_text =
                                if changed || config.hex_override.is_none_or(str::is_empty) {
                                    format!(
                                        "#{:02x}{:02x}{:02x}",
                                        (current_col.r.clamp(0.0, 1.0) * 255.0) as u8,
                                        (current_col.g.clamp(0.0, 1.0) * 255.0) as u8,
                                        (current_col.b.clamp(0.0, 1.0) * 255.0) as u8,
                                    )
                                } else {
                                    config.hex_override.unwrap_or("#ffffff").to_string()
                                };

                            bottom.label_styled_passive(
                                "LiveHexText",
                                &hex_text,
                                11.0,
                                Color::rgba(0.85, 0.88, 0.95, 1.0),
                                TextAlign::Left,
                                Style::new().flex_grow(1.0).height(20.0),
                            );
                        });
                    },
                );
            },
        );

        ColorPickerPopupResponse {
            rect: Rect::new(popup_x, popup_y, popup_w, popup_h),
            changed,
            committed,
            close_clicked,
            color: *color,
            hsv: [hsv_state.hue, hsv_state.saturation, hsv_state.value],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use iris_core::{MouseButton, UiEvent, UiTree};

    #[test]
    fn test_color_picker_popup_rgba_live_drag_and_atomic_commit() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("root");

        let tags = ColorPickerPopupTags::new(0x100, 0x101, 0x102, 0x103, 0x104);
        let bounds = Rect::new(0.0, 0.0, 320.0, 600.0);
        let (mut r, mut g, mut b, mut a) = (0.1_f32, 0.2_f32, 0.3_f32, 1.0_f32);
        let initial_color = Color::rgba(r, g, b, a);

        // 0. Initial render to register SV box and Hue bar rects
        {
            let mut scope = UiScope::new(&mut tree, root);
            let resp = scope.color_picker_popup_rgba(
                (&mut r, &mut g, &mut b, &mut a),
                ColorPickerPopupConfig::new("🎨 Color Picker", bounds, tags),
            );
            assert!(!resp.changed);
            assert!(resp.committed.is_none());
        }

        let sv_rect = tree
            .iter()
            .find(|(_, n)| n.tag == tags.sv_tag)
            .map(|(_, n)| n.computed_rect)
            .expect("SV box must exist");

        // 1. MouseDown inside SV box
        let start_pt = Point::new(sv_rect.x + 20.0, sv_rect.y + 20.0);
        let _ = tree.dispatch_event(&UiEvent::MouseDown {
            button: MouseButton::Left,
            point: start_pt,
        });
        {
            let _ = tree.clear_children(root);
            let mut scope = UiScope::new(&mut tree, root);
            let resp = scope.color_picker_popup_rgba(
                (&mut r, &mut g, &mut b, &mut a),
                ColorPickerPopupConfig::new("🎨 Color Picker", bounds, tags),
            );
            assert!(resp.changed);
            assert!(
                resp.committed.is_none(),
                "MouseDown must not emit commit prematurely"
            );
        }

        // 2. Simulate continuous drag frames
        for step in 1..=10 {
            let drag_pt = Point::new(
                sv_rect.x + 20.0 + (step as f32) * 8.0,
                sv_rect.y + 20.0 + (step as f32) * 5.0,
            );
            let _ = tree.dispatch_event(&UiEvent::MouseMove { point: drag_pt });
            let _ = tree.clear_children(root);
            let mut scope = UiScope::new(&mut tree, root);
            let resp = scope.color_picker_popup_rgba(
                (&mut r, &mut g, &mut b, &mut a),
                ColorPickerPopupConfig::new("🎨 Color Picker", bounds, tags),
            );
            assert!(resp.changed);
            assert!(
                resp.committed.is_none(),
                "Intermediate drag frames must not emit commit"
            );
        }

        let dragged_color = Color::rgba(r, g, b, a);
        assert_ne!(dragged_color, initial_color);

        // 3. MouseUp to finish drag -> must emit single atomic (initial_color, dragged_color) commit
        let end_pt = Point::new(sv_rect.x + 100.0, sv_rect.y + 70.0);
        let _ = tree.dispatch_event(&UiEvent::MouseUp {
            button: MouseButton::Left,
            point: end_pt,
        });
        {
            let _ = tree.clear_children(root);
            let mut scope = UiScope::new(&mut tree, root);
            let resp = scope.color_picker_popup_rgba(
                (&mut r, &mut g, &mut b, &mut a),
                ColorPickerPopupConfig::new("🎨 Color Picker", bounds, tags),
            );
            assert_eq!(resp.committed, Some((initial_color, dragged_color)));
        }
    }

    #[test]
    fn test_resolve_hsv_preserves_hue_at_black_and_white() {
        let black = Color::rgba(0.0, 0.0, 0.0, 1.0);
        let resolved_black = resolve_hsv_from_color(black, Some([240.0, 0.75, 0.0]));
        assert!((resolved_black.hue - 240.0).abs() < 1e-4);
        assert!((resolved_black.saturation - 0.75).abs() < 1e-4);

        let white = Color::rgba(1.0, 1.0, 1.0, 1.0);
        let resolved_white = resolve_hsv_from_color(white, Some([135.0, 0.0, 1.0]));
        assert!((resolved_white.hue - 135.0).abs() < 1e-4);
    }
}