// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Audio Components Inspector Cards
//!
//! Provides inspection cards for AudioSource and AudioListener components.

use super::super::registry::{ComponentInspectorHandler, ComponentRenderContext};
use super::super::types::ComponentCategory;
use irisui::prelude::*;

/// Opens the native audio file picker dialog and returns the selected file path string, if chosen.
#[must_use]
fn pick_audio_file_path() -> Option<String> {
    rfd::FileDialog::new()
        .add_filter("Audio Files", &["wav", "ogg", "mp3", "flac"])
        .pick_file()
        .map(|p| p.to_string_lossy().into_owned())
}

/// Applies a selected audio file path to an [`ae_audio::AudioSource`] component and records an atomic undo transaction.
pub fn apply_picked_audio_path(
    audio: &mut ae_audio::AudioSource,
    ctx: &mut ComponentRenderContext<'_>,
    path: String,
) {
    ctx.begin_edit("AudioSource");
    audio.sound_path = path;
    audio.is_playing = true;
    ctx.commit_edit("AudioSource");
}

/// Renders the declarative Audio Source component card with asset file slot, volume/pitch scrubbers,
/// and spatial/looping/play-on-start checkboxes.
pub fn draw_audio_source_card(
    scope: &mut UiScope<'_>,
    ctx: &mut ComponentRenderContext<'_>,
    audio: &mut ae_audio::AudioSource,
) {
    scope.card_panel("AudioSourceCard", |card| {
        AudioSourceHandler.render_header(card);

        card.asset_slot_row(
            AssetSlotConfig::untagged("Sound:", &audio.sound_path, "Select sound...")
                .with_play_state(audio.is_playing)
                .with_icon(crate::ui::iris_bridge::icons::ICON_FOLDER),
        )
        .on_pick(|| {
            if let Some(path) = pick_audio_file_path() {
                apply_picked_audio_path(audio, ctx, path);
            }
        })
        .on_play_toggle(|| {
            audio.is_playing = !audio.is_playing;
        });

        ctx.prop_f32(
            card,
            "AudioSource",
            "Volume:",
            &mut audio.volume,
            0.0..=1.0,
            0.02,
        );
        ctx.prop_f32(
            card,
            "AudioSource",
            "Pitch:",
            &mut audio.pitch,
            0.1..=10.0,
            0.05,
        );

        ctx.prop_checkbox(
            card,
            "AudioSource",
            "Spatial 3D Audio",
            &mut audio.is_spatial,
        );
        ctx.prop_checkbox(card, "AudioSource", "Looping", &mut audio.looping);
        ctx.prop_checkbox(
            card,
            "AudioSource",
            "Play on Start",
            &mut audio.play_on_start,
        );
    });
}

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
        if let Ok(mut audio) = ctx.world.get::<&mut ae_audio::AudioSource>(ctx.entity) {
            draw_audio_source_card(scope, ctx, &mut audio);
        }
    }

    fn spawn_default(&self, world: &mut hecs::World, entity: hecs::Entity) {
        drop(world.insert_one(entity, ae_audio::AudioSource::default()));
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
        drop(world.insert_one(entity, ae_audio::AudioListener));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::iris_bridge::inspector::tags::encode_component_delete_tag;
    use crate::ui::iris_bridge::inspector::types::InspectorPanelParams;

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
        let root = tree.create_root().expect("Root node must exist");
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
        {
            let mut scope = UiScope::new(&mut tree, root);
            handler.render_card(&mut scope, &mut ctx);
            scope.finish_layout(Rect::new(0.0, 0.0, 300.0, 600.0));
        }

        let del_tag = encode_component_delete_tag("AudioSource");
        assert!(
            tree.iter().any(|(_, n)| n.tag == del_tag),
            "AudioSource card must tag delete button with semantic delete tag"
        );

        let has_pick = tree.iter().any(|(_, n)| {
            n.name.as_deref() == Some("AudioPickBtn") && n.role == WidgetRole::Button
        });
        assert!(has_pick, "AudioPickBtn must exist as an interactive button");

        let has_play = tree.iter().any(|(_, n)| {
            n.name.as_deref() == Some("AudioPlayToggleBtn") && n.role == WidgetRole::Button
        });
        assert!(
            has_play,
            "AudioPlayToggleBtn must exist as an interactive button"
        );

        let numeric_pills = tree
            .iter()
            .filter(|(_, n)| n.name.as_deref() == Some("NumericDrag"))
            .count();
        assert_eq!(
            numeric_pills, 2,
            "AudioSource card must emit 2 numeric pills (Volume, Pitch)"
        );

        // Simulate clicking AudioPlayToggleBtn and verify in-place toggle of `is_playing`
        let play_rect = tree.assert_node_rect("AudioPlayToggleBtn");
        let click_pt = Point::new(play_rect.x + 5.0, play_rect.y + 5.0);
        let _ = tree.dispatch_event(&UiEvent::MouseDown {
            button: MouseButton::Left,
            point: click_pt,
        });
        let _ = tree.dispatch_event(&UiEvent::MouseUp {
            button: MouseButton::Left,
            point: click_pt,
        });

        {
            let mut scope = UiScope::new(&mut tree, root);
            handler.render_card(&mut scope, &mut ctx);
        }
        assert!(
            world
                .get::<&ae_audio::AudioSource>(entity)
                .unwrap()
                .is_playing,
            "Clicking AudioPlayToggleBtn must toggle is_playing in-place to true"
        );

        // Test in-place audio file path selection and undo snapshotting
        {
            let mut a = world.get::<&mut ae_audio::AudioSource>(entity).unwrap();
            apply_picked_audio_path(&mut a, &mut ctx, "assets/audio/new_hit.ogg".to_string());
        }

        {
            let a = world.get::<&ae_audio::AudioSource>(entity).unwrap();
            assert_eq!(a.sound_path, "assets/audio/new_hit.ogg");
            assert!(a.is_playing);
        }
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