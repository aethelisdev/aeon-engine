// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Iris UI WGPU Backend (`iris-wgpu`)
//!
//! GPU SDF rendering pipeline for Iris UI with sub-pixel antialiasing,
//! rounded rectangles, inner/outer borders, and gaussian drop shadows.
//!
//! Adheres strictly to a zero-unsafe policy (`#![forbid(unsafe_code)]`).

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod command;
pub mod quad;
pub mod renderer;
pub mod texture_pipeline;

pub use command::{DrawCommand, DrawCommandList};
pub use quad::QuadInstance;
pub use renderer::IrisRenderer;
pub use texture_pipeline::{TextureQuadInstance, TextureQuadPipeline};

#[cfg(test)]
mod tests {
    use super::*;
    use iris_core::{Color, Rect, Style};

    #[test]
    fn test_quad_instance_generation() {
        let rect = Rect::new(10.0, 20.0, 200.0, 100.0);
        let style = Style::new()
            .background(Color::RED)
            .border(2.0, Color::WHITE)
            .border_radius(12.0);

        let quad = QuadInstance::from_style(rect, &style, None);

        assert_eq!(quad.rect, [10.0, 20.0, 200.0, 100.0]);
        assert_eq!(quad.color, [1.0, 0.0, 0.0, 1.0]);
        assert_eq!(quad.border_color, [1.0, 1.0, 1.0, 1.0]);
        assert_eq!(quad.border_width, [2.0, 2.0, 2.0, 2.0]);
        assert_eq!(quad.corner_radii, [12.0, 12.0, 12.0, 12.0]);
    }

    #[test]
    fn test_texture_quad_command_stream() {
        let mut list = DrawCommandList::new();
        assert!(list.commands.is_empty());
        assert!(list.texture_quads.is_empty());

        let tq = TextureQuadInstance {
            rect: [10.0, 10.0, 24.0, 24.0],
            uv_rect: [0.0, 0.0, 0.25, 1.0],
            tint: [1.0, 1.0, 1.0, 1.0],
            clip_rect: [0.0, 0.0, 0.0, 0.0],
        };
        list.push_texture_quad(tq);

        assert_eq!(list.texture_quads.len(), 1);
        assert_eq!(list.commands.len(), 1);
        assert_eq!(
            list.commands[0],
            DrawCommand::DrawTexture { instance_index: 0 }
        );

        list.clear();
        assert!(list.commands.is_empty());
        assert!(list.texture_quads.is_empty());
    }
}