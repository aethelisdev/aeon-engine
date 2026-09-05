// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Audio Components Inspector Cards
//!
//! Provides inspection cards for  AudioSource and AudioListener components.

use super::super::registry::{ComponentInspectorHandler, ComponentRenderContext};
use super::super::types::{
    CompactNumericRowParams, ComponentCategory, ComponentCheckboxId, InspectorNumberInputId,
};
use super::physics::{render_checkbox_row, render_component_header, render_numeric_row_compact};
use irisui::prelude::*;

/// Inspector handler for AudioSource` component.
pub struct AudioSourceHandler;

impl ComponentInspectorHandler for AudioSourceHandler {
    fn component_name(&self) -> &'static str {
        "AudioSource"
    }

    fn display_title(&self) -> &'static str {
        "Audio Source"
    }

    fn icon(&self) -> &'static str {
        "🔊"
    }

    fn header_color(&self) -> Color {
        Color::rgba(0.22, 0.74, 0.97, 1.0) // Vibrant Sky Blue (#38bdf8)
    }

    fn category(&self) -> ComponentCategory {
        ComponentCategory::Audio
    }

    fn has_component(&self, world: &hecs::World, entity: hecs::Entity) -> bool {
        world.get::<&ae_audio::AudioSource>(entity).is_ok()
    }

    fn render_card(
        &self,
        tree: &mut UiTree,
        parent_id: WidgetId,
        ctx: &mut ComponentRenderContext<'_>,
    ) -> f32 {
        let (path_str, volume, pitch, is_spatial, looping, play_on_start, is_playing) =
            if let Ok(audio) = ctx.world.get::<&ae_audio::AudioSource>(ctx.entity) {
                (
                    audio.sound_path.clone(),
                    audio.volume,
                    audio.pitch,
                    audio.is_spatial,
                    audio.looping,
                    audio.play_on_start,
                    audio.is_playing,
                )
            } else {
                (String::new(), 1.0, 1.0, true, false, true, false)
            };

        let padding = 8.0;
        let row_h = 22.0;
        let spacing = 4.0;
        let card_h = 24.0 + 6.0 * (row_h + spacing) + padding * 2.0;
        let card_rect = Rect::new(ctx.base_x, ctx.base_y, ctx.card_w, card_h);

        let card_id = tree.create_node();
        if let Some(node) = tree.get_mut(card_id) {
            node.set_name("AudioSourceCard");
            node.computed_rect = card_rect;
            node.style = Style::new()
                .background(Color::rgba(0.090, 0.094, 0.110, 0.98))
                .border(1.0, Color::rgba(0.133, 0.141, 0.165, 0.85))
                .border_radius(6.0);
        }
        let _ = tree.add_child(parent_id, card_id);

        render_component_header(
            tree,
            card_id,
            ctx,
            self.icon(),
            self.display_title(),
            self.header_color(),
            self.component_name(),
        );

        let mut cur_y = ctx.base_y + padding + 24.0 + 4.0;

        // Row 1: Sound Path & Pick File Button & Play/Stop Preview Button
        let label_w = 46.0;
        let btn_size = 20.0;
        let path_box_w = (ctx.card_w - padding * 2.0 - label_w - (btn_size * 2.0) - 10.0).max(40.0);

        let lbl_id = tree.create_node();
        if let Some(node) = tree.get_mut(lbl_id) {
            node.set_name("AudioPathLbl");
            node.set_text("Sound:");
            node.font_size = 11.0;
            node.line_height = row_h;
            node.text_color = Color::rgba(0.620, 0.635, 0.678, 1.0);
            node.computed_rect = Rect::new(ctx.base_x + padding, cur_y, label_w, row_h);
        }
        let _ = tree.add_child(card_id, lbl_id);

        let path_box_rect = Rect::new(
            ctx.base_x + padding + label_w + 2.0,
            cur_y,
            path_box_w,
            row_h,
        );
        let path_box_id = tree.create_node();
        if let Some(node) = tree.get_mut(path_box_id) {
            node.set_name("AudioPathBox");
            node.computed_rect = path_box_rect;
            node.style = Style::new()
                .background(Color::rgba(0.157, 0.165, 0.188, 0.98))
                .border(1.0, Color::rgba(0.212, 0.220, 0.259, 0.85))
                .border_radius(4.0);
        }
        let _ = tree.add_child(card_id, path_box_id);

        let path_txt_id = tree.create_node();
        if let Some(node) = tree.get_mut(path_txt_id) {
            node.set_name("AudioPathTxt");
            let file_name = if path_str.is_empty() {
                "Select sound...".to_string()
            } else {
                std::path::Path::new(&path_str)
                    .file_name()
                    .map(|f| f.to_string_lossy().into_owned())
                    .unwrap_or_else(|| path_str.clone())
            };
            node.set_text(file_name);
            node.font_size = 10.0;
            node.line_height = row_h;
            node.text_color = if path_str.is_empty() {
                Color::rgba(0.45, 0.47, 0.52, 1.0)
            } else {
                Color::rgba(0.886, 0.894, 0.918, 1.0)
            };
            node.computed_rect = Rect::new(
                path_box_rect.x + 4.0,
                cur_y,
                path_box_rect.width - 8.0,
                row_h,
            );
        }
        let _ = tree.add_child(path_box_id, path_txt_id);

        // Pick File Button using custom master atlas folder icon
        let btn_rect = Rect::new(path_box_rect.right() + 4.0, cur_y + 1.0, btn_size, btn_size);
        let is_btn_hovered = btn_rect.contains_point(ctx.params.cursor_pos);
        let btn_id = tree.create_node();
        if let Some(node) = tree.get_mut(btn_id) {
            node.set_name("AudioPickBtn");
            node.computed_rect = btn_rect;
            let (bg, border) = if is_btn_hovered {
                (
                    Color::rgba(0.200, 0.208, 0.235, 1.0),
                    Color::rgba(0.271, 0.282, 0.329, 0.95),
                )
            } else {
                (
                    Color::rgba(0.157, 0.165, 0.188, 0.98),
                    Color::rgba(0.212, 0.220, 0.259, 0.85),
                )
            };
            node.style = Style::new()
                .background(bg)
                .border(1.0, border)
                .border_radius(4.0);
        }
        let _ = tree.add_child(card_id, btn_id);

        // Child node for custom vector folder icon from master atlas
        let ic_id = tree.create_node();
        if let Some(node) = tree.get_mut(ic_id) {
            node.set_name("AudioPickBtnIcon");
            node.texture_uv = Some(crate::ui::iris_bridge::icons::ICON_FOLDER);
            let tint = if is_btn_hovered {
                Color::WHITE
            } else {
                Color::rgba(0.85, 0.88, 0.92, 1.0)
            };
            node.texture_tint = Some(tint);
            let ic_size = 14.0;
            node.computed_rect = Rect::new(
                btn_rect.x + (btn_size - ic_size) * 0.5,
                btn_rect.y + (btn_size - ic_size) * 0.5,
                ic_size,
                ic_size,
            );
        }
        let _ = tree.add_child(btn_id, ic_id);
        ctx.targets.audio_pick_btn_rect = Some(btn_rect);

        // Play/Stop Preview Toggle Button `[ ▶ / ⏹ ]`
        let play_rect = Rect::new(btn_rect.right() + 4.0, cur_y + 1.0, btn_size, btn_size);
        let is_play_hovered = play_rect.contains_point(ctx.params.cursor_pos);
        let play_id = tree.create_node();
        if let Some(node) = tree.get_mut(play_id) {
            node.set_name("AudioPlayToggleBtn");
            node.computed_rect = play_rect;
            let (bg, border, text_col) = if is_playing {
                if is_play_hovered {
                    (
                        Color::rgba(0.70, 0.20, 0.20, 1.0),
                        Color::rgba(0.90, 0.35, 0.35, 1.0),
                        Color::WHITE,
                    )
                } else {
                    (
                        Color::rgba(0.50, 0.15, 0.15, 0.95),
                        Color::rgba(0.75, 0.25, 0.25, 0.90),
                        Color::rgba(1.0, 0.85, 0.85, 1.0),
                    )
                }
            } else if is_play_hovered {
                (
                    Color::rgba(0.200, 0.208, 0.235, 1.0),
                    Color::rgba(0.271, 0.282, 0.329, 0.95),
                    Color::rgba(0.40, 0.85, 0.55, 1.0),
                )
            } else {
                (
                    Color::rgba(0.157, 0.165, 0.188, 0.98),
                    Color::rgba(0.212, 0.220, 0.259, 0.85),
                    Color::rgba(0.35, 0.75, 0.48, 1.0),
                )
            };
            node.style = Style::new()
                .background(bg)
                .border(1.0, border)
                .border_radius(4.0);
            node.set_text(if is_playing { "⏹" } else { "▶" });
            node.font_size = 9.0;
            node.line_height = btn_size;
            node.text_align = TextAlign::Center;
            node.text_color = text_col;
        }
        let _ = tree.add_child(card_id, play_id);
        ctx.targets.audio_play_btn_rect = Some(play_rect);

        cur_y += row_h + spacing;

        // Row 2: Volume
        render_numeric_row_compact(
            tree,
            card_id,
            ctx,
            CompactNumericRowParams {
                label: "Volume:",
                input_id: InspectorNumberInputId::AudioVolume,
                val: volume,
                row_y: cur_y,
                label_w: 55.0,
                box_w: 48.0,
                unit: None,
            },
        );
        cur_y += row_h + spacing;

        // Row 3: Pitch
        render_numeric_row_compact(
            tree,
            card_id,
            ctx,
            CompactNumericRowParams {
                label: "Pitch:",
                input_id: InspectorNumberInputId::AudioPitch,
                val: pitch,
                row_y: cur_y,
                label_w: 55.0,
                box_w: 48.0,
                unit: None,
            },
        );
        cur_y += row_h + spacing;

        // Row 4: Spatial 3D
        render_checkbox_row(
            tree,
            card_id,
            ctx,
            "Spatial 3D Audio",
            ComponentCheckboxId::AudioSpatial,
            is_spatial,
            cur_y,
        );
        cur_y += row_h + spacing;

        // Row 5: Looping
        render_checkbox_row(
            tree,
            card_id,
            ctx,
            "Looping",
            ComponentCheckboxId::AudioLoop,
            looping,
            cur_y,
        );
        cur_y += row_h + spacing;

        // Row 6: Play on Start
        render_checkbox_row(
            tree,
            card_id,
            ctx,
            "Play on Start",
            ComponentCheckboxId::AudioPlayOnStart,
            play_on_start,
            cur_y,
        );

        card_h
    }

    fn spawn_default(&self, world: &mut hecs::World, entity: hecs::Entity) {
        let _ = world.insert_one(entity, ae_audio::AudioSource::default());
    }
}

/// Inspector handler for `👂 AudioListener` component.
pub struct AudioListenerHandler;

impl ComponentInspectorHandler for AudioListenerHandler {
    fn component_name(&self) -> &'static str {
        "AudioListener"
    }

    fn display_title(&self) -> &'static str {
        "Audio Listener"
    }

    fn icon(&self) -> &'static str {
        "👂"
    }

    fn header_color(&self) -> Color {
        Color::rgba(0.65, 0.55, 0.98, 1.0) // Soft Purple (#a78bfa)
    }

    fn category(&self) -> ComponentCategory {
        ComponentCategory::Audio
    }

    fn has_component(&self, world: &hecs::World, entity: hecs::Entity) -> bool {
        world.get::<&ae_audio::AudioListener>(entity).is_ok()
    }

    fn render_card(
        &self,
        tree: &mut UiTree,
        parent_id: WidgetId,
        ctx: &mut ComponentRenderContext<'_>,
    ) -> f32 {
        let padding = 8.0;
        let card_h = 24.0 + 26.0 + padding * 2.0;
        let card_rect = Rect::new(ctx.base_x, ctx.base_y, ctx.card_w, card_h);

        let card_id = tree.create_node();
        if let Some(node) = tree.get_mut(card_id) {
            node.set_name("AudioListenerCard");
            node.computed_rect = card_rect;
            node.style = Style::new()
                .background(Color::rgba(0.090, 0.094, 0.110, 0.98))
                .border(1.0, Color::rgba(0.133, 0.141, 0.165, 0.85))
                .border_radius(6.0);
        }
        let _ = tree.add_child(parent_id, card_id);

        render_component_header(
            tree,
            card_id,
            ctx,
            self.icon(),
            self.display_title(),
            self.header_color(),
            self.component_name(),
        );

        let desc_id = tree.create_node();
        if let Some(node) = tree.get_mut(desc_id) {
            node.set_name("AudioListenerDesc");
            node.set_text("Active 3D spatial microphone & ear for scene audio.");
            node.font_size = 10.5;
            node.line_height = 18.0;
            node.text_color = Color::rgba(0.620, 0.635, 0.678, 1.0);
            node.computed_rect = Rect::new(
                ctx.base_x + padding,
                ctx.base_y + padding + 24.0 + 4.0,
                ctx.card_w - padding * 2.0,
                20.0,
            );
        }
        let _ = tree.add_child(card_id, desc_id);

        card_h
    }

    fn spawn_default(&self, world: &mut hecs::World, entity: hecs::Entity) {
        let _ = world.insert_one(entity, ae_audio::AudioListener);
    }
}