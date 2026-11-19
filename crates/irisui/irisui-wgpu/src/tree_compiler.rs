// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Layer-Aware Tree-to-Command-Buffer Compiler
//!
//! Traverses a [`UiTree`] hierarchy and generates sorted, interleaved [`DrawCommandList`] instances
//! adhering to strict layer elevation priorities (`Background -> Content -> Floating -> Modal -> Popup -> Tooltip`),
//! automatic hardware scissor clip intersection, and elevated overlay clip decoupling.
#![deny(clippy::unwrap_used)]

use crate::command::DrawCommandList;
use crate::external_texture_pipeline::ExternalTextureQuadInstance;
use crate::quad::QuadInstance;
use crate::scissor::ScissorStack;
use crate::texture_pipeline::TextureQuadInstance;
use irisui_core::color::Color;
use irisui_core::geometry::{Border, Rect};
use irisui_core::id::WidgetId;
use irisui_core::node::{UiLayer, WidgetNode, WidgetRole};
use irisui_core::tree::UiTree;

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

    /// Populates hover and focus state tokens directly from the UI tree interaction state.
    #[must_use]
    #[inline]
    pub fn with_tree_focus(mut self, tree: &UiTree) -> Self {
        self.hovered_id = tree.event_state.hovered;
        self.focused_id = tree.event_state.focused;
        self.focused_tag = tree.event_state.focused_tag;
        self
    }

    /// Sets a custom drawing callback invoked during tree node traversal.
    #[inline]
    pub fn with_custom_drawer(mut self, drawer: CustomDrawCallback<'a>) -> Self {
        self.custom_drawer = Some(drawer);
        self
    }
}

pub use crate::tree_compiler_patch::{
    TreeCompilerScratch, compile_tree_draw_commands_with_scratch,
    patch_dirty_boundaries_with_scratch,
};

/// Internal context holding traversal parameters during tree compilation.
pub(crate) struct CompilationContext<'a, 'b> {
    pub(crate) scissor_stack: ScissorStack,
    pub(crate) inherited_layer: UiLayer,
    pub(crate) inherited_hovered: bool,
    pub(crate) inherited_focused: bool,
    pub(crate) hovered_tag: Option<u64>,
    pub(crate) hovered_id: Option<WidgetId>,
    pub(crate) focused_tag: Option<u64>,
    pub(crate) focused_id: Option<WidgetId>,
    pub(crate) blink_caret: bool,
    pub(crate) custom_drawer: &'b mut Option<CustomDrawCallback<'a>>,
    pub(crate) current_boundary: Option<WidgetId>,
}

/// Recursively compiles a [`UiTree`] into an existing [`DrawCommandList`],
/// sorting nodes across the 6 discrete elevation layers (`Background` to `Tooltip`).
pub fn compile_tree_draw_commands_into(
    tree: &UiTree,
    root_id: WidgetId,
    options: &mut TreeCompilerOptions<'_>,
    output: &mut DrawCommandList,
) {
    let mut scratch = TreeCompilerScratch::new();
    compile_tree_draw_commands_with_scratch(tree, root_id, options, &mut scratch, output);
}

#[inline]
pub(crate) fn resolve_node_hover(
    tree: &UiTree,
    id: WidgetId,
    node: &WidgetNode,
    hovered_id: Option<WidgetId>,
    hovered_tag: Option<u64>,
    inherited_hovered: bool,
) -> bool {
    let direct = (hovered_id.is_some() && hovered_id == Some(id))
        || (node.tag != 0 && hovered_tag == Some(node.tag))
        || (node.role == WidgetRole::DockTab
            && hovered_id.is_some_and(|hid| tree.is_descendant_of(hid, id)));
    direct || (inherited_hovered && node.tag == 0)
}

#[inline]
pub(crate) fn resolve_node_focus(
    id: WidgetId,
    node: &WidgetNode,
    focused_id: Option<WidgetId>,
    focused_tag: Option<u64>,
    inherited_focused: bool,
) -> bool {
    if node.role == WidgetRole::TextInputCaret {
        return inherited_focused;
    }
    let direct = (focused_id.is_some() && focused_id == Some(id))
        || (node.tag != 0 && focused_tag == Some(node.tag));
    direct || (inherited_focused && node.tag == 0)
}

#[inline]
pub(crate) fn resolve_effective_layer(node: &WidgetNode, inherited_layer: UiLayer) -> UiLayer {
    let explicit_or_default = if node.layer != UiLayer::Content {
        node.layer
    } else {
        node.role.default_layer()
    };
    if explicit_or_default > inherited_layer {
        explicit_or_default
    } else {
        inherited_layer
    }
}

#[inline]
pub(crate) fn is_node_elevated_root(
    node: &WidgetNode,
    effective_layer: UiLayer,
    inherited_layer: UiLayer,
) -> bool {
    effective_layer > inherited_layer
        || matches!(
            node.role,
            WidgetRole::DropdownPopup
                | WidgetRole::ModalWindow
                | WidgetRole::FloatingWindow
                | WidgetRole::Tooltip
        )
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
pub(crate) fn populate_layer_commands(
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

    let is_hovered = resolve_node_hover(
        tree,
        current,
        node,
        ctx.hovered_id,
        ctx.hovered_tag,
        ctx.inherited_hovered,
    );
    let is_focused = resolve_node_focus(
        current,
        node,
        ctx.focused_id,
        ctx.focused_tag,
        ctx.inherited_focused,
    );

    // Skip drawing text input caret if the field is not focused or currently in blink-off phase
    if node.role == WidgetRole::TextInputCaret && (!ctx.inherited_focused || !ctx.blink_caret) {
        return;
    }

    let effective_layer = resolve_effective_layer(node, ctx.inherited_layer);

    let effective_boundary = if node.is_boundary {
        Some(current)
    } else {
        ctx.current_boundary
    };

    // A node is an elevated overlay root if its explicit or role-inferred layer exceeds the inherited layer,
    // or if its semantic role designates an independent floating/overlay container
    // (DropdownPopup, ModalWindow, FloatingWindow, Tooltip).
    let is_elevated_root = is_node_elevated_root(node, effective_layer, ctx.inherited_layer);

    // Decouple scissor clip when transitioning into an elevated overlay layer
    // so that child popups or modal cards are not clipped by parent panel boundaries.
    let effective_clip = if is_elevated_root {
        None
    } else {
        ctx.scissor_stack.current()
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
    let is_clip_empty = effective_clip.is_some_and(|c| c.width <= 0.0 || c.height <= 0.0);

    if !is_clip_empty {
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
            target_list.push_texture_quad_with_boundary(
                TextureQuadInstance {
                    rect: [
                        node.computed_rect.x,
                        node.computed_rect.y,
                        node.computed_rect.width,
                        node.computed_rect.height,
                    ],
                    uv_rect: uv,
                    tint: [tint.r, tint.g, tint.b, tint.a],
                    clip_rect: clip_arr,
                },
                effective_boundary,
            );
        }

        // 3. External Texture Quad
        if let Some(id) = node.external_texture
            && node.computed_rect.width > 0.0
            && node.computed_rect.height > 0.0
        {
            let tint = node.texture_tint.unwrap_or(Color::WHITE);
            let uv = node.texture_uv.unwrap_or([0.0, 0.0, 1.0, 1.0]);
            target_list.push_external_texture_quad_with_boundary(
                id,
                ExternalTextureQuadInstance::with_uv(node.computed_rect, uv, tint, effective_clip),
                effective_boundary,
            );
        }

        // 4. Volatile Immediate Canvas Primitives
        if let Some(canvas_primitives) = tree.canvas_primitives(current) {
            emit_canvas_primitives(
                node.canvas_origin,
                node.computed_rect,
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
    }

    // 6. Traverse Children
    let prev_layer = ctx.inherited_layer;
    let prev_hovered = ctx.inherited_hovered;
    let prev_focused = ctx.inherited_focused;
    let prev_boundary = ctx.current_boundary;

    ctx.inherited_layer = effective_layer;
    ctx.inherited_hovered = is_hovered;
    ctx.inherited_focused = is_focused;
    ctx.current_boundary = effective_boundary;

    if is_elevated_root {
        // Elevated layers decouple completely from parent scissor stack
        let saved_stack = std::mem::take(&mut ctx.scissor_stack);
        let pushed_clip = if node.style.clip_children {
            ctx.scissor_stack.push(node.computed_rect);
            true
        } else {
            false
        };

        for &child_id in &node.children {
            populate_layer_commands(tree, child_id, ctx, layer_lists);
        }

        if pushed_clip {
            ctx.scissor_stack.pop();
        }
        ctx.scissor_stack = saved_stack;
    } else {
        let pushed_clip = if node.style.clip_children {
            ctx.scissor_stack.push(node.computed_rect);
            true
        } else {
            false
        };

        for &child_id in &node.children {
            populate_layer_commands(tree, child_id, ctx, layer_lists);
        }

        if pushed_clip {
            ctx.scissor_stack.pop();
        }
    }

    ctx.inherited_layer = prev_layer;
    ctx.inherited_hovered = prev_hovered;
    ctx.inherited_focused = prev_focused;
    ctx.current_boundary = prev_boundary;
}

/// Translates volatile immediate-mode canvas primitives into instanced GPU draw quads.
fn emit_canvas_primitives(
    origin_mode: irisui_core::CanvasOrigin,
    node_rect: Rect,
    primitives: &[irisui_core::CanvasPrimitive],
    target_list: &mut DrawCommandList,
    effective_clip: Option<Rect>,
    boundary: Option<WidgetId>,
) {
    let origin = origin_mode.resolve_offset(node_rect);
    for primitive in primitives {
        match primitive {
            irisui_core::CanvasPrimitive::Rect {
                rect,
                color,
                corner_radius,
            } => {
                let translated = Rect::new(
                    rect.x + origin.x,
                    rect.y + origin.y,
                    rect.width,
                    rect.height,
                );
                target_list.push_quad_with_boundary(
                    QuadInstance::from_style(
                        translated,
                        &irisui_core::Style::new()
                            .background(*color)
                            .border_radius(*corner_radius),
                        effective_clip,
                    ),
                    boundary,
                );
            }
            irisui_core::CanvasPrimitive::StyledRect {
                rect,
                color,
                border_color,
                border_width,
                corner_radius,
            } => {
                let translated = Rect::new(
                    rect.x + origin.x,
                    rect.y + origin.y,
                    rect.width,
                    rect.height,
                );
                target_list.push_quad_with_boundary(
                    QuadInstance::from_style(
                        translated,
                        &irisui_core::Style::new()
                            .background(*color)
                            .border(*border_width, *border_color)
                            .border_radius(*corner_radius),
                        effective_clip,
                    ),
                    boundary,
                );
            }
            irisui_core::CanvasPrimitive::Border {
                rect,
                border_width,
                color,
                corner_radius,
            } => {
                let translated = Rect::new(
                    rect.x + origin.x,
                    rect.y + origin.y,
                    rect.width,
                    rect.height,
                );
                target_list.push_quad_with_boundary(
                    QuadInstance::from_style(
                        translated,
                        &irisui_core::Style::new()
                            .border(*border_width, *color)
                            .border_radius(*corner_radius),
                        effective_clip,
                    ),
                    boundary,
                );
            }
            irisui_core::CanvasPrimitive::Line {
                start,
                end,
                thickness,
                color,
            } => {
                let sx = start.x + origin.x;
                let sy = start.y + origin.y;
                let ex = end.x + origin.x;
                let ey = end.y + origin.y;

                target_list.push_quad_with_boundary(
                    QuadInstance::line(
                        irisui_core::Point::new(sx, sy),
                        irisui_core::Point::new(ex, ey),
                        *color,
                        *thickness,
                        effective_clip,
                    ),
                    boundary,
                );
            }
            irisui_core::CanvasPrimitive::Text { .. } => {
                // Volatile canvas text primitives are shaped and rendered by the text collection pipeline.
            }
        }
    }
}

#[cfg(test)]
#[path = "tree_compiler_tests.rs"]
mod tests;