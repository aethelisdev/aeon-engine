// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Buttons & Interactive Clickable Controls
//!
//! Provides interactive push buttons, icon buttons, styled variants, and toggle pill
//! widgets returning immediate [`WidgetResponse`] interaction states.
//!

use super::core::UiScope;
use crate::declarative::types::{WidgetResponse, hash_label};
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
        let tag = hash_label(&label_str);
        self.button_tagged(label_str, tag)
    }

    /// Emits an interactive push button with a custom semantic tag.
    ///
    /// Automatically applies hover highlighting, pointer cursor, and layout styling internally.
    ///
    /// # Arguments
    /// * `label` - Button caption text.
    /// * `tag` - Unique semantic identifier inspected during interaction event dispatching.
    pub fn button_tagged(&mut self, label: impl Into<String>, tag: u64) -> WidgetResponse {
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
            node.line_height = 14.0;
            node.text_align = TextAlign::Center;
            node.text_color = if hovered {
                Color::WHITE
            } else {
                Color::rgba(0.0, 0.88, 1.0, 1.0)
            };
            node.set_style(
                Style::new()
                    .padding_insets(Insets::new(5.0, 14.0, 5.0, 14.0))
                    .background(if hovered {
                        Color::rgba(0.0, 0.45, 0.60, 0.80)
                    } else {
                        Color::rgba(0.0, 0.30, 0.42, 0.60)
                    })
                    .border(
                        1.0,
                        if hovered {
                            Color::rgba(0.0, 0.90, 1.0, 0.90)
                        } else {
                            Color::rgba(0.0, 0.75, 0.95, 0.60)
                        },
                    )
                    .border_radius(4.0)
                    .align_items(AlignItems::Center)
                    .justify_content(JustifyContent::Center),
            );
            node.style.width = Some(180.0);
            node.style.height = Some(26.0);
        }
        WidgetResponse::new(node_id, clicked, hovered, false)
    }

    /// Emits an interactive push button with a descriptive node name and a custom semantic tag.
    ///
    /// Automatically applies hover highlighting, pointer cursor, and layout styling internally.
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
            node.set_text(label);
            node.font_size = 11.0;
            node.line_height = 14.0;
            node.text_align = TextAlign::Center;
            node.text_color = if hovered {
                Color::WHITE
            } else {
                Color::rgba(0.0, 0.88, 1.0, 1.0)
            };
            node.set_style(
                Style::new()
                    .padding_insets(Insets::new(5.0, 14.0, 5.0, 14.0))
                    .background(if hovered {
                        Color::rgba(0.0, 0.45, 0.60, 0.80)
                    } else {
                        Color::rgba(0.0, 0.30, 0.42, 0.60)
                    })
                    .border(
                        1.0,
                        if hovered {
                            Color::rgba(0.0, 0.90, 1.0, 0.90)
                        } else {
                            Color::rgba(0.0, 0.75, 0.95, 0.60)
                        },
                    )
                    .border_radius(4.0)
                    .align_items(AlignItems::Center)
                    .justify_content(JustifyContent::Center),
            );
            node.style.width = Some(180.0);
            node.style.height = Some(26.0);
        }
        WidgetResponse::new(node_id, clicked, hovered, false)
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
                .border(
                    1.0,
                    if hovered {
                        Color::rgba(0.0, 0.90, 1.0, 0.95)
                    } else {
                        Color::rgba(0.20, 0.24, 0.30, 0.85)
                    },
                )
                .border_radius(4.0)
                .height(24.0);
            if let Some(w) = width {
                style.width = Some(w);
            }
            node.set_style(style);
        }

        let icon_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(icon_id) {
            node.interactive = false;
            node.set_texture_uv(icon_uv);
            node.set_texture_tint(if hovered { Color::WHITE } else { icon_tint });
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
        let tag = hash_label(&label_str);
        self.button_with_icon_tagged(icon_uv, label_str, tag)
    }

    /// Emits an interactive pill toggle button for mode or option selection.
    ///
    /// # Arguments
    /// * `label` - Button caption text.
    /// * `is_active` - Whether this option is currently selected.
    /// * `tag` - Semantic tag for tracking interactions.
    pub fn toggle_pill_tagged(&mut self, label: &str, is_active: bool, tag: u64) -> WidgetResponse {
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
            node.font_size = 9.5;
            node.line_height = 12.0;
            node.text_align = TextAlign::Center;
            let (bg, border, text_color) = if is_active {
                (
                    Color::rgba(0.0, 0.38, 0.50, 0.95),
                    Color::rgba(0.0, 0.85, 1.0, 0.95),
                    Color::WHITE,
                )
            } else if hovered {
                (
                    Color::rgba(0.16, 0.18, 0.22, 0.95),
                    Color::rgba(0.35, 0.40, 0.50, 0.90),
                    Color::rgba(0.85, 0.88, 0.92, 1.0),
                )
            } else {
                (
                    Color::rgba(0.11, 0.12, 0.15, 0.95),
                    Color::rgba(0.18, 0.20, 0.24, 0.85),
                    Color::rgba(0.65, 0.68, 0.74, 1.0),
                )
            };
            node.text_color = text_color;
            node.set_style(
                Style::new()
                    .padding_insets(Insets::new(3.0, 6.0, 3.0, 6.0))
                    .background(bg)
                    .border(1.0, border)
                    .border_radius(3.0)
                    .align_items(AlignItems::Center)
                    .justify_content(JustifyContent::Center)
                    .height(22.0),
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
            node.set_style(
                Style::new()
                    .flex_row()
                    .align_items(AlignItems::Center)
                    .justify_content(JustifyContent::Center)
                    .width(32.0)
                    .height(32.0)
                    .background(bg)
                    .border(1.0, border_color)
                    .border_radius(4.0)
                    .box_shadow(0.0, 2.0, 6.0, Color::rgba(0.0, 0.0, 0.0, 0.35)),
            );
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
            node.set_style(
                Style::new()
                    .flex_row()
                    .align_items(AlignItems::Center)
                    .justify_content(JustifyContent::Center)
                    .gap(4.0)
                    .width(width)
                    .height(32.0)
                    .background(bg)
                    .corner_radii(radii),
            );
        }

        if let Some(uv) = icon_uv {
            let icon_id = self.tree.create_node();
            if let Some(node) = self.tree.get_mut(icon_id) {
                node.interactive = false;
                node.set_name("ToolbarModeIcon");
                node.set_texture_uv(uv);
                node.set_texture_tint(text_color);
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
        }
        let _ = self.tree.add_child(btn_id, txt_id);

        WidgetResponse::new(btn_id, clicked, hovered, false)
    }

    /// Emits a circular interactive compass knob button for 3D orientation gizmos.
    ///
    /// Positioned absolutely within its parent compass canvas container. Applies circular
    /// geometry, pointer cursor, interactive event response, and automatically binds an embedded
    /// non-interactive text label centered within the knob geometry.
    ///
    /// # Arguments
    /// * `label` - Single-character axis indicator caption (e.g. "X", "Y", "Z").
    /// * `tag` - Unique 64-bit semantic tag for hit-testing and action dispatching.
    /// * `x` - Left coordinate offset relative to parent compass canvas in logical points.
    /// * `y` - Top coordinate offset relative to parent compass canvas in logical points.
    /// * `size` - Diameter of the circular knob button in logical points.
    /// * `color` - Idle background fill color representing the target axis.
    pub fn compass_knob(
        &mut self,
        label: impl Into<String>,
        tag: u64,
        x: f32,
        y: f32,
        size: f32,
        color: Color,
    ) -> WidgetResponse {
        let label_str = label.into();
        let knob_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(knob_id) {
            node.tag = tag;
        }
        let _ = self.tree.add_child(self.parent, knob_id);
        let (clicked, hovered, _) = self.check_interaction(knob_id);

        if let Some(node) = self.tree.get_mut(knob_id) {
            node.interactive = true;
            node.role = WidgetRole::Button;
            node.cursor = Some(WidgetCursor::Pointer);
            node.set_name("CompassKnob");
            node.set_style(
                Style::new()
                    .position_absolute()
                    .left(x)
                    .top(y)
                    .width(size)
                    .height(size)
                    .background(if hovered { Color::WHITE } else { color })
                    .border_radius(size * 0.5),
            );
        }

        let txt_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(txt_id) {
            node.interactive = false;
            node.set_name("CompassKnobText");
            node.set_text(label_str);
            node.font_size = 8.5;
            node.line_height = size;
            node.text_align = TextAlign::Center;
            node.text_color = if hovered { Color::BLACK } else { Color::WHITE };
            node.set_style(
                Style::new()
                    .position_absolute()
                    .left(0.0)
                    .top(0.0)
                    .width(size)
                    .height(size),
            );
        }
        let _ = self.tree.add_child(knob_id, txt_id);

        WidgetResponse::new(knob_id, clicked, hovered, false)
    }

    /// Emits a small circular interactive compass dot representing a negative axis endpoint.
    ///
    /// Positioned absolutely within its parent compass canvas container. Applies circular
    /// geometry, pointer cursor, and interactive event response for clicking and hover highlighting.
    ///
    /// # Arguments
    /// * `tag` - Unique 64-bit semantic tag for hit-testing and action dispatching.
    /// * `x` - Left coordinate offset relative to parent compass canvas in logical points.
    /// * `y` - Top coordinate offset relative to parent compass canvas in logical points.
    /// * `size` - Diameter of the circular dot in logical points.
    /// * `color` - Idle background fill color representing the negative axis.
    pub fn compass_dot(
        &mut self,
        tag: u64,
        x: f32,
        y: f32,
        size: f32,
        color: Color,
    ) -> WidgetResponse {
        let dot_id = self.tree.create_node();
        if let Some(node) = self.tree.get_mut(dot_id) {
            node.tag = tag;
        }
        let _ = self.tree.add_child(self.parent, dot_id);
        let (clicked, hovered, _) = self.check_interaction(dot_id);

        if let Some(node) = self.tree.get_mut(dot_id) {
            node.interactive = true;
            node.role = WidgetRole::Button;
            node.cursor = Some(WidgetCursor::Pointer);
            node.set_name("CompassDot");
            node.set_style(
                Style::new()
                    .position_absolute()
                    .left(x)
                    .top(y)
                    .width(size)
                    .height(size)
                    .background(if hovered { Color::WHITE } else { color })
                    .border_radius(size * 0.5),
            );
        }

        WidgetResponse::new(dot_id, clicked, hovered, false)
    }
}