// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Backend-agnostic In-Game UI, Canvas Layout, and HUD Engine.
//!
//! Designed for 100% decoupling from any specific GUI library (such as egui,
//! Slint, or custom WGPU GPU quad renderers). Game logic interacts solely with
//! ECS components (`UiElement`, `UiPanel`, `UiText`, `UiProgressBar`, `UiButton`, `UiImage`, `UiSlider`, `UiCheckbox`), while the
//! `UiLayoutResolver` generates hardware-agnostic `UiDrawCommand` batches.
//!

pub use ae_plugin_api::{
    PlayerHealthBarTag, ReticleTag, ScoreDisplayTag, UiAnchor, UiButton, UiCheckbox, UiElement,
    UiImage, UiLayoutGroup, UiLayoutType, UiPanel, UiProgressBar, UiRect, UiSliceMode, UiSlider,
    UiText, UiTextAlignment, UiTextInput,
};

/// Marker component for Pause Menu UI entities to allow clean teardown and HUD suppression.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PauseMenuUiTag;

/// Backend-agnostic drawing primitive emitted by the layout resolver.
#[derive(Debug, Clone, PartialEq)]
pub enum UiDrawCommand {
    /// Filled rectangle with optional border and rounded corners.
    Rect {
        rect: UiRect,
        fill_color: [f32; 4],
        border_color: [f32; 4],
        border_width: f32,
        border_radius: f32,
        z_index: i32,
    },
    /// Text label draw command.
    Text {
        pos: [f32; 2],
        text: String,
        font_size: f32,
        color: [f32; 4],
        alignment: UiTextAlignment,
        shadow_color: Option<[f32; 4]>,
        z_index: i32,
    },
    /// Textured quad draw command.
    Image {
        rect: UiRect,
        sprite_id: Option<u64>,
        uv_rect: [f32; 4],
        tint: [f32; 4],
        z_index: i32,
    },
}

impl UiDrawCommand {
    /// Returns z-index for depth sorting.
    pub fn z_index(&self) -> i32 {
        match self {
            Self::Rect { z_index, .. } => *z_index,
            Self::Text { z_index, .. } => *z_index,
            Self::Image { z_index, .. } => *z_index,
        }
    }
}

/// Layout resolver: resolves all UI components in the ECS world into sorted draw commands.
pub struct UiLayoutResolver;

impl UiLayoutResolver {
    /// Resolves active UI components into ordered `UiDrawCommand` primitives.
    pub fn resolve_draw_commands(
        world: &hecs::World,
        screen_w: f32,
        screen_h: f32,
        mouse_pos: Option<[f32; 2]>,
        _mouse_clicked: bool,
    ) -> Vec<UiDrawCommand> {
        let mut commands = Vec::new();

        // 1. Process Panels / Canvas Containers
        for (elem, panel) in world.query::<(&UiElement, &UiPanel)>().iter() {
            if !elem.visible {
                continue;
            }
            let rect = elem.compute_rect(screen_w, screen_h);
            commands.push(UiDrawCommand::Rect {
                rect,
                fill_color: panel.background_color,
                border_color: panel.border_color,
                border_width: panel.border_width,
                border_radius: panel.corner_radius,
                z_index: elem.z_index,
            });
        }

        // 2. Process Buttons
        for (elem, btn) in world.query::<(&UiElement, &UiButton)>().iter() {
            if !elem.visible {
                continue;
            }
            let rect = elem.compute_rect(screen_w, screen_h);
            let hovered = mouse_pos.map(|p| rect.contains(p)).unwrap_or(false);

            let color = if !btn.is_enabled {
                btn.disabled_color
            } else if hovered {
                btn.hover_color
            } else {
                btn.normal_color
            };

            commands.push(UiDrawCommand::Rect {
                rect,
                fill_color: color,
                border_color: [0.3, 0.4, 0.5, 0.8],
                border_width: 1.0,
                border_radius: 4.0,
                z_index: elem.z_index,
            });

            commands.push(UiDrawCommand::Text {
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

        // 3. Process Progress Bars
        for (elem, bar) in world.query::<(&UiElement, &UiProgressBar)>().iter() {
            if !elem.visible {
                continue;
            }
            let rect = elem.compute_rect(screen_w, screen_h);

            // Background track
            commands.push(UiDrawCommand::Rect {
                rect,
                fill_color: bar.background_color,
                border_color: bar.border_color,
                border_width: 1.0,
                border_radius: bar.corner_radius,
                z_index: elem.z_index,
            });

            // Fill meter
            let fill_width = rect.width() * bar.fraction();
            if fill_width > 0.0 {
                let fill_rect =
                    UiRect::new(rect.min_x, rect.min_y, rect.min_x + fill_width, rect.max_y);
                commands.push(UiDrawCommand::Rect {
                    rect: fill_rect,
                    fill_color: bar.fill_color,
                    border_color: [0.0, 0.0, 0.0, 0.0],
                    border_width: 0.0,
                    border_radius: bar.corner_radius,
                    z_index: elem.z_index + 1,
                });
            }
        }

        // 4. Process Sliders
        for (elem, slider) in world.query::<(&UiElement, &UiSlider)>().iter() {
            if !elem.visible {
                continue;
            }
            let rect = elem.compute_rect(screen_w, screen_h);
            let track_height = (rect.height() * 0.25).max(4.0);
            let track_y = rect.min_y + (rect.height() - track_height) * 0.5;

            // Track background
            let track_rect = UiRect::new(rect.min_x, track_y, rect.max_x, track_y + track_height);
            commands.push(UiDrawCommand::Rect {
                rect: track_rect,
                fill_color: slider.track_color,
                border_color: [0.3, 0.4, 0.5, 0.8],
                border_width: 1.0,
                border_radius: 2.0,
                z_index: elem.z_index,
            });

            // Slider thumb handle
            let range = slider.max - slider.min;
            let fraction = if range.abs() < 1e-4 {
                0.0
            } else {
                ((slider.value - slider.min) / range).clamp(0.0, 1.0)
            };
            let thumb_size = rect.height() * 0.8;
            let thumb_x = rect.min_x + (rect.width() - thumb_size) * fraction;
            let thumb_rect = UiRect::new(
                thumb_x,
                rect.min_y + (rect.height() - thumb_size) * 0.5,
                thumb_x + thumb_size,
                rect.min_y + (rect.height() + thumb_size) * 0.5,
            );
            commands.push(UiDrawCommand::Rect {
                rect: thumb_rect,
                fill_color: slider.thumb_color,
                border_color: [1.0, 1.0, 1.0, 0.9],
                border_width: 1.0,
                border_radius: thumb_size * 0.5,
                z_index: elem.z_index + 1,
            });
        }

        // 5. Process Checkboxes
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

            commands.push(UiDrawCommand::Rect {
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

            // Label
            commands.push(UiDrawCommand::Text {
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

        // 6. Process Standalone Texts
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
            commands.push(UiDrawCommand::Text {
                pos,
                text: text.text.clone(),
                font_size: text.font_size,
                color: text.color,
                alignment: text.alignment,
                shadow_color: text.shadow_color,
                z_index: elem.z_index,
            });
        }

        // 7. Process Images
        for (elem, img) in world.query::<(&UiElement, &UiImage)>().iter() {
            if !elem.visible {
                continue;
            }
            let rect = elem.compute_rect(screen_w, screen_h);
            commands.push(UiDrawCommand::Image {
                rect,
                sprite_id: img.sprite_id,
                uv_rect: img.uv_rect,
                tint: img.tint,
                z_index: elem.z_index,
            });
        }

        // Sort commands by z_index ascending (lower drawn first)
        commands.sort_by_key(|cmd| cmd.z_index());
        commands
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hecs::World;

    #[test]
    fn test_ui_anchor_origin_calculation() {
        let (w, h) = (1920.0, 1080.0);
        assert_eq!(UiAnchor::TopLeft.compute_origin(w, h), [0.0, 0.0]);
        assert_eq!(UiAnchor::TopCenter.compute_origin(w, h), [960.0, 0.0]);
        assert_eq!(UiAnchor::Center.compute_origin(w, h), [960.0, 540.0]);
        assert_eq!(UiAnchor::BottomRight.compute_origin(w, h), [1920.0, 1080.0]);
    }

    #[test]
    fn test_ui_element_bounding_rect_calculation() {
        let elem = UiElement::new(UiAnchor::Center, [10.0, -20.0], [200.0, 50.0]);
        let rect = elem.compute_rect(1920.0, 1080.0);

        let center_x = 960.0 + 10.0;
        let center_y = 540.0 - 20.0;
        assert_eq!(rect.min_x, center_x - 100.0);
        assert_eq!(rect.max_x, center_x + 100.0);
        assert_eq!(rect.min_y, center_y - 25.0);
        assert_eq!(rect.max_y, center_y + 25.0);
    }

    #[test]
    fn test_ui_progress_bar_fraction() {
        let mut bar = UiProgressBar {
            min: 0.0,
            max: 200.0,
            value: 50.0,
            ..Default::default()
        };
        assert!((bar.fraction() - 0.25).abs() < 1e-4);

        bar.value = 300.0;
        assert!((bar.fraction() - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_ui_layout_resolver_sorting() {
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

        let cmds = UiLayoutResolver::resolve_draw_commands(&world, 1920.0, 1080.0, None, false);
        assert_eq!(cmds.len(), 2);
        assert_eq!(cmds[0].z_index(), 0);
        assert_eq!(cmds[1].z_index(), 10);
    }
}