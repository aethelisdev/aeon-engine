// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # System Modules Preferences Tab
//!
//! Renders hardware-accelerated configuration cards for toggling core engine systems
//! (Physics, Audio, 3D Render Viewport pass) with live status indicators and zero background CPU/GPU overhead.

use super::super::types::{PreferencesParams, PreferencesTargets, PreferencesToggleId};
use ae_core::modules::EngineModule;
use irisui::prelude::*;

/// System module descriptor with UI presentation metadata.
struct ModuleCardData {
    module: EngineModule,
    name: &'static str,
    desc: &'static str,
    detail: &'static str,
    color: Color,
}

const MODULES: [ModuleCardData; 3] = [
    ModuleCardData {
        module: EngineModule::Physics,
        name: "Physics (Fizik)",
        desc: "Runs position/velocity integration, collisions, and character controller updates. Disabling halts all physical simulations and saves CPU cycles.",
        detail: "⚙ FixedUpdate Loop",
        color: Color::rgba(0.92, 0.45, 0.23, 1.0), // Orange
    },
    ModuleCardData {
        module: EngineModule::Audio,
        name: "Audio (Ses)",
        desc: "Processes sound playback and environmental effects. Disabling stops all audio processing.",
        detail: "🔊 Audio Pipeline",
        color: Color::rgba(0.23, 0.65, 0.92, 1.0), // Blue
    },
    ModuleCardData {
        module: EngineModule::Render,
        name: "Render (Render)",
        desc: "Renders 3D geometry, shadows, skybox, and post-processing. Disable to bypass the render pipeline.",
        detail: "👁 3D Viewport Pass",
        color: Color::rgba(0.45, 0.92, 0.23, 1.0), // Green
    },
];

/// Builds the System Modules preferences tab content.
pub fn build_modules_tab(
    tree: &mut UiTree,
    parent_id: WidgetId,
    content_rect: Rect,
    params: &PreferencesParams<'_>,
    targets: &mut PreferencesTargets,
) -> f32 {
    let mut virtual_y = 16.0;
    let scroll_y = params.scroll_offset_y;
    let content_w = content_rect.width - 32.0;
    let base_x = content_rect.x + 16.0;

    // 1. Heading
    let heading_id = tree.create_node();
    if let Some(node) = tree.get_mut(heading_id) {
        node.set_name("ModulesHeading");
        node.set_text("🧩  System Modules");
        node.font_size = 17.0;
        node.line_height = 22.0;
        node.text_color = Color::rgba(1.0, 1.0, 1.0, 1.0);
        node.computed_rect = Rect::new(
            base_x,
            content_rect.y + virtual_y - scroll_y,
            content_w,
            22.0,
        );
    }
    let _ = tree.add_child(parent_id, heading_id);
    virtual_y += 24.0;

    // 2. Subtitle
    let sub_id = tree.create_node();
    if let Some(node) = tree.get_mut(sub_id) {
        node.set_name("ModulesSubtitle");
        node.set_text("Enable or disable core systems to optimize performance or isolate systems. Disabled modules consume zero background CPU/GPU cycles.");
        node.font_size = 11.5;
        node.line_height = 16.0;
        node.text_color = Color::rgba(0.65, 0.68, 0.76, 1.0);
        node.computed_rect = Rect::new(
            base_x,
            content_rect.y + virtual_y - scroll_y,
            content_w,
            16.0,
        );
    }
    let _ = tree.add_child(parent_id, sub_id);
    virtual_y += 24.0;

    // 3. Separator
    let sep_id = tree.create_node();
    if let Some(node) = tree.get_mut(sep_id) {
        node.set_name("ModulesSep");
        node.style = Style::new().background(Color::rgba(0.20, 0.22, 0.30, 0.70));
        node.computed_rect = Rect::new(
            base_x,
            content_rect.y + virtual_y - scroll_y,
            content_w,
            1.0,
        );
    }
    let _ = tree.add_child(parent_id, sep_id);
    virtual_y += 16.0;

    // 4. Module Cards
    for card_data in &MODULES {
        let is_enabled = params.enabled_modules.contains(&card_data.module);
        let card_h = 88.0;
        let card_rect = Rect::new(
            base_x,
            content_rect.y + virtual_y - scroll_y,
            content_w,
            card_h,
        );

        let card_node = tree.create_node();
        if let Some(node) = tree.get_mut(card_node) {
            node.set_name("ModuleCard");
            node.computed_rect = card_rect;
            node.style = Style::new()
                .background(Color::rgba(0.09, 0.10, 0.14, 0.85))
                .border(1.0, Color::rgba(0.18, 0.20, 0.28, 0.90))
                .border_radius(6.0);
        }
        let _ = tree.add_child(parent_id, card_node);

        // Header Row: Dot + Name + Detail Badge + Status + Checkbox
        let dot_rect = Rect::new(
            base_x + 14.0,
            content_rect.y + virtual_y - scroll_y + 14.0,
            10.0,
            10.0,
        );
        let dot_node = tree.create_node();
        if let Some(node) = tree.get_mut(dot_node) {
            node.set_name("ModuleDot");
            node.computed_rect = dot_rect;
            let dot_col = if is_enabled {
                card_data.color
            } else {
                Color::rgba(0.24, 0.26, 0.32, 1.0)
            };
            node.style = Style::new().background(dot_col).border_radius(5.0);
        }
        let _ = tree.add_child(card_node, dot_node);

        let name_node = tree.create_node();
        if let Some(node) = tree.get_mut(name_node) {
            node.set_name("ModuleName");
            node.set_text(card_data.name);
            node.font_size = 13.5;
            node.line_height = 18.0;
            node.text_color = Color::rgba(1.0, 1.0, 1.0, 1.0);
            node.computed_rect = Rect::new(
                base_x + 30.0,
                content_rect.y + virtual_y - scroll_y + 10.0,
                200.0,
                18.0,
            );
        }
        let _ = tree.add_child(card_node, name_node);

        // Right side: Detail badge + Status label ("ENABLED"/"DISABLED") + Checkbox
        let cb_rect = Rect::new(
            base_x + content_w - 30.0,
            content_rect.y + virtual_y - scroll_y + 11.0,
            16.0,
            16.0,
        );
        let is_cb_hovered = Rect::new(
            base_x + content_w - 220.0,
            content_rect.y + virtual_y - scroll_y + 8.0,
            220.0,
            22.0,
        )
        .contains_point(params.cursor_pos);

        let cb_box = tree.create_node();
        if let Some(node) = tree.get_mut(cb_box) {
            node.set_name("ModuleCb");
            node.computed_rect = cb_rect;
            let bg = if is_enabled {
                Color::rgba(0.0, 0.70, 0.85, 1.0)
            } else if is_cb_hovered {
                Color::rgba(0.18, 0.20, 0.28, 1.0)
            } else {
                Color::rgba(0.11, 0.12, 0.16, 1.0)
            };
            node.style = Style::new()
                .background(bg)
                .border(1.0, Color::rgba(0.25, 0.30, 0.42, 1.0))
                .border_radius(3.0);
        }
        let _ = tree.add_child(card_node, cb_box);

        if is_enabled {
            let chk = tree.create_node();
            if let Some(node) = tree.get_mut(chk) {
                node.set_name("ModuleCheckMark");
                node.set_text("✓");
                node.font_size = 11.0;
                node.line_height = 16.0;
                node.text_align = TextAlign::Center;
                node.text_color = Color::rgba(0.05, 0.06, 0.08, 1.0);
                node.computed_rect = cb_rect;
            }
            let _ = tree.add_child(cb_box, chk);
        }

        let status_text = if is_enabled { "ENABLED" } else { "DISABLED" };
        let status_color = if is_enabled {
            Color::rgba(0.39, 0.86, 0.39, 1.0)
        } else {
            Color::rgba(0.86, 0.39, 0.39, 1.0)
        };
        let status_node = tree.create_node();
        if let Some(node) = tree.get_mut(status_node) {
            node.set_name("ModuleStatus");
            node.set_text(status_text);
            node.font_size = 11.0;
            node.line_height = 18.0;
            node.text_align = TextAlign::Right;
            node.text_color = status_color;
            node.computed_rect = Rect::new(
                base_x + content_w - 105.0,
                content_rect.y + virtual_y - scroll_y + 10.0,
                70.0,
                18.0,
            );
        }
        let _ = tree.add_child(card_node, status_node);

        let detail_node = tree.create_node();
        if let Some(node) = tree.get_mut(detail_node) {
            node.set_name("ModuleDetail");
            node.set_text(card_data.detail);
            node.font_size = 11.0;
            node.line_height = 18.0;
            node.text_align = TextAlign::Right;
            node.text_color = Color::rgba(0.55, 0.58, 0.68, 1.0);
            node.computed_rect = Rect::new(
                base_x + content_w - 240.0,
                content_rect.y + virtual_y - scroll_y + 10.0,
                130.0,
                18.0,
            );
        }
        let _ = tree.add_child(card_node, detail_node);

        // Description Paragraph
        let desc_node = tree.create_node();
        if let Some(node) = tree.get_mut(desc_node) {
            node.set_name("ModuleDesc");
            node.set_text(card_data.desc);
            node.font_size = 11.0;
            node.line_height = 15.0;
            node.text_color = Color::rgba(0.67, 0.69, 0.76, 1.0);
            node.computed_rect = Rect::new(
                base_x + 14.0,
                content_rect.y + virtual_y - scroll_y + 36.0,
                content_w - 28.0,
                42.0,
            );
        }
        let _ = tree.add_child(card_node, desc_node);

        targets.toggles.push((
            PreferencesToggleId::Module(card_data.module),
            Rect::new(
                base_x,
                content_rect.y + virtual_y - scroll_y,
                content_w,
                card_h,
            ),
        ));

        virtual_y += card_h + 14.0;
    }

    virtual_y += 10.0;
    virtual_y
}