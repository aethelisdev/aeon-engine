// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Scratch-Buffered Tree Compilation & In-Place Boundary Patching
//!
//! Eliminates per-frame layer buffer allocations via [`TreeCompilerScratch`] and enables
//! $O(M)$ in-place boundary patching inside existing [`DrawCommandList`] buffers when
//! hover, focus, or live canvas state mutates without altering slice counts or layout.

use crate::command::{BoundarySlice, DrawCommand, DrawCommandList};
use crate::scissor::ScissorStack;
use crate::tree_compiler::{
    CompilationContext, TreeCompilerOptions, is_node_elevated_root, populate_layer_commands,
    resolve_effective_layer, resolve_node_focus, resolve_node_hover,
};
use irisui_core::id::WidgetId;
use irisui_core::node::UiLayer;
use irisui_core::tree::UiTree;

/// Persistent scratch buffers reused across frame compilations to eliminate
/// temporary [`DrawCommandList`] heap allocations inside the tree compiler.
#[derive(Debug, Default)]
pub struct TreeCompilerScratch {
    layer_lists: [DrawCommandList; 6],
    dirty_boundaries: Vec<WidgetId>,
}

impl TreeCompilerScratch {
    /// Creates a new reusable [`TreeCompilerScratch`] buffer set.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

/// Compiles a [`UiTree`] into `output` reusing persistent heap buffers in `scratch`.
///
/// Eliminates per-frame temporary vector allocations for the 6 elevation layer buckets once
/// steady-state capacity is reached.
pub fn compile_tree_draw_commands_with_scratch(
    tree: &UiTree,
    root_id: WidgetId,
    options: &mut TreeCompilerOptions<'_>,
    scratch: &mut TreeCompilerScratch,
    output: &mut DrawCommandList,
) {
    for list in &mut scratch.layer_lists {
        list.clear();
    }
    let mut ctx = CompilationContext {
        scissor_stack: ScissorStack::with_initial_clip(options.initial_clip),
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
    populate_layer_commands(tree, root_id, &mut ctx, &mut scratch.layer_lists);
    output.clear();

    let mut had_scissor = false;
    if let Some(clip) = options.initial_clip {
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

    for (layer_idx, list) in scratch.layer_lists.iter_mut().enumerate() {
        if let Some((min_layer, max_layer)) = options.layer_range
            && (layer_idx < min_layer.index() || layer_idx > max_layer.index())
        {
            continue;
        }
        if list.is_empty() {
            continue;
        }
        if layer_idx >= UiLayer::Modal.index() && had_scissor {
            output.push_reset_scissor();
            had_scissor = false;
        }
        if list
            .commands
            .iter()
            .any(|c| matches!(c, DrawCommand::SetScissor { .. }))
        {
            had_scissor = true;
        }
        output.append_drain(list);
    }

    if had_scissor {
        output.push_reset_scissor();
    }
}

/// Attempts to patch only `tree.dirty_boundaries()` in place inside an already compiled
/// `command_list`, avoiding full-tree traversal during hover, focus, or live canvas updates.
///
/// Returns `true` when all dirty boundaries were cleanly patched in place (or none were dirty).
/// Returns `false` if slice counts, layers, or visibility shifted, signaling the caller to execute
/// [`compile_tree_draw_commands_with_scratch`].
pub fn patch_dirty_boundaries_with_scratch(
    tree: &UiTree,
    options: &mut TreeCompilerOptions<'_>,
    scratch: &mut TreeCompilerScratch,
    command_list: &mut DrawCommandList,
) -> bool {
    if !tree.has_dirty_boundaries() {
        return !command_list.is_empty();
    }
    if command_list.is_empty() || options.layer_range.is_some() {
        return false;
    }
    let Some(root_id) = tree.root() else {
        return false;
    };

    collect_topmost_dirty_boundaries(tree, &mut scratch.dirty_boundaries);

    for i in 0..scratch.dirty_boundaries.len() {
        let b_id = scratch.dirty_boundaries[i];
        if !tree.is_boundary(b_id) {
            return false;
        }
        let Some(mut sub_ctx) = build_boundary_ancestor_context(tree, root_id, b_id, options)
        else {
            return false;
        };
        sub_ctx.current_boundary = Some(b_id);

        for list in &mut scratch.layer_lists {
            list.clear();
        }
        populate_layer_commands(tree, b_id, &mut sub_ctx, &mut scratch.layer_lists);

        let mut active_layer: Option<usize> = None;
        for (idx, list) in scratch.layer_lists.iter().enumerate() {
            if !list.is_empty() {
                if active_layer.is_some() {
                    return false;
                }
                active_layer = Some(idx);
            }
        }

        let Some(layer_idx) = active_layer else {
            if boundary_has_existing_slices(command_list, b_id) {
                return false;
            }
            continue;
        };

        let compiled = &scratch.layer_lists[layer_idx];
        let Some(quad_pos) = find_matching_slice_run(
            &command_list.boundary_slices,
            &compiled.boundary_slices,
            b_id,
        ) else {
            return false;
        };
        let Some(tex_pos) = find_matching_slice_run(
            &command_list.texture_boundary_slices,
            &compiled.texture_boundary_slices,
            b_id,
        ) else {
            return false;
        };
        let Some(ext_pos) = find_matching_slice_run(
            &command_list.external_boundary_slices,
            &compiled.external_boundary_slices,
            b_id,
        ) else {
            return false;
        };

        if let Some(pos) = quad_pos {
            apply_slice_run(
                &mut command_list.quads,
                &command_list.boundary_slices[pos..],
                &compiled.quads,
                &compiled.boundary_slices,
            );
        }
        if let Some(pos) = tex_pos {
            apply_slice_run(
                &mut command_list.texture_quads,
                &command_list.texture_boundary_slices[pos..],
                &compiled.texture_quads,
                &compiled.texture_boundary_slices,
            );
        }
        if let Some(pos) = ext_pos {
            apply_slice_run(
                &mut command_list.external_texture_quads,
                &command_list.external_boundary_slices[pos..],
                &compiled.external_texture_quads,
                &compiled.external_boundary_slices,
            );
        }
    }

    true
}

fn collect_topmost_dirty_boundaries(tree: &UiTree, out: &mut Vec<WidgetId>) {
    out.clear();
    let dirty_set = tree.dirty_boundaries();
    for &b_id in dirty_set {
        let mut anc = tree.get(b_id).and_then(|n| n.parent);
        let mut covered_by_ancestor = false;
        while let Some(parent_id) = anc {
            if dirty_set.contains(&parent_id) {
                covered_by_ancestor = true;
                break;
            }
            anc = tree.get(parent_id).and_then(|n| n.parent);
        }
        if !covered_by_ancestor {
            out.push(b_id);
        }
    }
}

#[inline]
fn boundary_has_existing_slices(command_list: &DrawCommandList, b_id: WidgetId) -> bool {
    command_list
        .boundary_slices
        .iter()
        .any(|(id, _)| *id == b_id)
        || command_list
            .texture_boundary_slices
            .iter()
            .any(|(id, _)| *id == b_id)
        || command_list
            .external_boundary_slices
            .iter()
            .any(|(id, _)| *id == b_id)
}

fn find_matching_slice_run(
    dst_slices: &[(WidgetId, BoundarySlice)],
    src_slices: &[(WidgetId, BoundarySlice)],
    owner_boundary: WidgetId,
) -> Option<Option<usize>> {
    if src_slices.is_empty() {
        return dst_slices
            .iter()
            .all(|(id, _)| *id != owner_boundary)
            .then_some(None);
    }
    let first_pos = dst_slices
        .iter()
        .position(|(id, _)| *id == src_slices[0].0)?;
    let end_pos = first_pos + src_slices.len();
    let candidate = dst_slices.get(first_pos..end_pos)?;
    for ((d_id, d_slice), (s_id, s_slice)) in candidate.iter().zip(src_slices.iter()) {
        if d_id != s_id || d_slice.count != s_slice.count {
            return None;
        }
    }
    let extra_outside = dst_slices[..first_pos]
        .iter()
        .any(|(id, _)| *id == owner_boundary)
        || dst_slices[end_pos..]
            .iter()
            .any(|(id, _)| *id == owner_boundary);
    if extra_outside {
        return None;
    }
    Some(Some(first_pos))
}

fn apply_slice_run<T: Copy>(
    dst_items: &mut [T],
    dst_slices: &[(WidgetId, BoundarySlice)],
    src_items: &[T],
    src_slices: &[(WidgetId, BoundarySlice)],
) {
    for ((_, d_slice), (_, s_slice)) in dst_slices.iter().zip(src_slices.iter()) {
        if d_slice.count == 0 {
            continue;
        }
        let dst_start = d_slice.start as usize;
        let dst_end = dst_start + d_slice.count as usize;
        let src_start = s_slice.start as usize;
        let src_end = src_start + s_slice.count as usize;
        dst_items[dst_start..dst_end].copy_from_slice(&src_items[src_start..src_end]);
    }
}

fn build_boundary_ancestor_context<'a, 'b>(
    tree: &UiTree,
    root_id: WidgetId,
    target_id: WidgetId,
    options: &'b mut TreeCompilerOptions<'a>,
) -> Option<CompilationContext<'a, 'b>> {
    let mut ancestors = [target_id; 64];
    let mut depth = 0usize;
    let mut curr = tree.get(target_id)?.parent;
    while let Some(anc) = curr {
        if depth >= ancestors.len() {
            return None;
        }
        ancestors[depth] = anc;
        depth += 1;
        if anc == root_id {
            break;
        }
        curr = tree.get(anc)?.parent;
    }
    if depth > 0 && ancestors[depth - 1] != root_id {
        return None;
    }

    let mut scissor_stack = ScissorStack::with_initial_clip(options.initial_clip);
    let mut inherited_layer = UiLayer::Background;
    let mut inherited_hovered = false;
    let mut inherited_focused = false;

    for i in (0..depth).rev() {
        let anc_id = ancestors[i];
        let node = tree.get(anc_id)?;
        if !node.visible {
            return None;
        }
        let is_hovered = resolve_node_hover(
            tree,
            anc_id,
            node,
            options.hovered_id,
            options.hovered_tag,
            inherited_hovered,
        );
        let is_focused = resolve_node_focus(
            anc_id,
            node,
            options.focused_id,
            options.focused_tag,
            inherited_focused,
        );
        let effective_layer = resolve_effective_layer(node, inherited_layer);
        if is_node_elevated_root(node, effective_layer, inherited_layer) {
            scissor_stack = ScissorStack::default();
        }
        if node.style.clip_children {
            scissor_stack.push(node.computed_rect);
        }
        inherited_layer = effective_layer;
        inherited_hovered = is_hovered;
        inherited_focused = is_focused;
    }

    Some(CompilationContext {
        scissor_stack,
        inherited_layer,
        inherited_hovered,
        inherited_focused,
        hovered_tag: options.hovered_tag,
        hovered_id: options.hovered_id,
        focused_tag: options.focused_tag,
        focused_id: options.focused_id,
        blink_caret: options.blink_caret,
        custom_drawer: &mut options.custom_drawer,
        current_boundary: Some(target_id),
    })
}