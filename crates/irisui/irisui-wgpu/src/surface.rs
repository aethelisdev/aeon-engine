// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Retained UI Surface & GPU Render Host
//!
//! Provides [`IrisSurfaceHost`], the autonomous WGPU surface coordinator that pairs a retained
//! [`UiTree`] and [`TextSystem`] with a compiled [`DrawCommandList`], [`IrisRenderer`], and
//! split [`LayeredTextRenderer`] pipeline.
//!
//! Lives inside `irisui-wgpu` with zero dependency on `irisui-workbench`, enabling any standalone
//! game HUD or desktop workbench shell to share the same incremental quad/text GPU pipeline.

use irisui_core::{ExternalTextureId, Rect, UiTree, WidgetId, WidgetNode};
pub use irisui_core::{IrisSurfaceFrameFlags, SurfaceRefreshMode};
use irisui_text::{
    CaretBlinkUpdate, LayeredTextRenderer, TextCollectionOptions, TextSection, TextSystem,
    TextVolatilityFilter,
};

use crate::command::DrawCommandList;
use crate::renderer::{CommandListRenderMode, IrisRenderer};
use crate::surface_cache::{StaticUiSurfaceCache, SurfaceCacheFrameAction};
use crate::texture_array_atlas::TextureArrayAtlas;
use crate::tree_compiler::TreeCompilerOptions;
use crate::tree_compiler_patch::{
    TreeCompilerScratch, compile_tree_draw_commands_with_scratch,
    patch_dirty_boundaries_with_scratch,
};

/// Signature for custom node canvas draw callbacks invoked during [`DrawCommandList`] compilation.
pub type CustomDrawFn<'a> =
    &'a mut dyn FnMut(&mut DrawCommandList, WidgetId, &WidgetNode, Option<Rect>);

/// Parameters passed to [`IrisSurfaceHost::render_to_view`].
///
/// Groups GPU command submission handles, viewport scaling, and host-provided text occlusion/focus
/// metadata without exposing low-level quad or text buffers to callers.
pub struct IrisSurfaceRenderParams<'a> {
    /// Active WGPU logical rendering device.
    pub device: &'a wgpu::Device,
    /// Active WGPU hardware command queue.
    pub queue: &'a wgpu::Queue,
    /// Primary frame command encoder recording UI overlay commands.
    pub encoder: &'a mut wgpu::CommandEncoder,
    /// Target framebuffer surface texture view.
    pub target_view: &'a wgpu::TextureView,
    /// Target window/surface resolution in physical pixels.
    pub physical_screen_size: (u32, u32),
    /// Active UI zoom scaling factor.
    pub zoom_factor: f32,
    /// Additional floating/menu rectangles that occlude underlying UI text sections.
    pub extra_text_occluders: &'a [Rect],
}

/// Queued CPU-side atlas upload flushed automatically on the first [`IrisSurfaceHost::render_to_view`] pass.
enum PendingAtlasUpload {
    SpriteSheet {
        start_layer: u32,
        width: u32,
        height: u32,
        rgba: Vec<u8>,
    },
    CroppedLayer {
        layer: u32,
        width: u32,
        height: u32,
        crop: [u32; 4],
        rgba: Vec<u8>,
    },
    RgbaLayer {
        layer: u32,
        rgba: Vec<u8>,
    },
}

/// Autonomous GPU render surface and command compiler for retained [`UiTree`] hosts.
///
/// Owns [`IrisRenderer`], optional [`LayeredTextRenderer`], retained [`DrawCommandList`],
/// [`TreeCompilerScratch`], optional secondary texture-array atlas slot storage, and incremental
/// text/quad GPU upload invalidation state.
pub struct IrisSurfaceHost {
    /// Core SDF quad and external texture WGPU renderer.
    pub renderer: IrisRenderer,
    /// Split static/volatile Glyphon GPU text renderer initialized lazily on first render.
    pub text_renderer: Option<LayeredTextRenderer>,
    /// Retained GPU quad/clip/texture command buffer synchronized with the active [`UiTree`].
    pub command_list: DrawCommandList,
    /// Reusable compiler scratch buffers avoiding per-frame allocations during patching/compilation.
    pub compiler_scratch: TreeCompilerScratch,
    /// Surface pixel format configured on the WGPU swapchain/render target.
    pub target_format: wgpu::TextureFormat,
    /// Optional mipmapped 2D texture array atlas bound for icon sheets and dynamic thumbnails.
    pub texture_atlas: Option<TextureArrayAtlas>,
    pending_atlas_uploads: Vec<PendingAtlasUpload>,
    /// Offscreen rasterization cache and 1-quad blit pipeline for dormant UI frames.
    pub surface_cache: StaticUiSurfaceCache,
    /// Refresh mode observed in the most recent [`Self::apply_frame_flags`] call.
    pub last_frame_mode: SurfaceRefreshMode,
    /// Whether the static typography layer requires GPU glyph shaping before rendering.
    pub needs_text_prepare: bool,
    /// Whether the high-frequency volatile typography layer requires GPU glyph shaping.
    pub needs_volatile_text_prepare: bool,
    /// Whether the entire quad instance buffer must be re-uploaded rather than patched by boundary.
    pub needs_full_quad_upload: bool,
    /// Master visibility toggle gating UI overlay rendering.
    pub is_visible: bool,
}

impl IrisSurfaceHost {
    /// Creates a new [`IrisSurfaceHost`] targeting `target_format` on `device`.
    #[must_use]
    pub fn new(device: &wgpu::Device, target_format: wgpu::TextureFormat) -> Self {
        Self {
            renderer: IrisRenderer::new(device, target_format),
            text_renderer: None,
            command_list: DrawCommandList::default(),
            compiler_scratch: TreeCompilerScratch::default(),
            target_format,
            texture_atlas: None,
            pending_atlas_uploads: Vec::new(),
            surface_cache: StaticUiSurfaceCache::new(device, target_format),
            last_frame_mode: SurfaceRefreshMode::StructuralRebuild {
                is_full_shell: true,
            },
            needs_text_prepare: true,
            needs_volatile_text_prepare: true,
            needs_full_quad_upload: true,
            is_visible: true,
        }
    }

    /// Marks the dormant offscreen UI cache stale so the next dormant frame re-rasterizes offscreen.
    #[inline]
    pub fn invalidate_surface_cache(&mut self) {
        self.surface_cache.state_mut().invalidate();
    }

    /// Enables or disables dormant offscreen caching and 1-quad blitting.
    #[inline]
    pub fn set_surface_cache_enabled(&mut self, enabled: bool) {
        self.surface_cache.state_mut().set_enabled(enabled);
    }

    /// Returns `true` if the most recent [`Self::render_to_view`] call used only the 1-quad cache blit.
    #[inline]
    #[must_use]
    pub const fn did_last_frame_hit_surface_cache(&self) -> bool {
        self.surface_cache.state().did_last_frame_hit()
    }

    /// Returns total dormant frames satisfied via single-quad cache blits since startup.
    #[inline]
    #[must_use]
    pub const fn surface_cache_hit_count(&self) -> u64 {
        self.surface_cache.state().cache_hit_count()
    }

    /// Returns total offscreen UI rasterization passes executed to populate [`Self::surface_cache`].
    #[inline]
    #[must_use]
    pub const fn surface_cache_offscreen_render_count(&self) -> u64 {
        self.surface_cache.state().offscreen_render_count()
    }

    /// Returns `true` once [`Self::texture_atlas`] has been initialized on the GPU.
    #[inline]
    #[must_use]
    pub const fn has_texture_atlas(&self) -> bool {
        self.texture_atlas.is_some()
    }

    /// Ensures the surface's [`TextureArrayAtlas`] is allocated and bound to [`IrisRenderer`],
    /// returning a shared reference to the initialized atlas.
    ///
    /// If the atlas already exists, returns it without reallocating GPU memory.
    pub fn ensure_texture_atlas(
        &mut self,
        device: &wgpu::Device,
        tile_size: u32,
        layer_count: u32,
        label: Option<&str>,
    ) -> &TextureArrayAtlas {
        if self.texture_atlas.is_none() {
            let atlas = TextureArrayAtlas::new(
                device,
                &self.renderer.texture_pipeline,
                tile_size,
                layer_count,
                label,
            );
            self.renderer
                .set_texture_bind_group(Some(atlas.bind_group().clone()));
            self.texture_atlas = Some(atlas);
            self.invalidate_surface_cache();
        }
        // Invariant: `self.texture_atlas` is guaranteed `Some` by the guard immediately above.
        self.texture_atlas
            .as_ref()
            .unwrap_or_else(|| unreachable!("texture_atlas initialized immediately above"))
    }

    /// Initializes the surface's [`TextureArrayAtlas`] once on `device` with `reserved_static_layers`
    /// preserved for application icons before dynamic allocations start.
    pub fn init_texture_atlas(
        &mut self,
        device: &wgpu::Device,
        tile_size: u32,
        layer_count: u32,
        reserved_static_layers: u32,
        label: Option<&str>,
    ) {
        let mut atlas = TextureArrayAtlas::new(
            device,
            &self.renderer.texture_pipeline,
            tile_size,
            layer_count,
            label,
        );
        atlas.set_reserved_static_layers(reserved_static_layers);
        self.renderer
            .set_texture_bind_group(Some(atlas.bind_group().clone()));
        self.texture_atlas = Some(atlas);
        self.invalidate_surface_cache();
    }

    /// Stages a packed RGBA8 sprite sheet to be uploaded automatically on the next render pass.
    pub fn stage_atlas_sprite_sheet(
        &mut self,
        start_layer: u32,
        width: u32,
        height: u32,
        rgba: Vec<u8>,
    ) {
        self.pending_atlas_uploads
            .push(PendingAtlasUpload::SpriteSheet {
                start_layer,
                width,
                height,
                rgba,
            });
    }

    /// Stages a cropped and box-resampled RGBA8 layer upload executed automatically on the next render pass.
    pub fn stage_atlas_cropped_rgba_layer(
        &mut self,
        layer: u32,
        width: u32,
        height: u32,
        crop: [u32; 4],
        rgba: Vec<u8>,
    ) {
        self.pending_atlas_uploads
            .push(PendingAtlasUpload::CroppedLayer {
                layer,
                width,
                height,
                crop,
                rgba,
            });
    }

    /// Stages a single pre-sized `tile_size x tile_size` RGBA8 layer upload for the next render pass.
    pub fn stage_atlas_rgba_layer(&mut self, layer: u32, rgba: Vec<u8>) {
        self.pending_atlas_uploads
            .push(PendingAtlasUpload::RgbaLayer { layer, rgba });
    }

    /// Allocates the next dynamic slot on the bound [`TextureArrayAtlas`], uploads `rgba`, and
    /// returns its assigned array layer index, or `None` if uninitialized/full.
    pub fn alloc_atlas_rgba_layer(&mut self, queue: &wgpu::Queue, rgba: &[u8]) -> Option<u32> {
        self.texture_atlas.as_mut()?.alloc_rgba_layer(queue, rgba)
    }

    /// Flushes any staged atlas sprite sheets or layers to GPU memory immediately using `queue`.
    pub fn flush_pending_atlas_uploads(&mut self, queue: &wgpu::Queue) {
        if self.pending_atlas_uploads.is_empty() {
            return;
        }
        let Some(atlas) = self.texture_atlas.as_ref() else {
            return;
        };
        for upload in self.pending_atlas_uploads.drain(..) {
            match upload {
                PendingAtlasUpload::SpriteSheet {
                    start_layer,
                    width,
                    height,
                    rgba,
                } => {
                    let _ =
                        atlas.upload_sprite_sheet_tiles(queue, width, height, &rgba, start_layer);
                }
                PendingAtlasUpload::CroppedLayer {
                    layer,
                    width,
                    height,
                    crop,
                    rgba,
                } => {
                    let _ =
                        atlas.upload_cropped_rgba_layer(queue, layer, width, height, crop, &rgba);
                }
                PendingAtlasUpload::RgbaLayer { layer, rgba } => {
                    let _ = atlas.upload_rgba_layer(queue, layer, &rgba);
                }
            }
        }
        self.pending_atlas_uploads.shrink_to_fit();
    }

    /// Uploads a single `tile_size x tile_size` RGBA8 image into `layer` of [`Self::texture_atlas`]
    /// along with its full 2x2 box-filtered mipmap chain.
    ///
    /// Returns `false` if no texture atlas is initialized or the layer/buffer bounds are invalid.
    pub fn upload_atlas_rgba_layer(&self, queue: &wgpu::Queue, layer: u32, rgba: &[u8]) -> bool {
        let Some(atlas) = &self.texture_atlas else {
            return false;
        };
        atlas.upload_rgba_layer(queue, layer, rgba)
    }

    /// Registers or removes an external texture view binding on [`IrisRenderer`] for
    /// `DrawExternalTexture` commands referencing `id`.
    pub fn set_external_texture(
        &mut self,
        device: &wgpu::Device,
        id: ExternalTextureId,
        view: Option<&wgpu::TextureView>,
    ) {
        let changed = match view {
            Some(view) => self.renderer.external_textures.set(
                device,
                &self.renderer.external_pipeline,
                id,
                view,
            ),
            None => self.renderer.external_textures.remove(id),
        };
        if changed {
            self.invalidate_surface_cache();
        }
    }

    /// Marks static typography preparation dirty when `update` flipped caret visibility.
    #[inline]
    pub fn apply_caret_blink_update(&mut self, update: CaretBlinkUpdate) {
        if update.needs_redraw() {
            self.needs_text_prepare = true;
            self.invalidate_surface_cache();
        }
    }

    /// Propagates hover transition flags onto GPU text and full quad re-upload flags.
    #[inline]
    pub fn apply_hover_flags(
        &mut self,
        hover_target_changed: bool,
        hover_affects_text: bool,
        hover_requires_full_upload: bool,
    ) {
        if hover_target_changed {
            if hover_affects_text {
                self.needs_text_prepare = true;
            }
            if hover_requires_full_upload {
                self.needs_full_quad_upload = true;
            }
            if hover_affects_text || hover_requires_full_upload {
                self.invalidate_surface_cache();
            }
        }
    }

    /// Applies [`IrisSurfaceFrameFlags`] against `tree` and incrementally compiles or boundary-patches
    /// [`Self::command_list`] when quad geometry mutated.
    pub fn apply_frame_flags(
        &mut self,
        tree: &mut UiTree,
        blink_caret: bool,
        hovered_tag: Option<u64>,
        flags: IrisSurfaceFrameFlags,
        custom_drawer: Option<CustomDrawFn<'_>>,
    ) {
        self.last_frame_mode = flags.mode;
        if flags.caret_blink_redraw {
            self.needs_text_prepare = true;
        }
        match flags.mode {
            SurfaceRefreshMode::StructuralRebuild { .. } => {
                if flags.needs_text_prepare {
                    self.needs_text_prepare = true;
                    self.needs_volatile_text_prepare = true;
                }
            }
            SurfaceRefreshMode::InPlaceRefresh => {
                if flags.volatile_needs_text_prepare {
                    self.needs_volatile_text_prepare = true;
                }
                if tree.take_static_text_sync_needed()
                    || (flags.needs_text_prepare && !flags.volatile_needs_text_prepare)
                {
                    self.needs_text_prepare = true;
                }
            }
            SurfaceRefreshMode::Dormant => {
                if flags.needs_text_prepare {
                    self.needs_text_prepare = true;
                }
            }
        }
        if !matches!(
            flags.mode,
            SurfaceRefreshMode::Dormant | SurfaceRefreshMode::InPlaceRefresh
        ) || flags.caret_blink_redraw
            || flags.needs_quad_reupload
            || self.needs_text_prepare
        {
            self.invalidate_surface_cache();
        }

        if flags.needs_quad_reupload {
            let (allow_in_place_patch, force_full_upload) = match flags.mode {
                SurfaceRefreshMode::StructuralRebuild { is_full_shell } => (false, is_full_shell),
                SurfaceRefreshMode::InPlaceRefresh | SurfaceRefreshMode::Dormant => (true, false),
            };
            self.recompile_or_patch_draw_commands(
                tree,
                blink_caret,
                hovered_tag,
                allow_in_place_patch,
                force_full_upload,
                custom_drawer,
            );
        }
    }

    /// Incrementally patches dirty invalidation boundaries in [`Self::command_list`] or recompiles
    /// all draw commands directly from `tree`.
    pub fn recompile_or_patch_draw_commands(
        &mut self,
        tree: &UiTree,
        blink_caret: bool,
        hovered_tag: Option<u64>,
        allow_in_place_patch: bool,
        force_full_upload: bool,
        custom_drawer: Option<CustomDrawFn<'_>>,
    ) {
        self.invalidate_surface_cache();
        let Some(root) = tree.root() else {
            self.command_list.clear();
            return;
        };

        let mut options = TreeCompilerOptions::new()
            .with_hovered_tag(hovered_tag)
            .with_hovered_id(tree.event_state.hovered)
            .with_focused_tag(tree.event_state.focused_tag)
            .with_focused_id(tree.event_state.focused)
            .with_blink_caret(blink_caret);
        if let Some(drawer) = custom_drawer {
            options = options.with_custom_drawer(drawer);
        }

        if allow_in_place_patch
            && tree.has_dirty_boundaries()
            && !self.command_list.is_empty()
            && patch_dirty_boundaries_with_scratch(
                tree,
                &mut options,
                &mut self.compiler_scratch,
                &mut self.command_list,
            )
        {
            return;
        }

        self.command_list.clear();
        compile_tree_draw_commands_with_scratch(
            tree,
            root,
            &mut options,
            &mut self.compiler_scratch,
            &mut self.command_list,
        );
        if force_full_upload || !tree.has_dirty_boundaries() {
            self.needs_full_quad_upload = true;
        }
    }

    /// Collects visible text sections from `tree` applying extra floating/modal/dropdown occluders.
    #[must_use]
    pub fn collect_text_sections_from_tree<'a>(
        tree: &'a UiTree,
        active_dropdown_rects: &[Rect],
        active_modal_rects: &[Rect],
        floating_window_rects: &[Rect],
    ) -> Vec<TextSection<'a>> {
        Self::collect_text_sections_from_tree_with_hover(
            tree,
            active_dropdown_rects,
            active_modal_rects,
            floating_window_rects,
            None,
        )
    }

    /// Collects visible text sections from `tree` applying hover emphasis and extra occluders.
    #[must_use]
    pub fn collect_text_sections_from_tree_with_hover<'a>(
        tree: &'a UiTree,
        active_dropdown_rects: &[Rect],
        active_modal_rects: &[Rect],
        floating_window_rects: &[Rect],
        hovered_tag: Option<u64>,
    ) -> Vec<TextSection<'a>> {
        let mut extra_occluders = Vec::with_capacity(
            active_dropdown_rects.len() + active_modal_rects.len() + floating_window_rects.len(),
        );
        extra_occluders.extend_from_slice(active_dropdown_rects);
        extra_occluders.extend_from_slice(active_modal_rects);
        extra_occluders.extend_from_slice(floating_window_rects);

        let mut options = TextCollectionOptions::new().with_hovered_tag(hovered_tag);
        options.extra_occluders = extra_occluders;
        irisui_text::collect_text_sections_with_options(tree, &options)
    }

    /// Prepares split static/volatile text layers and submits the WGPU UI overlay render pass for
    /// `tree` and `text_system`, clearing consumed tree dirty boundary markers on completion.
    pub fn render_to_view(
        &mut self,
        tree: &mut UiTree,
        text_system: &mut TextSystem,
        hovered_tag: Option<u64>,
        params: IrisSurfaceRenderParams<'_>,
    ) {
        self.flush_pending_atlas_uploads(params.queue);
        if !self.is_visible
            || (self.command_list.quads.is_empty()
                && self.command_list.texture_quads.is_empty()
                && tree.root().is_none())
        {
            return;
        }

        let zoom = if params.zoom_factor.is_finite() && params.zoom_factor > 0.1 {
            params.zoom_factor
        } else {
            1.0
        };
        let logical_screen_size = (
            (params.physical_screen_size.0 as f32 / zoom)
                .round()
                .max(1.0) as u32,
            (params.physical_screen_size.1 as f32 / zoom)
                .round()
                .max(1.0) as u32,
        );

        let static_surface_dirty =
            self.needs_text_prepare || self.needs_full_quad_upload || tree.has_dirty_boundaries();
        let action = self.surface_cache.state().resolve_frame_action(
            self.last_frame_mode,
            static_surface_dirty,
            params.physical_screen_size,
            zoom,
        );

        match action {
            SurfaceCacheFrameAction::BlitCachedOnly => {
                if self.needs_volatile_text_prepare {
                    self.prepare_text_layers(tree, text_system, hovered_tag, zoom, &params);
                }
                self.encode_cached_composite_pass(
                    logical_screen_size,
                    params.physical_screen_size,
                    params.target_view,
                    params.encoder,
                );
                self.surface_cache.state_mut().record_cache_hit();
            }
            SurfaceCacheFrameAction::PopulateAndBlit => {
                self.prepare_text_layers(tree, text_system, hovered_tag, zoom, &params);
                self.prepare_quads_for_render(
                    tree,
                    logical_screen_size,
                    params.device,
                    params.queue,
                );
                let has_drawable = self.has_drawable_layers();
                if has_drawable {
                    let offscreen_view = self
                        .surface_cache
                        .ensure_offscreen_view(params.device, params.physical_screen_size);
                    Self::encode_ui_pass(
                        SurfaceRenderSources {
                            renderer: &self.renderer,
                            command_list: &self.command_list,
                            text_renderer: self.text_renderer.as_ref(),
                        },
                        UiPassConfig {
                            logical_screen_size,
                            target_view: offscreen_view,
                            load_op: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                            mode: CommandListRenderMode::StaticWithCutout,
                            include_volatile_text: false,
                        },
                        params.encoder,
                    );
                    self.surface_cache
                        .state_mut()
                        .record_populated(params.physical_screen_size, zoom);
                    self.encode_cached_composite_pass(
                        logical_screen_size,
                        params.physical_screen_size,
                        params.target_view,
                        params.encoder,
                    );
                } else {
                    self.surface_cache.state_mut().record_direct_frame();
                }
                tree.clear_dirty_boundaries();
                self.needs_full_quad_upload = false;
            }
            SurfaceCacheFrameAction::DirectRender => {
                self.prepare_text_layers(tree, text_system, hovered_tag, zoom, &params);
                self.prepare_quads_for_render(
                    tree,
                    logical_screen_size,
                    params.device,
                    params.queue,
                );
                if self.has_drawable_layers() {
                    Self::encode_ui_pass(
                        SurfaceRenderSources {
                            renderer: &self.renderer,
                            command_list: &self.command_list,
                            text_renderer: self.text_renderer.as_ref(),
                        },
                        UiPassConfig {
                            logical_screen_size,
                            target_view: params.target_view,
                            load_op: wgpu::LoadOp::Load,
                            mode: CommandListRenderMode::Full,
                            include_volatile_text: true,
                        },
                        params.encoder,
                    );
                }
                self.surface_cache.state_mut().record_direct_frame();
                tree.clear_dirty_boundaries();
                self.needs_full_quad_upload = false;
            }
        }
    }

    fn prepare_text_layers(
        &mut self,
        tree: &UiTree,
        text_system: &mut TextSystem,
        hovered_tag: Option<u64>,
        zoom: f32,
        params: &IrisSurfaceRenderParams<'_>,
    ) {
        let mut prepare_static = self.needs_text_prepare;
        let mut prepare_volatile = self.needs_volatile_text_prepare;
        if self.text_renderer.is_none() {
            self.text_renderer = Some(LayeredTextRenderer::new(
                params.device,
                params.queue,
                self.target_format,
            ));
            prepare_static = true;
            prepare_volatile = true;
        }
        if !prepare_static && !prepare_volatile {
            return;
        }

        let mut text_options = TextCollectionOptions {
            hovered_tag,
            hovered_id: tree.event_state.hovered,
            focused_tag: tree.event_state.focused_tag,
            focused_id: tree.event_state.focused,
            blink_caret: text_system.is_caret_visible(),
            extra_occluders: params.extra_text_occluders.to_vec(),
            ..Default::default()
        };

        let Some(txt_renderer) = &mut self.text_renderer else {
            return;
        };

        if prepare_static && prepare_volatile {
            let (static_sec, volatile_sec) =
                irisui_text::collect_split_text_sections_with_options(tree, &text_options);
            txt_renderer.prepare_static(
                params.device,
                params.queue,
                text_system,
                params.physical_screen_size,
                zoom,
                &static_sec,
            );
            txt_renderer.prepare_volatile(
                params.device,
                params.queue,
                text_system,
                params.physical_screen_size,
                zoom,
                &volatile_sec,
            );
        } else if prepare_static {
            text_options.volatility_filter = TextVolatilityFilter::StaticOnly;
            let static_sec = irisui_text::collect_text_sections_with_options(tree, &text_options);
            txt_renderer.prepare_static(
                params.device,
                params.queue,
                text_system,
                params.physical_screen_size,
                zoom,
                &static_sec,
            );
        } else {
            text_options.volatility_filter = TextVolatilityFilter::VolatileOnly;
            let volatile_sec = irisui_text::collect_text_sections_with_options(tree, &text_options);
            txt_renderer.prepare_volatile(
                params.device,
                params.queue,
                text_system,
                params.physical_screen_size,
                zoom,
                &volatile_sec,
            );
        }

        self.needs_text_prepare = false;
        self.needs_volatile_text_prepare = false;
    }

    fn has_quads(&self) -> bool {
        !self.command_list.commands.is_empty() || !self.command_list.quads.is_empty()
    }

    fn has_drawable_layers(&self) -> bool {
        self.has_quads() || self.text_renderer.is_some()
    }

    fn prepare_quads_for_render(
        &mut self,
        tree: &UiTree,
        logical_screen_size: (u32, u32),
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) {
        if !self.has_quads() {
            return;
        }
        let dirty_boundaries = if self.needs_full_quad_upload {
            None
        } else {
            Some(tree.dirty_boundaries())
        };
        self.renderer.prepare_command_list_with_dirty_boundaries(
            device,
            queue,
            [logical_screen_size.0 as f32, logical_screen_size.1 as f32],
            &self.command_list,
            dirty_boundaries,
        );
    }

    fn encode_ui_pass(
        sources: SurfaceRenderSources<'_>,
        config: UiPassConfig<'_>,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        let has_quads =
            !sources.command_list.commands.is_empty() || !sources.command_list.quads.is_empty();
        let static_text = sources
            .text_renderer
            .map(LayeredTextRenderer::static_renderer);
        let volatile_text = config
            .include_volatile_text
            .then_some(sources.text_renderer)
            .flatten()
            .map(LayeredTextRenderer::volatile_renderer);
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Iris UI Surface Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: config.target_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: config.load_op,
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });

        if has_quads {
            sources.renderer.render_command_list_with_mode(
                &mut render_pass,
                sources.command_list,
                config.logical_screen_size,
                config.mode,
            );
        }
        if let Some(txt) = static_text {
            txt.render(&mut render_pass);
        }
        if let Some(volatile_txt) = volatile_text {
            volatile_txt.render(&mut render_pass);
        }
    }

    fn encode_cached_composite_pass(
        &self,
        logical_screen_size: (u32, u32),
        physical_screen_size: (u32, u32),
        target_view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Iris UI Cached Composite Pass"),
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

        if !self.command_list.external_texture_quads.is_empty() {
            self.renderer.render_command_list_with_mode(
                &mut render_pass,
                &self.command_list,
                logical_screen_size,
                CommandListRenderMode::ExternalTexturesOnly,
            );
            render_pass.set_scissor_rect(
                0,
                0,
                physical_screen_size.0.max(1),
                physical_screen_size.1.max(1),
            );
        }

        self.surface_cache.draw_blit_in_pass(&mut render_pass);

        if let Some(volatile_txt) = self
            .text_renderer
            .as_ref()
            .map(LayeredTextRenderer::volatile_renderer)
        {
            volatile_txt.render(&mut render_pass);
        }
    }
}

struct SurfaceRenderSources<'a> {
    renderer: &'a IrisRenderer,
    command_list: &'a DrawCommandList,
    text_renderer: Option<&'a LayeredTextRenderer>,
}

struct UiPassConfig<'a> {
    logical_screen_size: (u32, u32),
    target_view: &'a wgpu::TextureView,
    load_op: wgpu::LoadOp<wgpu::Color>,
    mode: CommandListRenderMode,
    include_volatile_text: bool,
}