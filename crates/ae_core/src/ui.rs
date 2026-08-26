// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Backend-agnostic In-Game UI, Canvas Layout, and HUD Engine.
//!
//! Designed for 100% decoupling from any specific GUI library (such as egui,
//! Slint, or custom WGPU GPU quad renderers). Game logic interacts solely with
//! ECS components (`UiElement`, `UiText`, `UiProgressBar`, `UiButton`), while the
//! `UiLayoutResolver` generates hardware-agnostic `UiDrawCommand` batches.
//!

/// Marker component for Pause Menu UI entities to allow clean teardown and HUD suppression.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PauseMenuUiTag;

/// Anchor points defining alignment on the screen canvas.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum UiAnchor {
    #[default]
    TopLeft,
    TopCenter,
    TopRight,
    CenterLeft,
    Center,
    CenterRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
}

impl UiAnchor {
    /// Computes the pixel origin `[x, y]` on a screen of size `(screen_w, screen_h)`.
    #[inline]
    pub fn compute_origin(self, screen_w: f32, screen_h: f32) -> [f32; 2] {
        match self {
            Self::TopLeft => [0.0, 0.0],
            Self::TopCenter => [screen_w * 0.5, 0.0],
            Self::TopRight => [screen_w, 0.0],
            Self::CenterLeft => [0.0, screen_h * 0.5],
            Self::Center => [screen_w * 0.5, screen_h * 0.5],
            Self::CenterRight => [screen_w, screen_h * 0.5],
            Self::BottomLeft => [0.0, screen_h],
            Self::BottomCenter => [screen_w * 0.5, screen_h],
            Self::BottomRight => [screen_w, screen_h],
        }
    }
}

/// Screen-space 2D bounding rectangle.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct UiRect {
    pub min_x: f32,
    pub min_y: f32,
    pub max_x: f32,
    pub max_y: f32,
}

impl UiRect {
    /// Constructs a new rectangle from min and max coordinates.
    pub fn new(min_x: f32, min_y: f32, max_x: f32, max_y: f32) -> Self {
        Self {
            min_x,
            min_y,
            max_x,
            max_y,
        }
    }

    /// Computes width of the rectangle.
    pub fn width(self) -> f32 {
        (self.max_x - self.min_x).max(0.0)
    }

    /// Computes height of the rectangle.
    pub fn height(self) -> f32 {
        (self.max_y - self.min_y).max(0.0)
    }

    /// Checks if a 2D point lies within the rectangle bounds.
    pub fn contains(self, point: [f32; 2]) -> bool {
        point[0] >= self.min_x
            && point[0] <= self.max_x
            && point[1] >= self.min_y
            && point[1] <= self.max_y
    }
}

/// Primary UI positioning and visibility component.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UiElement {
    pub anchor: UiAnchor,
    pub offset: [f32; 2],
    pub size: [f32; 2],
    pub visible: bool,
    pub z_index: i32,
}

impl Default for UiElement {
    fn default() -> Self {
        Self {
            anchor: UiAnchor::TopLeft,
            offset: [0.0, 0.0],
            size: [100.0, 30.0],
            visible: true,
            z_index: 0,
        }
    }
}

impl UiElement {
    /// Creates a new positioned UI element.
    pub fn new(anchor: UiAnchor, offset: [f32; 2], size: [f32; 2]) -> Self {
        Self {
            anchor,
            offset,
            size,
            visible: true,
            z_index: 0,
        }
    }

    /// Computes screen-space bounding rectangle given canvas dimensions.
    pub fn compute_rect(&self, screen_w: f32, screen_h: f32) -> UiRect {
        let origin = self.anchor.compute_origin(screen_w, screen_h);
        let center_x = origin[0] + self.offset[0];
        let center_y = origin[1] + self.offset[1];
        let half_w = self.size[0] * 0.5;
        let half_h = self.size[1] * 0.5;

        UiRect::new(
            center_x - half_w,
            center_y - half_h,
            center_x + half_w,
            center_y + half_h,
        )
    }
}

/// Text alignment option for UI rendering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum UiTextAlignment {
    #[default]
    Left,
    Center,
    Right,
}

/// Text label component rendered on a UI element.
#[derive(Debug, Clone, PartialEq)]
pub struct UiText {
    pub text: String,
    pub font_size: f32,
    pub color: [f32; 4],
    pub alignment: UiTextAlignment,
}

impl UiText {
    /// Creates a new text label with default white color and left alignment.
    pub fn new(text: impl Into<String>, font_size: f32) -> Self {
        Self {
            text: text.into(),
            font_size,
            color: [1.0, 1.0, 1.0, 1.0],
            alignment: UiTextAlignment::Left,
        }
    }

    /// Builder method to specify text RGBA color.
    pub fn with_color(mut self, color: [f32; 4]) -> Self {
        self.color = color;
        self
    }

    /// Builder method to specify text alignment.
    pub fn with_alignment(mut self, alignment: UiTextAlignment) -> Self {
        self.alignment = alignment;
        self
    }
}

/// Progress bar / health bar component for meters and gauges.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UiProgressBar {
    pub min: f32,
    pub max: f32,
    pub value: f32,
    pub fill_color: [f32; 4],
    pub background_color: [f32; 4],
    pub border_color: [f32; 4],
}

impl Default for UiProgressBar {
    fn default() -> Self {
        Self {
            min: 0.0,
            max: 100.0,
            value: 100.0,
            fill_color: [0.2, 0.8, 0.2, 1.0], // Green
            background_color: [0.08, 0.10, 0.14, 0.85],
            border_color: [0.3, 0.3, 0.3, 1.0],
        }
    }
}

impl UiProgressBar {
    /// Creates a new progress bar with specified max and initial value.
    pub fn new(max: f32, initial_value: f32) -> Self {
        Self {
            min: 0.0,
            max,
            value: initial_value,
            ..Default::default()
        }
    }

    /// Computes normalized fill fraction `[0.0, 1.0]`.
    pub fn fraction(self) -> f32 {
        let range = self.max - self.min;
        if range.abs() < f32::EPSILON {
            0.0
        } else {
            ((self.value - self.min) / range).clamp(0.0, 1.0)
        }
    }
}

/// Interactive button component.
#[derive(Debug, Clone, PartialEq)]
pub struct UiButton {
    pub label: String,
    pub normal_color: [f32; 4],
    pub hovered_color: [f32; 4],
    pub pressed_color: [f32; 4],
    pub is_hovered: bool,
    pub is_pressed: bool,
    pub clicked: bool,
}

impl UiButton {
    /// Creates a new interactive UI button with default styling.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            normal_color: [0.18, 0.22, 0.30, 0.95],
            hovered_color: [0.28, 0.42, 0.65, 1.0],
            pressed_color: [0.14, 0.30, 0.50, 1.0],
            is_hovered: false,
            is_pressed: false,
            clicked: false,
        }
    }
}

/// Image / Icon component rendered on a UI element.
#[derive(Debug, Clone, PartialEq)]
pub struct UiImage {
    pub texture_name: String,
    pub tint: [f32; 4],
    pub uv_rect: [f32; 4],
}

impl UiImage {
    /// Creates a new UI image.
    pub fn new(texture_name: impl Into<String>) -> Self {
        Self {
            texture_name: texture_name.into(),
            tint: [1.0, 1.0, 1.0, 1.0],
            uv_rect: [0.0, 0.0, 1.0, 1.0],
        }
    }
}

/// Backend-agnostic drawing primitive emitted by the layout resolver.
#[derive(Debug, Clone, PartialEq)]
pub enum UiDrawCommand {
    /// Filled rectangle with optional border.
    Rect {
        rect: UiRect,
        fill_color: [f32; 4],
        border_color: [f32; 4],
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
        z_index: i32,
    },
    /// Textured quad draw command.
    Image {
        rect: UiRect,
        texture_name: String,
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

        // 1. Process buttons
        for (elem, btn) in world.query::<(&UiElement, &UiButton)>().iter() {
            if !elem.visible {
                continue;
            }
            let rect = elem.compute_rect(screen_w, screen_h);
            let hovered = mouse_pos.map(|p| rect.contains(p)).unwrap_or(false);

            let color = if btn.is_pressed {
                btn.pressed_color
            } else if hovered {
                btn.hovered_color
            } else {
                btn.normal_color
            };

            commands.push(UiDrawCommand::Rect {
                rect,
                fill_color: color,
                border_color: [1.0, 1.0, 1.0, 0.4],
                border_radius: 4.0,
                z_index: elem.z_index,
            });

            commands.push(UiDrawCommand::Text {
                pos: [
                    rect.min_x + rect.width() * 0.5,
                    rect.min_y + rect.height() * 0.5,
                ],
                text: btn.label.clone(),
                font_size: 14.0,
                color: [1.0, 1.0, 1.0, 1.0],
                alignment: UiTextAlignment::Center,
                z_index: elem.z_index + 1,
            });
        }

        // 2. Process progress bars
        for (elem, bar) in world.query::<(&UiElement, &UiProgressBar)>().iter() {
            if !elem.visible {
                continue;
            }
            let rect = elem.compute_rect(screen_w, screen_h);

            // Background
            commands.push(UiDrawCommand::Rect {
                rect,
                fill_color: bar.background_color,
                border_color: bar.border_color,
                border_radius: 2.0,
                z_index: elem.z_index,
            });

            // Fill
            let fill_width = rect.width() * bar.fraction();
            if fill_width > 0.0 {
                let fill_rect =
                    UiRect::new(rect.min_x, rect.min_y, rect.min_x + fill_width, rect.max_y);
                commands.push(UiDrawCommand::Rect {
                    rect: fill_rect,
                    fill_color: bar.fill_color,
                    border_color: [0.0, 0.0, 0.0, 0.0],
                    border_radius: 2.0,
                    z_index: elem.z_index + 1,
                });
            }
        }

        // 3. Process standalone texts
        for (elem, text) in world.query::<(&UiElement, &UiText)>().iter() {
            if !elem.visible {
                continue;
            }
            let rect = elem.compute_rect(screen_w, screen_h);
            let pos = match text.alignment {
                UiTextAlignment::Left => [rect.min_x, rect.min_y],
                UiTextAlignment::Center => [
                    rect.min_x + rect.width() * 0.5,
                    rect.min_y + rect.height() * 0.5,
                ],
                UiTextAlignment::Right => [rect.max_x, rect.min_y],
            };
            commands.push(UiDrawCommand::Text {
                pos,
                text: text.text.clone(),
                font_size: text.font_size,
                color: text.color,
                alignment: text.alignment,
                z_index: elem.z_index,
            });
        }

        // 4. Process images
        for (elem, img) in world.query::<(&UiElement, &UiImage)>().iter() {
            if !elem.visible {
                continue;
            }
            let rect = elem.compute_rect(screen_w, screen_h);
            commands.push(UiDrawCommand::Image {
                rect,
                texture_name: img.texture_name.clone(),
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
        let rect = elem.compute_rect(1000.0, 1000.0);
        assert_eq!(rect.min_x, 500.0 + 10.0 - 100.0); // 410.0
        assert_eq!(rect.max_x, 500.0 + 10.0 + 100.0); // 610.0
        assert_eq!(rect.min_y, 500.0 - 20.0 - 25.0); // 455.0
        assert_eq!(rect.max_y, 500.0 - 20.0 + 25.0); // 505.0
        assert!(rect.contains([510.0, 480.0]));
        assert!(!rect.contains([0.0, 0.0]));
    }

    #[test]
    fn test_ui_progress_bar_fraction() {
        let bar = UiProgressBar::new(200.0, 50.0);
        assert_eq!(bar.fraction(), 0.25);

        let full_bar = UiProgressBar::new(100.0, 150.0);
        assert_eq!(full_bar.fraction(), 1.0); // clamped to 1.0
    }

    #[test]
    fn test_ui_layout_resolver_emits_sorted_draw_commands() {
        let mut world = World::new();

        // 1. Text at TopLeft
        world.spawn((
            UiElement {
                anchor: UiAnchor::TopLeft,
                offset: [20.0, 20.0],
                size: [150.0, 30.0],
                visible: true,
                z_index: 10,
            },
            UiText::new("Score: 500", 18.0),
        ));

        // 2. Health bar at BottomLeft
        world.spawn((
            UiElement {
                anchor: UiAnchor::BottomLeft,
                offset: [100.0, -30.0],
                size: [200.0, 20.0],
                visible: true,
                z_index: 0,
            },
            UiProgressBar::new(100.0, 75.0),
        ));

        let commands = UiLayoutResolver::resolve_draw_commands(
            &world,
            1920.0,
            1080.0,
            Some([100.0, 100.0]),
            false,
        );

        assert!(!commands.is_empty());
        // Lowest z_index (0, progress bar background) must come first
        assert_eq!(commands[0].z_index(), 0);
        // Text with z_index 10 must come last
        assert_eq!(commands.last().unwrap().z_index(), 10);
    }
}