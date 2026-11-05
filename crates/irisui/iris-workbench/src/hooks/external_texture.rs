// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # External Hardware Texture View Component & Aspect Ratio Solver
//!
//! Provides the declarative [`ExternalTextureView`] component and [`AspectRatio`]
//! solver for embedding offscreen hardware render targets (3D viewports, render-to-texture
//! passes, video decoders) inside workbench panels with mathematical aspect-ratio preservation.

use iris_core::ExternalTextureId;
use iris_core::geometry::Rect;
use iris_core::style::Style;
use iris_widgets::declarative::UiScope;

/// Aspect ratio constraint policies governing how external textures map into host container bounds.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum AspectRatio {
    /// Stretches the texture to completely fill the container rectangle without preserving aspect ratio.
    #[default]
    Stretch,

    /// Preserves the texture's native aspect ratio (`width / height`) by fitting entirely within the container (pillarbox or letterbox).
    Fit,

    /// Scales the texture to completely cover the container, preserving aspect ratio and cropping excess overflow edges.
    Fill,

    /// Preserves an explicit custom width-to-height ratio (e.g. `16.0 / 9.0` or `4.0 / 3.0`).
    Custom(f32),
}

impl AspectRatio {
    /// Resolves the effective numerical ratio (`width / height`) for the given source dimensions.
    ///
    /// Returns `None` for [`AspectRatio::Stretch`], or if the computed ratio is non-positive or non-finite.
    #[inline]
    #[must_use]
    pub fn numerical_ratio(&self, source_w: f32, source_h: f32) -> Option<f32> {
        match self {
            Self::Stretch => None,
            Self::Fit | Self::Fill => {
                if source_w > 0.0 && source_h > 0.0 {
                    let r = source_w / source_h;
                    if r.is_finite() && r > 0.0 {
                        Some(r)
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            Self::Custom(r) => {
                if r.is_finite() && *r > 0.0 {
                    Some(*r)
                } else {
                    None
                }
            }
        }
    }
}

/// Declarative view component for presenting external GPU hardware textures inside workbench panels.
///
/// Automatically evaluates container dimensions from Taffy or panel layout scopes,
/// calculates mathematically centered letterbox/pillarbox/fill bounds, and registers
/// non-interactive texture quads into [`UiScope`] with zero engine-side scissoring boilerplate.
#[derive(Debug, Clone, PartialEq)]
pub struct ExternalTextureView {
    /// 64-bit identifier of the registered hardware texture in the GPU renderer backend.
    pub texture_id: u64,

    /// Native pixel width of the source texture.
    pub width: f32,

    /// Native pixel height of the source texture.
    pub height: f32,

    /// Aspect ratio constraint policy applied when fitting inside the host container.
    pub aspect_ratio: AspectRatio,
}

impl ExternalTextureView {
    /// Constructs a new [`ExternalTextureView`] with default stretch behavior.
    ///
    /// # Arguments
    /// * `texture_id` - 64-bit texture identifier recognized by [`iris_wgpu::ExternalTextures`].
    /// * `width` - Source texture native width in pixels.
    /// * `height` - Source texture native height in pixels.
    #[inline]
    #[must_use]
    pub const fn new(texture_id: u64, width: f32, height: f32) -> Self {
        Self {
            texture_id,
            width,
            height,
            aspect_ratio: AspectRatio::Stretch,
        }
    }

    /// Sets the aspect ratio constraint policy for this texture view.
    ///
    /// # Arguments
    /// * `aspect_ratio` - Desired [`AspectRatio`] policy (Fit, Fill, Stretch, or Custom).
    #[inline]
    #[must_use]
    pub const fn with_aspect_ratio(mut self, aspect_ratio: AspectRatio) -> Self {
        self.aspect_ratio = aspect_ratio;
        self
    }

    /// Computes the exact sub-rectangle within `container_rect` according to [`self.aspect_ratio`](Self::aspect_ratio).
    ///
    /// Guarantees centered placement with zero distortion:
    /// - [`AspectRatio::Stretch`]: returns `container_rect` unchanged.
    /// - [`AspectRatio::Fit`]:
    ///   - Container wider than texture ratio: pillarbox (fits height, centers horizontally).
    ///   - Container taller than texture ratio: letterbox (fits width, centers vertically).
    /// - [`AspectRatio::Fill`]:
    ///   - Container wider than texture ratio: scales by width, centers vertically (crops top/bottom).
    ///   - Container taller than texture ratio: scales by height, centers horizontally (crops left/right).
    /// - [`AspectRatio::Custom`]: behaves like Fit using the explicit numerical ratio.
    ///
    /// Handles non-positive dimensions, `NaN`, and `Inf` with mathematical safety,
    /// returning `container_rect` without panic.
    #[must_use]
    pub fn compute_fitted_rect(&self, container_rect: Rect) -> Rect {
        if container_rect.width <= 0.0 || container_rect.height <= 0.0 {
            return container_rect;
        }

        let Some(target_ratio) = self.aspect_ratio.numerical_ratio(self.width, self.height) else {
            return container_rect;
        };

        let container_ratio = container_rect.width / container_rect.height;

        match self.aspect_ratio {
            AspectRatio::Fit | AspectRatio::Custom(_) => {
                if container_ratio > target_ratio {
                    // Container is wider than target ratio -> pillarbox (fit height)
                    let fitted_w = container_rect.height * target_ratio;
                    let offset_x = (container_rect.width - fitted_w) * 0.5;
                    Rect::new(
                        container_rect.x + offset_x,
                        container_rect.y,
                        fitted_w,
                        container_rect.height,
                    )
                } else {
                    // Container is taller than target ratio -> letterbox (fit width)
                    let fitted_h = container_rect.width / target_ratio;
                    let offset_y = (container_rect.height - fitted_h) * 0.5;
                    Rect::new(
                        container_rect.x,
                        container_rect.y + offset_y,
                        container_rect.width,
                        fitted_h,
                    )
                }
            }
            AspectRatio::Fill => {
                if container_ratio > target_ratio {
                    // Container is wider -> expand to fit width, excess height cropped
                    let filled_h = container_rect.width / target_ratio;
                    let offset_y = (container_rect.height - filled_h) * 0.5;
                    Rect::new(
                        container_rect.x,
                        container_rect.y + offset_y,
                        container_rect.width,
                        filled_h,
                    )
                } else {
                    // Container is taller -> expand to fit height, excess width cropped
                    let filled_w = container_rect.height * target_ratio;
                    let offset_x = (container_rect.width - filled_w) * 0.5;
                    Rect::new(
                        container_rect.x + offset_x,
                        container_rect.y,
                        filled_w,
                        container_rect.height,
                    )
                }
            }
            AspectRatio::Stretch => container_rect,
        }
    }

    /// Declaratively renders the external GPU texture quad into the provided [`UiScope`].
    ///
    /// Resolves container bounds from `scope.parent_rect()`, computes the fitted sub-rect,
    /// and attaches a non-interactive node equipped with [`ExternalTextureId`].
    ///
    /// # Arguments
    /// * `scope` - Declarative UI builder scope.
    pub fn render(&self, scope: &mut UiScope<'_>) {
        let parent_rect = scope.parent_rect();
        if parent_rect.width <= 0.0 || parent_rect.height <= 0.0 {
            return;
        }

        let fitted = self.compute_fitted_rect(parent_rect);
        let offset_x = fitted.x - parent_rect.x;
        let offset_y = fitted.y - parent_rect.y;

        let parent = scope.parent();
        let tree = scope.tree_mut();
        let node_id = tree.create_node();
        if let Some(node) = tree.get_mut(node_id) {
            node.interactive = false;
            node.set_name("ExternalTextureView");
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

#[cfg(test)]
mod tests {
    use super::*;
    use iris_core::UiTree;

    #[test]
    fn test_aspect_ratio_numerical_ratio_resolution() {
        assert_eq!(AspectRatio::Stretch.numerical_ratio(1920.0, 1080.0), None);
        assert_eq!(
            AspectRatio::Fit.numerical_ratio(1920.0, 1080.0),
            Some(1920.0 / 1080.0)
        );
        assert_eq!(
            AspectRatio::Fill.numerical_ratio(800.0, 600.0),
            Some(800.0 / 600.0)
        );
        assert_eq!(
            AspectRatio::Custom(1.5).numerical_ratio(100.0, 100.0),
            Some(1.5)
        );

        // Degenerate inputs
        assert_eq!(AspectRatio::Fit.numerical_ratio(0.0, 1080.0), None);
        assert_eq!(AspectRatio::Fit.numerical_ratio(1920.0, 0.0), None);
        assert_eq!(AspectRatio::Fit.numerical_ratio(-100.0, 50.0), None);
        assert_eq!(
            AspectRatio::Custom(-1.0).numerical_ratio(100.0, 100.0),
            None
        );
        assert_eq!(
            AspectRatio::Custom(f32::NAN).numerical_ratio(100.0, 100.0),
            None
        );
    }

    #[test]
    fn test_external_texture_view_fit_pillarbox_wide_container() {
        let container = Rect::new(100.0, 50.0, 1600.0, 800.0); // 2:1 container
        let view = ExternalTextureView::new(1, 1920.0, 1080.0) // 16:9 texture
            .with_aspect_ratio(AspectRatio::Fit);

        let fitted = view.compute_fitted_rect(container);
        assert_eq!(fitted.height, 800.0);
        let expected_w = 800.0 * (16.0 / 9.0);
        assert!((fitted.width - expected_w).abs() < 1e-3);
        let expected_x = 100.0 + (1600.0 - expected_w) * 0.5;
        assert!((fitted.x - expected_x).abs() < 1e-3);
        assert_eq!(fitted.y, 50.0);
    }

    #[test]
    fn test_external_texture_view_fit_letterbox_tall_container() {
        let container = Rect::new(10.0, 20.0, 800.0, 1200.0); // tall container
        let view = ExternalTextureView::new(1, 1920.0, 1080.0) // 16:9 texture
            .with_aspect_ratio(AspectRatio::Fit);

        let fitted = view.compute_fitted_rect(container);
        assert_eq!(fitted.width, 800.0);
        let expected_h = 800.0 / (16.0 / 9.0);
        assert!((fitted.height - expected_h).abs() < 1e-3);
        let expected_y = 20.0 + (1200.0 - expected_h) * 0.5;
        assert!((fitted.y - expected_y).abs() < 1e-3);
        assert_eq!(fitted.x, 10.0);
    }

    #[test]
    fn test_external_texture_view_fill_mode() {
        let container = Rect::new(0.0, 0.0, 1000.0, 1000.0); // 1:1 container
        let view = ExternalTextureView::new(2, 1600.0, 900.0) // 16:9 texture
            .with_aspect_ratio(AspectRatio::Fill);

        let filled = view.compute_fitted_rect(container);
        // In 1:1 container with 16:9 texture, fill scales by height so height=1000, width=1000*(16/9) = 1777.77
        assert_eq!(filled.height, 1000.0);
        let expected_w = 1000.0 * (16.0 / 9.0);
        assert!((filled.width - expected_w).abs() < 1e-3);
        let expected_x = (1000.0 - expected_w) * 0.5;
        assert!((filled.x - expected_x).abs() < 1e-3);
    }

    #[test]
    fn test_external_texture_view_stretch_mode() {
        let container = Rect::new(50.0, 60.0, 1280.0, 720.0);
        let view =
            ExternalTextureView::new(3, 1920.0, 1080.0).with_aspect_ratio(AspectRatio::Stretch);

        let fitted = view.compute_fitted_rect(container);
        assert_eq!(fitted, container);
    }

    #[test]
    fn test_external_texture_view_custom_ratio() {
        let container = Rect::new(0.0, 0.0, 1000.0, 500.0);
        let view = ExternalTextureView::new(4, 100.0, 100.0)
            .with_aspect_ratio(AspectRatio::Custom(4.0 / 3.0));

        let fitted = view.compute_fitted_rect(container);
        assert_eq!(fitted.height, 500.0);
        let expected_w = 500.0 * (4.0 / 3.0);
        assert!((fitted.width - expected_w).abs() < 1e-3);
    }

    #[test]
    fn test_external_texture_view_degenerate_bounds_safety() {
        let view = ExternalTextureView::new(5, 1920.0, 1080.0).with_aspect_ratio(AspectRatio::Fit);
        assert_eq!(view.compute_fitted_rect(Rect::ZERO), Rect::ZERO);
        let neg_container = Rect::new(0.0, 0.0, -100.0, 50.0);
        assert_eq!(view.compute_fitted_rect(neg_container), neg_container);
    }

    #[test]
    fn test_external_texture_view_render_emits_node_in_tree() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("Root created");
        let parent = tree.create_node();
        if let Some(node) = tree.get_mut(parent) {
            node.computed_rect = Rect::new(10.0, 20.0, 800.0, 600.0);
        }
        let _ = tree.add_child(root, parent);

        {
            let mut scope = UiScope::new(&mut tree, parent);
            let view =
                ExternalTextureView::new(42, 1920.0, 1080.0).with_aspect_ratio(AspectRatio::Fit);
            view.render(&mut scope);
        }

        let parent_node = tree.get(parent).expect("Parent node exists");
        assert_eq!(parent_node.children.len(), 1);
        let child_id = parent_node.children[0];
        let child_node = tree.get(child_id).expect("Child quad exists");
        assert_eq!(child_node.name.as_deref(), Some("ExternalTextureView"));
        assert_eq!(child_node.external_texture, Some(ExternalTextureId(42)));
        assert!(!child_node.interactive);
        assert_eq!(
            child_node.style.position,
            iris_core::style::Position::Absolute
        );
    }
}