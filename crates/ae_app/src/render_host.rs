// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Shared 3D/2D rendering state, scene extraction, and camera/spatial coordination host.
//!
//! Eliminates duplicated `extract_render_scene`, light synchronization, 2D overlay assembly,
//! and resize logic between the editor host and standalone runtime executable.

#[cfg(feature = "2d")]
use ae_2d::{ActiveDimensionMode, Sprite2DOverlay, Sprite2DSystem, configure_2d_camera};
use ae_camera::Camera;
use ae_core::Resources;
use ae_core::events::DynamicEventBus;
use ae_core::modules::EngineModule;
use ae_ecs::{Entity, World};
use ae_light::{Light, LightUniform};
use ae_plugin_api::{Position, VisibleEntities};
use ae_renderer::asset::AssetManager;
use ae_renderer::render::RenderState;
use ae_renderer::render::types::RenderScene;
use ae_transform::{SpatialGrid, update_hierarchy_transforms};
use std::collections::{HashMap, HashSet};

/// Cohesive GPU device/pipeline, asset storage, camera, spatial culling grid, and 2D sprite host.
pub struct RenderHost {
    /// Core WGPU surface, swapchain, and 3D PBR/post-process renderer.
    pub render_state: RenderState,
    /// Loaded 3D models, textures, shaders, and path-deduplicated caches.
    pub asset_manager: AssetManager,
    /// Active viewport or gameplay camera.
    pub camera: Camera,
    /// Lazy-synced spatial grid accelerating 3D frustum/distance culling.
    pub spatial_grid: SpatialGrid,
    /// Dimension isolation mode (`Mode2D` vs `Mode3D`).
    #[cfg(feature = "2d")]
    pub dimension_mode: ActiveDimensionMode,
    /// Optional 2D instanced sprite renderer pair allocated exclusively in `Mode2D`.
    #[cfg(feature = "2d")]
    pub sprite_2d: Option<Sprite2DSystem>,
}

/// Prepared per-frame scene geometry ready for overlay preparation and [`RenderState::render`].
pub struct PreparedRenderScene {
    /// Extracted 3D primitives, models, transparent sorted objects, and lighting uniforms.
    pub scene: RenderScene,
    /// Whether `dimension_mode` is currently 2D.
    pub is_2d: bool,
    /// Whether the active render module (`Render2D` or `Render`) is enabled.
    pub render_enabled: bool,
}

/// Prepares the 2D instanced sprite batch overlay when `render_enabled && is_2d`.
///
/// Taking `sprite_2d` separately keeps the mutable borrow disjoint from `render_state`, `camera`,
/// and `asset_manager` during [`RenderState::render`].
#[cfg(feature = "2d")]
#[must_use]
pub fn prepare_2d_overlay<'a>(
    sprite_2d: Option<&'a mut Sprite2DSystem>,
    render_state: &RenderState,
    world: &World,
    camera: &Camera,
    asset_manager: &'a AssetManager,
    render_enabled: bool,
    is_2d: bool,
) -> Option<Sprite2DOverlay<'a>> {
    (render_enabled && is_2d).then_some(()).and_then(|()| {
        sprite_2d.map(|sprite_sys| {
            sprite_sys.prepare_overlay(
                &render_state.device,
                &render_state.queue,
                world,
                camera,
                &asset_manager.textures,
            )
        })
    })
}

impl RenderHost {
    /// Assembles a [`RenderHost`] from pre-initialized GPU and asset components, automatically
    /// provisioning [`Sprite2DSystem`] and configuring the 2D orthographic camera in 2D mode.
    #[cfg(feature = "2d")]
    #[must_use]
    pub fn new(
        render_state: RenderState,
        asset_manager: AssetManager,
        mut camera: Camera,
        cell_size: f32,
        dimension_mode: ActiveDimensionMode,
    ) -> Self {
        let sprite_2d = (dimension_mode == ActiveDimensionMode::Mode2D).then(|| {
            configure_2d_camera(&mut camera);
            Sprite2DSystem::new(
                &render_state.device,
                &render_state.queue,
                render_state.config.format,
            )
        });
        Self {
            render_state,
            asset_manager,
            camera,
            spatial_grid: SpatialGrid::new(cell_size.max(128.0)),
            dimension_mode,
            sprite_2d,
        }
    }

    /// Assembles a 3D-only [`RenderHost`] when the `2d` feature is disabled.
    #[cfg(not(feature = "2d"))]
    #[must_use]
    pub fn new(
        render_state: RenderState,
        asset_manager: AssetManager,
        camera: Camera,
        cell_size: f32,
    ) -> Self {
        Self {
            render_state,
            asset_manager,
            camera,
            spatial_grid: SpatialGrid::new(cell_size.max(128.0)),
        }
    }

    /// Returns `true` if `dimension_mode` is 2D.
    #[inline]
    #[must_use]
    pub fn is_2d(&self) -> bool {
        #[cfg(feature = "2d")]
        {
            self.dimension_mode.is_2d()
        }
        #[cfg(not(feature = "2d"))]
        {
            false
        }
    }

    /// Queries [`DynamicEventBus`] for whether the active dimension's render module is enabled.
    #[inline]
    #[must_use]
    pub fn is_render_enabled(&self, event_bus: &DynamicEventBus) -> bool {
        if self.is_2d() {
            event_bus.is_module_enabled(EngineModule::Render2D)
        } else {
            event_bus.is_module_enabled(EngineModule::Render)
        }
    }

    /// Rebuilds 2D sprite pipelines when swapchain/MSAA settings change.
    pub fn rebuild_sprite_2d(&mut self) {
        #[cfg(feature = "2d")]
        if let Some(ref mut sprite_sys) = self.sprite_2d {
            sprite_sys.rebuild(
                &self.render_state.device,
                &self.render_state.queue,
                self.render_state.config.format,
            );
        }
    }

    /// Updates swapchain size and perspective aspect ratio for physical `(width, height)`.
    pub fn resize_dimensions(&mut self, width: u32, height: u32) {
        self.render_state.resize_dimensions(width, height);
        if width > 0 && height > 0 {
            self.camera.aspect = width as f32 / height as f32;
        }
    }

    /// Synchronizes camera aspect ratio with [`RenderState::last_viewport_rect`] after rendering.
    pub fn sync_camera_aspect_to_viewport(&mut self) {
        let vp_rect = self.render_state.last_viewport_rect;
        let vp_w = vp_rect.max_x - vp_rect.min_x;
        let vp_h = vp_rect.max_y - vp_rect.min_y;
        if vp_w > 0.0 && vp_h > 0.0 {
            self.camera.aspect = vp_w / vp_h;
        }
    }

    /// Syncs spatial grid, hierarchy transforms, light positions, extracts [`RenderScene`],
    /// and updates [`VisibleEntities`] without retaining any long-lived borrow on `self`.
    pub fn prepare_scene(
        &mut self,
        world: &mut World,
        resources: &mut Resources,
        event_bus: &DynamicEventBus,
        selected_entities: &HashSet<Entity>,
        primary_selected: Option<Entity>,
    ) -> PreparedRenderScene {
        let is_2d = self.is_2d();
        let render_enabled = self.is_render_enabled(event_bus);

        if render_enabled && !is_2d {
            self.spatial_grid.sync(world);
        }
        update_hierarchy_transforms(world);

        if render_enabled && !is_2d {
            for (pos, light) in world.query_mut::<(&Position, &mut Light)>() {
                light.position = [pos.x, pos.y, pos.z];
            }
        }

        let mut scene = if render_enabled && !is_2d {
            RenderScene::extract(
                world,
                &self.camera,
                &self.asset_manager,
                selected_entities,
                primary_selected,
                &self.spatial_grid,
            )
        } else {
            empty_render_scene()
        };

        if render_enabled {
            resources.insert(VisibleEntities {
                entities: std::mem::take(&mut scene.visible_entities),
            });
        }

        PreparedRenderScene {
            scene,
            is_2d,
            render_enabled,
        }
    }
}

/// Constructs an empty [`RenderScene`] for disabled-render or pure 2D frames.
#[must_use]
pub fn empty_render_scene() -> RenderScene {
    RenderScene {
        light_uniform: LightUniform::default(),
        triangle_instances: Vec::new(),
        cube_instances: Vec::new(),
        sphere_instances: Vec::new(),
        cylinder_instances: Vec::new(),
        capsule_instances: Vec::new(),
        torus_instances: Vec::new(),
        transparent_objs: Vec::new(),
        model_instance_data: HashMap::new(),
        selected_primitive_instances: Vec::new(),
        selected_model_instances: Vec::new(),
        visible_entities: Vec::new(),
        culled_entities_count: 0,
    }
}