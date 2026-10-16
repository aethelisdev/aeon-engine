// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Animation Timeline Studio Root Panel Builder
//!
//! Orchestrates panel container instantiation, responsive empty-state placeholder cards,
//! and child transport and ruler subsystems using 100% pure declarative [`UiScope`] architecture
//! with zero external node identifier retention or post-layout tree inspections.
//!

use super::lanes::build_dope_sheet_lanes;
use super::ruler::build_ruler_and_scrubber;
use super::transport::build_transport_toolbar;
use super::types::{TIMELINE_TAG_PANEL_ROOT, TimelineAction, TimelinePanelParams};
use irisui::prelude::*;

/// Builds the complete Animation Timeline Studio docked panel via pure declarative [`UiScope`].
///
/// Dispatches to declarative placeholder cards when no valid entity or
/// `AnimationPlayer` is active, or instantiates the playback transport toolbar,
/// time ruler scrubber controls, and dope sheet channel track lanes.
///
/// Returns the resolved clip duration in seconds.
pub fn build_timeline_panel(
    tree: &mut UiTree,
    parent_id: WidgetId,
    params: &TimelinePanelParams<'_>,
    actions: &mut Vec<TimelineAction>,
) -> f32 {
    let mut resolved_duration = 0.0;

    let mut scope = UiScope::with_tagged_interactions(tree, parent_id, params.events, None);

    scope
        .panel("AnimationTimelinePanelRoot")
        .tagged(TIMELINE_TAG_PANEL_ROOT)
        .background_themed(|t| t.panel_bg)
        .border_themed(1.0, |t| t.panel_border)
        .show(|panel_scope| {
            // ── Case A: No Entity Selected ──
            let Some(entity) = params.entity else {
                render_empty_placeholder(
                    panel_scope,
                    params.panel_rect.height,
                    "🎬",
                    "Animation Timeline Studio",
                    "No entity selected. Select an animated 3D model in the viewport or hierarchy.",
                );
                return;
            };

            // ── Case B: Selected Entity Missing AnimationPlayer Component ──
            let Some(player) = params.animation_player else {
                render_missing_player_card(panel_scope, params.panel_rect.height, actions, entity);
                return;
            };

            // ── Case C: Active AnimationPlayer Present ──
            let duration = player
                .current_clip
                .as_ref()
                .map_or(1.0, |c| c.duration.max(0.1));
            resolved_duration = duration;

            build_transport_toolbar(panel_scope, params, duration);
            build_ruler_and_scrubber(panel_scope, params, duration);
            build_dope_sheet_lanes(panel_scope, params, duration);
        });

    resolved_duration
}

/// Renders a centered empty-state information card when no animated entity is selected.
///
/// Uses pure flexbox centering (`JustifyContent::Center`, `AlignItems::Center`) without
/// manual coordinate math or pixel offsets.
fn render_empty_placeholder(
    scope: &mut UiScope<'_>,
    height: f32,
    icon: &str,
    title: &str,
    subtitle: &str,
) {
    scope
        .column_with()
        .name("TimelineEmptyCenterWrapper")
        .height(height)
        .centered()
        .show(|center_col| {
            center_col
                .column_with()
                .name("TimelineEmptyCard")
                .width(460.0)
                .background_themed(|t| t.card_bg)
                .border_radius(6.0)
                .border_themed(1.0, |t| t.card_border)
                .align_items(AlignItems::Center)
                .padding_insets(Insets::new(12.0, 16.0, 12.0, 16.0))
                .gap(4.0)
                .show(|card_scope| {
                    card_scope.colored_label(
                        card_scope.theme().text_primary,
                        format!("{}  {}", icon, title),
                    );
                    card_scope.colored_label(card_scope.theme().text_muted, subtitle);
                });
        });
}

/// Renders an informational card with an action button when an entity is selected without AnimationPlayer.
///
/// Emits declarative title and action button via pure [`UiScope`], reacting to button
/// activation via the declarative `.clicked()` pattern with automatic flexbox centering.
fn render_missing_player_card(
    scope: &mut UiScope<'_>,
    height: f32,
    actions: &mut Vec<TimelineAction>,
    entity: hecs::Entity,
) {
    scope
        .column_with()
        .name("TimelineMissingCenterWrapper")
        .height(height)
        .centered()
        .show(|center_col| {
            center_col
                .column_with()
                .name("TimelineMissingPlayerCard")
                .width(480.0)
                .background_themed(|t| t.card_bg)
                .border_radius(6.0)
                .border_themed(1.0, |t| t.card_border)
                .align_items(AlignItems::Center)
                .padding_insets(Insets::new(12.0, 16.0, 12.0, 16.0))
                .gap(8.0)
                .show(|card_scope| {
                    card_scope.colored_label(
                        card_scope.theme().text_secondary,
                        "Selected entity does not have an AnimationPlayer component.",
                    );

                    if card_scope.button("➕ Add AnimationPlayer").clicked() {
                        actions.push(TimelineAction::AddAnimationPlayer(entity));
                    }
                });
        });
}