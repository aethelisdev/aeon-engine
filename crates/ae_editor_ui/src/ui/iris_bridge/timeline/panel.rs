// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Animation Timeline Studio Root Panel Builder
//!
//! Orchestrates panel container instantiation, responsive empty-state placeholder cards,
//! and child transport, ruler, scrubber, and dope sheet lanes using 100% pure zero-`let`
//! declarative [`UiScope`] composition and direct [`ActionBus`] emission.

use super::lanes::{build_dope_sheet_lanes, refresh_lanes_playhead_live};
use super::ruler::{build_ruler_and_scrubber, refresh_scrubber_live};
use super::transport::{build_transport_toolbar, update_time_readout_live};
use super::types::{
    TIMELINE_TAG_PANEL_ROOT, TimelineClipCache, TimelinePanelState, timeline_live_hash,
    timeline_structural_hash,
};
use crate::ui::panel_layout::PanelId;
use crate::ui::panel_registry::EditorPanelContext;
use crate::ui::types::EngineUiAction;
use irisui::prelude::*;
use irisui::workbench::ActionBus;

impl TimelinePanelState {
    /// Draws the Animation Timeline Studio panel directly using typed timeline state and workbench context.
    pub(crate) fn draw_panel(&mut self, scope: &mut UiScope<'_>, ctx: &mut EditorPanelContext<'_>) {
        self.selected_entity = ctx.data.editor_state.primary_selected();
        self.clip_duration = self
            .selected_entity
            .and_then(|ent| {
                ctx.data
                    .world
                    .inspect::<ae_animation::AnimationPlayer, _>(ent, |player| {
                        build_timeline_panel(
                            scope,
                            Some(ent),
                            Some(player),
                            &mut self.cache,
                            ctx.action_bus,
                        )
                    })
                    .ok()
            })
            .unwrap_or_else(|| {
                build_timeline_panel(
                    scope,
                    self.selected_entity,
                    None,
                    &mut self.cache,
                    ctx.action_bus,
                )
            });
    }

    /// Streams 60 FPS playhead, scrubber, and time readout updates in place during `InPlaceRefresh`
    /// via [`EditorPanelContext::sync_live_revision`], triggering granular rebuilds on structural edits.
    pub(crate) fn refresh_panel(
        &mut self,
        scope: &mut UiScope<'_>,
        ctx: &mut EditorPanelContext<'_>,
    ) {
        ctx.data
            .editor_state
            .primary_selected()
            .and_then(|ent| {
                ctx.data
                    .world
                    .inspect::<ae_animation::AnimationPlayer, _>(ent, |player| {
                        refresh_timeline_with_player(scope, ctx, Some(ent), Some(player));
                    })
                    .ok()
            })
            .unwrap_or_else(|| {
                refresh_timeline_with_player(
                    scope,
                    ctx,
                    ctx.data.editor_state.primary_selected(),
                    None,
                );
            });
    }
}

fn refresh_timeline_with_player(
    scope: &mut UiScope<'_>,
    ctx: &mut EditorPanelContext<'_>,
    entity: Option<ae_ecs::Entity>,
    player: Option<&ae_animation::AnimationPlayer>,
) {
    ctx.sync_live_revision(
        scope,
        TIMELINE_TAG_PANEL_ROOT,
        PanelId::AnimationTimeline,
        timeline_structural_hash(entity, player),
        timeline_live_hash(player),
        |live_scope| {
            player.into_iter().for_each(|p| {
                refresh_player_widgets(
                    live_scope,
                    p.current_time,
                    p.current_clip.as_ref().map_or(1.0, |c| c.duration.max(0.1)),
                );
            });
        },
    );
}

fn refresh_player_widgets(scope: &mut UiScope<'_>, current_time: f32, duration: f32) {
    update_time_readout_live(scope, current_time, duration, 30.0);
    refresh_scrubber_live(scope, duration, current_time);
    refresh_lanes_playhead_live(scope, duration, current_time);
}

/// Builds the complete Animation Timeline Studio docked panel via pure declarative [`UiScope`].
///
/// Emits centered empty-state cards when no entity or `AnimationPlayer` is active, or renders
/// the transport toolbar, adaptive time ruler scrubber, and dope sheet channel lanes.
/// Returns the resolved clip duration in seconds.
pub fn build_timeline_panel(
    scope: &mut UiScope<'_>,
    entity: Option<ae_ecs::Entity>,
    animation_player: Option<&ae_animation::AnimationPlayer>,
    cache: &mut TimelineClipCache,
    action_bus: &mut ActionBus<EngineUiAction>,
) -> f32 {
    scope.record_retained_revision(
        TIMELINE_TAG_PANEL_ROOT,
        timeline_structural_hash(entity, animation_player),
        timeline_live_hash(animation_player),
    );
    match (entity, animation_player) {
        (None, _) => {
            cache.sync(None);
            render_empty_placeholder(scope);
            0.0
        }
        (Some(entity), None) => {
            cache.sync(None);
            render_missing_player_card(scope, entity, action_bus);
            0.0
        }
        (Some(entity), Some(player)) => build_active_timeline(
            scope,
            entity,
            player,
            player
                .current_clip
                .as_ref()
                .map_or(1.0, |c| c.duration.max(0.1)),
            cache,
            action_bus,
        ),
    }
}

fn build_active_timeline(
    scope: &mut UiScope<'_>,
    entity: ae_ecs::Entity,
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
    entity: ae_ecs::Entity,
    action_bus: &mut ActionBus<EngineUiAction>,
) {
    render_missing_player_card_themed(scope, entity, action_bus, scope.theme());
}

fn render_missing_player_card_themed(
    scope: &mut UiScope<'_>,
    entity: ae_ecs::Entity,
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