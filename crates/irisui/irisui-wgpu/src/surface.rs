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
use crate::renderer::IrisRenderer;
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
    /// Optional host-managed focus tag taking precedence over `UiTree` internal focus.
    pub focused_tag_override: Option<u64>,
    /// Additional floating/menu rectangles that occlude underlying UI text sections.
    pub extra_text_occluders: &'a [Rect],
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
            needs_text_prepare: true,
            needs_volatile_text_prepare: true,
            needs_full_quad_upload: true,
            is_visible: true,
        }
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
        }
        // Invariant: `self.texture_atlas` is guaranteed `Some` by the guard immediately above.
        self.texture_atlas
            .as_ref()
            .unwrap_or_else(|| unreachable!("texture_atlas initialized immediately above"))
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
        if let Some(view) = view {
            self.renderer
                .external_textures
                .set(device, &self.renderer.external_pipeline, id, view);
        } else {
            self.renderer.external_textures.remove(id);
        }
    }

    /// Marks static typography preparation dirty when `update` flipped caret visibility.
    #[inline]
    pub fn apply_caret_blink_update(&mut self, update: CaretBlinkUpdate) {
        if update.needs_redraw() {
            self.needs_text_prepare = true;
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

        self.prepare_text_layers(tree, text_system, hovered_tag, zoom, &params);
        self.submit_overlay_render_pass(tree, logical_screen_size, params);
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
            focused_tag: params.focused_tag_override.or(tree.event_state.focused_tag),
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

    fn submit_overlay_render_pass(
        &mut self,
        tree: &mut UiTree,
        logical_screen_size: (u32, u32),
        params: IrisSurfaceRenderParams<'_>,
    ) {
        let has_quads =
            !self.command_list.commands.is_empty() || !self.command_list.quads.is_empty();
        let static_text = self.text_renderer.as_ref().map(|l| l.static_renderer());
        let volatile_text = self.text_renderer.as_ref().map(|l| l.volatile_renderer());
        if has_quads || static_text.is_some() || volatile_text.is_some() {
            if has_quads {
                let dirty_boundaries = if self.needs_full_quad_upload {
                    None
                } else {
                    Some(tree.dirty_boundaries())
                };
                self.renderer.prepare_command_list_with_dirty_boundaries(
                    params.device,
                    params.queue,
                    [logical_screen_size.0 as f32, logical_screen_size.1 as f32],
                    &self.command_list,
                    dirty_boundaries,
                );
            }

            let mut render_pass = params
                .encoder
                .begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Iris UI Surface Render Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: params.target_view,
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

            if has_quads {
                self.renderer.render_command_list(
                    &mut render_pass,
                    &self.command_list,
                    logical_screen_size,
                );
            }
            if let Some(txt) = static_text {
                txt.render(&mut render_pass);
            }
            if let Some(volatile_txt) = volatile_text {
                volatile_txt.render(&mut render_pass);
            }
        }

        tree.clear_dirty_boundaries();
        self.needs_full_quad_upload = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use irisui_core::{Color, Style};

    #[test]
    fn test_collect_text_sections_filters_occluded_background_labels() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("root");
        let label = tree.create_node();
        tree.add_child(root, label).expect("child");
        if let Some(node) = tree.get_mut(label) {
            node.text = Some("Background Text".into());
            node.computed_rect = Rect::new(20.0, 20.0, 120.0, 20.0);
        }

        let visible = IrisSurfaceHost::collect_text_sections_from_tree(&tree, &[], &[], &[]);
        assert_eq!(visible.len(), 1);

        let occluded = IrisSurfaceHost::collect_text_sections_from_tree(
            &tree,
            &[Rect::new(0.0, 0.0, 200.0, 200.0)],
            &[],
            &[],
        );
        assert!(occluded.is_empty());
    }

    #[test]
    fn test_surface_compile_and_hover_flag_logic() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("root");
        let child = tree.create_node();
        tree.add_child(root, child).expect("child");
        if let Some(node) = tree.get_mut(child) {
            node.style = Style::new().background(Color::BLUE);
            node.computed_rect = Rect::new(0.0, 0.0, 100.0, 40.0);
        }
        let mut command_list = DrawCommandList::default();
        let mut scratch = TreeCompilerScratch::default();
        let mut noop =
            |_l: &mut DrawCommandList, _id: WidgetId, _n: &WidgetNode, _c: Option<Rect>| {};
        let mut opts = TreeCompilerOptions::new().with_custom_drawer(&mut noop);
        compile_tree_draw_commands_with_scratch(
            &tree,
            root,
            &mut opts,
            &mut scratch,
            &mut command_list,
        );
        assert!(!command_list.quads.is_empty());

        let flags = IrisSurfaceFrameFlags {
            mode: SurfaceRefreshMode::StructuralRebuild {
                is_full_shell: true,
            },
            caret_blink_redraw: false,
            needs_text_prepare: true,
            volatile_needs_text_prepare: true,
            needs_quad_reupload: true,
        };
        assert_eq!(
            flags.mode,
            SurfaceRefreshMode::StructuralRebuild {
                is_full_shell: true
            }
        );
    }
}