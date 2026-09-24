// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Animation Timeline Studio Root Panel Builder
//!
//! Orchestrates panel container instantiation, responsive empty-state placeholder cards,
//! and child transport and ruler subsystems using 100% pure declarative [`UiScope`] architecture
//! with zero external node identifier retention or post-layout tree inspections.
//!

use super::ruler::build_ruler_and_scrubber;
use super::transport::build_transport_toolbar;
use super::types::{TimelineAction, TimelinePanelParams};
use irisui::prelude::*;

/// Builds the complete Animation Timeline Studio docked panel via pure declarative [`UiScope`].
///
/// Dispatches to declarative placeholder cards when no valid entity or
/// `AnimationPlayer` is active, or instantiates the playback transport toolbar
/// and time ruler scrubber controls.
///
/// Returns the resolved clip duration in seconds.
pub fn build_timeline_panel(
    tree: &mut UiTree,
    parent_id: WidgetId,
    params: &TimelinePanelParams<'_>,
    actions: &mut Vec<TimelineAction>,
) -> f32 {
    let mut resolved_duration = 0.0;

    let mut scope =
        UiScope::with_tagged_interactions(tree, parent_id, params.events, params.hovered_tag);

    let root_style = Style::new()
        .flex_col()
        .background(Color::rgba(0.06, 0.07, 0.09, 0.98))
        .border(1.0, Color::rgba(0.16, 0.18, 0.24, 0.60))
        .clip_children(true);

    scope.container_tagged(
        "AnimationTimelinePanelRoot",
        root_style,
        WidgetRole::Default,
        TIMELINE_TAG_PANEL_ROOT,
        |panel_scope| {
            // ── Case A: No Entity Selected ──
            let Some(entity) = params.entity else {
                render_empty_placeholder(
                    panel_scope,
                    params.panel_rect.height,
                    "🎬",
                    "Animation Timeline Studio",
                    "No entity selected. Select an animated 3D model in the viewport or hierarchy.",
                );
                panel_scope.finish_layout(params.panel_rect);
                return;
            };

            // ── Case B: Selected Entity Missing AnimationPlayer Component ──
            let Some(player) = params.animation_player else {
                render_missing_player_card(panel_scope, params.panel_rect.height, actions, entity);
                panel_scope.finish_layout(params.panel_rect);
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

            panel_scope.finish_layout(params.panel_rect);
        },
    );

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
    scope.container_named(
        "TimelineEmptyCenterWrapper",
        Style::new()
            .flex_col()
            .height(height)
            .justify_content(JustifyContent::Center)
            .align_items(AlignItems::Center),
        |center_col| {
            let card_style = Style::new()
                .width(460.0)
                .background(Color::rgba(0.09, 0.11, 0.15, 0.85))
                .border_radius(6.0)
                .border(1.0, Color::rgba(0.20, 0.24, 0.32, 0.50))
                .flex_col()
                .align_items(AlignItems::Center)
                .padding_insets(Insets::new(12.0, 16.0, 12.0, 16.0))
                .gap(4.0);

            center_col.container_named("TimelineEmptyCard", card_style, |card_scope| {
                card_scope.label(
                    format!("{}  {}", icon, title),
                    13.0,
                    Color::rgba(0.70, 0.75, 0.85, 1.0),
                    TextAlign::Center,
                );
                card_scope.label(
                    subtitle,
                    10.5,
                    Color::rgba(0.48, 0.52, 0.62, 1.0),
                    TextAlign::Center,
                );
            });
        },
    );
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
    scope.container_named(
        "TimelineMissingCenterWrapper",
        Style::new()
            .flex_col()
            .height(height)
            .justify_content(JustifyContent::Center)
            .align_items(AlignItems::Center),
        |center_col| {
            let card_style = Style::new()
                .width(480.0)
                .background(Color::rgba(0.09, 0.11, 0.15, 0.85))
                .border_radius(6.0)
                .border(1.0, Color::rgba(0.25, 0.28, 0.38, 0.60))
                .flex_col()
                .align_items(AlignItems::Center)
                .padding_insets(Insets::new(12.0, 16.0, 12.0, 16.0))
                .gap(8.0);

            center_col.container_named("TimelineMissingPlayerCard", card_style, |card_scope| {
                card_scope.label(
                    "Selected entity does not have an AnimationPlayer component.",
                    11.5,
                    Color::rgba(0.72, 0.76, 0.86, 1.0),
                    TextAlign::Center,
                );

                if card_scope.button("➕ Add AnimationPlayer").clicked() {
                    actions.push(TimelineAction::AddAnimationPlayer(entity));
                }
            });
        },
    );
}