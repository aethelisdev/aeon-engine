// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Hierarchical Folder Tree Sidebar for Iris UI Asset Browser.
//!
//! Renders an interactive, collapsible directory tree using the engine's canonical
//! vector folder logo (`ICON_FOLDER`, Layer 6) with depth indents and selection pills.
//!

use super::types::{AssetsPanelParams, AssetsPanelTargets, FolderTreeNodeTarget};
use crate::ui::iris_bridge::icons::{ICON_FOLDER, ICON_PLUS};
use irisui::prelude::*;
use std::path::{Path, PathBuf};

/// Height of an individual folder tree node row in pixels.
pub const FOLDER_ROW_HEIGHT: f32 = 24.0;

/// Header height for the "FOLDERS" label and new subfolder button.
pub const FOLDER_HEADER_HEIGHT: f32 = 28.0;

/// Constructs the complete hierarchical folder tree sidebar into the Iris `UiTree`.
pub fn build_folder_tree_sidebar(
    tree: &mut UiTree,
    parent_id: WidgetId,
    sidebar_rect: Rect,
    params: &AssetsPanelParams<'_>,
    targets: &mut AssetsPanelTargets,
) {
    targets.sidebar_rect = Some(sidebar_rect);

    // 1. Sidebar Container with Hardware Scissor Clipping
    let sb_id = tree.create_node();
    if let Some(node) = tree.get_mut(sb_id) {
        node.set_name("FolderTreeSidebarRoot");
        node.computed_rect = sidebar_rect;
        node.style = Style::new()
            .background(Color::rgba(0.06, 0.07, 0.09, 0.98))
            .border(1.0, Color::rgba(0.18, 0.20, 0.26, 0.70))
            .clip_children(true);
    }
    let _ = tree.add_child(parent_id, sb_id);

    // 2. Header Bar ("FOLDERS" + "+" button)
    let hdr_rect = Rect::new(
        sidebar_rect.x,
        sidebar_rect.y,
        sidebar_rect.width,
        FOLDER_HEADER_HEIGHT,
    );
    let hdr_id = tree.create_node();
    if let Some(node) = tree.get_mut(hdr_id) {
        node.set_name("FolderTreeHeader");
        node.computed_rect = hdr_rect;
        node.style = Style::new()
            .background(Color::rgba(0.08, 0.09, 0.12, 0.95))
            .border(1.0, Color::rgba(0.16, 0.18, 0.24, 0.50));
    }
    let _ = tree.add_child(sb_id, hdr_id);

    // Header Label: "FOLDERS"
    let lbl_id = tree.create_node();
    if let Some(node) = tree.get_mut(lbl_id) {
        node.set_name("FoldersLabel");
        node.set_text("FOLDERS");
        node.font_size = 11.0;
        node.line_height = FOLDER_HEADER_HEIGHT;
        node.text_color = Color::rgba(0.65, 0.70, 0.80, 1.0);
        node.computed_rect = Rect::new(
            hdr_rect.x + 8.0,
            hdr_rect.y,
            hdr_rect.width - 36.0,
            FOLDER_HEADER_HEIGHT,
        );
    }
    let _ = tree.add_child(hdr_id, lbl_id);

    // "+" Add Subfolder Button
    let plus_btn_rect = Rect::new(hdr_rect.right() - 26.0, hdr_rect.y + 4.0, 20.0, 20.0);
    targets.new_subfolder_btn_rect = Some(plus_btn_rect);
    let is_plus_hovered = plus_btn_rect.contains_point(params.cursor_pos);

    let plus_id = tree.create_node();
    if let Some(node) = tree.get_mut(plus_id) {
        node.set_name("NewSubfolderBtn");
        node.computed_rect = plus_btn_rect;
        node.style = Style::new()
            .background(if is_plus_hovered {
                Color::rgba(0.20, 0.24, 0.32, 1.0)
            } else {
                Color::rgba(0.12, 0.14, 0.18, 0.80)
            })
            .border_radius(3.0)
            .border(
                1.0,
                if is_plus_hovered {
                    Color::rgba(0.35, 0.42, 0.55, 0.80)
                } else {
                    Color::rgba(0.20, 0.23, 0.30, 0.40)
                },
            );
    }
    let _ = tree.add_child(hdr_id, plus_id);

    // Canonical vector plus icon quad (12x12 px centered)
    let icon_dim = 12.0;
    let icon_x = plus_btn_rect.x + (plus_btn_rect.width - icon_dim) * 0.5;
    let icon_y = plus_btn_rect.y + (plus_btn_rect.height - icon_dim) * 0.5;
    let icon_rect = Rect::new(icon_x, icon_y, icon_dim, icon_dim);
    let icon_id = tree.create_node();
    if let Some(node) = tree.get_mut(icon_id) {
        node.set_name("NewSubfolderPlusIcon");
        node.computed_rect = icon_rect;
        node.set_texture_uv(ICON_PLUS);
        node.set_texture_tint(if is_plus_hovered {
            Color::WHITE
        } else {
            Color::rgba(0.70, 0.75, 0.85, 1.0)
        });
    }
    let _ = tree.add_child(plus_id, icon_id);

    // 3. Scrollable Tree Viewport
    let vp_rect = Rect::new(
        sidebar_rect.x,
        sidebar_rect.y + FOLDER_HEADER_HEIGHT,
        sidebar_rect.width,
        sidebar_rect.height - FOLDER_HEADER_HEIGHT,
    );
    let vp_id = tree.create_node();
    if let Some(node) = tree.get_mut(vp_id) {
        node.set_name("FolderTreeViewport");
        node.computed_rect = vp_rect;
        node.style = Style::new().clip_children(true);
    }
    let _ = tree.add_child(sb_id, vp_id);

    // 4. Recursively build root directory nodes starting at "assets"
    let root_path = PathBuf::from("assets");
    let mut cur_y = vp_rect.y + 4.0 - params.tree_scroll_y;
    let mut ctx = FolderTreeContext {
        vp_rect,
        params,
        targets,
    };
    render_folder_recursive(tree, vp_id, &root_path, 0, &mut cur_y, &mut ctx);
}

/// Context descriptor bundling tree traversal layout parameters and hit targets.
struct FolderTreeContext<'a, 'p> {
    /// Scissor-clipped scrollable viewport bounding box.
    pub vp_rect: Rect,
    /// Read-only panel rendering parameters.
    pub params: &'a AssetsPanelParams<'p>,
    /// Mutable hit target registry.
    pub targets: &'a mut AssetsPanelTargets,
}

/// Recursively builds folder nodes, chevrons, and custom vector `ICON_FOLDER` quads.
fn render_folder_recursive(
    tree: &mut UiTree,
    parent_id: WidgetId,
    path: &Path,
    depth: usize,
    cur_y: &mut f32,
    ctx: &mut FolderTreeContext<'_, '_>,
) {
    let folder_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("assets");

    // Discover child directories
    let mut child_dirs = Vec::new();
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            let child_path = entry.path();
            if child_path.is_dir() {
                child_dirs.push(child_path);
            }
        }
    }
    child_dirs.sort();
    let has_children = !child_dirs.is_empty();

    let is_selected = ctx.params.current_folder == path;
    let is_expanded = path == Path::new("assets") || ctx.params.current_folder.starts_with(path);

    let row_y = *cur_y;
    *cur_y += FOLDER_ROW_HEIGHT;

    // Viewport scissor cull: skip generating quad if completely outside viewport
    if row_y + FOLDER_ROW_HEIGHT < ctx.vp_rect.y || row_y > ctx.vp_rect.bottom() {
        if is_expanded {
            for child in &child_dirs {
                render_folder_recursive(tree, parent_id, child, depth + 1, cur_y, ctx);
            }
        }
        return;
    }

    let row_rect = Rect::new(
        ctx.vp_rect.x + 4.0,
        row_y,
        ctx.vp_rect.width - 8.0,
        FOLDER_ROW_HEIGHT - 2.0,
    );
    let is_hovered = row_rect.contains_point(ctx.params.cursor_pos);

    // Tree Node Row Capsule
    let row_id = tree.create_node();
    if let Some(node) = tree.get_mut(row_id) {
        node.set_name("FolderRow");
        node.computed_rect = row_rect;
        let bg_color = if is_selected {
            Color::rgba(0.08, 0.22, 0.32, 0.90)
        } else if is_hovered {
            Color::rgba(0.14, 0.16, 0.22, 0.70)
        } else {
            Color::TRANSPARENT
        };
        let border_color = if is_selected {
            Color::rgba(0.0, 0.85, 1.0, 0.80)
        } else {
            Color::TRANSPARENT
        };
        node.style = Style::new()
            .background(bg_color)
            .border_radius(4.0)
            .border(1.0, border_color);
    }
    let _ = tree.add_child(parent_id, row_id);

    let mut cur_x = row_rect.x + 4.0 + (depth as f32 * 14.0);

    // 1. Expand / Collapse Chevron indicator (▾ / ▸)
    let chevron_rect = if has_children {
        let ch_rect = Rect::new(cur_x, row_rect.y, 14.0, row_rect.height);
        let ch_id = tree.create_node();
        if let Some(node) = tree.get_mut(ch_id) {
            node.set_name("FolderChevron");
            node.set_text(if is_expanded { "▾" } else { "▸" });
            node.font_size = 11.0;
            node.line_height = row_rect.height;
            node.text_align = TextAlign::Center;
            node.text_color = if is_selected {
                Color::rgba(0.0, 0.90, 1.0, 1.0)
            } else {
                Color::rgba(0.60, 0.65, 0.75, 1.0)
            };
            node.computed_rect = ch_rect;
        }
        let _ = tree.add_child(row_id, ch_id);
        Some(ch_rect)
    } else {
        None
    };
    cur_x += 16.0;

    // 2. Canonical Vector Folder Icon (`ICON_FOLDER`, Layer 6)
    let icon_size = 16.0;
    let icon_y = row_rect.y + (row_rect.height - icon_size) * 0.5;
    let icon_rect = Rect::new(cur_x, icon_y, icon_size, icon_size);
    let icon_id = tree.create_node();
    if let Some(node) = tree.get_mut(icon_id) {
        node.set_name("FolderIcon");
        node.computed_rect = icon_rect;
        node.set_texture_uv(ICON_FOLDER);
        node.set_texture_tint(if is_selected {
            Color::rgba(0.0, 0.90, 1.0, 1.0) // Cyan when selected
        } else {
            Color::rgba(0.95, 0.76, 0.28, 1.0) // Warm folder amber
        });
    }
    let _ = tree.add_child(row_id, icon_id);
    cur_x += icon_size + 6.0;

    // 3. Folder Name Label
    let name_w = (row_rect.right() - cur_x).max(20.0);
    let name_rect = Rect::new(cur_x, row_rect.y, name_w, row_rect.height);
    let name_id = tree.create_node();
    if let Some(node) = tree.get_mut(name_id) {
        node.set_name("FolderName");
        node.set_text(folder_name);
        node.font_size = 11.5;
        node.line_height = row_rect.height;
        node.text_color = if is_selected {
            Color::WHITE
        } else if is_hovered {
            Color::rgba(0.90, 0.92, 0.96, 1.0)
        } else {
            Color::rgba(0.75, 0.78, 0.85, 1.0)
        };
        node.computed_rect = name_rect;
    }
    let _ = tree.add_child(row_id, name_id);

    // Register Target
    ctx.targets.folder_nodes.push(FolderTreeNodeTarget {
        row_rect,
        chevron_rect,
        path: path.to_path_buf(),
        has_children,
        is_expanded,
    });

    // Recurse children if expanded
    if is_expanded {
        for child in &child_dirs {
            render_folder_recursive(tree, parent_id, child, depth + 1, cur_y, ctx);
        }
    }
}