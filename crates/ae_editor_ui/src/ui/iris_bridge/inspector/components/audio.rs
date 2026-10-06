// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Audio Components Inspector Cards
//!
//! Provides inspection cards for  AudioSource and AudioListener components.

use super::super::registry::{ComponentInspectorHandler, ComponentRenderContext};
use super::super::tags::{TAG_INSPECTOR_AUDIO_PICK, TAG_INSPECTOR_AUDIO_PLAY};
use super::super::types::{ComponentCategory, ComponentCheckboxId, InspectorNumberInputId};
use super::physics::helpers::{
    ComponentHeaderProps, DeclarativeNumericRowParams, build_declarative_card_header,
    render_declarative_checkbox_row, render_declarative_numeric_row,
};
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

        let get_edit = |id| {
            ctx.params
                .active_number_input
                .filter(|s| s.id == id)
                .map(|s| s.to_edit_state(ctx.params.blink_caret))
        };

        let card_style = Style::new()
            .flex_col()
            .background(Color::rgba(0.090, 0.094, 0.110, 0.98))
            .border(1.0, Color::rgba(0.133, 0.141, 0.165, 0.85))
            .border_radius(6.0)
            .padding_insets(Insets::new(6.0, 8.0, 6.0, 8.0))
            .gap(4.0);

        scope.container_named("AudioSourceCard", card_style, |card| {
            build_declarative_card_header(
                card,
                ComponentHeaderProps {
                    atlas_icon: None,
                    icon: self.icon(),
                    display_title: self.display_title(),
                    header_color: self.header_color(),
                    component_name: self.component_name(),
                },
                false,
            );

            // Row 1: Sound Path + Pick File Button + Play/Stop Preview Button
            let row_style = Style::new()
                .flex_row()
                .align_items(AlignItems::Center)
                .height(22.0)
                .gap(4.0);

            card.container_named("AudioSoundRow", row_style, |row| {
                row.label_styled_passive(
                    "AudioPathLbl",
                    "Sound:",
                    11.0,
                    Color::rgba(0.620, 0.635, 0.678, 1.0),
                    TextAlign::Left,
                    Style::new().width(46.0).height(22.0),
                );

                let file_name = if path_str.is_empty() {
                    "Select sound...".to_string()
                } else {
                    std::path::Path::new(&path_str)
                        .file_name()
                        .map(|f| f.to_string_lossy().into_owned())
                        .unwrap_or_else(|| path_str.clone())
                };

                let path_box_style = Style::new()
                    .flex_row()
                    .align_items(AlignItems::Center)
                    .flex_grow(1.0)
                    .height(22.0)
                    .background(Color::rgba(0.157, 0.165, 0.188, 0.98))
                    .border(1.0, Color::rgba(0.212, 0.220, 0.259, 0.85))
                    .border_radius(4.0)
                    .padding_insets(Insets::new(0.0, 6.0, 0.0, 6.0));

                row.container_named("AudioPathBox", path_box_style, |pbox| {
                    pbox.label_styled_passive(
                        "AudioPathTxt",
                        &file_name,
                        10.0,
                        if path_str.is_empty() {
                            Color::rgba(0.45, 0.47, 0.52, 1.0)
                        } else {
                            Color::rgba(0.886, 0.894, 0.918, 1.0)
                        },
                        TextAlign::Left,
                        Style::new().flex_grow(1.0).height(20.0),
                    );
                });

                // Pick File Button using custom master atlas folder icon
                let pick_style = Style::new()
                    .flex_row()
                    .align_items(AlignItems::Center)
                    .justify_content(JustifyContent::Center)
                    .width(20.0)
                    .height(20.0)
                    .background(Color::rgba(0.157, 0.165, 0.188, 0.98))
                    .border(1.0, Color::rgba(0.212, 0.220, 0.259, 0.85))
                    .border_radius(4.0);

                row.container_tagged(
                    "AudioPickBtn",
                    pick_style,
                    WidgetRole::Button,
                    TAG_INSPECTOR_AUDIO_PICK,
                    |btn| {
                        btn.icon_named(
                            "AudioPickBtnIcon",
                            crate::ui::iris_bridge::icons::ICON_FOLDER,
                            Color::rgba(0.85, 0.88, 0.92, 1.0),
                            14.0,
                        );
                    },
                );

                // Play/Stop Preview Toggle Button
                let (bg, border, text_col) = if is_playing {
                    (
                        Color::rgba(0.50, 0.15, 0.15, 0.95),
                        Color::rgba(0.75, 0.25, 0.25, 0.90),
                        Color::rgba(1.0, 0.85, 0.85, 1.0),
                    )
                } else {
                    (
                        Color::rgba(0.157, 0.165, 0.188, 0.98),
                        Color::rgba(0.212, 0.220, 0.259, 0.85),
                        Color::rgba(0.35, 0.75, 0.48, 1.0),
                    )
                };

                let play_style = Style::new()
                    .flex_row()
                    .align_items(AlignItems::Center)
                    .justify_content(JustifyContent::Center)
                    .width(20.0)
                    .height(20.0)
                    .background(bg)
                    .border(1.0, border)
                    .border_radius(4.0);

                row.container_tagged(
                    "AudioPlayToggleBtn",
                    play_style,
                    WidgetRole::Button,
                    TAG_INSPECTOR_AUDIO_PLAY,
                    |btn| {
                        btn.label_styled_passive(
                            "PlayIcon",
                            if is_playing { "⏹" } else { "▶" },
                            9.0,
                            text_col,
                            TextAlign::Center,
                            Style::new().width(20.0).height(20.0),
                        );
                    },
                );
            });

            // Row 2: Volume
            render_declarative_numeric_row(
                card,
                DeclarativeNumericRowParams {
                    input_id: InspectorNumberInputId::AudioVolume,
                    label: "Volume:",
                    val: volume,
                    label_w: 55.0,
                    box_w: 48.0,
                    unit: None,
                    edit_state: get_edit(InspectorNumberInputId::AudioVolume),
                    is_hovered: false,
                },
            );

            // Row 3: Pitch
            render_declarative_numeric_row(
                card,
                DeclarativeNumericRowParams {
                    input_id: InspectorNumberInputId::AudioPitch,
                    label: "Pitch:",
                    val: pitch,
                    label_w: 55.0,
                    box_w: 48.0,
                    unit: None,
                    edit_state: get_edit(InspectorNumberInputId::AudioPitch),
                    is_hovered: false,
                },
            );

            // Row 4: Spatial 3D Audio
            render_declarative_checkbox_row(
                card,
                ComponentCheckboxId::AudioSpatial,
                "Spatial 3D Audio",
                is_spatial,
                false,
            );

            // Row 5: Looping
            render_declarative_checkbox_row(
                card,
                ComponentCheckboxId::AudioLoop,
                "Looping",
                looping,
                false,
            );

            // Row 6: Play on Start
            render_declarative_checkbox_row(
                card,
                ComponentCheckboxId::AudioPlayOnStart,
                "Play on Start",
                play_on_start,
                false,
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
        let card_style = Style::new()
            .flex_col()
            .background(Color::rgba(0.090, 0.094, 0.110, 0.98))
            .border(1.0, Color::rgba(0.133, 0.141, 0.165, 0.85))
            .border_radius(6.0)
            .padding_insets(Insets::new(6.0, 8.0, 6.0, 8.0))
            .gap(4.0);

        scope.container_named("AudioListenerCard", card_style, |card| {
            build_declarative_card_header(
                card,
                ComponentHeaderProps {
                    atlas_icon: None,
                    icon: self.icon(),
                    display_title: self.display_title(),
                    header_color: self.header_color(),
                    component_name: self.component_name(),
                },
                false,
            );

            card.label_styled_passive(
                "AudioListenerDesc",
                "Active 3D spatial microphone & ear for scene audio.",
                10.5,
                Color::rgba(0.620, 0.635, 0.678, 1.0),
                TextAlign::Left,
                Style::new().height(20.0),
            );
        });
    }

    fn spawn_default(&self, world: &mut hecs::World, entity: hecs::Entity) {
        let _ = world.insert_one(entity, ae_audio::AudioListener);
    }
}