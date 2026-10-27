// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Workbench UI Extensions
//!
//! Provides high-level declarative extension methods on [`UiScope`] for embedding
//! 3D viewport surfaces, aspect-ratio-fitted external GPU textures, and volatile
//! immediate hardware overlays with zero boilerplate.
//!

use iris_core::Color;
use iris_core::geometry::Rect;
use iris_core::style::{AlignItems, JustifyContent, Style, TextAlign};
use iris_widgets::canvas::ImmediateCanvas;
use iris_widgets::declarative::UiScope;

use super::overlay::CustomOverlayHook;
use super::texture::ExternalTextureHook;

/// Extension trait providing high-level declarative workbench primitives on [`UiScope`].
///
/// Designed to eliminate boilerplate glue code on the application and engine side.
/// Instead of manually defining custom hook structs, managing absolute container styles,
/// and wiring low-level texture IDs, consumers call [`Self::viewport_surface`] and
/// [`Self::volatile_overlay`] declaratively.
pub trait WorkbenchUiScopeExt {
    /// Declaratively renders a hardware-accelerated viewport surface with optional aspect ratio
    /// fitting, fallback placeholder messaging, and a nested content scope for overlays.
    ///
    /// When `texture` is `Some(hook)`, renders the GPU texture quad with aspect ratio preservation.
    /// When `texture` is `None` and `placeholder_text` is provided, displays centered fallback text.
    /// The `content_fn` closure receives an inner [`UiScope`] allowing child HUD elements,
    /// floating toolbars, and buttons to be declared on top of the viewport canvas.
    ///
    /// # Arguments
    /// * `texture` - Optional [`ExternalTextureHook`] descriptor.
    /// * `placeholder_text` - Human-readable label displayed when `texture` is `None`.
    /// * `content_fn` - Declarative closure building viewport overlays, toolbars, and controls.
    fn viewport_surface<F>(
        &mut self,
        texture: Option<ExternalTextureHook>,
        placeholder_text: Option<&str>,
        content_fn: F,
    ) where
        F: FnOnce(&mut UiScope<'_>);

    /// Declaratively embeds a volatile immediate-mode hardware overlay directly into an
    /// [`ImmediateCanvas`] without persistent child nodes in the tree.
    ///
    /// # Arguments
    /// * `name` - Diagnostic debug name for the canvas container.
    /// * `tag` - 64-bit semantic identifier for hit-testing and tag retrieval.
    /// * `rect` - Bounding rectangle of the overlay in panel coordinates.
    /// * `draw_fn` - Immediate drawing closure executed to populate per-frame volatile primitives.
    fn volatile_overlay<F>(&mut self, name: &'static str, tag: u64, rect: Rect, draw_fn: F)
    where
        F: FnOnce(&mut ImmediateCanvas<'_>);

    /// Declaratively embeds a volatile overlay using a type implementing [`CustomOverlayHook`].
    ///
    /// # Arguments
    /// * `name` - Diagnostic debug name for the canvas container.
    /// * `tag` - 64-bit semantic identifier for hit-testing and tag retrieval.
    /// * `rect` - Bounding rectangle of the overlay in panel coordinates.
    /// * `hook` - Concrete type implementing [`CustomOverlayHook`].
    fn volatile_overlay_hook<H>(&mut self, name: &'static str, tag: u64, rect: Rect, hook: H)
    where
        H: CustomOverlayHook;
}

impl<'a> WorkbenchUiScopeExt for UiScope<'a> {
    fn viewport_surface<F>(
        &mut self,
        texture: Option<ExternalTextureHook>,
        placeholder_text: Option<&str>,
        content_fn: F,
    ) where
        F: FnOnce(&mut UiScope<'_>),
    {
        let parent_rect = self.parent_rect();
        let root_style = Style::new()
            .position_absolute()
            .left(0.0)
            .top(0.0)
            .width(parent_rect.width)
            .height(parent_rect.height)
            .background(Color::TRANSPARENT)
            .clip_children(true);

        self.container_passive(root_style, |vp_scope| {
            if let Some(tex) = texture {
                tex.render(vp_scope);
            } else if let Some(placeholder) = placeholder_text {
                let ph_style = Style::new()
                    .position_absolute()
                    .left(0.0)
                    .top(0.0)
                    .width(parent_rect.width)
                    .height(parent_rect.height)
                    .justify_content(JustifyContent::Center)
                    .align_items(AlignItems::Center);
                vp_scope.container_passive(ph_style, |ph| {
                    ph.label_custom(placeholder, 14.0, Color::hex("#888888"), TextAlign::Center);
                });
            }

            content_fn(vp_scope);
        });
    }

    fn volatile_overlay<F>(&mut self, name: &'static str, tag: u64, rect: Rect, draw_fn: F)
    where
        F: FnOnce(&mut ImmediateCanvas<'_>),
    {
        let style = Style::new()
            .position_absolute()
            .left(rect.x)
            .top(rect.y)
            .width(rect.width)
            .height(rect.height);

        self.immediate_canvas_styled(name, tag, style, draw_fn);
    }

    fn volatile_overlay_hook<H>(&mut self, name: &'static str, tag: u64, rect: Rect, mut hook: H)
    where
        H: CustomOverlayHook,
    {
        let style = Style::new()
            .position_absolute()
            .left(rect.x)
            .top(rect.y)
            .width(rect.width)
            .height(rect.height);

        self.immediate_canvas_styled(name, tag, style, |canvas| {
            hook.render_overlay(canvas, rect);
        });
    }
}
