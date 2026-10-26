// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Layer-Aware Tree-to-Command-Buffer Compiler
//!
//! Traverses a [`UiTree`] hierarchy and generates sorted, interleaved [`DrawCommandList`] instances
//! adhering to strict layer elevation priorities (`Background -> Content -> Floating -> Modal -> Popup -> Tooltip`),
//! automatic hardware scissor clip intersection, and elevated overlay clip decoupling.

use crate::command::DrawCommandList;
use crate::external_texture_pipeline::ExternalTextureQuadInstance;
use crate::quad::QuadInstance;
use crate::texture_pipeline::TextureQuadInstance;
use iris_core::color::Color;
use iris_core::geometry::{Border, Rect};
use iris_core::id::WidgetId;
use iris_core::node::{UiLayer, WidgetNode, WidgetRole};
use iris_core::tree::UiTree;

/// Callback function type for appending custom drawing commands during tree traversal.
pub type CustomDrawCallback<'a> =
    &'a mut dyn FnMut(&mut DrawCommandList, WidgetId, &WidgetNode, Option<Rect>);

/// Options and configuration for compiling a [`UiTree`] into a [`DrawCommandList`].
#[derive(Default)]
pub struct TreeCompilerOptions<'a> {
    /// Initial scissor clipping rectangle applied to root nodes.
    pub initial_clip: Option<Rect>,
    /// Optional hovered semantic tag for highlighting backgrounds and borders without AST re-creation.
    pub hovered_tag: Option<u64>,
    /// Optional hovered widget identifier for highlighting backgrounds and borders without AST re-creation.
    pub hovered_id: Option<WidgetId>,
    /// Optional focused semantic tag for rendering focus rings and active text input highlights.
    pub focused_tag: Option<u64>,
    /// Optional focused widget identifier for rendering focus rings and active text input highlights.
    pub focused_id: Option<WidgetId>,
    /// Whether the text input caret cursor is currently in the visible blink phase.
    pub blink_caret: bool,
    /// Optional callback for handling custom widget roles or canvas primitives at exact Z-order.
    pub custom_drawer: Option<CustomDrawCallback<'a>>,
    /// Optional inclusive layer elevation range `(min, max)` to compile.
    pub layer_range: Option<(UiLayer, UiLayer)>,
}

impl<'a> TreeCompilerOptions<'a> {
    /// Creates a new default compiler options structure without clipping or custom drawers.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets an inclusive layer elevation range `(min, max)` to filter compiled commands.
    #[must_use]
    #[inline]
    pub fn with_layer_range(mut self, min: UiLayer, max: UiLayer) -> Self {
        self.layer_range = Some((min, max));
        self
    }

    /// Sets the initial boundary clipping rectangle.
    #[inline]
    pub fn with_clip(mut self, clip: Option<Rect>) -> Self {
        self.initial_clip = clip;
        self
    }

    /// Sets the currently hovered semantic tag for dynamic hover styling.
    #[must_use]
    #[inline]
    pub fn with_hovered_tag(mut self, tag: Option<u64>) -> Self {
        self.hovered_tag = tag;
        self
    }

    /// Sets the currently hovered widget identifier for dynamic hover styling.
    #[must_use]
    #[inline]
    pub fn with_hovered_id(mut self, id: Option<WidgetId>) -> Self {
        self.hovered_id = id;
        self
    }

    /// Sets the currently focused semantic tag for dynamic focus styling.
    #[must_use]
    #[inline]
    pub fn with_focused_tag(mut self, tag: Option<u64>) -> Self {
        self.focused_tag = tag;
        self
    }

    /// Sets the currently focused widget identifier for dynamic focus styling.
    #[must_use]
    #[inline]
    pub fn with_focused_id(mut self, id: Option<WidgetId>) -> Self {
        self.focused_id = id;
        self
    }

    /// Sets whether the text caret is in the visible blink cycle.
    #[must_use]
    #[inline]
    pub fn with_blink_caret(mut self, blink: bool) -> Self {
        self.blink_caret = blink;
        self
    }

    /// Sets a custom drawing callback invoked during tree node traversal.
    #[inline]
    pub fn with_custom_drawer(mut self, drawer: CustomDrawCallback<'a>) -> Self {
        self.custom_drawer = Some(drawer);
        self
    }
}

/// Internal context holding traversal parameters during tree compilation.
struct CompilationContext<'a, 'b> {
    clip_rect: Option<Rect>,
    inherited_layer: UiLayer,
    inherited_hovered: bool,
    inherited_focused: bool,
    hovered_tag: Option<u64>,
    hovered_id: Option<WidgetId>,
    focused_tag: Option<u64>,
    focused_id: Option<WidgetId>,
    blink_caret: bool,
    custom_drawer: &'b mut Option<CustomDrawCallback<'a>>,
    current_boundary: Option<WidgetId>,
}

/// Recursively compiles a [`UiTree`] into an existing [`DrawCommandList`],
/// sorting nodes across the 6 discrete elevation layers (`Background` to `Tooltip`).
pub fn compile_tree_draw_commands_into(
    tree: &UiTree,
    root_id: WidgetId,
    options: &mut TreeCompilerOptions<'_>,
    output: &mut DrawCommandList,
) {
    let mut layer_lists: [DrawCommandList; 6] = Default::default();
    let mut ctx = CompilationContext {
        clip_rect: options.initial_clip,
        inherited_layer: UiLayer::Background,
        inherited_hovered: false,
        inherited_focused: false,
        hovered_tag: options.hovered_tag,
        hovered_id: options.hovered_id,
        focused_tag: options.focused_tag,
        focused_id: options.focused_id,
        blink_caret: options.blink_caret,
        custom_drawer: &mut options.custom_drawer,
        current_boundary: if tree.is_boundary(root_id) {
            Some(root_id)
        } else {
            None
        },
    };
    populate_layer_commands(tree, root_id, &mut ctx, &mut layer_lists);
    output.clear();

    let mut had_scissor = false;
    if let Some(clip) = options.initial_clip {
        // Only apply initial hardware scissor if compiling base/content layers
        let includes_base_layers = options
            .layer_range
            .is_none_or(|(min, _)| min < UiLayer::Modal);
        if includes_base_layers {
            let sx = clip.x.max(0.0).floor() as u32;
            let sy = clip.y.max(0.0).floor() as u32;
            let sw = clip.width.max(1.0).ceil() as u32;
            let sh = clip.height.max(1.0).ceil() as u32;
            output.push_scissor(sx, sy, sw, sh);
            had_scissor = true;
        }
    }

    for (layer_idx, list) in layer_lists.into_iter().enumerate() {
        if let Some((min_layer, max_layer)) = options.layer_range
            && (layer_idx < min_layer.index() || layer_idx > max_layer.index())
        {
            continue;
        }
        if list.is_empty() {
            continue;
        }
        // When transitioning to elevated overlay layers (Modal, Popup, Tooltip), ensure any
        // hardware scissor pushed by lower content layers is reset so overlays are never cut off.
        if layer_idx >= UiLayer::Modal.index() && had_scissor {
            output.push_reset_scissor();
            had_scissor = false;
        }
        if list
            .commands
            .iter()
            .any(|c| matches!(c, crate::command::DrawCommand::SetScissor { .. }))
        {
            had_scissor = true;
        }
        output.append(list);
    }

    // Ensure hardware scissor does not leak into subsequent frame operations
    if had_scissor {
        output.push_reset_scissor();
    }
}

/// Recursively compiles a [`UiTree`] starting at `root_id` into a new [`DrawCommandList`].
pub fn compile_tree_draw_commands(
    tree: &UiTree,
    root_id: WidgetId,
    options: &mut TreeCompilerOptions<'_>,
) -> DrawCommandList {
    let mut command_list = DrawCommandList::new();
    compile_tree_draw_commands_into(tree, root_id, options, &mut command_list);
    command_list
}

/// Internal recursive helper traversing the UI hierarchy and sorting into per-layer command lists.
fn populate_layer_commands(
    tree: &UiTree,
    current: WidgetId,
    ctx: &mut CompilationContext<'_, '_>,
    layer_lists: &mut [DrawCommandList; 6],
) {
    let Some(node) = tree.get(current) else {
        return;
    };
    if !node.visible {
        return;
    }

    let node_matches_hover = (ctx.hovered_id.is_some() && ctx.hovered_id == Some(current))
        || (node.tag != 0 && ctx.hovered_tag == Some(node.tag));
    let is_hovered = node_matches_hover || (ctx.inherited_hovered && node.tag == 0);

    let node_matches_focus = (ctx.focused_id.is_some() && ctx.focused_id == Some(current))
        || (node.tag != 0 && ctx.focused_tag == Some(node.tag));
    let is_focused = if node.role == WidgetRole::TextInputCaret {
        ctx.inherited_focused
    } else {
        node_matches_focus || (ctx.inherited_focused && node.tag == 0)
    };

    // Skip drawing text input caret if the field is not focused or currently in blink-off phase
    if node.role == WidgetRole::TextInputCaret && (!ctx.inherited_focused || !ctx.blink_caret) {
        return;
    }

    let explicit_or_default_layer = if node.layer != UiLayer::Content {
        node.layer
    } else {
        node.role.default_layer()
    };

    let effective_layer = if explicit_or_default_layer > ctx.inherited_layer {
        explicit_or_default_layer
    } else {
        ctx.inherited_layer
    };

    let effective_boundary = if node.is_boundary {
        Some(current)
    } else {
        ctx.current_boundary
    };

    // A node is an elevated overlay root if its explicit or role-inferred layer exceeds the inherited layer,
    // or if its semantic role designates an independent floating/overlay container
    // (DropdownPopup, ModalWindow, FloatingWindow, Tooltip).
    let is_elevated_root = effective_layer > ctx.inherited_layer
        || matches!(
            node.role,
            WidgetRole::DropdownPopup
                | WidgetRole::ModalWindow
                | WidgetRole::FloatingWindow
                | WidgetRole::Tooltip
        );

    // Decouple scissor clip when transitioning into an elevated overlay layer
    // so that child popups or modal cards are not clipped by parent panel boundaries.
    let effective_clip = if is_elevated_root {
        None
    } else {
        ctx.clip_rect
    };

    let child_clip = if node.style.clip_children {
        match effective_clip {
            Some(existing) => Some(existing.intersect(node.computed_rect)),
            None => Some(node.computed_rect),
        }
    } else {
        effective_clip
    };

    let effective_style = if is_focused {
        let mut s = node.style;
        if let Some(bg) = s.focus_background {
            s.background_color = bg;
        } else if is_hovered && let Some(bg) = s.hover_background {
            s.background_color = bg;
        }
        if let Some(border) = s.focus_border {
            s.border = border;
        } else if node.role == WidgetRole::TextInput {
            s.border = Border::uniform(1.0, Color::rgba(0.0, 0.90, 1.0, 0.95));
        } else if is_hovered && let Some(border) = s.hover_border {
            s.border = border;
        }
        s
    } else if is_hovered {
        let mut s = node.style;
        if let Some(bg) = s.hover_background {
            s.background_color = bg;
        } else if matches!(
            node.role,
            WidgetRole::Button
                | WidgetRole::MenuBarItem
                | WidgetRole::DropdownItem
                | WidgetRole::DockTab
        ) && s.background_color.a > 0.01
        {
            s.background_color = Color::rgba(
                (s.background_color.r * 1.25).min(1.0),
                (s.background_color.g * 1.25).min(1.0),
                (s.background_color.b * 1.25).min(1.0),
                s.background_color.a,
            );
        }
        if let Some(border) = s.hover_border {
            s.border = border;
        } else if matches!(
            node.role,
            WidgetRole::Button
                | WidgetRole::MenuBarItem
                | WidgetRole::DropdownItem
                | WidgetRole::DockTab
        ) && (s.border.width.top > 0.0
            || s.border.width.bottom > 0.0
            || s.border.width.left > 0.0
            || s.border.width.right > 0.0)
            && s.border.color.a > 0.01
        {
            let mut b = s.border;
            b.color = Color::rgba(
                (b.color.r * 1.30).min(1.0),
                (b.color.g * 1.30).min(1.0),
                (b.color.b * 1.30).min(1.0),
                b.color.a,
            );
            s.border = b;
        }
        s
    } else {
        node.style
    };

    let has_border = (effective_style.border.width.top > 0.0
        || effective_style.border.width.bottom > 0.0
        || effective_style.border.width.left > 0.0
        || effective_style.border.width.right > 0.0)
        && effective_style.border.color.a > 0.0;

    let target_list = &mut layer_lists[effective_layer.index()];

    // 1. SDF Quad (Background, Border, Shadow)
    if node.computed_rect.width > 0.0
        && node.computed_rect.height > 0.0
        && (effective_style.background_color.a > 0.0
            || has_border
            || effective_style.box_shadow.is_some())
    {
        target_list.push_quad_with_boundary(
            QuadInstance::from_style(node.computed_rect, &effective_style, effective_clip),
            effective_boundary,
        );
    }

    // 2. Atlas Texture Quad
    if let Some(uv) = node.texture_uv
        && node.computed_rect.width > 0.0
        && node.computed_rect.height > 0.0
    {
        let base_tint = node.texture_tint.unwrap_or(Color::WHITE);
        let tint = if is_hovered {
            node.hover_texture_tint.unwrap_or(base_tint)
        } else {
            base_tint
        };
        let clip_arr = match effective_clip {
            Some(c) => [c.x, c.y, c.x + c.width, c.y + c.height],
            None => [0.0, 0.0, 0.0, 0.0],
        };
        target_list.push_texture_quad(TextureQuadInstance {
            rect: [
                node.computed_rect.x,
                node.computed_rect.y,
                node.computed_rect.width,
                node.computed_rect.height,
            ],
            uv_rect: uv,
            tint: [tint.r, tint.g, tint.b, tint.a],
            clip_rect: clip_arr,
        });
    }

    // 3. External Texture Quad
    if let Some(id) = node.external_texture
        && node.computed_rect.width > 0.0
        && node.computed_rect.height > 0.0
    {
        let tint = node.texture_tint.unwrap_or(Color::WHITE);
        let uv = node.texture_uv.unwrap_or([0.0, 0.0, 1.0, 1.0]);
        target_list.push_external_texture_quad(
            id,
            ExternalTextureQuadInstance::with_uv(node.computed_rect, uv, tint, effective_clip),
        );
    }

    // 4. Volatile Immediate Canvas Primitives
    if let Some(canvas_primitives) = tree.canvas_primitives(current) {
        emit_canvas_primitives(
            canvas_primitives,
            target_list,
            effective_clip,
            effective_boundary,
        );
    }

    // 5. Custom Drawing Hook
    if let Some(drawer) = ctx.custom_drawer.as_deref_mut() {
        drawer(target_list, current, node, effective_clip);
    }

    // 6. Traverse Children
    let prev_clip = ctx.clip_rect;
    let prev_layer = ctx.inherited_layer;
    let prev_hovered = ctx.inherited_hovered;
    let prev_focused = ctx.inherited_focused;
    let prev_boundary = ctx.current_boundary;

    ctx.clip_rect = child_clip;
    ctx.inherited_layer = effective_layer;
    ctx.inherited_hovered = is_hovered;
    ctx.inherited_focused = is_focused;
    ctx.current_boundary = effective_boundary;

    for &child_id in &node.children {
        populate_layer_commands(tree, child_id, ctx, layer_lists);
    }

    ctx.clip_rect = prev_clip;
    ctx.inherited_layer = prev_layer;
    ctx.inherited_hovered = prev_hovered;
    ctx.inherited_focused = prev_focused;
    ctx.current_boundary = prev_boundary;
}

/// Translates volatile immediate-mode canvas primitives into instanced GPU draw quads.
fn emit_canvas_primitives(
    primitives: &[iris_core::CanvasPrimitive],
    target_list: &mut DrawCommandList,
    effective_clip: Option<Rect>,
    boundary: Option<WidgetId>,
) {
    for primitive in primitives {
        match primitive {
            iris_core::CanvasPrimitive::Rect {
                rect,
                color,
                corner_radius,
            } => {
                target_list.push_quad_with_boundary(
                    QuadInstance::from_style(
                        *rect,
                        &iris_core::Style::new()
                            .background(*color)
                            .border_radius(*corner_radius),
                        effective_clip,
                    ),
                    boundary,
                );
            }
            iris_core::CanvasPrimitive::Border {
                rect,
                border_width,
                color,
                corner_radius,
            } => {
                target_list.push_quad_with_boundary(
                    QuadInstance::from_style(
                        *rect,
                        &iris_core::Style::new()
                            .border(*border_width, *color)
                            .border_radius(*corner_radius),
                        effective_clip,
                    ),
                    boundary,
                );
            }
            iris_core::CanvasPrimitive::Line {
                start,
                end,
                thickness,
                color,
            } => {
                let seg_min_x = start.x.min(end.x);
                let seg_min_y = start.y.min(end.y);
                let seg_w = (end.x - start.x).abs().max(*thickness);
                let seg_h = (end.y - start.y).abs().max(*thickness);

                target_list.push_quad_with_boundary(
                    QuadInstance::from_style(
                        Rect::new(seg_min_x, seg_min_y, seg_w, seg_h),
                        &iris_core::Style::new()
                            .background(*color)
                            .border_radius(*thickness * 0.5),
                        effective_clip,
                    ),
                    boundary,
                );
            }
            iris_core::CanvasPrimitive::Text { .. } => {
                // Volatile canvas text primitives are shaped and rendered by the text collection pipeline.
            }
        }
    }
}

#[cfg(test)]
#[path = "tree_compiler_tests.rs"]
mod tests;