// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Animation Timeline Studio Root Panel Builder
//!
//! Orchestrates panel container instantiation, responsive empty-state placeholder cards,
//! and child transport, ruler, scrubber, and dope sheet lanes using 100% pure zero-`let`
//! declarative [`UiScope`] composition and direct [`ActionBus`] emission.

use super::lanes::build_dope_sheet_lanes;
use super::ruler::build_ruler_and_scrubber;
use super::transport::build_transport_toolbar;
use super::types::{TIMELINE_TAG_PANEL_ROOT, TimelineClipCache, TimelinePanelParams};
use crate::ui::types::EngineUiAction;
use irisui::prelude::*;
use irisui::workbench::ActionBus;

/// Builds the complete Animation Timeline Studio docked panel via pure declarative [`UiScope`].
///
/// Emits centered empty-state cards when no entity or `AnimationPlayer` is active, or renders
/// the transport toolbar, adaptive time ruler scrubber, and dope sheet channel lanes.
/// Returns the resolved clip duration in seconds.
pub fn build_timeline_panel(
    scope: &mut UiScope<'_>,
    params: &TimelinePanelParams<'_>,
    cache: &mut TimelineClipCache,
    action_bus: &mut ActionBus<EngineUiAction>,
) -> f32 {
    scope
        .panel("AnimationTimelinePanelRoot")
        .tagged(TIMELINE_TAG_PANEL_ROOT)
        .background_themed(|t| t.panel_bg)
        .border_themed(1.0, |t| t.panel_border)
        .show(
            |panel_scope| match (params.entity, params.animation_player) {
                (None, _) => {
                    cache.sync(None);
                    render_empty_placeholder(panel_scope);
                    0.0
                }
                (Some(entity), None) => {
                    cache.sync(None);
                    render_missing_player_card(panel_scope, entity, action_bus);
                    0.0
                }
                (Some(entity), Some(player)) => build_active_timeline(
                    panel_scope,
                    entity,
                    player,
                    player
                        .current_clip
                        .as_ref()
                        .map_or(1.0, |c| c.duration.max(0.1)),
                    cache,
                    action_bus,
                ),
            },
        )
        .inner
}

fn build_active_timeline(
    scope: &mut UiScope<'_>,
    entity: hecs::Entity,
    player: &ae_animation::AnimationPlayer,
    duration: f32,
    cache: &mut TimelineClipCache,
    action_bus: &mut ActionBus<EngineUiAction>,
) -> f32 {
    build_transport_toolbar(scope, entity, player, duration, action_bus);
    build_ruler_and_scrubber(scope, entity, player, duration, cache, action_bus);
    build_dope_sheet_lanes(scope, player, duration);
    duration
}

/// Renders a flex-centered empty-state information card when no animated entity is selected.
fn render_empty_placeholder(scope: &mut UiScope<'_>) {
    render_empty_placeholder_themed(scope, scope.theme());
}

fn render_empty_placeholder_themed(scope: &mut UiScope<'_>, theme: Theme) {
    scope.centered_fill(12.0, |center_col| {
        center_col.container_named(
            "TimelineEmptyCard",
            Style::new()
                .flex_col()
                .min_width(240.0)
                .max_width(520.0)
                .background(theme.card_bg)
                .border_radius(6.0)
                .border(1.0, theme.card_border)
                .align_items(AlignItems::Center)
                .padding_insets(Insets::new(14.0, 20.0, 14.0, 20.0))
                .gap(6.0),
            |card_scope| {
                card_scope.colored_label(theme.text_primary, "🎬  Animation Timeline Studio");
                card_scope.label_styled_passive_wrapped(
                    "TimelineEmptyMessage",
                    "No entity selected. Select an animated 3D model in the viewport or hierarchy.",
                    WrappedLabelDescriptor::new(11.0, theme.text_muted, Style::new())
                        .align(TextAlign::Center),
                );
            },
        );
    });
}

/// Renders a flex-centered prompt card with an action button to attach an `AnimationPlayer`.
fn render_missing_player_card(
    scope: &mut UiScope<'_>,
    entity: hecs::Entity,
    action_bus: &mut ActionBus<EngineUiAction>,
) {
    render_missing_player_card_themed(scope, entity, action_bus, scope.theme());
}

fn render_missing_player_card_themed(
    scope: &mut UiScope<'_>,
    entity: hecs::Entity,
    action_bus: &mut ActionBus<EngineUiAction>,
    theme: Theme,
) {
    scope.centered_fill(12.0, |center_col| {
        center_col.container_named(
            "TimelineMissingPlayerCard",
            Style::new()
                .flex_col()
                .min_width(240.0)
                .max_width(520.0)
                .background(theme.card_bg)
                .border_radius(6.0)
                .border(1.0, theme.card_border)
                .align_items(AlignItems::Center)
                .padding_insets(Insets::new(14.0, 20.0, 14.0, 20.0))
                .gap(8.0),
            |card_scope| {
                card_scope.label_styled_passive_wrapped(
                    "TimelineMissingPlayerMessage",
                    "Selected entity does not have an AnimationPlayer component.",
                    WrappedLabelDescriptor::new(11.5, theme.text_secondary, Style::new())
                        .align(TextAlign::Center),
                );
                card_scope.button("➕ Add AnimationPlayer").on_click(|| {
                    action_bus.emit(EngineUiAction::AddComponent(entity, "AnimationPlayer"))
                });
            },
        );
    });
}