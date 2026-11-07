// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Asset File Slot Row (`asset_slot_row`, `AssetSlotConfig`, `AssetSlotResponse`)
//!
//! Provides the [`UiScope::asset_slot_row`] declarative widget, [`AssetSlotConfig`] builder,
//! and [`AssetSlotResponse`] fluent callback chain (`.on_pick(...)`, `.on_play_toggle(...)`,
//! `.on_change(...)`) for inspecting file paths with optional media preview controls.

use super::core::UiScope;
use crate::declarative::types::WidgetResponse;
use iris_core::{AlignItems, Color, Insets, JustifyContent, Style, TextAlign, WidgetRole};

/// Interaction response returned by [`UiScope::asset_slot_row`].
///
/// Captures the [`WidgetResponse`] of both the file picker button and the optional
/// preview play/stop toggle button, exposing fluent `.on_pick(...)`, `.on_play_toggle(...)`,
/// and `.on_change(...)` callback chains without requiring intermediate `let` bindings in UI views.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct AssetSlotResponse {
    /// Interaction response of the file picker trigger button (`"AudioPickBtn"`).
    pub pick: WidgetResponse,
    /// Interaction response of the optional preview play/stop toggle button (`"AudioPlayToggleBtn"`).
    pub play: WidgetResponse,
}

impl AssetSlotResponse {
    /// Returns `true` if either the file picker button or the preview toggle button was clicked.
    #[inline]
    #[must_use]
    pub const fn changed(&self) -> bool {
        self.pick.clicked() || self.play.clicked()
    }

    /// Returns `true` if the file picker trigger button was clicked in the current frame.
    #[inline]
    #[must_use]
    pub const fn pick_clicked(&self) -> bool {
        self.pick.clicked()
    }

    /// Returns `true` if the preview play/stop toggle button was clicked in the current frame.
    #[inline]
    #[must_use]
    pub const fn play_clicked(&self) -> bool {
        self.play.clicked()
    }

    /// Invokes `f` when the file picker button is clicked, returning `self` for fluent chaining.
    #[inline]
    pub fn on_pick(self, f: impl FnOnce()) -> Self {
        if self.pick.clicked() {
            f();
        }
        self
    }

    /// Invokes `f` when the preview play/stop toggle button is clicked, returning `self` for fluent chaining.
    #[inline]
    pub fn on_play_toggle(self, f: impl FnOnce()) -> Self {
        if self.play.clicked() {
            f();
        }
        self
    }

    /// Invokes `f` when either the file picker button or the preview play/stop toggle button is clicked.
    #[inline]
    pub fn on_change(self, f: impl FnOnce()) -> Self {
        if self.changed() {
            f();
        }
        self
    }
}

/// Configuration for rendering a declarative asset file slot with picker and optional play preview buttons.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AssetSlotConfig<'a> {
    /// Property label displayed on the left (e.g. "Sound:").
    pub label: &'a str,
    /// Current asset path or name.
    pub path: &'a str,
    /// Placeholder text shown when `path` is empty (e.g. "Select sound...").
    pub empty_label: &'a str,
    /// Semantic 64-bit tag assigned to the file picker button (`0` uses automatic scope tag hashing).
    pub pick_tag: u64,
    /// Semantic 64-bit tag assigned to the preview toggle button, if enabled.
    pub play_tag: Option<u64>,
    /// Whether the preview media is currently playing.
    pub is_playing: bool,
    /// Optional atlas UV coordinates for the picker button icon.
    pub picker_icon_uv: Option<[f32; 4]>,
}

impl<'a> AssetSlotConfig<'a> {
    /// Constructs a new asset slot configuration with label, path, empty label, and pick tag.
    #[inline]
    #[must_use]
    pub const fn new(label: &'a str, path: &'a str, empty_label: &'a str, pick_tag: u64) -> Self {
        Self {
            label,
            path,
            empty_label,
            pick_tag,
            play_tag: None,
            is_playing: false,
            picker_icon_uv: None,
        }
    }

    /// Constructs a new asset slot configuration with automatic scope tag hashing for the file picker button.
    #[inline]
    #[must_use]
    pub const fn untagged(label: &'a str, path: &'a str, empty_label: &'a str) -> Self {
        Self::new(label, path, empty_label, 0)
    }

    /// Sets the optional preview toggle button tag and current playback state.
    #[inline]
    #[must_use]
    pub const fn with_play(mut self, play_tag: u64, is_playing: bool) -> Self {
        self.play_tag = Some(play_tag);
        self.is_playing = is_playing;
        self
    }

    /// Enables the preview play/stop toggle button with automatic scope tag hashing and current playback state.
    #[inline]
    #[must_use]
    pub const fn with_play_state(mut self, is_playing: bool) -> Self {
        self.play_tag = Some(0);
        self.is_playing = is_playing;
        self
    }

    /// Sets the optional atlas icon UV coordinates for the picker button.
    #[inline]
    #[must_use]
    pub const fn with_icon(mut self, uv: [f32; 4]) -> Self {
        self.picker_icon_uv = Some(uv);
        self
    }
}

impl<'a> UiScope<'a> {
    /// Emits a declarative asset file slot row with path display, file picker trigger, and optional play/stop toggle.
    ///
    /// Renders a horizontal container with label, styled path box, picker button, and preview button,
    /// returning an [`AssetSlotResponse`] supporting fluent `.on_pick(...)`, `.on_play_toggle(...)`,
    /// and `.on_change(...)` callback chains without requiring intermediate `let` bindings in view callers.
    pub fn asset_slot_row(&mut self, cfg: AssetSlotConfig<'_>) -> AssetSlotResponse {
        let row_style = Style::new()
            .flex_row()
            .align_items(AlignItems::Center)
            .height(22.0)
            .gap(4.0);

        let mut pick_resp = WidgetResponse::default();
        let mut play_resp = WidgetResponse::default();

        self.container_named("AudioSoundRow", row_style, |row| {
            row.label_styled_passive(
                "AudioPathLbl",
                cfg.label,
                11.0,
                row.theme().text_muted,
                TextAlign::Left,
                Style::new().width(46.0).height(22.0),
            );

            let file_name = if cfg.path.is_empty() {
                cfg.empty_label.to_string()
            } else {
                std::path::Path::new(cfg.path)
                    .file_name()
                    .map(|f| f.to_string_lossy().into_owned())
                    .unwrap_or_else(|| cfg.path.to_string())
            };

            let path_box_style = Style::new()
                .flex_row()
                .align_items(AlignItems::Center)
                .flex_grow(1.0)
                .height(22.0)
                .background(row.theme().card_header_bg)
                .border(1.0, row.theme().card_border)
                .border_radius(4.0)
                .padding_insets(Insets::new(0.0, 6.0, 0.0, 6.0));

            row.container_named("AudioPathBox", path_box_style, |pbox| {
                pbox.label_styled_passive(
                    "AudioPathTxt",
                    &file_name,
                    10.0,
                    if cfg.path.is_empty() {
                        pbox.theme().text_muted
                    } else {
                        pbox.theme().text_primary
                    },
                    TextAlign::Left,
                    Style::new().flex_grow(1.0).height(20.0),
                );
            });

            // Pick File Button
            let pick_style = Style::new()
                .flex_row()
                .align_items(AlignItems::Center)
                .justify_content(JustifyContent::Center)
                .width(20.0)
                .height(20.0)
                .background(row.theme().button_idle_bg)
                .border(1.0, row.theme().button_border)
                .border_radius(4.0);

            let pick_tag = if cfg.pick_tag != 0 {
                cfg.pick_tag
            } else {
                row.tag_for("AudioPickBtn")
            };

            let pick_container = row.container_tagged(
                "AudioPickBtn",
                pick_style,
                WidgetRole::Button,
                pick_tag,
                |btn| {
                    if let Some(uv) = cfg.picker_icon_uv {
                        btn.icon_named("AudioPickBtnIcon", uv, btn.theme().text_secondary, 14.0);
                    } else {
                        btn.label_styled_passive(
                            "AudioPickBtnText",
                            "…",
                            12.0,
                            btn.theme().text_secondary,
                            TextAlign::Center,
                            Style::new().width(20.0).height(20.0),
                        );
                    }
                },
            );
            pick_resp = pick_container.response;

            // Optional Play/Stop Preview Toggle Button
            if let Some(play_tag) = cfg.play_tag {
                let (bg, border, text_col) = if cfg.is_playing {
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

                let resolved_play_tag = if play_tag != 0 {
                    play_tag
                } else {
                    row.tag_for("AudioPlayToggleBtn")
                };

                let play_container = row.container_tagged(
                    "AudioPlayToggleBtn",
                    play_style,
                    WidgetRole::Button,
                    resolved_play_tag,
                    |btn| {
                        btn.label_styled_passive(
                            "PlayIcon",
                            if cfg.is_playing { "⏹" } else { "▶" },
                            9.0,
                            text_col,
                            TextAlign::Center,
                            Style::new().width(20.0).height(20.0),
                        );
                    },
                );
                play_resp = play_container.response;
            }
        });

        AssetSlotResponse {
            pick: pick_resp,
            play: play_resp,
        }
    }
}