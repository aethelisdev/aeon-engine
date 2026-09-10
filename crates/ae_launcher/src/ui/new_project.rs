// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

use irisui::prelude::*;

use crate::icons::{ICON_CUBE, ICON_FOLDER, ICON_PLUS};
use crate::ui::types::{LauncherUiState, ViewLayoutContext};

/// Builds the New Project wizard tab view into the UI tree.
/// Constructs interactive project name input (with focus and blinking caret support),
/// project parent directory locator with folder browser button, dimension mode cards
/// (including 2D Closed Beta locked badge), Create button, and transient feedback text.
pub fn build_new_project_view(
    tree: &mut UiTree,
    parent_id: WidgetId,
    ctx: ViewLayoutContext<'_>,
    state: &LauncherUiState,
) {
    let start_x = ctx.start_x;
    let cursor_pos = ctx.cursor_pos;

    let header_id = tree.create_node();
    if let Some(node) = tree.get_mut(header_id) {
        node.set_text("Create New Project");
        node.font_size = 20.0;
        node.text_color = Color::hex("#ffffff");
        node.computed_rect = Rect::new(start_x + 32.0, 24.0, 300.0, 28.0);
    }
    let _ = tree.add_child(parent_id, header_id);

    let subhead_id = tree.create_node();
    if let Some(node) = tree.get_mut(subhead_id) {
        node.set_text("Configure your project name, location, and engine dimension mode.");
        node.font_size = 11.5;
        node.text_color = Color::hex("#73798c");
        node.computed_rect = Rect::new(start_x + 32.0, 54.0, 500.0, 20.0);
    }
    let _ = tree.add_child(parent_id, subhead_id);

    let mut form_y = 100.0;

    // 1. Project Name
    let name_lbl_id = tree.create_node();
    if let Some(node) = tree.get_mut(name_lbl_id) {
        node.set_text("PROJECT NAME");
        node.font_size = 10.0;
        node.text_color = Color::hex("#8a91a3");
        node.computed_rect = Rect::new(start_x + 32.0, form_y, 200.0, 16.0);
    }
    let _ = tree.add_child(parent_id, name_lbl_id);

    form_y += 22.0;

    let name_box_rect = Rect::new(start_x + 32.0, form_y, 420.0, 36.0);
    let name_box_hovered = name_box_rect.contains_point(cursor_pos);
    let name_box_id = tree.create_node();
    if let Some(node) = tree.get_mut(name_box_id) {
        node.computed_rect = name_box_rect;
        let border = if state.is_name_focused {
            Color::hex("#00e5ff")
        } else if name_box_hovered {
            Color::hex("#3a4358")
        } else {
            Color::hex("#2a3142")
        };
        let bg = if state.is_name_focused {
            Color::hex("#181d28")
        } else {
            Color::hex("#151821")
        };
        node.style = Style::new()
            .background(bg)
            .border_radius(6.0)
            .border(if state.is_name_focused { 1.5 } else { 1.0 }, border);
    }
    let _ = tree.add_child(parent_id, name_box_id);

    let display_name = if state.is_name_focused && state.cursor_blink_visible {
        format!("{}|", state.new_project_name)
    } else {
        state.new_project_name.clone()
    };
    let name_val_id = tree.create_node();
    if let Some(node) = tree.get_mut(name_val_id) {
        node.set_text(&display_name);
        node.font_size = 12.0;
        node.text_color = Color::hex("#ffffff");
        node.computed_rect = Rect::new(start_x + 44.0, form_y + 9.0, 400.0, 18.0);
    }
    let _ = tree.add_child(parent_id, name_val_id);

    form_y += 54.0;

    // 2. Project Location
    let loc_lbl_id = tree.create_node();
    if let Some(node) = tree.get_mut(loc_lbl_id) {
        node.set_text("PROJECT LOCATION");
        node.font_size = 10.0;
        node.text_color = Color::hex("#8a91a3");
        node.computed_rect = Rect::new(start_x + 32.0, form_y, 200.0, 16.0);
    }
    let _ = tree.add_child(parent_id, loc_lbl_id);

    form_y += 22.0;

    let loc_box_id = tree.create_node();
    if let Some(node) = tree.get_mut(loc_box_id) {
        node.computed_rect = Rect::new(start_x + 32.0, form_y, 372.0, 36.0);
        node.style = Style::new()
            .background(Color::hex("#151821"))
            .border_radius(6.0)
            .border(1.0, Color::hex("#2a3142"));
    }
    let _ = tree.add_child(parent_id, loc_box_id);

    let loc_val_id = tree.create_node();
    if let Some(node) = tree.get_mut(loc_val_id) {
        node.set_text(&state.new_project_dir);
        node.font_size = 11.5;
        node.text_color = Color::hex("#a0a6b5");
        node.computed_rect = Rect::new(start_x + 44.0, form_y + 9.0, 350.0, 18.0);
    }
    let _ = tree.add_child(parent_id, loc_val_id);

    // Browse Button (Icon only, enlarged folder icon)
    let browse_rect = Rect::new(start_x + 32.0 + 380.0, form_y, 40.0, 36.0);
    let browse_hovered = browse_rect.contains_point(cursor_pos);
    let browse_btn_id = tree.create_node();
    if let Some(node) = tree.get_mut(browse_btn_id) {
        node.computed_rect = browse_rect;
        node.style = Style::new()
            .background(if browse_hovered {
                Color::hex("#252c3c")
            } else {
                Color::hex("#1c212c")
            })
            .border_radius(6.0)
            .border(
                1.0,
                if browse_hovered {
                    Color::hex("#00e5ff")
                } else {
                    Color::hex("#2d3648")
                },
            );
    }
    let _ = tree.add_child(parent_id, browse_btn_id);

    let browse_icon_id = tree.create_node();
    if let Some(node) = tree.get_mut(browse_icon_id) {
        node.computed_rect = Rect::new(start_x + 32.0 + 380.0 + 9.0, form_y + 7.0, 22.0, 22.0);
        node.set_texture_uv(ICON_FOLDER);
        node.set_texture_tint(if browse_hovered {
            Color::hex("#ffe082")
        } else {
            Color::hex("#ffd166")
        });
    }
    let _ = tree.add_child(parent_id, browse_icon_id);

    form_y += 56.0;

    // 3. Engine Dimension Mode (2D vs 3D)
    let mode_lbl_id = tree.create_node();
    if let Some(node) = tree.get_mut(mode_lbl_id) {
        node.set_text("DIMENSION MODE");
        node.font_size = 10.0;
        node.text_color = Color::hex("#8a91a3");
        node.computed_rect = Rect::new(start_x + 32.0, form_y, 200.0, 16.0);
    }
    let _ = tree.add_child(parent_id, mode_lbl_id);

    form_y += 22.0;

    let is_3d_selected = state.selected_mode == "3D";

    // Mode Card 1: 2D Sprite Engine (Locked - Closed Beta)
    let mode_2d_rect = Rect::new(start_x + 32.0, form_y, 204.0, 84.0);
    let mode_2d_hovered = mode_2d_rect.contains_point(cursor_pos);
    let mode_2d_id = tree.create_node();
    if let Some(node) = tree.get_mut(mode_2d_id) {
        node.computed_rect = mode_2d_rect;
        let border = if mode_2d_hovered {
            Color::hex("#3a4358")
        } else {
            Color::hex("#1e2330")
        };
        node.style = Style::new()
            .background(Color::hex("#12141c"))
            .border_radius(8.0)
            .border(1.0, border);
    }
    let _ = tree.add_child(parent_id, mode_2d_id);

    let m2d_icon_id = tree.create_node();
    if let Some(node) = tree.get_mut(m2d_icon_id) {
        node.computed_rect = Rect::new(start_x + 44.0, form_y + 14.0, 16.0, 16.0);
        node.set_texture_uv(ICON_FOLDER);
        node.set_texture_tint(Color::hex("#73798c"));
    }
    let _ = tree.add_child(parent_id, m2d_icon_id);

    let m2d_title_id = tree.create_node();
    if let Some(node) = tree.get_mut(m2d_title_id) {
        node.set_text("2D Sprite Engine");
        node.font_size = 11.5;
        node.text_color = Color::hex("#9aa0b2");
        node.computed_rect = Rect::new(start_x + 66.0, form_y + 14.0, 100.0, 18.0);
    }
    let _ = tree.add_child(parent_id, m2d_title_id);

    // Closed Beta Badge
    let beta_badge_id = tree.create_node();
    if let Some(node) = tree.get_mut(beta_badge_id) {
        node.computed_rect = Rect::new(start_x + 32.0 + 204.0 - 74.0, form_y + 10.0, 68.0, 16.0);
        node.style = Style::new()
            .background(Color::hex("#241c0f"))
            .border_radius(4.0)
            .border(1.0, Color::hex("#b45309"));
    }
    let _ = tree.add_child(parent_id, beta_badge_id);

    let beta_lbl_id = tree.create_node();
    if let Some(node) = tree.get_mut(beta_lbl_id) {
        node.set_text("🔒 BETA");
        node.font_size = 8.5;
        node.text_color = Color::hex("#f59e0b");
        node.computed_rect = Rect::new(start_x + 32.0 + 204.0 - 64.0, form_y + 10.5, 55.0, 14.0);
    }
    let _ = tree.add_child(parent_id, beta_lbl_id);

    let m2d_desc_id = tree.create_node();
    if let Some(node) = tree.get_mut(m2d_desc_id) {
        node.set_text("Under active development.\nAvailable in future releases.");
        node.font_size = 9.5;
        node.text_color = Color::hex("#5d6373");
        node.computed_rect = Rect::new(start_x + 44.0, form_y + 38.0, 180.0, 32.0);
    }
    let _ = tree.add_child(parent_id, m2d_desc_id);

    // Mode Card 2: 3D Spatial Engine
    let mode_3d_rect = Rect::new(start_x + 248.0, form_y, 204.0, 84.0);
    let mode_3d_hovered = mode_3d_rect.contains_point(cursor_pos);
    let mode_3d_id = tree.create_node();
    if let Some(node) = tree.get_mut(mode_3d_id) {
        node.computed_rect = mode_3d_rect;
        let border = if is_3d_selected {
            Color::hex("#00e5ff")
        } else if mode_3d_hovered {
            Color::hex("#3a4358")
        } else {
            Color::hex("#1e2330")
        };
        node.style = Style::new()
            .background(if is_3d_selected {
                Color::hex("#0f2129")
            } else {
                Color::hex("#151821")
            })
            .border_radius(8.0)
            .border(1.5, border);
    }
    let _ = tree.add_child(parent_id, mode_3d_id);

    let m3d_icon_id = tree.create_node();
    if let Some(node) = tree.get_mut(m3d_icon_id) {
        node.computed_rect = Rect::new(start_x + 262.0, form_y + 14.0, 18.0, 18.0);
        node.set_texture_uv(ICON_CUBE);
        node.set_texture_tint(Color::hex("#00e5ff"));
    }
    let _ = tree.add_child(parent_id, m3d_icon_id);

    let m3d_title_id = tree.create_node();
    if let Some(node) = tree.get_mut(m3d_title_id) {
        node.set_text("3D Spatial Engine");
        node.font_size = 12.5;
        node.text_color = Color::hex("#ffffff");
        node.computed_rect = Rect::new(start_x + 288.0, form_y + 14.0, 150.0, 18.0);
    }
    let _ = tree.add_child(parent_id, m3d_title_id);

    let m3d_desc_id = tree.create_node();
    if let Some(node) = tree.get_mut(m3d_desc_id) {
        node.set_text("Full PBR forward passes,\nCSM shadows & skeletal mesh.");
        node.font_size = 10.0;
        node.text_color = Color::hex("#73798c");
        node.computed_rect = Rect::new(start_x + 262.0, form_y + 38.0, 180.0, 32.0);
    }
    let _ = tree.add_child(parent_id, m3d_desc_id);

    form_y += 110.0;

    // 4. Create & Launch Button
    let create_rect = Rect::new(start_x + 32.0, form_y, 420.0, 42.0);
    let create_hovered = create_rect.contains_point(cursor_pos);
    let create_btn_id = tree.create_node();
    if let Some(node) = tree.get_mut(create_btn_id) {
        node.computed_rect = create_rect;
        node.style = Style::new()
            .background(if create_hovered {
                Color::hex("#1c2738")
            } else {
                Color::hex("#131a24")
            })
            .border_radius(7.0)
            .border(
                1.5,
                if create_hovered {
                    Color::hex("#38efff")
                } else {
                    Color::hex("#00b4d8")
                },
            );
    }
    let _ = tree.add_child(parent_id, create_btn_id);

    let create_icon_id = tree.create_node();
    if let Some(node) = tree.get_mut(create_icon_id) {
        node.computed_rect = Rect::new(start_x + 138.0, form_y + 13.0, 16.0, 16.0);
        node.set_texture_uv(ICON_PLUS);
        node.set_texture_tint(if create_hovered {
            Color::hex("#38efff")
        } else {
            Color::hex("#00e5ff")
        });
    }
    let _ = tree.add_child(parent_id, create_icon_id);

    let create_lbl_id = tree.create_node();
    if let Some(node) = tree.get_mut(create_lbl_id) {
        node.set_text("Create & Launch Project");
        node.font_size = 13.0;
        node.text_color = Color::hex("#ffffff");
        node.computed_rect = Rect::new(start_x + 162.0, form_y + 12.0, 200.0, 20.0);
    }
    let _ = tree.add_child(parent_id, create_lbl_id);

    if let Some(ref status_msg) = state.status_message {
        let status_id = tree.create_node();
        if let Some(node) = tree.get_mut(status_id) {
            node.set_text(status_msg);
            node.font_size = 11.0;
            node.text_color = Color::hex("#f59e0b");
            node.computed_rect = Rect::new(start_x + 32.0, form_y + 50.0, 420.0, 18.0);
        }
        let _ = tree.add_child(parent_id, status_id);
    }
}