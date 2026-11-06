// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Audio Components Inspector Cards
//!
//! Provides inspection cards for AudioSource and AudioListener components.

use super::super::registry::{ComponentInspectorHandler, ComponentRenderContext};
use super::super::tags::{TAG_INSPECTOR_AUDIO_PICK, TAG_INSPECTOR_AUDIO_PLAY};
use super::super::types::ComponentCategory;
use irisui::prelude::*;

/// Inspector handler for `AudioSource` component.
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

    fn render_card(&self, scope: &mut UiScope<'_>, ctx: &mut ComponentRenderContext<'_>) {
        let Ok(mut audio) = ctx.world.get::<&mut ae_audio::AudioSource>(ctx.entity) else {
            return;
        };

        scope.card_panel("AudioSourceCard", |card| {
            self.render_header(card);

            card.asset_slot_row(
                AssetSlotConfig::new(
                    "Sound:",
                    &audio.sound_path,
                    "Select sound...",
                    TAG_INSPECTOR_AUDIO_PICK,
                )
                .with_play(TAG_INSPECTOR_AUDIO_PLAY, audio.is_playing)
                .with_icon(crate::ui::iris_bridge::icons::ICON_FOLDER),
            );

            ctx.prop_f32(
                card,
                self.component_name(),
                "Volume:",
                &mut audio.volume,
                0.0..=1.0,
                0.02,
            );
            ctx.prop_f32(
                card,
                self.component_name(),
                "Pitch:",
                &mut audio.pitch,
                0.1..=10.0,
                0.05,
            );

            ctx.prop_checkbox(
                card,
                self.component_name(),
                "Spatial 3D Audio",
                &mut audio.is_spatial,
            );
            ctx.prop_checkbox(card, self.component_name(), "Looping", &mut audio.looping);
            ctx.prop_checkbox(
                card,
                self.component_name(),
                "Play on Start",
                &mut audio.play_on_start,
            );
        });
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

    fn render_card(&self, scope: &mut UiScope<'_>, _ctx: &mut ComponentRenderContext<'_>) {
        scope.card_panel("AudioListenerCard", |card| {
            self.render_header(card);

            card.label_styled_passive(
                "AudioListenerDesc",
                "Active 3D spatial microphone & ear for scene audio.",
                10.5,
                card.theme().text_muted,
                TextAlign::Left,
                Style::new().height(20.0),
            );
        });
    }

    fn spawn_default(&self, world: &mut hecs::World, entity: hecs::Entity) {
        let _ = world.insert_one(entity, ae_audio::AudioListener);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::iris_bridge::inspector::tags::{
        encode_component_delete_tag, resolve_audio_pick_tag, resolve_audio_play_tag,
    };
    use crate::ui::iris_bridge::inspector::types::{InspectorAction, InspectorPanelParams};

    #[test]
    fn test_audio_source_render_and_in_place_mutation() {
        let mut world = hecs::World::new();
        let entity = world.spawn((ae_audio::AudioSource {
            sound_path: "assets/audio/test.wav".to_string(),
            volume: 0.85,
            pitch: 1.10,
            looping: false,
            is_spatial: true,
            play_on_start: true,
            is_playing: false,
            ..Default::default()
        },));
        let mut tree = UiTree::new();
        let root = WidgetId::default();
        let euler = [0.0, 0.0, 0.0];
        let swatches = [];
        let params = InspectorPanelParams {
            panel_rect: Rect::new(0.0, 0.0, 300.0, 600.0),
            world: &world,
            selected_entity: Some(entity),
            inspector_euler: &euler,
            inspector_color_hex: "#ffffff",
            saved_swatches: &swatches,
            cursor_pos: Point::new(0.0, 0.0),
            active_submenu: None,
            is_add_menu_open: false,
            is_color_picker_open: false,
            active_text_input: None,
            active_rename_buffer: None,
            is_rename_all_selected: false,
            active_hex_buffer: None,
            inspector_hsv: [0.0, 0.0, 1.0],
            blink_caret: false,
        };

        let mut actions = Vec::new();
        let mut ctx = ComponentRenderContext::with_actions(
            entity,
            &world,
            &params,
            &mut actions,
            10.0,
            20.0,
            260.0,
        );
        let handler = AudioSourceHandler;
        let mut scope = UiScope::new(&mut tree, root);
        handler.render_card(&mut scope, &mut ctx);

        let del_tag = encode_component_delete_tag("AudioSource");
        assert!(
            tree.iter().any(|(_, n)| n.tag == del_tag),
            "AudioSource card must tag delete button with semantic delete tag"
        );

        let has_pick = tree.iter().any(|(_, n)| resolve_audio_pick_tag(n.tag));
        assert!(has_pick, "AudioPickBtn must be tagged");

        let has_play = tree.iter().any(|(_, n)| resolve_audio_play_tag(n.tag));
        assert!(has_play, "AudioPlayToggleBtn must be tagged");

        let numeric_pills = tree
            .iter()
            .filter(|(_, n)| n.name.as_deref() == Some("NumericDrag"))
            .count();
        assert_eq!(
            numeric_pills, 2,
            "AudioSource card must emit 2 numeric pills (Volume, Pitch)"
        );

        // Test in-place mutation and undo snapshotting
        {
            let mut a = world.get::<&mut ae_audio::AudioSource>(entity).unwrap();
            ctx.begin_edit(handler.component_name());
            a.volume = 0.50;
            a.pitch = 1.25;
            a.looping = true;
            a.is_spatial = false;
            ctx.commit_edit(handler.component_name());
        }

        {
            let a = world.get::<&ae_audio::AudioSource>(entity).unwrap();
            assert_eq!(a.volume, 0.50);
            assert_eq!(a.pitch, 1.25);
            assert!(a.looping);
            assert!(!a.is_spatial);
        }

        assert!(actions.contains(&InspectorAction::StartComponentEdit(
            entity,
            handler.component_name(),
            None,
        )));
        assert!(actions.contains(&InspectorAction::CommitComponentEdit(
            entity,
            handler.component_name()
        )));
    }

    #[test]
    fn test_audio_listener_declarative_render() {
        let mut world = hecs::World::new();
        let entity = world.spawn((ae_audio::AudioListener,));
        let mut tree = UiTree::new();
        let root = WidgetId::default();
        let euler = [0.0, 0.0, 0.0];
        let swatches = [];
        let params = InspectorPanelParams {
            panel_rect: Rect::new(0.0, 0.0, 300.0, 600.0),
            world: &world,
            selected_entity: Some(entity),
            inspector_euler: &euler,
            inspector_color_hex: "#ffffff",
            saved_swatches: &swatches,
            cursor_pos: Point::new(0.0, 0.0),
            active_submenu: None,
            is_add_menu_open: false,
            is_color_picker_open: false,
            active_text_input: None,
            active_rename_buffer: None,
            is_rename_all_selected: false,
            active_hex_buffer: None,
            inspector_hsv: [0.0, 0.0, 1.0],
            blink_caret: false,
        };

        let mut ctx = ComponentRenderContext::new(entity, &world, &params, 10.0, 20.0, 260.0);
        let handler = AudioListenerHandler;
        let mut scope = UiScope::new(&mut tree, root);
        handler.render_card(&mut scope, &mut ctx);

        let del_tag = encode_component_delete_tag("AudioListener");
        assert!(
            tree.iter().any(|(_, n)| n.tag == del_tag),
            "AudioListener card must tag delete button with semantic delete tag"
        );
    }
}