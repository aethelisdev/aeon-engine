// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # External Hardware Texture Hooks
//!
//! Provides descriptors and mathematical aspect-ratio solvers for presenting
//! offscreen render targets (3D viewports, video decoders, compute canvases)
//! inside workbench panels with zero visual distortion.

use iris_core::ExternalTextureId;
use iris_core::geometry::Rect;
use iris_core::style::Style;
use iris_widgets::declarative::UiScope;

/// Describes an external hardware texture binding presented inside a workbench panel.
///
/// Designed to decouple the high-level UI shell from graphics backend intricacies (WGPU,
/// Vulkan, DirectX, Metal). Host panels specify the registered texture identifier and target
/// screen bounds, with optional aspect ratio fitting (letterbox / pillarbox calculation).
#[derive(Debug, Clone, PartialEq)]
pub struct ExternalTextureHook {
    /// Identifier of the external texture registered with the rendering backend.
    pub texture_id: u64,

    /// Bounding rectangle in screen coordinates where the texture should be presented.
    pub rect: Rect,

    /// Optional aspect ratio (width divided by height) to maintain within `rect`.
    /// When specified, [`Self::fit_aspect_ratio`] computes the centered sub-rect.
    pub aspect_ratio: Option<f32>,
}

impl ExternalTextureHook {
    /// Creates a new external texture hook with the given identifier and bounding rectangle.
    ///
    /// By default, no aspect ratio constraint is applied (`aspect_ratio: None`), stretching
    /// the presentation across the entirety of `rect`.
    ///
    /// # Arguments
    /// * `texture_id` - Unique identifier of the GPU texture in the renderer backend.
    /// * `rect` - Bounding rectangle where the texture is displayed.
    #[inline]
    #[must_use]
    pub const fn new(texture_id: u64, rect: Rect) -> Self {
        Self {
            texture_id,
            rect,
            aspect_ratio: None,
        }
    }

    /// Sets an explicit aspect ratio (width / height) to maintain within the container bounds.
    ///
    /// # Arguments
    /// * `aspect_ratio` - Desired width-to-height ratio (e.g., `16.0 / 9.0`).
    #[inline]
    #[must_use]
    pub const fn with_aspect_ratio(mut self, aspect_ratio: f32) -> Self {
        self.aspect_ratio = Some(aspect_ratio);
        self
    }

    /// Computes the largest centered sub-rectangle within [`self.rect`](Self::rect) that preserves
    /// the target [`aspect_ratio`](Self::aspect_ratio).
    ///
    /// If no aspect ratio is set, or if the ratio is non-positive, or if the container
    /// bounds have non-positive dimensions, this returns [`self.rect`](Self::rect) unchanged.
    ///
    /// - If container is wider than ratio: applies horizontal letterboxing (pillarbox).
    /// - If container is taller than ratio: applies vertical letterboxing.
    #[inline]
    #[must_use]
    pub fn fit_aspect_ratio(&self) -> Rect {
        let ratio = match self.aspect_ratio {
            Some(r) if r > 0.0 => r,
            _ => return self.rect,
        };

        if self.rect.width <= 0.0 || self.rect.height <= 0.0 {
            return self.rect;
        }

        let container_ratio = self.rect.width / self.rect.height;

        if container_ratio > ratio {
            // Container is wider than the target aspect ratio -> pillarbox (fit height)
            let fitted_w = self.rect.height * ratio;
            let offset_x = (self.rect.width - fitted_w) * 0.5;
            Rect::new(
                self.rect.x + offset_x,
                self.rect.y,
                fitted_w,
                self.rect.height,
            )
        } else {
            // Container is taller than the target aspect ratio -> letterbox (fit width)
            let fitted_h = self.rect.width / ratio;
            let offset_y = (self.rect.height - fitted_h) * 0.5;
            Rect::new(
                self.rect.x,
                self.rect.y + offset_y,
                self.rect.width,
                fitted_h,
            )
        }
    }

    /// Returns `true` if the texture hook has valid positive screen dimensions.
    #[inline]
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.rect.width > 0.0 && self.rect.height > 0.0
    }

    /// Declaratively emits the external hardware texture quad into the given [`UiScope`].
    ///
    /// Evaluates [`fit_aspect_ratio`](Self::fit_aspect_ratio) and constructs an absolute
    /// quad container presenting the external GPU texture with zero distortion.
    ///
    /// # Arguments
    /// * `scope` - Declarative UI builder scope.
    pub fn render(&self, scope: &mut UiScope<'_>) {
        if !self.is_valid() {
            return;
        }
        let fitted = self.fit_aspect_ratio();
        let offset_x = fitted.x - self.rect.x;
        let offset_y = fitted.y - self.rect.y;

        let parent = scope.parent();
        let tree = scope.tree_mut();
        let node_id = tree.create_node();
        if let Some(node) = tree.get_mut(node_id) {
            node.interactive = false;
            node.set_name("ExternalTextureQuad");
            node.set_external_texture(Some(ExternalTextureId(self.texture_id)));
            node.set_style(
                Style::new()
                    .position_absolute()
                    .left(offset_x)
                    .top(offset_y)
                    .width(fitted.width)
                    .height(fitted.height),
            );
        }
        let _ = tree.add_child(parent, node_id);
    }
}
