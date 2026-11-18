// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Iris UI WGPU Backend (`irisui-wgpu`)
//!
//! GPU SDF rendering pipeline for Iris UI with sub-pixel antialiasing,
//! rounded rectangles, inner/outer borders, and gaussian drop shadows.
//!
//! Adheres strictly to a zero-unsafe policy (`#![forbid(unsafe_code)]`).

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod command;
pub mod external_texture_pipeline;
pub mod external_textures;
pub mod layer_cache;
pub mod quad;
pub mod renderer;
pub mod scissor;
pub mod surface;
pub mod texture_array_atlas;
pub mod texture_pipeline;
pub mod tree_compiler;
mod tree_compiler_patch;

pub use command::{BoundarySlice, DrawCommand, DrawCommandList};
pub use external_texture_pipeline::{ExternalTexturePipeline, ExternalTextureQuadInstance};
pub use external_textures::ExternalTextures;
pub use layer_cache::{BoundaryLayerCache, InstanceBufferTarget};
pub use quad::QuadInstance;
pub use renderer::{IrisRenderer, IrisTreePrepareParams, compile_tree_for_frame};
pub use scissor::ScissorStack;
pub use surface::{
    IrisSurfaceFrameFlags, IrisSurfaceHost, IrisSurfaceRenderParams, SurfaceRefreshMode,
};
pub use texture_array_atlas::{TextureArrayAtlas, compute_mip_level_count};
pub use texture_pipeline::{TextureQuadInstance, TextureQuadPipeline};
pub use tree_compiler::{
    CustomDrawCallback, TreeCompilerOptions, compile_tree_draw_commands,
    compile_tree_draw_commands_into,
};
pub use tree_compiler_patch::{
    TreeCompilerScratch, compile_tree_draw_commands_with_scratch,
    patch_dirty_boundaries_with_scratch,
};

/// WGPU rendering backend prelude re-exporting renderer, quad instances, and draw commands.
///
/// Designed to be imported via wildcard `use irisui_wgpu::prelude::*;` or through the `irisui` facade.
pub mod prelude {
    pub use crate::{
        BoundarySlice, CustomDrawCallback, DrawCommand, DrawCommandList, ExternalTexturePipeline,
        ExternalTextureQuadInstance, ExternalTextures, InstanceBufferTarget, IrisRenderer,
        IrisSurfaceFrameFlags, IrisSurfaceHost, IrisSurfaceRenderParams, IrisTreePrepareParams,
        QuadInstance, ScissorStack, SurfaceRefreshMode, TextureArrayAtlas, TextureQuadInstance,
        TextureQuadPipeline, TreeCompilerOptions, TreeCompilerScratch, compile_tree_draw_commands,
        compile_tree_draw_commands_into, compile_tree_draw_commands_with_scratch,
        compile_tree_for_frame, patch_dirty_boundaries_with_scratch,
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use irisui_core::{Color, Rect, Style};

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

    #[test]
    fn test_external_texture_quad_command_stream() {
        use irisui_core::node::ExternalTextureId;

        let mut list = DrawCommandList::new();
        assert!(list.external_texture_quads.is_empty());

        let ext_id = ExternalTextureId(42);
        let eq =
            ExternalTextureQuadInstance::new(Rect::new(0.0, 0.0, 800.0, 600.0), Color::WHITE, None);
        list.push_external_texture_quad(ext_id, eq);

        assert_eq!(list.external_texture_quads.len(), 1);
        assert_eq!(list.commands.len(), 1);
        assert_eq!(
            list.commands[0],
            DrawCommand::DrawExternalTexture {
                id: ext_id,
                instance_index: 0,
            }
        );

        list.clear();
        assert!(list.commands.is_empty());
        assert!(list.external_texture_quads.is_empty());
    }
}