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
}

impl<'a> TreeCompilerOptions<'a> {
    /// Creates a new default compiler options structure without clipping or custom drawers.
    #[inline]
    pub fn new() -> Self {
        Self::default()
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
    };
    populate_layer_commands(tree, root_id, &mut ctx, &mut layer_lists);
    output.clear();
    for list in layer_lists {
        output.append(list);
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
    let is_focused = node_matches_focus || (ctx.inherited_focused && node.tag == 0);

    // Skip drawing text input caret if the field is not focused or currently in blink-off phase
    if node.role == WidgetRole::TextInputCaret && (!ctx.inherited_focused || !ctx.blink_caret) {
        return;
    }

    let effective_layer = if node.layer > ctx.inherited_layer {
        node.layer
    } else {
        ctx.inherited_layer
    };

    // Decouple scissor clip when transitioning into an elevated overlay layer
    // so that child popups or modal cards are not clipped by parent panel boundaries.
    let effective_clip = if effective_layer > ctx.inherited_layer {
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
        target_list.push_quad(QuadInstance::from_style(
            node.computed_rect,
            &effective_style,
            effective_clip,
        ));
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

    // 4. Custom Drawing Hook
    if let Some(drawer) = ctx.custom_drawer.as_deref_mut() {
        drawer(target_list, current, node, effective_clip);
    }

    // 5. Traverse Children
    let prev_clip = ctx.clip_rect;
    let prev_layer = ctx.inherited_layer;
    let prev_hovered = ctx.inherited_hovered;
    let prev_focused = ctx.inherited_focused;

    ctx.clip_rect = child_clip;
    ctx.inherited_layer = effective_layer;
    ctx.inherited_hovered = is_hovered;
    ctx.inherited_focused = is_focused;

    for &child_id in &node.children {
        populate_layer_commands(tree, child_id, ctx, layer_lists);
    }

    ctx.clip_rect = prev_clip;
    ctx.inherited_layer = prev_layer;
    ctx.inherited_hovered = prev_hovered;
    ctx.inherited_focused = prev_focused;
}

#[cfg(test)]
mod tests {
    use super::*;
    use iris_core::style::Style;

    #[test]
    fn test_tree_compiler_layer_sorting() {
        let mut tree = UiTree::new();
        let root = tree.create_node();
        if let Some(node) = tree.get_mut(root) {
            node.computed_rect = Rect::new(0.0, 0.0, 800.0, 600.0);
            node.style = Style::new().background(Color::BLACK);
            node.layer = UiLayer::Background;
        }

        let content = tree.create_node();
        if let Some(node) = tree.get_mut(content) {
            node.computed_rect = Rect::new(50.0, 50.0, 200.0, 100.0);
            node.style = Style::new().background(Color::WHITE);
            node.layer = UiLayer::Content;
        }
        let _ = tree.add_child(root, content);

        let modal = tree.create_node();
        if let Some(node) = tree.get_mut(modal) {
            node.computed_rect = Rect::new(100.0, 100.0, 300.0, 200.0);
            node.style = Style::new().background(Color::RED);
            node.layer = UiLayer::Modal;
        }
        let _ = tree.add_child(content, modal);

        let mut options = TreeCompilerOptions::new();
        let cmd_list = compile_tree_draw_commands(&tree, root, &mut options);

        // 3 quads should be present in order: Background, Content, Modal
        assert_eq!(cmd_list.quads.len(), 3);
        assert_eq!(cmd_list.quads[0].rect[2], 800.0); // Background
        assert_eq!(cmd_list.quads[1].rect[2], 200.0); // Content
        assert_eq!(cmd_list.quads[2].rect[2], 300.0); // Modal
    }

    #[test]
    fn test_tree_compiler_clip_decoupling_for_elevated_layer() {
        let mut tree = UiTree::new();
        let root = tree.create_node();
        if let Some(node) = tree.get_mut(root) {
            node.computed_rect = Rect::new(0.0, 0.0, 400.0, 400.0);
            node.style = Style::new().clip_children(true);
            node.layer = UiLayer::Content;
        }

        // Popup node child inside clipped container
        let popup = tree.create_node();
        if let Some(node) = tree.get_mut(popup) {
            node.computed_rect = Rect::new(300.0, 300.0, 200.0, 200.0);
            node.style = Style::new().background(Color::BLUE);
            node.layer = UiLayer::Popup;
        }
        let _ = tree.add_child(root, popup);

        let mut options = TreeCompilerOptions::new();
        let cmd_list = compile_tree_draw_commands(&tree, root, &mut options);

        assert_eq!(cmd_list.quads.len(), 1);
        // Scissor clip must be decoupled (0.0, 0.0, 0.0, 0.0) for elevated layer
        assert_eq!(cmd_list.quads[0].clip_rect, [0.0, 0.0, 0.0, 0.0]);
    }

    #[test]
    fn test_tree_compiler_custom_hook() {
        let mut tree = UiTree::new();
        let root = tree.create_node();
        if let Some(node) = tree.get_mut(root) {
            node.computed_rect = Rect::new(0.0, 0.0, 100.0, 100.0);
        }

        let mut custom_called = false;
        let mut custom_drawer =
            |_list: &mut DrawCommandList, id: WidgetId, _node: &WidgetNode, _clip: Option<Rect>| {
                if id == root {
                    custom_called = true;
                }
            };

        let mut options = TreeCompilerOptions::new().with_custom_drawer(&mut custom_drawer);
        let _ = compile_tree_draw_commands(&tree, root, &mut options);

        assert!(custom_called);
    }

    #[test]
    fn test_tree_compiler_hover_style_in_place() {
        let mut tree = UiTree::new();
        let root = tree.create_node();
        let btn_tag = 999u64;
        if let Some(node) = tree.get_mut(root) {
            node.computed_rect = Rect::new(0.0, 0.0, 100.0, 30.0);
            node.tag = btn_tag;
            node.style = Style::new()
                .background(Color::BLACK)
                .hover_background(Color::RED);
        }

        // 1. Unhovered compilation
        let mut normal_options = TreeCompilerOptions::new();
        let normal_cmds = compile_tree_draw_commands(&tree, root, &mut normal_options);
        assert_eq!(normal_cmds.quads.len(), 1);
        assert_eq!(
            normal_cmds.quads[0].color,
            Color::BLACK.to_linear().to_array()
        );

        // 2. Hovered compilation with exact tag
        let mut hover_options = TreeCompilerOptions::new().with_hovered_tag(Some(btn_tag));
        let hover_cmds = compile_tree_draw_commands(&tree, root, &mut hover_options);
        assert_eq!(hover_cmds.quads.len(), 1);
        assert_eq!(hover_cmds.quads[0].color, Color::RED.to_linear().to_array());

        // Tree node in memory was never mutated
        assert_eq!(tree.get(root).unwrap().style.background_color, Color::BLACK);
    }

    #[test]
    fn test_tree_compiler_focus_style_in_place() {
        let mut tree = UiTree::new();
        let root = tree.create_node();
        let input_tag = 1234u64;
        if let Some(node) = tree.get_mut(root) {
            node.computed_rect = Rect::new(0.0, 0.0, 150.0, 30.0);
            node.tag = input_tag;
            node.role = WidgetRole::TextInput;
            node.style = Style::new()
                .background(Color::BLACK)
                .border(1.0, Color::rgb(0.5, 0.5, 0.5))
                .focus_border(2.0, Color::CYAN);
        }

        let caret = tree.create_node();
        if let Some(node) = tree.get_mut(caret) {
            node.computed_rect = Rect::new(10.0, 5.0, 1.5, 20.0);
            node.role = WidgetRole::TextInputCaret;
            node.style = Style::new().background(Color::WHITE);
        }
        let _ = tree.add_child(root, caret);

        // 1. Unfocused compilation -> caret not rendered
        let mut unfocused_options = TreeCompilerOptions::new();
        let unfocused_cmds = compile_tree_draw_commands(&tree, root, &mut unfocused_options);
        // Only the input box quad, caret skipped
        assert_eq!(unfocused_cmds.quads.len(), 1);
        assert_eq!(
            unfocused_cmds.quads[0].border_color,
            Color::rgb(0.5, 0.5, 0.5).to_linear().to_array()
        );

        // 2. Focused compilation with blink visible -> caret rendered, focus border applied
        let mut focused_options = TreeCompilerOptions::new()
            .with_focused_tag(Some(input_tag))
            .with_blink_caret(true);
        let focused_cmds = compile_tree_draw_commands(&tree, root, &mut focused_options);
        assert_eq!(focused_cmds.quads.len(), 2);
        assert_eq!(
            focused_cmds.quads[0].border_color,
            Color::CYAN.to_linear().to_array()
        );

        // 3. Focused compilation with blink off -> caret hidden
        let mut blink_off_options = TreeCompilerOptions::new()
            .with_focused_tag(Some(input_tag))
            .with_blink_caret(false);
        let blink_off_cmds = compile_tree_draw_commands(&tree, root, &mut blink_off_options);
        assert_eq!(blink_off_cmds.quads.len(), 1);
    }
}