// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Dormant Offscreen UI Surface Cache & Single-Quad Blit Pipeline
//!
//! Caches the fully rasterized UI overlay into an offscreen [`wgpu::Texture`] when the surface
//! enters [`SurfaceRefreshMode::Dormant`]. Subsequent idle frames skip SDF quad uploads, scissor
//! evaluation, and Glyphon text rendering completely, compositing the retained UI layer back onto
//! the active target surface view using a single premultiplied-alpha fullscreen quad.

use irisui_core::SurfaceRefreshMode;

/// Execution strategy selected by [`SurfaceCacheState`] for the current UI render frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SurfaceCacheFrameAction {
    /// Render directly onto the caller's target view and mark any offscreen cache stale.
    DirectRender,
    /// Rasterize the UI pass once into the offscreen cache texture, then blit 1 quad to target view.
    PopulateAndBlit,
    /// Skip SDF quad compilation/draw and Glyphon text passes; blit only the cached 1-quad texture.
    BlitCachedOnly,
}

/// Pure-state validity and telemetry tracker governing [`StaticUiSurfaceCache`] invalidation,
/// HiDPI zoom transitions, resize reallocations, and hit/miss counters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SurfaceCacheState {
    cached_size: (u32, u32),
    cached_zoom_bits: u32,
    has_target: bool,
    valid: bool,
    enabled: bool,
    offscreen_render_count: u64,
    cache_hit_count: u64,
    last_frame_hit: bool,
}

impl Default for SurfaceCacheState {
    fn default() -> Self {
        Self::new()
    }
}

impl SurfaceCacheState {
    /// Constructs a new invalid, enabled [`SurfaceCacheState`].
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            cached_size: (0, 0),
            cached_zoom_bits: 1.0_f32.to_bits(),
            has_target: false,
            valid: false,
            enabled: true,
            offscreen_render_count: 0,
            cache_hit_count: 0,
            last_frame_hit: false,
        }
    }

    /// Enables or disables offscreen dormant caching, invalidating any cached texture when disabled.
    #[inline]
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            self.valid = false;
            self.last_frame_hit = false;
        }
    }

    /// Returns whether dormant offscreen surface caching is currently enabled.
    #[inline]
    #[must_use]
    pub const fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Returns whether the offscreen cache currently contains valid rasterized pixels.
    #[inline]
    #[must_use]
    pub const fn is_valid(&self) -> bool {
        self.valid
    }

    /// Returns whether the most recently rendered frame satisfied rendering solely via the 1-quad blit.
    #[inline]
    #[must_use]
    pub const fn did_last_frame_hit(&self) -> bool {
        self.last_frame_hit
    }

    /// Returns total number of offscreen rasterization passes executed to populate the cache.
    #[inline]
    #[must_use]
    pub const fn offscreen_render_count(&self) -> u64 {
        self.offscreen_render_count
    }

    /// Returns total number of dormant frames satisfied by the 1-quad cache blit fast path.
    #[inline]
    #[must_use]
    pub const fn cache_hit_count(&self) -> u64 {
        self.cache_hit_count
    }

    /// Returns the clamped physical `(width, height)` currently allocated for the offscreen texture.
    #[inline]
    #[must_use]
    pub const fn cached_size(&self) -> (u32, u32) {
        self.cached_size
    }

    /// Clamps physical dimensions to at least `1x1` pixel so minimized surfaces cannot panic WGPU.
    #[inline]
    #[must_use]
    pub fn clamp_size(physical_size: (u32, u32)) -> (u32, u32) {
        (physical_size.0.max(1), physical_size.1.max(1))
    }

    /// Marks the cached texture stale so the next dormant frame re-rasterizes offscreen.
    #[inline]
    pub fn invalidate(&mut self) {
        self.valid = false;
    }

    /// Returns `true` when the offscreen texture matches `physical_size` and `zoom_factor` and is clean.
    #[inline]
    #[must_use]
    pub fn is_valid_for(&self, physical_size: (u32, u32), zoom_factor: f32) -> bool {
        let clamped = Self::clamp_size(physical_size);
        self.enabled
            && self.valid
            && self.has_target
            && self.cached_size == clamped
            && self.cached_zoom_bits == zoom_factor.to_bits()
    }

    /// Evaluates the optimal [`SurfaceCacheFrameAction`] given frame mode, static dirty flags, and dimensions.
    ///
    /// Both [`SurfaceRefreshMode::Dormant`] and volatile-only [`SurfaceRefreshMode::InPlaceRefresh`]
    /// frames reuse the cached static UI surface when static quads/text are unchanged.
    #[inline]
    #[must_use]
    pub fn resolve_frame_action(
        &self,
        mode: SurfaceRefreshMode,
        any_static_dirty: bool,
        physical_size: (u32, u32),
        zoom_factor: f32,
    ) -> SurfaceCacheFrameAction {
        if !self.enabled || matches!(mode, SurfaceRefreshMode::StructuralRebuild { .. }) {
            return SurfaceCacheFrameAction::DirectRender;
        }
        if !any_static_dirty && self.is_valid_for(physical_size, zoom_factor) {
            SurfaceCacheFrameAction::BlitCachedOnly
        } else if mode == SurfaceRefreshMode::Dormant || !any_static_dirty {
            SurfaceCacheFrameAction::PopulateAndBlit
        } else {
            SurfaceCacheFrameAction::DirectRender
        }
    }

    /// Returns `true` if the backing GPU offscreen texture needs allocation or resize.
    #[inline]
    #[must_use]
    pub fn needs_target_reallocation(&self, physical_size: (u32, u32)) -> bool {
        let clamped = Self::clamp_size(physical_size);
        !self.has_target || self.cached_size != clamped
    }

    /// Records allocation of a new offscreen backing texture and clears validity until populated.
    #[inline]
    pub fn record_target_allocated(&mut self, physical_size: (u32, u32)) {
        self.cached_size = Self::clamp_size(physical_size);
        self.has_target = true;
        self.valid = false;
    }

    /// Records completion of an offscreen UI rasterization pass at `(physical_size, zoom_factor)`.
    #[inline]
    pub fn record_populated(&mut self, physical_size: (u32, u32), zoom_factor: f32) {
        self.cached_size = Self::clamp_size(physical_size);
        self.cached_zoom_bits = zoom_factor.to_bits();
        self.has_target = true;
        self.valid = true;
        self.last_frame_hit = false;
        self.offscreen_render_count += 1;
    }

    /// Records a dormant 1-quad cache hit frame.
    #[inline]
    pub fn record_cache_hit(&mut self) {
        self.last_frame_hit = true;
        self.cache_hit_count += 1;
    }

    /// Records a direct swapchain render frame, invalidating stale cached pixels.
    #[inline]
    pub fn record_direct_frame(&mut self) {
        self.valid = false;
        self.last_frame_hit = false;
    }
}

struct CachedSurfaceTarget {
    _texture: wgpu::Texture,
    view: wgpu::TextureView,
    bind_group: wgpu::BindGroup,
}

/// GPU offscreen render target and single-quad premultiplied blit pipeline for dormant UI surfaces.
///
/// Renders SDF quads, textures, and Glyphon typography into a retained `target_format` offscreen
/// [`wgpu::Texture`] once upon entering [`SurfaceRefreshMode::Dormant`], then composites subsequent
/// dormant frames onto the window surface via a single fullscreen textured quad.
pub struct StaticUiSurfaceCache {
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    target_format: wgpu::TextureFormat,
    offscreen: Option<CachedSurfaceTarget>,
    state: SurfaceCacheState,
}

impl StaticUiSurfaceCache {
    /// Creates a new [`StaticUiSurfaceCache`] compiled for `target_format` with premultiplied alpha blending.
    #[must_use]
    pub fn new(device: &wgpu::Device, target_format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Iris UI Surface Blit Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/surface_blit.wgsl").into()),
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Iris UI Surface Blit Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Iris UI Surface Blit Nearest Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Iris UI Surface Blit Pipeline Layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Iris UI Surface Blit Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: target_format,
                    blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        Self {
            pipeline,
            bind_group_layout,
            sampler,
            target_format,
            offscreen: None,
            state: SurfaceCacheState::new(),
        }
    }

    /// Returns a shared reference to the underlying [`SurfaceCacheState`].
    #[inline]
    #[must_use]
    pub const fn state(&self) -> &SurfaceCacheState {
        &self.state
    }

    /// Returns a mutable reference to the underlying [`SurfaceCacheState`].
    #[inline]
    pub fn state_mut(&mut self) -> &mut SurfaceCacheState {
        &mut self.state
    }

    /// Allocates or resizes the offscreen backing texture when `physical_size` changes, returning its [`wgpu::TextureView`].
    pub fn ensure_offscreen_view(
        &mut self,
        device: &wgpu::Device,
        physical_size: (u32, u32),
    ) -> &wgpu::TextureView {
        let clamped = SurfaceCacheState::clamp_size(physical_size);
        if self.offscreen.is_none() || self.state.needs_target_reallocation(clamped) {
            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Iris UI Static Offscreen Cache Texture"),
                size: wgpu::Extent3d {
                    width: clamped.0,
                    height: clamped.1,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: self.target_format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });
            let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Iris UI Static Offscreen Cache Bind Group"),
                layout: &self.bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&self.sampler),
                    },
                ],
            });
            self.offscreen = Some(CachedSurfaceTarget {
                _texture: texture,
                view,
                bind_group,
            });
            self.state.record_target_allocated(clamped);
        }

        // Invariant: `self.offscreen` is populated by the guard immediately above.
        &self
            .offscreen
            .as_ref()
            .unwrap_or_else(|| unreachable!("offscreen target initialized immediately above"))
            .view
    }

    /// Records the fullscreen premultiplied-alpha blit draw call inside an already open [`wgpu::RenderPass`].
    ///
    /// No-ops safely if the offscreen target has not yet been allocated.
    pub fn draw_blit_in_pass<'rp>(&'rp self, render_pass: &mut wgpu::RenderPass<'rp>) {
        let Some(target) = &self.offscreen else {
            return;
        };
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &target.bind_group, &[]);
        render_pass.draw(0..6, 0..1);
    }

    /// Encodes a single fullscreen premultiplied-alpha textured quad pass blitting the cached UI onto `target_view`.
    ///
    /// No-ops safely if the offscreen target has not yet been allocated.
    pub fn blit_to_view(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        target_view: &wgpu::TextureView,
    ) {
        if self.offscreen.is_none() {
            return;
        }

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Iris UI Surface Cache Blit Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });

        self.draw_blit_in_pass(&mut render_pass);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_surface_cache_lifecycle_dormant_populate_then_one_quad_hits() {
        let mut state = SurfaceCacheState::new();
        let size = (1920, 1080);

        // 1. Interactive frame renders directly without populating offscreen cache
        assert_eq!(
            state.resolve_frame_action(
                SurfaceRefreshMode::StructuralRebuild {
                    is_full_shell: true,
                },
                true,
                size,
                1.0,
            ),
            SurfaceCacheFrameAction::DirectRender
        );
        state.record_direct_frame();
        assert!(!state.did_last_frame_hit());
        assert_eq!(state.offscreen_render_count(), 0);
        assert_eq!(state.cache_hit_count(), 0);

        // 2. First Dormant frame rasterizes offscreen + blits
        assert_eq!(
            state.resolve_frame_action(SurfaceRefreshMode::Dormant, false, size, 1.0),
            SurfaceCacheFrameAction::PopulateAndBlit
        );
        state.record_target_allocated(size);
        state.record_populated(size, 1.0);
        assert!(state.is_valid_for(size, 1.0));
        assert!(!state.did_last_frame_hit());
        assert_eq!(state.offscreen_render_count(), 1);
        assert_eq!(state.cache_hit_count(), 0);

        // 3. Subsequent Dormant frames hit the 1-quad cache blit fast path
        for expected_hits in 1..=5 {
            assert_eq!(
                state.resolve_frame_action(SurfaceRefreshMode::Dormant, false, size, 1.0),
                SurfaceCacheFrameAction::BlitCachedOnly
            );
            state.record_cache_hit();
            assert!(state.did_last_frame_hit());
            assert_eq!(state.cache_hit_count(), expected_hits);
            assert_eq!(state.offscreen_render_count(), 1);
        }
    }

    #[test]
    fn test_surface_cache_invalidates_on_resize_zoom_dirty_and_toggle() {
        let mut state = SurfaceCacheState::new();
        let size = (1280, 720);
        state.record_target_allocated(size);
        state.record_populated(size, 1.0);

        // Unchanged -> hit
        assert_eq!(
            state.resolve_frame_action(SurfaceRefreshMode::Dormant, false, size, 1.0),
            SurfaceCacheFrameAction::BlitCachedOnly
        );

        // Window resize invalidates target reallocation & forces repopulation
        assert!(state.needs_target_reallocation((1600, 900)));
        assert_eq!(
            state.resolve_frame_action(SurfaceRefreshMode::Dormant, false, (1600, 900), 1.0),
            SurfaceCacheFrameAction::PopulateAndBlit
        );

        // HiDPI zoom factor delta invalidates without target reallocation
        assert!(!state.needs_target_reallocation(size));
        assert_eq!(
            state.resolve_frame_action(SurfaceRefreshMode::Dormant, false, size, 1.25),
            SurfaceCacheFrameAction::PopulateAndBlit
        );

        // Internal dirty flags force repopulation even in Dormant mode
        assert_eq!(
            state.resolve_frame_action(SurfaceRefreshMode::Dormant, true, size, 1.0),
            SurfaceCacheFrameAction::PopulateAndBlit
        );

        // InPlaceRefresh with dirty static quads bypasses cache and invalidates
        assert_eq!(
            state.resolve_frame_action(SurfaceRefreshMode::InPlaceRefresh, true, size, 1.0),
            SurfaceCacheFrameAction::DirectRender
        );

        // Volatile-only InPlaceRefresh preserves populated static cache and hits BlitCachedOnly
        assert_eq!(
            state.resolve_frame_action(SurfaceRefreshMode::InPlaceRefresh, false, size, 1.0),
            SurfaceCacheFrameAction::BlitCachedOnly
        );

        // Explicit invalidate drops validity
        state.invalidate();
        assert!(!state.is_valid());
        assert_eq!(
            state.resolve_frame_action(SurfaceRefreshMode::Dormant, false, size, 1.0),
            SurfaceCacheFrameAction::PopulateAndBlit
        );

        // Disabling forces DirectRender and resets hit tracking
        state.record_populated(size, 1.0);
        state.record_cache_hit();
        state.set_enabled(false);
        assert!(!state.is_enabled());
        assert!(!state.did_last_frame_hit());
        assert_eq!(
            state.resolve_frame_action(SurfaceRefreshMode::Dormant, false, size, 1.0),
            SurfaceCacheFrameAction::DirectRender
        );
    }

    #[test]
    fn test_surface_cache_clamps_minimized_dimensions() {
        let mut state = SurfaceCacheState::new();
        assert_eq!(SurfaceCacheState::clamp_size((0, 0)), (1, 1));
        state.record_target_allocated((0, 0));
        state.record_populated((0, 0), 1.0);
        assert_eq!(state.cached_size(), (1, 1));
        assert!(state.is_valid_for((0, 0), 1.0));
        assert!(!state.needs_target_reallocation((1, 1)));
    }
}