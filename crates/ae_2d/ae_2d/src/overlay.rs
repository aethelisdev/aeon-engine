// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Engine-facing 2D sprite rendering system, world collection, and [`OverlayRenderer`] bridge.
//!
//! Serves as the single source of truth shared between the editor host and standalone runtime
//! player for 2D orthographic camera setup, ECS sprite extraction, batch upload, and overlay drawing.

use ae_2d_core::components::{SpriteRenderer, SpriteSortKey};
use ae_2d_render::{Sprite2DPipeline, SpriteBatcher, SpriteInstance};
use ae_camera::{Camera, ProjectionMode};
use ae_ecs::World;
use ae_plugin_api::{Hidden, Position, Scale};
use ae_renderer::asset::{AssetHandle, AssetStorage};
use ae_renderer::render::{OverlayRenderer, TextureAsset};
use ae_transform::compose_2d_model_matrix;

/// Owns the GPU 2D sprite pipeline and dynamic instanced batcher pair.
pub struct Sprite2DSystem {
    /// Compiled WGPU pipeline for textured 2D quad instancing.
    pub pipeline: Sprite2DPipeline,
    /// Dynamic vertex/instance batcher partitioned by texture handle.
    pub batcher: SpriteBatcher,
}

impl Sprite2DSystem {
    /// Allocates a new 2D sprite pipeline and batcher targeting `format`.
    #[must_use]
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, format: wgpu::TextureFormat) -> Self {
        let pipeline = Sprite2DPipeline::new(device, format, None, 1);
        let batcher = SpriteBatcher::new(device, queue, &pipeline);
        Self { pipeline, batcher }
    }

    /// Rebuilds the 2D pipeline and resets its batcher after swapchain/settings changes.
    pub fn rebuild(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    ) {
        *self = Self::new(device, queue, format);
    }

    /// Collects visible ECS [`SpriteRenderer`] entities, uploads GPU buffers, and builds a draw overlay.
    pub fn prepare_overlay<'a>(
        &'a mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        world: &World,
        camera: &Camera,
        textures: &'a AssetStorage<TextureAsset>,
    ) -> Sprite2DOverlay<'a> {
        self.batcher.begin_frame();
        let view_proj = camera.build_view_projection_matrix();
        self.batcher.update_camera(queue, &view_proj);
        collect_world_sprites(&mut self.batcher, world);
        self.batcher.finish_and_upload(device, queue);
        Sprite2DOverlay {
            pipeline: &self.pipeline,
            batcher: &self.batcher,
            textures,
        }
    }
}

/// Configures `camera` with standard 2D orthographic framing along the -Z view axis.
pub fn configure_2d_camera(camera: &mut Camera) {
    camera.position = cgmath::Point3::new(0.0, 0.0, 10.0);
    camera.target = cgmath::Point3::new(0.0, 0.0, 0.0);
    camera.yaw = cgmath::Deg(-90.0).into();
    camera.pitch = cgmath::Deg(0.0).into();
    camera.mode = ProjectionMode::Orthographic;
    camera.ortho_scale = 10.0;
}

/// Builds the sort key and GPU instance payload for a single visible [`SpriteRenderer`].
#[inline]
#[must_use]
fn build_sprite_draw_item(
    pos: Position,
    scale: Scale,
    sprite: &SpriteRenderer,
) -> (SpriteSortKey, SpriteInstance) {
    let tex_id = sprite
        .texture
        .map(|t| {
            use slotmap::Key;
            t.data().as_ffi() as u32
        })
        .unwrap_or(0);
    let key = SpriteSortKey::from_layer_order_texture(
        sprite.sorting_layer,
        sprite.order_in_layer,
        tex_id,
    );
    let model = compose_2d_model_matrix(pos, scale.x, scale.y);
    let instance = SpriteInstance {
        model_matrix: model.into(),
        uv_rect: sprite.uv_rect,
        tint: sprite.tint,
    };
    (key, instance)
}

/// Extracts all non-hidden [`SpriteRenderer`] entities from `world` into `batcher`.
///
/// Assumes `batcher.begin_frame()` has already been called by the caller.
/// Returns the number of visible sprite instances pushed.
pub fn collect_world_sprites(batcher: &mut SpriteBatcher, world: &World) -> usize {
    let mut pushed = 0usize;
    for (pos, scale, sprite, hidden_opt) in world
        .query::<(&Position, &Scale, &SpriteRenderer, Option<&Hidden>)>()
        .iter()
    {
        if hidden_opt.is_some() {
            continue;
        }
        let (key, instance) = build_sprite_draw_item(*pos, *scale, sprite);
        batcher.push_instance(key, instance);
        pushed += 1;
    }
    pushed
}

/// Forward-pass overlay adapter rendering sorted 2D sprite batches into [`ae_renderer`].
pub struct Sprite2DOverlay<'a> {
    pipeline: &'a Sprite2DPipeline,
    batcher: &'a SpriteBatcher,
    textures: &'a AssetStorage<TextureAsset>,
}

impl OverlayRenderer for Sprite2DOverlay<'_> {
    fn draw_overlay<'b>(&'b self, _queue: &wgpu::Queue, pass: &mut wgpu::RenderPass<'b>) {
        self.batcher.render(pass, self.pipeline, |id| {
            let handle = AssetHandle::from(slotmap::KeyData::from_ffi(id as u64));
            self.textures.get(handle).map(|t| &t.bind_group)
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_configure_2d_camera_sets_orthographic_axis_aligned_view() {
        let mut cam = Camera {
            position: cgmath::Point3::new(3.0, 4.0, 5.0),
            yaw: cgmath::Rad(0.0),
            pitch: cgmath::Rad(-0.5),
            aspect: 16.0 / 9.0,
            fovy: 45.0,
            znear: 0.1,
            zfar: 1000.0,
            mode: ProjectionMode::Perspective,
            ortho_scale: 1.0,
            target: cgmath::Point3::new(1.0, 2.0, 3.0),
        };
        configure_2d_camera(&mut cam);

        assert_eq!(cam.mode, ProjectionMode::Orthographic);
        assert!((cam.position.z - 10.0).abs() < f32::EPSILON);
        assert!((cam.ortho_scale - 10.0).abs() < f32::EPSILON);
        assert!(cam.pitch.0.abs() < 1e-6);
    }

    #[test]
    fn test_build_sprite_draw_item_scales_and_orders_correctly() {
        let sprite = SpriteRenderer {
            sorting_layer: -2,
            order_in_layer: 5,
            tint: [0.25, 0.5, 0.75, 1.0],
            uv_rect: [0.0, 0.0, 0.5, 0.5],
            ..Default::default()
        };
        let (key, instance) = build_sprite_draw_item(
            Position::new(3.0, -4.0, 1.0),
            Scale::new(2.0, 3.0, 1.0),
            &sprite,
        );
        assert_eq!(key, SpriteSortKey::from_layer_order_texture(-2, 5, 0));
        assert_eq!(instance.tint, [0.25, 0.5, 0.75, 1.0]);
        assert!((instance.model_matrix[3][0] - 3.0).abs() < 1e-6);
        assert!((instance.model_matrix[3][1] + 4.0).abs() < 1e-6);
        assert!((instance.model_matrix[0][0] - 2.0).abs() < 1e-6);
        assert!((instance.model_matrix[1][1] - 3.0).abs() < 1e-6);
    }
}