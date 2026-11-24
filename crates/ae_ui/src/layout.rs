// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Backend-agnostic In-Game UI canvas layout resolver emitting sorted `UiDrawCommand` batches
//! from ECS UI components.

use ae_ecs::World;
use ae_plugin_api::{
    UiButton, UiCheckbox, UiElement, UiImage, UiPanel, UiProgressBar, UiRect, UiSlider, UiText,
    UiTextAlignment, UiTextInput,
};

/// Marker component for Pause Menu UI entities to allow clean teardown and HUD suppression.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PauseMenuUiTag;

/// Backend-agnostic drawing primitive emitted by the layout resolver.
#[derive(Debug, Clone, PartialEq)]
pub enum UiDrawCommand {
    /// Filled rectangle with optional border and rounded corners.
    Rect {
        /// Screen-space bounding rectangle.
        rect: UiRect,
        /// Linear RGBA fill color.
        fill_color: [f32; 4],
        /// Linear RGBA border color.
        border_color: [f32; 4],
        /// Border stroke width in logical pixels.
        border_width: f32,
        /// Corner rounding radius in logical pixels.
        border_radius: f32,
        /// Painter's order depth index.
        z_index: i32,
    },
    /// Text label draw command.
    Text {
        /// Anchor point `[x, y]` in logical pixels.
        pos: [f32; 2],
        /// Text string payload.
        text: String,
        /// Font point size in logical pixels.
        font_size: f32,
        /// Linear RGBA text color.
        color: [f32; 4],
        /// Horizontal alignment relative to `pos`.
        alignment: UiTextAlignment,
        /// Optional drop shadow color.
        shadow_color: Option<[f32; 4]>,
        /// Painter's order depth index.
        z_index: i32,
    },
    /// Textured quad draw command.
    Image {
        /// Screen-space bounding rectangle.
        rect: UiRect,
        /// Optional texture handle identifier.
        sprite_id: Option<u64>,
        /// Normalized `[u0, v0, u1, v1]` UV rect.
        uv_rect: [f32; 4],
        /// Linear RGBA tint color multiplier.
        tint: [f32; 4],
        /// Painter's order depth index.
        z_index: i32,
    },
}

impl UiDrawCommand {
    /// Returns the z-index used for painter-order depth sorting.
    #[must_use]
    pub fn z_index(&self) -> i32 {
        match self {
            Self::Rect { z_index, .. }
            | Self::Text { z_index, .. }
            | Self::Image { z_index, .. } => *z_index,
        }
    }
}

/// Resolves active ECS UI components into depth-sorted [`UiDrawCommand`] batches.
pub struct UiLayoutResolver;

impl UiLayoutResolver {
    /// Resolves active UI components into ordered [`UiDrawCommand`] primitives for a viewport of
    /// size `(screen_w, screen_h)`.
    ///
    /// Returns an empty list if dimensions are non-finite or non-positive.
    #[must_use]
    pub fn resolve_draw_commands(
        world: &World,
        screen_w: f32,
        screen_h: f32,
        mouse_pos: Option<[f32; 2]>,
        _mouse_clicked: bool,
    ) -> Vec<UiDrawCommand> {
        if !screen_w.is_finite() || !screen_h.is_finite() || screen_w <= 0.0 || screen_h <= 0.0 {
            return Vec::new();
        }

        let mut commands = Vec::new();
        Self::emit_panels(world, screen_w, screen_h, &mut commands);
        Self::emit_buttons(world, screen_w, screen_h, mouse_pos, &mut commands);
        Self::emit_progress_bars(world, screen_w, screen_h, &mut commands);
        Self::emit_sliders(world, screen_w, screen_h, &mut commands);
        Self::emit_checkboxes(world, screen_w, screen_h, &mut commands);
        Self::emit_texts(world, screen_w, screen_h, &mut commands);
        Self::emit_images(world, screen_w, screen_h, &mut commands);
        Self::emit_text_inputs(world, screen_w, screen_h, &mut commands);
        commands.sort_by_key(UiDrawCommand::z_index);
        commands
    }

    fn emit_panels(world: &World, screen_w: f32, screen_h: f32, out: &mut Vec<UiDrawCommand>) {
        for (elem, panel) in world.query::<(&UiElement, &UiPanel)>().iter() {
            if !elem.visible {
                continue;
            }
            out.push(UiDrawCommand::Rect {
                rect: elem.compute_rect(screen_w, screen_h),
                fill_color: panel.background_color,
                border_color: panel.border_color,
                border_width: panel.border_width,
                border_radius: panel.corner_radius,
                z_index: elem.z_index,
            });
        }
    }

    fn emit_buttons(
        world: &World,
        screen_w: f32,
        screen_h: f32,
        mouse_pos: Option<[f32; 2]>,
        out: &mut Vec<UiDrawCommand>,
    ) {
        for (elem, btn) in world.query::<(&UiElement, &UiButton)>().iter() {
            if !elem.visible {
                continue;
            }
            let rect = elem.compute_rect(screen_w, screen_h);
            let hovered = mouse_pos.is_some_and(|p| rect.contains(p));
            let color = if !btn.is_enabled {
                btn.disabled_color
            } else if hovered {
                btn.hover_color
            } else {
                btn.normal_color
            };

            out.push(UiDrawCommand::Rect {
                rect,
                fill_color: color,
                border_color: [0.3, 0.4, 0.5, 0.8],
                border_width: 1.0,
                border_radius: 4.0,
                z_index: elem.z_index,
            });
            out.push(UiDrawCommand::Text {
                pos: [
                    rect.min_x + rect.width() * 0.5,
                    rect.min_y + rect.height() * 0.5,
                ],
                text: btn.text.clone(),
                font_size: 14.0,
                color: [1.0, 1.0, 1.0, 1.0],
                alignment: UiTextAlignment::Center,
                shadow_color: Some([0.0, 0.0, 0.0, 0.6]),
                z_index: elem.z_index + 1,
            });
        }
    }

    fn emit_progress_bars(
        world: &World,
        screen_w: f32,
        screen_h: f32,
        out: &mut Vec<UiDrawCommand>,
    ) {
        for (elem, bar) in world.query::<(&UiElement, &UiProgressBar)>().iter() {
            if !elem.visible {
                continue;
            }
            let rect = elem.compute_rect(screen_w, screen_h);
            out.push(UiDrawCommand::Rect {
                rect,
                fill_color: bar.background_color,
                border_color: bar.border_color,
                border_width: 1.0,
                border_radius: bar.corner_radius,
                z_index: elem.z_index,
            });

            let fill_width = rect.width() * bar.fraction();
            if fill_width > 0.0 {
                out.push(UiDrawCommand::Rect {
                    rect: UiRect::new(rect.min_x, rect.min_y, rect.min_x + fill_width, rect.max_y),
                    fill_color: bar.fill_color,
                    border_color: [0.0, 0.0, 0.0, 0.0],
                    border_width: 0.0,
                    border_radius: bar.corner_radius,
                    z_index: elem.z_index + 1,
                });
            }
        }
    }

    fn emit_sliders(world: &World, screen_w: f32, screen_h: f32, out: &mut Vec<UiDrawCommand>) {
        for (elem, slider) in world.query::<(&UiElement, &UiSlider)>().iter() {
            if !elem.visible {
                continue;
            }
            let rect = elem.compute_rect(screen_w, screen_h);
            let track_height = (rect.height() * 0.25).max(4.0);
            let track_y = rect.min_y + (rect.height() - track_height) * 0.5;
            out.push(UiDrawCommand::Rect {
                rect: UiRect::new(rect.min_x, track_y, rect.max_x, track_y + track_height),
                fill_color: slider.track_color,
                border_color: [0.3, 0.4, 0.5, 0.8],
                border_width: 1.0,
                border_radius: 2.0,
                z_index: elem.z_index,
            });

            let range = slider.max - slider.min;
            let fraction = if range.abs() < 1e-4 {
                0.0
            } else {
                ((slider.value - slider.min) / range).clamp(0.0, 1.0)
            };
            let thumb_size = rect.height() * 0.8;
            let thumb_x = rect.min_x + (rect.width() - thumb_size) * fraction;
            out.push(UiDrawCommand::Rect {
                rect: UiRect::new(
                    thumb_x,
                    rect.min_y + (rect.height() - thumb_size) * 0.5,
                    thumb_x + thumb_size,
                    rect.min_y + (rect.height() + thumb_size) * 0.5,
                ),
                fill_color: slider.thumb_color,
                border_color: [1.0, 1.0, 1.0, 0.9],
                border_width: 1.0,
                border_radius: thumb_size * 0.5,
                z_index: elem.z_index + 1,
            });
        }
    }

    fn emit_checkboxes(world: &World, screen_w: f32, screen_h: f32, out: &mut Vec<UiDrawCommand>) {
        for (elem, chk) in world.query::<(&UiElement, &UiCheckbox)>().iter() {
            if !elem.visible {
                continue;
            }
            let rect = elem.compute_rect(screen_w, screen_h);
            let box_size = rect.height().min(20.0);
            let box_rect = UiRect::new(
                rect.min_x,
                rect.min_y + (rect.height() - box_size) * 0.5,
                rect.min_x + box_size,
                rect.min_y + (rect.height() + box_size) * 0.5,
            );

            out.push(UiDrawCommand::Rect {
                rect: box_rect,
                fill_color: if chk.is_checked {
                    chk.check_color
                } else {
                    chk.box_color
                },
                border_color: [0.3, 0.4, 0.5, 0.8],
                border_width: 1.0,
                border_radius: 3.0,
                z_index: elem.z_index,
            });
            out.push(UiDrawCommand::Text {
                pos: [
                    rect.min_x + box_size + 8.0,
                    rect.min_y + rect.height() * 0.5,
                ],
                text: chk.label.clone(),
                font_size: 13.0,
                color: [0.9, 0.9, 0.9, 1.0],
                alignment: UiTextAlignment::Left,
                shadow_color: Some([0.0, 0.0, 0.0, 0.6]),
                z_index: elem.z_index + 1,
            });
        }
    }

    fn emit_texts(world: &World, screen_w: f32, screen_h: f32, out: &mut Vec<UiDrawCommand>) {
        for (elem, text) in world.query::<(&UiElement, &UiText)>().iter() {
            if !elem.visible {
                continue;
            }
            let rect = elem.compute_rect(screen_w, screen_h);
            let pos = match text.alignment {
                UiTextAlignment::Left => [rect.min_x, rect.min_y + rect.height() * 0.5],
                UiTextAlignment::Center => [
                    rect.min_x + rect.width() * 0.5,
                    rect.min_y + rect.height() * 0.5,
                ],
                UiTextAlignment::Right => [rect.max_x, rect.min_y + rect.height() * 0.5],
            };
            out.push(UiDrawCommand::Text {
                pos,
                text: text.text.clone(),
                font_size: text.font_size,
                color: text.color,
                alignment: text.alignment,
                shadow_color: text.shadow_color,
                z_index: elem.z_index,
            });
        }
    }

    fn emit_images(world: &World, screen_w: f32, screen_h: f32, out: &mut Vec<UiDrawCommand>) {
        for (elem, img) in world.query::<(&UiElement, &UiImage)>().iter() {
            if !elem.visible {
                continue;
            }
            out.push(UiDrawCommand::Image {
                rect: elem.compute_rect(screen_w, screen_h),
                sprite_id: img.sprite_id,
                uv_rect: img.uv_rect,
                tint: img.tint,
                z_index: elem.z_index,
            });
        }
    }

    fn emit_text_inputs(world: &World, screen_w: f32, screen_h: f32, out: &mut Vec<UiDrawCommand>) {
        for (elem, input) in world.query::<(&UiElement, &UiTextInput)>().iter() {
            if !elem.visible {
                continue;
            }
            let rect = elem.compute_rect(screen_w, screen_h);
            let border_color = if input.is_focused {
                [0.0, 0.75, 1.0, 1.0]
            } else {
                [0.25, 0.35, 0.45, 0.8]
            };
            out.push(UiDrawCommand::Rect {
                rect,
                fill_color: [0.08, 0.10, 0.14, 0.95],
                border_color,
                border_width: 1.0,
                border_radius: 4.0,
                z_index: elem.z_index,
            });

            let (display_text, text_color) = if !input.text.is_empty() {
                (input.text.clone(), [0.95, 0.95, 0.95, 1.0])
            } else {
                (input.placeholder.clone(), [0.50, 0.55, 0.65, 0.75])
            };
            out.push(UiDrawCommand::Text {
                pos: [rect.min_x + 10.0, rect.min_y + rect.height() * 0.5],
                text: display_text,
                font_size: 13.0,
                color: text_color,
                alignment: UiTextAlignment::Left,
                shadow_color: None,
                z_index: elem.z_index + 1,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ae_ecs::World;
    use ae_plugin_api::UiAnchor;

    #[test]
    fn test_ui_anchor_and_bounding_rect_calculation() {
        let (w, h) = (1920.0, 1080.0);
        assert_eq!(UiAnchor::TopLeft.compute_origin(w, h), [0.0, 0.0]);
        assert_eq!(UiAnchor::TopCenter.compute_origin(w, h), [960.0, 0.0]);
        assert_eq!(UiAnchor::Center.compute_origin(w, h), [960.0, 540.0]);
        assert_eq!(UiAnchor::BottomRight.compute_origin(w, h), [1920.0, 1080.0]);

        let elem = UiElement::new(UiAnchor::Center, [10.0, -20.0], [200.0, 50.0]);
        let rect = elem.compute_rect(w, h);
        assert_eq!(rect.min_x, 870.0);
        assert_eq!(rect.max_x, 1070.0);
        assert_eq!(rect.min_y, 495.0);
        assert_eq!(rect.max_y, 545.0);
    }

    #[test]
    fn test_ui_layout_resolver_sorting_and_invalid_viewport_guard() {
        let mut world = World::new();
        world.spawn((
            UiElement {
                z_index: 10,
                ..Default::default()
            },
            UiText::new("Layer 10", 12.0),
        ));
        world.spawn((
            UiElement {
                z_index: 0,
                ..Default::default()
            },
            UiPanel::default(),
        ));

        assert!(
            UiLayoutResolver::resolve_draw_commands(&world, f32::NAN, 1080.0, None, false)
                .is_empty()
        );
        assert!(
            UiLayoutResolver::resolve_draw_commands(&world, 1920.0, 0.0, None, false).is_empty()
        );

        let cmds = UiLayoutResolver::resolve_draw_commands(&world, 1920.0, 1080.0, None, false);
        assert_eq!(cmds.len(), 2);
        assert_eq!(cmds[0].z_index(), 0);
        assert_eq!(cmds[1].z_index(), 10);
    }

    #[test]
    fn test_ui_text_input_resolver() {
        let mut world = World::new();
        world.spawn((
            UiElement {
                anchor: UiAnchor::Center,
                offset: [0.0, 0.0],
                size: [180.0, 32.0],
                z_index: 5,
                ..Default::default()
            },
            UiTextInput {
                text: "Player1".to_string(),
                placeholder: "Enter name...".to_string(),
                max_length: Some(16),
                is_focused: true,
            },
        ));

        let cmds = UiLayoutResolver::resolve_draw_commands(&world, 1920.0, 1080.0, None, false);
        assert_eq!(cmds.len(), 2);
        match &cmds[0] {
            UiDrawCommand::Rect {
                border_color,
                z_index,
                ..
            } => {
                assert_eq!(*z_index, 5);
                assert!((border_color[1] - 0.75).abs() < 1e-3);
            }
            _ => panic!("Expected Rect command first"),
        }
        match &cmds[1] {
            UiDrawCommand::Text { text, z_index, .. } => {
                assert_eq!(*z_index, 6);
                assert_eq!(text, "Player1");
            }
            _ => panic!("Expected Text command second"),
        }
    }
}