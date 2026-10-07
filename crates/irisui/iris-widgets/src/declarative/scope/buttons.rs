// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Buttons & Interactive Clickable Controls
//!
//! Provides interactive push buttons, icon buttons, styled variants, and toggle pill
//! widgets returning immediate [`WidgetResponse`] interaction states.
//!

use super::core::UiScope;
use crate::declarative::types::WidgetResponse;
use iris_core::{
    AlignItems, Color, CornerRadii, Insets, JustifyContent, Style, TextAlign, WidgetCursor,
    WidgetRole,
};

impl<'a> UiScope<'a> {
    /// Emits an interactive push button returning immediate [`WidgetResponse`].
    ///
    /// Generates a deterministic persistent tag from `label` to support cross-frame
    /// interaction tracking, hover highlighting, and hit testing without manual identifier management.
    ///
    /// # Arguments
    /// * `label` - Button caption text.
    pub fn button(&mut self, label: impl Into<String>) -> WidgetResponse {
        let label_str = label.into();
        let (visible, _) = crate::declarative::types::split_label_id(&label_str);
        let tag = self.tag_for(&label_str);
        self.button_tagged(visible, tag)
    }

    /// Emits an interactive push button with a custom semantic tag.
    ///
    /// Automatically applies hover highlighting, pointer cursor, and layout styling internally.
    ///
    /// # Arguments
    /// * `label` - Button caption text.
    /// * `tag` - Unique semantic identifier inspected during interaction event dispatching.
    pub fn button_tagged(&mut self, label: impl Into<String>, tag: u64) -> WidgetResponse {
        self.button_named_tagged("Button", label, tag)
    }

    /// Emits an interactive push button with a custom [`Style`] and semantic tag.
    ///
    /// Automatically applies subtle hover brightening to the background and border
    /// while fully respecting custom sizing, padding, and layout properties from `style`.
    ///
    /// # Arguments
    /// * `label` - Button caption text.
    /// * `style` - Custom layout and visual appearance style.
    /// * `tag` - Unique semantic identifier inspected during interaction event dispatching.
    pub fn button_styled_tagged(
        &mut self,
        label: impl Into<String>,
        style: Style,
        tag: u64,
    ) -> WidgetResponse {
        self.button_named_styled_tagged("Button", label, style, tag)
    }

    /// Emits an interactive push button with a descriptive node name, custom [`Style`], and semantic tag.
    ///
    /// # Arguments
    /// * `name` - Descriptive identifier assigned to the UI node for debugging and tree inspection.
    /// * `label` - Button caption text.
    /// * `style` - Custom layout and visual appearance style.
    /// * `tag` - Unique semantic identifier inspected during interaction event dispatching.
    pub fn button_named_styled_tagged(
        &mut self,
        name: &'static str,
        label: impl Into<String>,
        style: Style,
        tag: u64,
    ) -> WidgetResponse {
        let label_str = label.into();
        let node_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(node_id) {
            node.set_name(name);
            node.tag = tag;
        }
        let _ = self.tree.add_child(self.parent, node_id);
        let (clicked, hovered, _) = self.check_interaction(node_id);
        if let Some(node) = self.tree.get_mut(node_id) {
            node.interactive = true;
            node.role = WidgetRole::Button;
            node.cursor = Some(WidgetCursor::Pointer);
            node.set_text(label_str);
            node.font_size = 11.0;
            node.line_height = 14.0;
            node.text_align = TextAlign::Center;
            node.text_color = if hovered {
                Color::WHITE
            } else {
                Color::rgba(0.0, 0.88, 1.0, 1.0)
            };
            node.hover_text_color = Some(Color::WHITE);
            let mut resolved_style = style;
            if resolved_style.hover_background.is_none() {
                let bg = resolved_style.background_color;
                if bg.a > 0.01 {
                    resolved_style.hover_background = Some(Color::rgba(
                        (bg.r * 1.30).min(1.0),
                        (bg.g * 1.30).min(1.0),
                        (bg.b * 1.30).min(1.0),
                        bg.a,
                    ));
                }
            }
            if resolved_style.hover_border.is_none() {
                let bc = resolved_style.border.color;
                if bc.a > 0.01 {
                    let mut b = resolved_style.border;
                    b.color = Color::rgba(
                        (bc.r * 1.35).min(1.0),
                        (bc.g * 1.35).min(1.0),
                        (bc.b * 1.35).min(1.0),
                        bc.a,
                    );
                    resolved_style.hover_border = Some(b);
                }
            }
            if hovered {
                if let Some(hbg) = resolved_style.hover_background {
                    resolved_style.background_color = hbg;
                }
                if let Some(hb) = resolved_style.hover_border {
                    resolved_style.border = hb;
                }
            }
            node.set_style(resolved_style);
        }
        WidgetResponse::new(node_id, clicked, hovered, false)
    }

    /// Emits an interactive push button with a descriptive node name and a custom semantic tag.
    ///
    /// Automatically applies hover highlighting, pointer cursor, and layout styling internally.
    /// Uses content-based snug width by default.
    ///
    /// # Arguments
    /// * `name` - Descriptive identifier assigned to the UI node for debugging and tree inspection.
    /// * `label` - Button caption text.
    /// * `tag` - Unique semantic identifier inspected during interaction event dispatching.
    pub fn button_named_tagged(
        &mut self,
        name: &'static str,
        label: impl Into<String>,
        tag: u64,
    ) -> WidgetResponse {
        let default_style = Style::new()
            .height(26.0)
            .padding_insets(Insets::new(5.0, 14.0, 5.0, 14.0))
            .background(Color::rgba(0.0, 0.30, 0.42, 0.60))
            .border(1.0, Color::rgba(0.0, 0.75, 0.95, 0.60))
            .border_radius(4.0)
            .align_items(AlignItems::Center)
            .justify_content(JustifyContent::Center);
        self.button_named_styled_tagged(name, label, default_style, tag)
    }

    /// Emits an interactive push button with an explicit width constraint and semantic tag.
    ///
    /// # Arguments
    /// * `label` - Button caption text.
    /// * `width` - Explicit button width in logical points.
    /// * `tag` - Semantic tag for tracking interactions.
    pub fn button_with_width_tagged(
        &mut self,
        label: impl Into<String>,
        width: f32,
        tag: u64,
    ) -> WidgetResponse {
        let mut style = Style::new()
            .height(26.0)
            .padding_insets(Insets::new(5.0, 14.0, 5.0, 14.0))
            .background(Color::rgba(0.0, 0.30, 0.42, 0.60))
            .border(1.0, Color::rgba(0.0, 0.75, 0.95, 0.60))
            .border_radius(4.0)
            .align_items(AlignItems::Center)
            .justify_content(JustifyContent::Center);
        style.width = Some(width);
        self.button_named_styled_tagged("Button", label, style, tag)
    }

    /// Emits an interactive push button with explicit width, custom icon tint, and semantic tag.
    ///
    /// # Arguments
    /// * `icon_uv` - Atlas texture UV coordinates for the leading icon.
    /// * `icon_tint` - Idle tint color for the icon.
    /// * `label` - Button caption text.
    /// * `width` - Optional fixed width constraint in logical points.
    /// * `tag` - Semantic tag for tracking interactions.
    pub fn button_with_icon_styled_tagged(
        &mut self,
        icon_uv: [f32; 4],
        icon_tint: Color,
        label: impl Into<String>,
        width: Option<f32>,
        tag: u64,
    ) -> WidgetResponse {
        let label_str = label.into();
        let btn_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(btn_id) {
            node.tag = tag;
        }
        let _ = self.tree.add_child(self.parent, btn_id);
        let (clicked, hovered, _) = self.check_interaction(btn_id);

        if let Some(node) = self.tree.get_mut(btn_id) {
            node.interactive = true;
            node.role = WidgetRole::Button;
            node.cursor = Some(WidgetCursor::Pointer);
            let mut style = Style::new()
                .flex_row()
                .align_items(AlignItems::Center)
                .justify_content(JustifyContent::Center)
                .gap(6.0)
                .padding_insets(Insets::new(3.0, 8.0, 3.0, 8.0))
                .background(if hovered {
                    Color::rgba(0.0, 0.45, 0.60, 0.85)
                } else {
                    Color::rgba(0.12, 0.14, 0.18, 0.90)
                })
                .hover_background(Color::rgba(0.0, 0.45, 0.60, 0.85))
                .border(
                    1.0,
                    if hovered {
                        Color::rgba(0.0, 0.90, 1.0, 0.95)
                    } else {
                        Color::rgba(0.20, 0.24, 0.30, 0.85)
                    },
                )
                .hover_border(1.0, Color::rgba(0.0, 0.90, 1.0, 0.95))
                .border_radius(4.0)
                .height(24.0);
            let btn_w = width.unwrap_or_else(|| {
                let text_w = (label_str.chars().count() as f32 * 6.5).ceil().max(20.0);
                36.0 + text_w
            });
            style.width = Some(btn_w);
            node.set_style(style);
        }

        let icon_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(icon_id) {
            node.interactive = false;
            node.set_texture_uv(icon_uv);
            node.set_texture_tint(if hovered { Color::WHITE } else { icon_tint });
            node.hover_texture_tint = Some(Color::WHITE);
            node.set_style(Style::new().width(14.0).height(14.0));
        }
        let _ = self.tree.add_child(btn_id, icon_id);

        let text_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(text_id) {
            node.interactive = false;
            node.set_text(label_str);
            node.font_size = 10.0;
            node.line_height = 14.0;
            node.text_align = TextAlign::Center;
            node.text_color = if hovered {
                Color::WHITE
            } else {
                Color::rgba(0.90, 0.92, 0.96, 1.0)
            };
            node.hover_text_color = Some(Color::WHITE);
        }
        let _ = self.tree.add_child(btn_id, text_id);

        WidgetResponse::new(btn_id, clicked, hovered, false)
    }

    /// Emits an interactive push button with a leading icon quad and text label.
    ///
    /// # Arguments
    /// * `icon_uv` - Atlas texture UV coordinates for the leading icon.
    /// * `label` - Button caption text.
    /// * `tag` - Semantic tag for tracking interactions.
    pub fn button_with_icon_tagged(
        &mut self,
        icon_uv: [f32; 4],
        label: impl Into<String>,
        tag: u64,
    ) -> WidgetResponse {
        self.button_with_icon_styled_tagged(
            icon_uv,
            Color::rgba(0.0, 0.85, 1.0, 0.95),
            label,
            None,
            tag,
        )
    }

    /// Emits an interactive push button with a leading icon quad and text label using a generated tag.
    ///
    /// # Arguments
    /// * `icon_uv` - Atlas texture UV coordinates for the leading icon.
    /// * `label` - Button caption text.
    pub fn button_with_icon(
        &mut self,
        icon_uv: [f32; 4],
        label: impl Into<String>,
    ) -> WidgetResponse {
        let label_str = label.into();
        let (visible, _) = crate::declarative::types::split_label_id(&label_str);
        let tag = self.tag_for(&label_str);
        self.button_with_icon_tagged(icon_uv, visible, tag)
    }

    /// Emits an interactive pill toggle button for mode or option selection.
    ///
    /// # Arguments
    /// * `label` - Button caption text.
    /// * `is_active` - Whether this option is currently selected.
    /// * `tag` - Semantic tag for tracking interactions.
    pub fn toggle_pill_tagged(&mut self, label: &str, is_active: bool, tag: u64) -> WidgetResponse {
        self.emit_toggle_pill(label, is_active, tag, false)
    }

    /// Emits an interactive toggle pill button that expands to fill available flex space (`flex_grow: 1.0`).
    ///
    /// Ideal for segmented mode selector button groups where items evenly divide the row width.
    ///
    /// # Arguments
    /// * `label` - Button caption text.
    /// * `is_active` - Whether this option is currently selected.
    /// * `tag` - Semantic tag for tracking interactions.
    pub fn toggle_pill_flex_tagged(
        &mut self,
        label: &str,
        is_active: bool,
        tag: u64,
    ) -> WidgetResponse {
        self.emit_toggle_pill(label, is_active, tag, true)
    }

    fn emit_toggle_pill(
        &mut self,
        label: &str,
        is_active: bool,
        tag: u64,
        is_flex: bool,
    ) -> WidgetResponse {
        let node_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(node_id) {
            node.tag = tag;
        }
        let _ = self.tree.add_child(self.parent, node_id);
        let (clicked, hovered, _) = self.check_interaction(node_id);
        if let Some(node) = self.tree.get_mut(node_id) {
            node.interactive = true;
            node.role = WidgetRole::Button;
            node.cursor = Some(WidgetCursor::Pointer);
            node.set_text(label);
            node.font_size = 11.0;
            node.line_height = 24.0;
            node.text_align = TextAlign::Center;
            let (bg, border, text_color) = if is_active {
                (
                    Color::rgba(0.06, 0.22, 0.32, 0.75),
                    Color::rgba(0.14, 0.65, 0.95, 0.65),
                    Color::rgba(0.25, 0.85, 1.0, 1.0),
                )
            } else if hovered {
                (
                    Color::rgba(0.18, 0.21, 0.28, 0.85),
                    Color::rgba(0.35, 0.40, 0.50, 0.65),
                    Color::WHITE,
                )
            } else {
                (
                    Color::rgba(0.11, 0.13, 0.17, 0.70),
                    Color::rgba(0.22, 0.25, 0.32, 0.50),
                    Color::rgba(0.65, 0.70, 0.78, 1.0),
                )
            };
            node.text_color = text_color;
            let mut style = Style::new()
                .height(24.0)
                .background(bg)
                .border(1.0, border)
                .border_radius(4.0)
                .align_items(AlignItems::Center)
                .justify_content(JustifyContent::Center);
            if !is_active {
                style = style
                    .hover_background(Color::rgba(0.18, 0.21, 0.28, 0.85))
                    .hover_border(1.0, Color::rgba(0.35, 0.40, 0.50, 0.65));
                node.hover_text_color = Some(Color::WHITE);
            }
            if is_flex {
                style = style
                    .flex_grow(1.0)
                    .padding_insets(Insets::new(2.0, 4.0, 2.0, 4.0));
            } else {
                style = style.padding_insets(Insets::new(2.0, 10.0, 2.0, 10.0));
            }
            node.set_style(style);
        }
        WidgetResponse::new(node_id, clicked, hovered, false)
    }

    /// Emits a severity-colored filter pill tab for toolbar strips.
    ///
    /// Unlike solid toggle pills, uses an elevated dark slate background (`rgba(0.12, 0.16, 0.24, 0.95)`)
    /// and a severity-specific accent outline border (`accent_color`) when active, preventing visual overload.
    ///
    /// # Arguments
    /// * `label` - Filter caption text (e.g. `"Errors (1)"`).
    /// * `is_active` - Whether this filter level is currently selected.
    /// * `accent_color` - Severity accent color applied to the outline border when active.
    /// * `width` - Explicit button width in physical pixels.
    /// * `tag` - Semantic identifier inspected during interaction event dispatching.
    pub fn filter_pill_tagged(
        &mut self,
        label: &str,
        is_active: bool,
        accent_color: Color,
        width: f32,
        tag: u64,
    ) -> WidgetResponse {
        let node_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(node_id) {
            node.tag = tag;
        }
        let _ = self.tree.add_child(self.parent, node_id);
        let (clicked, hovered, _) = self.check_interaction(node_id);
        if let Some(node) = self.tree.get_mut(node_id) {
            node.interactive = true;
            node.role = WidgetRole::Button;
            node.cursor = Some(WidgetCursor::Pointer);
            node.set_text(label);
            node.font_size = 11.0;
            node.line_height = 24.0;
            node.text_align = TextAlign::Center;

            let (bg, border_c, border_w, text_color) = if is_active {
                let bg_c = Color::rgba(
                    (accent_color.r * 0.45).clamp(0.08, 0.50),
                    (accent_color.g * 0.45).clamp(0.08, 0.50),
                    (accent_color.b * 0.45).clamp(0.08, 0.50),
                    0.52,
                );
                (bg_c, accent_color, 1.5, Color::WHITE)
            } else if hovered {
                let bg_c = Color::rgba(
                    (accent_color.r * 0.28).clamp(0.05, 0.35),
                    (accent_color.g * 0.28).clamp(0.05, 0.35),
                    (accent_color.b * 0.28).clamp(0.05, 0.35),
                    0.32,
                );
                (
                    bg_c,
                    Color::rgba(accent_color.r, accent_color.g, accent_color.b, 0.75),
                    1.0,
                    Color::WHITE,
                )
            } else {
                let bg_c = Color::rgba(
                    (accent_color.r * 0.18).clamp(0.03, 0.22),
                    (accent_color.g * 0.18).clamp(0.03, 0.22),
                    (accent_color.b * 0.18).clamp(0.03, 0.22),
                    0.22,
                );
                let txt_c = Color::rgba(
                    (0.70 + accent_color.r * 0.30).min(1.0),
                    (0.70 + accent_color.g * 0.30).min(1.0),
                    (0.70 + accent_color.b * 0.30).min(1.0),
                    1.0,
                );
                (
                    bg_c,
                    Color::rgba(accent_color.r, accent_color.g, accent_color.b, 0.42),
                    1.0,
                    txt_c,
                )
            };

            node.text_color = text_color;
            let mut style = Style::new()
                .width(width)
                .height(24.0)
                .padding_insets(Insets::new(2.0, 6.0, 2.0, 6.0))
                .background(bg)
                .border(border_w, border_c)
                .border_radius(4.0)
                .align_items(AlignItems::Center)
                .justify_content(JustifyContent::Center);
            if !is_active {
                let hover_bg_c = Color::rgba(
                    (accent_color.r * 0.28).clamp(0.05, 0.35),
                    (accent_color.g * 0.28).clamp(0.05, 0.35),
                    (accent_color.b * 0.28).clamp(0.05, 0.35),
                    0.32,
                );
                let hover_border_c =
                    Color::rgba(accent_color.r, accent_color.g, accent_color.b, 0.75);
                style = style
                    .hover_background(hover_bg_c)
                    .hover_border(1.0, hover_border_c);
                node.hover_text_color = Some(Color::WHITE);
            }
            node.set_style(style);
        }
        WidgetResponse::new(node_id, clicked, hovered, false)
    }

    /// Emits an interactive compact button designed for toolbar header strips.
    ///
    /// # Arguments
    /// * `label` - Button caption text.
    /// * `width` - Explicit button width in physical pixels.
    /// * `tag` - Semantic identifier inspected during interaction event dispatching.
    pub fn toolbar_button_tagged(
        &mut self,
        label: impl Into<String>,
        width: f32,
        tag: u64,
    ) -> WidgetResponse {
        let label_str = label.into();
        let node_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(node_id) {
            node.tag = tag;
        }
        let _ = self.tree.add_child(self.parent, node_id);
        let (clicked, hovered, _) = self.check_interaction(node_id);
        if let Some(node) = self.tree.get_mut(node_id) {
            node.interactive = true;
            node.role = WidgetRole::Button;
            node.cursor = Some(WidgetCursor::Pointer);
            node.set_text(label_str);
            node.font_size = 11.0;
            node.line_height = 24.0;
            node.text_align = TextAlign::Center;
            node.text_color = if hovered {
                Color::WHITE
            } else {
                Color::rgba(0.90, 0.93, 0.98, 1.0)
            };
            node.hover_text_color = Some(Color::WHITE);
            node.set_style(
                Style::new()
                    .width(width)
                    .height(24.0)
                    .padding_insets(Insets::new(2.0, 6.0, 2.0, 6.0))
                    .background(if hovered {
                        Color::rgba(0.24, 0.29, 0.39, 1.0)
                    } else {
                        Color::rgba(0.15, 0.18, 0.25, 0.95)
                    })
                    .hover_background(Color::rgba(0.24, 0.29, 0.39, 1.0))
                    .border(
                        1.0,
                        if hovered {
                            Color::rgba(0.0, 0.85, 1.0, 0.85)
                        } else {
                            Color::rgba(0.30, 0.36, 0.48, 0.75)
                        },
                    )
                    .hover_border(1.0, Color::rgba(0.0, 0.85, 1.0, 0.85))
                    .border_radius(4.0)
                    .align_items(AlignItems::Center)
                    .justify_content(JustifyContent::Center),
            );
        }
        WidgetResponse::new(node_id, clicked, hovered, false)
    }

    /// Emits a 32×32 pixel square toolbar icon button for gizmos, tools, and coordinate toggles.
    ///
    /// Provides built-in hover lifting, active state coloring, and pointer cursor styling.
    ///
    /// # Arguments
    /// * `icon_uv` - Atlas texture UV coordinates for the icon.
    /// * `tag` - Semantic tag for tracking interactions.
    /// * `is_active` - Whether this tool is currently selected or active.
    pub fn toolbar_icon_button(
        &mut self,
        icon_uv: [f32; 4],
        tag: u64,
        is_active: bool,
    ) -> WidgetResponse {
        let btn_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(btn_id) {
            node.set_name("ToolbarIconButton");
            node.set_role(WidgetRole::Button);
            node.set_tag(tag);
        }
        let _ = self.tree.add_child(self.parent, btn_id);
        let (clicked, hovered, _) = self.check_interaction(btn_id);

        let (bg, border_color) = if is_active {
            (
                Color::rgba(0.06, 0.46, 0.92, 1.0),
                Color::rgba(0.25, 0.60, 1.0, 0.95),
            )
        } else if hovered {
            (
                Color::rgba(0.20, 0.23, 0.30, 0.95),
                Color::rgba(0.35, 0.40, 0.50, 0.90),
            )
        } else {
            (
                Color::rgba(0.12, 0.13, 0.16, 0.92),
                Color::rgba(0.24, 0.26, 0.32, 0.85),
            )
        };

        if let Some(node) = self.tree.get_mut(btn_id) {
            node.interactive = true;
            node.cursor = Some(WidgetCursor::Pointer);
            let mut style = Style::new()
                .flex_row()
                .align_items(AlignItems::Center)
                .justify_content(JustifyContent::Center)
                .width(32.0)
                .height(32.0)
                .background(bg)
                .border(1.0, border_color)
                .border_radius(4.0)
                .box_shadow(0.0, 2.0, 6.0, Color::rgba(0.0, 0.0, 0.0, 0.35));
            if !is_active {
                style = style
                    .hover_background(Color::rgba(0.20, 0.23, 0.30, 0.95))
                    .hover_border(1.0, Color::rgba(0.35, 0.40, 0.50, 0.90));
            }
            node.set_style(style);
        }

        let icon_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(icon_id) {
            node.interactive = false;
            node.set_name("ToolbarIcon");
            node.set_texture_uv(icon_uv);
            let tint = if is_active {
                Color::rgba(1.0, 1.0, 1.0, 1.0)
            } else if hovered {
                Color::rgba(0.95, 0.98, 1.0, 1.0)
            } else {
                Color::rgba(0.80, 0.84, 0.90, 0.90)
            };
            node.set_texture_tint(tint);
            node.hover_texture_tint = Some(Color::rgba(0.95, 0.98, 1.0, 1.0));
            node.set_style(Style::new().width(22.0).height(22.0));
        }
        let _ = self.tree.add_child(btn_id, icon_id);

        WidgetResponse::new(btn_id, clicked, hovered, false)
    }

    /// Emits a viewport toolbar mode selector button (e.g. Camera Mode or Shading Mode).
    ///
    /// # Arguments
    /// * `icon_uv` - Optional leading icon texture UV coordinates.
    /// * `label` - Caption text displayed on the mode button.
    /// * `tag` - Semantic tag for tracking interactions.
    /// * `is_open` - Whether the dropdown popup associated with this mode is open.
    /// * `width` - Explicit button width in logical points.
    /// * `radii` - Corner radii styling.
    pub fn toolbar_mode_button(
        &mut self,
        icon_uv: Option<[f32; 4]>,
        label: impl Into<String>,
        tag: u64,
        is_open: bool,
        width: f32,
        radii: CornerRadii,
    ) -> WidgetResponse {
        let label_str = label.into();
        let btn_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(btn_id) {
            node.set_name("ToolbarModeButton");
            node.set_role(WidgetRole::Button);
            node.set_tag(tag);
        }
        let _ = self.tree.add_child(self.parent, btn_id);
        let (clicked, hovered, _) = self.check_interaction(btn_id);

        let is_highlighted = is_open || hovered;
        let bg = if is_highlighted {
            Color::rgba(0.20, 0.23, 0.30, 0.90)
        } else {
            Color::TRANSPARENT
        };
        let text_color = if is_highlighted {
            Color::rgba(1.0, 1.0, 1.0, 1.0)
        } else {
            Color::rgba(0.85, 0.88, 0.94, 1.0)
        };

        if let Some(node) = self.tree.get_mut(btn_id) {
            node.interactive = true;
            node.cursor = Some(WidgetCursor::Pointer);
            let mut style = Style::new()
                .flex_row()
                .align_items(AlignItems::Center)
                .justify_content(JustifyContent::Center)
                .gap(4.0)
                .width(width)
                .height(32.0)
                .background(bg)
                .corner_radii(radii);
            if !is_open {
                style = style.hover_background(Color::rgba(0.20, 0.23, 0.30, 0.90));
            }
            node.set_style(style);
        }

        if let Some(uv) = icon_uv {
            let icon_id = self.tree.create_node();
            if let Some(node) = self.tree.get_mut(icon_id) {
                node.interactive = false;
                node.set_name("ToolbarModeIcon");
                node.set_texture_uv(uv);
                node.set_texture_tint(text_color);
                node.hover_texture_tint = Some(Color::WHITE);
                node.set_style(Style::new().width(18.0).height(18.0));
            }
            let _ = self.tree.add_child(btn_id, icon_id);
        }

        let approx_text_w = (label_str.chars().count() as f32 * 6.8).ceil();
        let txt_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(txt_id) {
            node.interactive = false;
            node.set_name("ToolbarModeText");
            node.set_style(Style::new().width(approx_text_w).height(14.0));
            node.set_text(label_str);
            node.font_size = 11.0;
            node.line_height = 14.0;
            node.text_align = TextAlign::Center;
            node.text_color = text_color;
            node.hover_text_color = Some(Color::WHITE);
        }
        let _ = self.tree.add_child(btn_id, txt_id);

        WidgetResponse::new(btn_id, clicked, hovered, false)
    }
}