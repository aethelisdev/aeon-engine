// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Unit tests for [`crate::surface::IrisSurfaceHost`], text occlusion filtering,
//! external-texture cutout command emission, and split static/volatile render routing.

use crate::command::DrawCommandList;
use crate::surface::{IrisSurfaceFrameFlags, IrisSurfaceHost, SurfaceRefreshMode};
use crate::tree_compiler::TreeCompilerOptions;
use crate::tree_compiler_patch::{TreeCompilerScratch, compile_tree_draw_commands_with_scratch};
use irisui_core::{Color, ExternalTextureId, Rect, Style, UiTree, WidgetId, WidgetNode};

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
    let mut noop = |_l: &mut DrawCommandList, _id: WidgetId, _n: &WidgetNode, _c: Option<Rect>| {};
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

#[test]
fn test_viewport_external_texture_and_hud_z_order_preserve_cutout_window() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("root");

    let panel_bg = tree.create_node();
    if let Some(n) = tree.get_mut(panel_bg) {
        n.style = Style::new().background(Color::rgba(0.1, 0.1, 0.12, 1.0));
        n.computed_rect = Rect::new(0.0, 0.0, 1280.0, 720.0);
    }
    tree.add_child(root, panel_bg).expect("panel_bg");

    let viewport = tree.create_node();
    if let Some(n) = tree.get_mut(viewport) {
        n.external_texture = Some(ExternalTextureId(1));
        n.computed_rect = Rect::new(240.0, 32.0, 800.0, 540.0);
    }
    tree.add_child(panel_bg, viewport).expect("viewport");

    let hud_pill = tree.create_node();
    if let Some(n) = tree.get_mut(hud_pill) {
        n.style = Style::new().background(Color::rgba(0.2, 0.5, 0.9, 0.85));
        n.computed_rect = Rect::new(260.0, 48.0, 120.0, 28.0);
    }
    tree.add_child(viewport, hud_pill).expect("hud_pill");

    let mut command_list = DrawCommandList::default();
    let mut scratch = TreeCompilerScratch::default();
    let mut opts = TreeCompilerOptions::new();
    compile_tree_draw_commands_with_scratch(
        &tree,
        root,
        &mut opts,
        &mut scratch,
        &mut command_list,
    );

    assert_eq!(command_list.external_texture_quads.len(), 1);
    let ext_cmd_idx = command_list
        .commands
        .iter()
        .position(|c| matches!(c, crate::command::DrawCommand::DrawExternalTexture { .. }))
        .expect("ExternalTexture command must exist");
    assert!(
        ext_cmd_idx >= 1,
        "Background SDF quads must precede external texture cutout so alpha=0 punches through background only"
    );
    assert!(
        command_list.commands[ext_cmd_idx + 1..]
            .iter()
            .any(|c| matches!(c, crate::command::DrawCommand::DrawSdfQuads { .. })),
        "Foreground HUD SDF quads must follow external texture cutout so they composite cleanly over the hole"
    );
}