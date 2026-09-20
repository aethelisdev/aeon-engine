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
use iris_core::geometry::Rect;
use iris_core::id::WidgetId;
use iris_core::node::{UiLayer, WidgetNode};
use iris_core::tree::UiTree;

/// Callback function type for appending custom drawing commands during tree traversal.
pub type CustomDrawCallback<'a> =
    &'a mut dyn FnMut(&mut DrawCommandList, WidgetId, &WidgetNode, Option<Rect>);

/// Options and configuration for compiling a [`UiTree`] into a [`DrawCommandList`].
#[derive(Default)]
pub struct TreeCompilerOptions<'a> {
    /// Initial scissor clipping rectangle applied to root nodes.
    pub initial_clip: Option<Rect>,
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

    /// Sets a custom drawing callback invoked during tree node traversal.
    #[inline]
    pub fn with_custom_drawer(mut self, drawer: CustomDrawCallback<'a>) -> Self {
        self.custom_drawer = Some(drawer);
        self
    }
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
    populate_layer_commands(
        tree,
        root_id,
        options.initial_clip,
        UiLayer::Background,
        &mut options.custom_drawer,
        &mut layer_lists,
    );
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
    clip_rect: Option<Rect>,
    inherited_layer: UiLayer,
    custom_drawer: &mut Option<CustomDrawCallback<'_>>,
    layer_lists: &mut [DrawCommandList; 6],
) {
    let Some(node) = tree.get(current) else {
        return;
    };
    if !node.visible {
        return;
    }

    let effective_layer = if node.layer > inherited_layer {
        node.layer
    } else {
        inherited_layer
    };

    // Decouple scissor clip when transitioning into an elevated overlay layer
    // so that child popups or modal cards are not clipped by parent panel boundaries.
    let effective_clip = if effective_layer > inherited_layer {
        None
    } else {
        clip_rect
    };

    let child_clip = if node.style.clip_children {
        match effective_clip {
            Some(existing) => Some(existing.intersect(node.computed_rect)),
            None => Some(node.computed_rect),
        }
    } else {
        effective_clip
    };

    let has_border = (node.style.border.width.top > 0.0
        || node.style.border.width.bottom > 0.0
        || node.style.border.width.left > 0.0
        || node.style.border.width.right > 0.0)
        && node.style.border.color.a > 0.0;

    let target_list = &mut layer_lists[effective_layer.index()];

    // 1. SDF Quad (Background, Border, Shadow)
    if node.computed_rect.width > 0.0
        && node.computed_rect.height > 0.0
        && (node.style.background_color.a > 0.0 || has_border || node.style.box_shadow.is_some())
    {
        target_list.push_quad(QuadInstance::from_style(
            node.computed_rect,
            &node.style,
            effective_clip,
        ));
    }

    // 2. Atlas Texture Quad
    if let Some(uv) = node.texture_uv
        && node.computed_rect.width > 0.0
        && node.computed_rect.height > 0.0
    {
        let tint = node.texture_tint.unwrap_or(Color::WHITE);
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
    if let Some(drawer) = custom_drawer {
        drawer(target_list, current, node, effective_clip);
    }

    // 5. Traverse Children
    for &child_id in &node.children {
        populate_layer_commands(
            tree,
            child_id,
            child_clip,
            effective_layer,
            custom_drawer,
            layer_lists,
        );
    }
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
}