// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Animation Timeline Studio Root Panel Builder
//!
//! Orchestrates panel container instantiation, hardware-accelerated clipping,
//! empty-state placeholder cards, and child transport and ruler subsystems
//! using the declarative [`PanelBuilder`] and [`UiScope`] architecture.
//!

use super::ruler::build_ruler_and_scrubber;
use super::transport::{TRANSPORT_TOOLBAR_HEIGHT, build_transport_toolbar};
use super::types::{TimelineAction, TimelinePanelParams, TimelinePanelTargets};
use irisui::prelude::*;

/// Builds the complete Animation Timeline Studio docked panel.
///
/// Dispatches to declarative placeholder cards when no valid entity or
/// `AnimationPlayer` is active, or instantiates the playback transport toolbar
/// and time ruler scrubber controls.
pub fn build_timeline_panel(
    tree: &mut UiTree,
    parent_id: WidgetId,
    params: &TimelinePanelParams<'_>,
    targets: &mut TimelinePanelTargets,
    actions: &mut Vec<TimelineAction>,
) {
    targets.panel_rect = params.panel_rect;
    targets.scrubber_track_rect = None;
    targets.playhead_needle_rect = None;
    targets.clip_duration = 0.0;

    let root_id = PanelBuilder::new(tree)
        .name("AnimationTimelinePanelRoot")
        .rect(params.panel_rect)
        .style(|s| {
            s.background(Color::rgba(0.06, 0.07, 0.09, 0.98))
                .border(1.0, Color::rgba(0.16, 0.18, 0.24, 0.60))
                .clip_children(true)
        })
        .build();
    let _ = tree.add_child(parent_id, root_id);

    // ── Case A: No Entity Selected ──
    let Some(entity) = params.entity else {
        render_empty_placeholder(
            tree,
            root_id,
            params.panel_rect,
            "🎬",
            "Animation Timeline Studio",
            "No entity selected. Select an animated 3D model in the viewport or hierarchy.",
        );
        return;
    };

    // ── Case B: Selected Entity Missing AnimationPlayer Component ──
    let Some(player) = params.animation_player else {
        render_missing_player_card(tree, root_id, params.panel_rect, params, actions, entity);
        return;
    };

    // ── Case C: Active AnimationPlayer Present ──
    let duration = player
        .current_clip
        .as_ref()
        .map_or(1.0, |c| c.duration.max(0.1));

    build_transport_toolbar(tree, root_id, params, targets, duration);

    build_ruler_and_scrubber(
        tree,
        root_id,
        params,
        targets,
        params.panel_rect.y + TRANSPORT_TOOLBAR_HEIGHT,
        duration,
    );
}

/// Renders a centered empty-state information card when no animated entity is selected.
///
/// Uses [`PanelBuilder`] for card container styling and [`UiScope`] for declarative
/// centered text hierarchy.
fn render_empty_placeholder(
    tree: &mut UiTree,
    parent_id: WidgetId,
    panel_rect: Rect,
    icon: &str,
    title: &str,
    subtitle: &str,
) {
    let card_w = 460.0_f32.min(panel_rect.width - 40.0).max(280.0);
    let card_h = 70.0;
    let card_x = panel_rect.x + (panel_rect.width - card_w) * 0.5;
    let card_y = panel_rect.y + (panel_rect.height - card_h) * 0.5;
    let card_rect = Rect::new(card_x, card_y, card_w, card_h);

    let card_id = PanelBuilder::new(tree)
        .name("TimelineEmptyCard")
        .rect(card_rect)
        .style(|s| {
            s.background(Color::rgba(0.09, 0.11, 0.15, 0.85))
                .border_radius(6.0)
                .border(1.0, Color::rgba(0.20, 0.24, 0.32, 0.50))
                .flex_col()
                .align_items(AlignItems::Center)
                .padding_insets(Insets::new(12.0, 10.0, 10.0, 10.0))
                .gap(4.0)
        })
        .build();
    let _ = tree.add_child(parent_id, card_id);

    let mut scope = UiScope::new(tree, card_id);
    scope.label(
        format!("{}  {}", icon, title),
        13.0,
        Color::rgba(0.70, 0.75, 0.85, 1.0),
        TextAlign::Center,
    );
    scope.label(
        subtitle,
        10.5,
        Color::rgba(0.48, 0.52, 0.62, 1.0),
        TextAlign::Center,
    );
    scope.finish_layout(card_rect);
}

/// Renders an informational card with an action button when an entity is selected without AnimationPlayer.
///
/// Emits declarative title and action button via [`UiScope`], immediately reacting to button
/// activation via the declarative `.clicked()` pattern.
fn render_missing_player_card(
    tree: &mut UiTree,
    parent_id: WidgetId,
    panel_rect: Rect,
    params: &TimelinePanelParams<'_>,
    actions: &mut Vec<TimelineAction>,
    entity: hecs::Entity,
) {
    let card_w = 480.0_f32.min(panel_rect.width - 40.0).max(280.0);
    let card_h = 80.0;
    let card_x = panel_rect.x + (panel_rect.width - card_w) * 0.5;
    let card_y = panel_rect.y + (panel_rect.height - card_h) * 0.5;
    let card_rect = Rect::new(card_x, card_y, card_w, card_h);

    let card_id = PanelBuilder::new(tree)
        .name("TimelineMissingPlayerCard")
        .rect(card_rect)
        .style(|s| {
            s.background(Color::rgba(0.09, 0.11, 0.15, 0.85))
                .border_radius(6.0)
                .border(1.0, Color::rgba(0.25, 0.28, 0.38, 0.60))
                .flex_col()
                .align_items(AlignItems::Center)
                .padding_insets(Insets::new(10.0, 10.0, 10.0, 10.0))
                .gap(8.0)
        })
        .build();
    let _ = tree.add_child(parent_id, card_id);

    let mut scope =
        UiScope::with_tagged_interactions(tree, card_id, params.events, params.hovered_tag);
    scope.label(
        "Selected entity does not have an AnimationPlayer component.",
        11.5,
        Color::rgba(0.72, 0.76, 0.86, 1.0),
        TextAlign::Center,
    );

    if scope.button("➕ Add AnimationPlayer").clicked() {
        actions.push(TimelineAction::AddAnimationPlayer(entity));
    }

    scope.finish_layout(card_rect);
}