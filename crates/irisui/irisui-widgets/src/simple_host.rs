// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Lightweight Standalone Overlay UI Host (`SimpleUiHost`)
//!
//! Provides [`SimpleUiHost`], a compact retained/immediate-rebuild WGPU UI host combining
//! [`UiTree`], [`LayoutEngine`], [`TextSystem`], and [`IrisSurfaceHost`] for in-game HUDs and
//! standalone viewport overlays that do not require `winit` or `irisui-workbench` docking shells.

use irisui_core::{MouseButton, Point, Rect, Size, UiEvent, UiTree};
use irisui_layout::LayoutEngine;
use irisui_text::TextSystem;
use irisui_wgpu::{IrisSurfaceHost, IrisSurfaceRenderParams};

use crate::UiScope;

/// Logical geometry and cursor coordinates passed to a [`SimpleUiHost::prepare_frame`] builder.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SimpleUiContext {
    /// Logical full-screen overlay rectangle (`(0, 0, logical_w, logical_h)`).
    pub viewport_rect: Rect,
    /// Current pointer coordinates in logical overlay space.
    pub cursor_pos: Point,
}

/// CPU-side state coordinator paired with [`SimpleUiHost`] managing tree lifecycle, DPI scaling,
/// Taffy layout solving, and pointer event dispatch without requiring a GPU device.
pub struct SimpleUiState {
    tree: UiTree,
    layout_engine: LayoutEngine,
    cursor_pos: Point,
    physical_size: (u32, u32),
    scale_factor: f32,
}

impl SimpleUiState {
    /// Creates a new overlay state for `physical_size` and DPI `scale_factor`.
    #[must_use]
    pub fn new(physical_size: (u32, u32), scale_factor: f32) -> Self {
        Self {
            tree: UiTree::new(),
            layout_engine: LayoutEngine::new(),
            cursor_pos: Point::ZERO,
            physical_size,
            scale_factor: sanitize_scale_factor(scale_factor),
        }
    }

    /// Updates surface `physical_size` and DPI `scale_factor` (ignoring non-finite or `<= 0.1` scales).
    pub fn resize(&mut self, physical_size: (u32, u32), scale_factor: f32) {
        self.physical_size = physical_size;
        if scale_factor.is_finite() && scale_factor > 0.1 {
            self.scale_factor = scale_factor;
        }
    }

    /// Returns the sanitized active DPI scale factor.
    #[inline]
    #[must_use]
    pub const fn scale_factor(&self) -> f32 {
        self.scale_factor
    }

    /// Returns the surface dimensions in physical pixels.
    #[inline]
    #[must_use]
    pub const fn physical_size(&self) -> (u32, u32) {
        self.physical_size
    }

    /// Returns the rounded logical dimensions derived from `physical_size` and `scale_factor`.
    #[inline]
    #[must_use]
    pub fn logical_size(&self) -> Size {
        logical_dimensions(self.physical_size, self.scale_factor)
    }

    /// Returns the logical full-viewport bounds `(0, 0, logical_width, logical_height)`.
    #[inline]
    #[must_use]
    pub fn viewport_rect(&self) -> Rect {
        let size = self.logical_size();
        Rect::new(0.0, 0.0, size.width, size.height)
    }

    /// Returns the current pointer coordinate in logical space.
    #[inline]
    #[must_use]
    pub const fn cursor_pos(&self) -> Point {
        self.cursor_pos
    }

    /// Updates pointer coordinates from physical pixels (rejecting non-finite inputs) and optionally
    /// dispatches a [`UiEvent::MouseMove`] event, returning whether the UI consumed it.
    pub fn handle_pointer_moved(
        &mut self,
        physical_x: f64,
        physical_y: f64,
        dispatch: bool,
    ) -> bool {
        if !physical_x.is_finite() || !physical_y.is_finite() {
            return false;
        }
        self.cursor_pos = Point::new(
            physical_x as f32 / self.scale_factor,
            physical_y as f32 / self.scale_factor,
        );
        if dispatch {
            self.tree
                .dispatch_event(&UiEvent::MouseMove {
                    point: self.cursor_pos,
                })
                .consumed
        } else {
            false
        }
    }

    /// Dispatches a mouse press or release at the current logical pointer coordinate.
    pub fn handle_mouse_button(&mut self, button: MouseButton, pressed: bool) -> bool {
        let event = if pressed {
            UiEvent::MouseDown {
                button,
                point: self.cursor_pos,
            }
        } else {
            UiEvent::MouseUp {
                button,
                point: self.cursor_pos,
            }
        };
        self.tree.dispatch_event(&event).consumed
    }

    /// Dispatches an arbitrary [`UiEvent`] to the internal retained tree.
    pub fn dispatch_event(&mut self, event: &UiEvent) -> bool {
        self.tree.dispatch_event(event).consumed
    }

    /// Returns the semantic tag clicked during the preceding event dispatch, if any.
    #[inline]
    #[must_use]
    pub const fn clicked_tag(&self) -> Option<u64> {
        self.tree.event_state.frame_clicked_tag
    }

    /// Returns `true` when `tag` was clicked during the preceding event dispatch.
    #[inline]
    #[must_use]
    pub fn is_tag_clicked(&self, tag: u64) -> bool {
        self.clicked_tag() == Some(tag)
    }

    /// Clears transient interaction, text edit, popup, and numeric undo history.
    pub fn clear_interaction_history(&mut self) {
        self.tree.clear_interaction_history();
    }

    /// Returns the semantic tag of the currently hovered widget, if any.
    #[inline]
    #[must_use]
    pub fn hovered_tag(&self) -> Option<u64> {
        self.tree
            .event_state
            .hovered
            .and_then(|id| self.tree.get(id))
            .map(|node| node.tag)
            .filter(|&tag| tag != 0)
    }

    /// Rebuilds the widget tree via `build` and computes Taffy layout for the active logical bounds.
    pub fn prepare_tree<F>(&mut self, build: F)
    where
        F: FnOnce(&mut UiScope<'_>, SimpleUiContext),
    {
        self.tree.clear_frame_clicks();
        self.tree.clear();
        let logical_size = self.logical_size();
        if let Ok(root) = self.tree.create_root() {
            let ctx = SimpleUiContext {
                viewport_rect: Rect::new(0.0, 0.0, logical_size.width, logical_size.height),
                cursor_pos: self.cursor_pos,
            };
            let mut scope = UiScope::new(&mut self.tree, root);
            build(&mut scope, ctx);
            let _ = self
                .layout_engine
                .compute_layout(&mut self.tree, logical_size);
        }
    }

    /// Returns an immutable reference to the internal [`UiTree`] for diagnostics or tests.
    #[inline]
    #[must_use]
    pub const fn tree(&self) -> &UiTree {
        &self.tree
    }
}

/// Lightweight WGPU UI overlay host combining [`SimpleUiState`], [`TextSystem`], and
/// [`IrisSurfaceHost`] for standalone HUDs without desktop workbench/docking overhead.
pub struct SimpleUiHost {
    state: SimpleUiState,
    text_system: TextSystem,
    surface: IrisSurfaceHost,
}

impl SimpleUiHost {
    /// Creates a new standalone overlay host targeting `target_format` on `device`.
    #[must_use]
    pub fn new(
        device: &wgpu::Device,
        target_format: wgpu::TextureFormat,
        physical_size: (u32, u32),
        scale_factor: f32,
    ) -> Self {
        let mut surface = IrisSurfaceHost::new(device, target_format);
        // SimpleUiHost rebuilds the tree per prepare_frame call, so direct rendering avoids
        // stale offscreen surface blits across dynamic HUD animations.
        surface.set_surface_cache_enabled(false);
        Self {
            state: SimpleUiState::new(physical_size, scale_factor),
            text_system: TextSystem::new(),
            surface,
        }
    }

    /// Toggles offscreen static surface caching for infrequent HUDs.
    #[inline]
    pub fn set_surface_cache_enabled(&mut self, enabled: bool) {
        self.surface.set_surface_cache_enabled(enabled);
    }

    /// Updates surface dimensions and DPI scale factor after window resize.
    #[inline]
    pub fn resize(&mut self, physical_size: (u32, u32), scale_factor: f32) {
        self.state.resize(physical_size, scale_factor);
    }

    /// Returns the sanitized active DPI scale factor.
    #[inline]
    #[must_use]
    pub const fn scale_factor(&self) -> f32 {
        self.state.scale_factor()
    }

    /// Returns the current logical pointer coordinate.
    #[inline]
    #[must_use]
    pub const fn cursor_pos(&self) -> Point {
        self.state.cursor_pos()
    }

    /// Returns the current logical viewport rectangle.
    #[inline]
    #[must_use]
    pub fn viewport_rect(&self) -> Rect {
        self.state.viewport_rect()
    }

    /// Updates pointer coordinates from physical pixels and optionally dispatches [`UiEvent::MouseMove`].
    #[inline]
    pub fn handle_pointer_moved(
        &mut self,
        physical_x: f64,
        physical_y: f64,
        dispatch: bool,
    ) -> bool {
        self.state
            .handle_pointer_moved(physical_x, physical_y, dispatch)
    }

    /// Dispatches a mouse press or release event at the active logical pointer coordinate.
    #[inline]
    pub fn handle_mouse_button(&mut self, button: MouseButton, pressed: bool) -> bool {
        self.state.handle_mouse_button(button, pressed)
    }

    /// Dispatches an arbitrary [`UiEvent`] onto the overlay widget tree.
    #[inline]
    pub fn dispatch_event(&mut self, event: &UiEvent) -> bool {
        self.state.dispatch_event(event)
    }

    /// Returns the semantic tag clicked during the preceding event dispatch, if any.
    #[inline]
    #[must_use]
    pub const fn clicked_tag(&self) -> Option<u64> {
        self.state.clicked_tag()
    }

    /// Returns `true` when `tag` was clicked during the preceding event dispatch.
    #[inline]
    #[must_use]
    pub fn is_tag_clicked(&self, tag: u64) -> bool {
        self.state.is_tag_clicked(tag)
    }

    /// Clears transient interaction, focus, text edit, and popup history.
    #[inline]
    pub fn clear_interaction_history(&mut self) {
        self.state.clear_interaction_history();
    }

    /// Rebuilds the declarative overlay tree, solves Taffy layout, and compiles GPU draw commands.
    pub fn prepare_frame<F>(&mut self, build: F)
    where
        F: FnOnce(&mut UiScope<'_>, SimpleUiContext),
    {
        self.state.prepare_tree(build);
        let hovered_tag = self.state.hovered_tag();
        let blink_caret = self.text_system.is_caret_visible();
        self.surface.recompile_or_patch_draw_commands(
            &self.state.tree,
            blink_caret,
            hovered_tag,
            false,
            true,
            None,
        );
        self.surface.needs_text_prepare = true;
        self.surface.needs_volatile_text_prepare = true;
    }

    /// Uploads prepared text/quad GPU resources and encodes the UI overlay pass onto `target_view`.
    pub fn render_to_view(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        target_view: &wgpu::TextureView,
    ) {
        if self.surface.command_list.quads.is_empty()
            && self.surface.command_list.texture_quads.is_empty()
        {
            return;
        }
        let hovered_tag = self.state.hovered_tag();
        let physical_screen_size = self.state.physical_size();
        let zoom_factor = self.state.scale_factor();
        self.surface.render_to_view(
            &mut self.state.tree,
            &mut self.text_system,
            hovered_tag,
            IrisSurfaceRenderParams {
                device,
                queue,
                encoder,
                target_view,
                physical_screen_size,
                zoom_factor,
                extra_text_occluders: &[],
            },
        );
    }

    /// Provides mutable access to the underlying [`TextSystem`] for custom font loading.
    #[inline]
    pub fn text_system_mut(&mut self) -> &mut TextSystem {
        &mut self.text_system
    }
}

#[inline]
#[must_use]
fn sanitize_scale_factor(scale_factor: f32) -> f32 {
    if scale_factor.is_finite() && scale_factor > 0.1 {
        scale_factor
    } else {
        1.0
    }
}

#[inline]
#[must_use]
fn logical_dimensions(physical_size: (u32, u32), scale_factor: f32) -> Size {
    let scale = sanitize_scale_factor(scale_factor);
    let w = (physical_size.0 as f32 / scale).round().max(1.0);
    let h = (physical_size.1 as f32 / scale).round().max(1.0);
    Size::new(w, h)
}

#[cfg(test)]
mod tests {
    use super::*;
    use irisui_core::UiTag;

    #[test]
    fn test_scale_factor_and_logical_dimension_guards() {
        let mut state = SimpleUiState::new((0, 0), f32::NAN);
        assert_eq!(state.scale_factor(), 1.0);
        assert_eq!(state.logical_size(), Size::new(1.0, 1.0));

        // Negative or non-finite resizes keep the last valid scale and reject bad pointers.
        state.resize((1920, 1080), 2.0);
        assert_eq!(state.scale_factor(), 2.0);
        assert_eq!(state.logical_size(), Size::new(960.0, 540.0));
        state.resize((1920, 1080), -3.0);
        assert_eq!(state.scale_factor(), 2.0);
        assert!(!state.handle_pointer_moved(f64::NAN, 10.0, true));
        assert_eq!(state.cursor_pos(), Point::ZERO);
    }

    #[test]
    fn test_prepare_tree_and_click_tag_roundtrip() {
        const BTN_TAG: UiTag = UiTag::from_str("simple_host.test_btn");
        let mut state = SimpleUiState::new((800, 600), 2.0);

        state.prepare_tree(|scope, ctx| {
            assert_eq!(ctx.viewport_rect, Rect::new(0.0, 0.0, 400.0, 300.0));
            let _ =
                scope.add(crate::declarative::widget::Button::new("Continue").tag(BTN_TAG.raw()));
        });

        let btn_rect = state
            .tree()
            .find_node_by_tag(BTN_TAG.raw())
            .and_then(|id| state.tree().get(id))
            .map(|n| n.computed_rect)
            .expect("button must be laid out");
        assert!(btn_rect.width > 0.0 && btn_rect.height > 0.0);

        // Physical coordinate = 2x logical center
        let px = f64::from(btn_rect.x + btn_rect.width * 0.5) * 2.0;
        let py = f64::from(btn_rect.y + btn_rect.height * 0.5) * 2.0;
        state.handle_pointer_moved(px, py, true);
        assert!(state.handle_mouse_button(MouseButton::Left, true));
        assert!(state.handle_mouse_button(MouseButton::Left, false));
        assert!(state.is_tag_clicked(BTN_TAG.raw()));
    }
}