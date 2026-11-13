// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Preferences Dialog Builder
//!
//! Assembles the complete hardware-accelerated GPU SDF Preferences modal dialog
//! declaratively using [`UiScope`], including glassmorphic card framing, titlebar,
//! sidebar tab navigation, content routing, scrollbar, and active dropdown popups.

use super::tabs::*;
use super::types::{
    PREF_TAG_CARD, PREF_TAG_CLOSE, PREF_TAG_CONTENT_VIEW, PREF_TAG_SCROLL_CONTAINER,
    PREF_TAG_SCROLLBAR_THUMB, PREF_TAG_SCROLLBAR_TRACK, PREF_TAG_TITLEBAR, PreferencesParams,
    encode_tab_tag, preferences_scroll_style,
};
use crate::ui::iris_bridge::types::{IrisEditorOverlay, OverlayUpdateParams};
use irisui::prelude::*;
use irisui::widgets::scroll_area::ScrollBarGeometry;

/// Width of the preferences modal card in physical pixels.
pub const PREF_CARD_WIDTH: f32 = 760.0;
/// Height of the preferences modal card in physical pixels.
pub const PREF_CARD_HEIGHT: f32 = 540.0;
/// Width of the left sidebar navigation tab strip in physical pixels.
pub const SIDEBAR_WIDTH: f32 = 160.0;
/// Height of the titlebar in physical pixels.
pub const TITLEBAR_HEIGHT: f32 = 36.0;

/// Sidebar tabs descriptor list: `(label, tab_index)`.
pub const SIDEBAR_TABS: [(&str, u8); 10] = [
    ("General", 0),
    ("Graphics", 1),
    ("Editor", 2),
    ("Navigation", 3),
    ("Input", 7),
    ("Keymap", 4),
    ("System", 5),
    ("Add-ons", 6),
    ("Modules", 9),
    ("Experimental", 8),
];

/// Sidebar tabs descriptor list with explicit semantic tags: `(label, tab_index, semantic_tag)`.
pub const SIDEBAR_TABS_TAGGED: [(&str, u8, u64); 10] = [
    ("General", 0, encode_tab_tag(0)),
    ("Graphics", 1, encode_tab_tag(1)),
    ("Editor", 2, encode_tab_tag(2)),
    ("Navigation", 3, encode_tab_tag(3)),
    ("Input", 7, encode_tab_tag(7)),
    ("Keymap", 4, encode_tab_tag(4)),
    ("System", 5, encode_tab_tag(5)),
    ("Add-ons", 6, encode_tab_tag(6)),
    ("Modules", 9, encode_tab_tag(9)),
    ("Experimental", 8, encode_tab_tag(8)),
];

/// Returns the virtual scrollable height for a given preferences tab.
#[inline]
pub fn tab_virtual_height(tab_idx: u8) -> f32 {
    match tab_idx {
        0 => 680.0,
        1 => 1800.0, // Graphics tab: complete Shadows, Performance, AA, Bloom, Atmosphere, Clouds & Fog
        2 => 1100.0,
        3 => 650.0,
        4 => 1100.0,
        5 => 650.0,
        6 => 700.0,
        7 => 600.0,
        8 => 650.0,
        9 => 850.0,
        _ => 600.0,
    }
}

/// Calculates the clamped preferences floating window position during titlebar dragging.
///
/// Delegates directly to the Iris UI floating panel drag solver.
pub fn calculate_preferences_drag_pos(
    cursor: Point,
    drag_offset: Point,
    screen_width: f32,
    screen_height: f32,
) -> Point {
    let min_y = crate::ui::iris_bridge::menubar::MENUBAR_HEIGHT;
    irisui::widgets::floating_panel::calculate_floating_drag_pos(
        cursor,
        drag_offset,
        (PREF_CARD_WIDTH, PREF_CARD_HEIGHT),
        screen_width,
        screen_height,
        min_y,
    )
}

/// Constructs the complete Preferences dialog tree declaratively using [`UiScope`].
///
/// Returns `(card_id, card_rect, content_rect, max_scroll_y)`.
pub fn build_preferences_dialog(
    tree: &mut UiTree,
    mut params: PreferencesParams<'_>,
    actions: &mut Vec<super::types::PreferencesAction>,
) -> (WidgetId, Rect, Rect, f32) {
    let screen_width = params.screen_width;
    let screen_height = params.screen_height;
    let cursor_pos = params.cursor_pos;
    let min_y = crate::ui::iris_bridge::menubar::MENUBAR_HEIGHT;

    let (left, top) = if let Some(pos) = params.window_pos {
        let max_x = (screen_width - PREF_CARD_WIDTH).max(0.0);
        let max_y = (screen_height - PREF_CARD_HEIGHT).max(min_y);
        (
            pos.x.clamp(0.0, max_x).round(),
            pos.y.clamp(min_y, max_y).round(),
        )
    } else {
        (
            ((screen_width - PREF_CARD_WIDTH) * 0.5).max(0.0).round(),
            ((screen_height - PREF_CARD_HEIGHT) * 0.5)
                .max(min_y)
                .round(),
        )
    };

    let card_rect = Rect::new(left, top, PREF_CARD_WIDTH, PREF_CARD_HEIGHT);
    let content_w = PREF_CARD_WIDTH - SIDEBAR_WIDTH - 1.0;
    let content_h = PREF_CARD_HEIGHT - TITLEBAR_HEIGHT;
    let content_rect = Rect::new(
        left + SIDEBAR_WIDTH + 1.0,
        top + TITLEBAR_HEIGHT,
        content_w,
        content_h,
    );

    let root_id = tree
        .root()
        .or_else(|| tree.create_root().ok())
        .unwrap_or_default();
    let mut root_scope = UiScope::with_tagged_interactions(tree, root_id, params.events, None)
        .with_active_text_input(params.active_number_input);

    // 1. Floating Preferences Dialog Card
    root_scope.container_tagged(
        "PreferencesDialogCard",
        Style::new()
            .position_absolute()
            .left(left)
            .top(top)
            .width(PREF_CARD_WIDTH)
            .height(PREF_CARD_HEIGHT)
            .flex_col()
            .background(root_scope.theme().modal_bg)
            .border(1.0, root_scope.theme().modal_border)
            .border_radius(10.0)
            .box_shadow(0.0, 8.0, 24.0, Color::rgba(0.0, 0.0, 0.0, 0.75))
            .clip_children(true),
        WidgetRole::FloatingWindow,
        PREF_TAG_CARD,
        |card| {
            // 2. Titlebar Strip - unified deep black matching sidebar with matching top rounded corners
            let titlebar_bg = card.theme().toolbar_bg;
            card.container_tagged_resp(
                "PreferencesTitlebar",
                Style::new()
                    .flex_row()
                    .align_items(AlignItems::Center)
                    .justify_content(JustifyContent::SpaceBetween)
                    .height(TITLEBAR_HEIGHT)
                    .padding_insets(Insets::new(0.0, 14.0, 0.0, 10.0))
                    .background(titlebar_bg)
                    .corner_radii(CornerRadii::new(10.0, 10.0, 0.0, 0.0)),
                WidgetRole::FloatingTitlebar,
                PREF_TAG_TITLEBAR,
                |titlebar| {
                    titlebar.icon(
                        crate::ui::iris_bridge::icons::ICON_GEAR,
                        Color::rgba(0.85, 0.88, 0.94, 1.0),
                        15.0,
                    );

                    titlebar.empty_box_passive_named(
                        "PrefTitleGap",
                        Style::new().width(8.0).height(1.0),
                    );

                    titlebar.label_styled_passive(
                        "PrefTitleText",
                        "Preferences",
                        13.0,
                        Color::rgba(1.0, 1.0, 1.0, 1.0),
                        TextAlign::Left,
                        Style::new().flex_grow(1.0),
                    );

                    // Close Button
                    if titlebar
                        .container_tagged_resp(
                            "PrefCloseButton",
                            Style::new()
                                .width(26.0)
                                .height(26.0)
                                .background(Color::TRANSPARENT)
                                .hover_background(Color::rgba(0.85, 0.20, 0.25, 0.85))
                                .border_radius(4.0)
                                .align_items(AlignItems::Center)
                                .justify_content(JustifyContent::Center),
                            WidgetRole::Button,
                            PREF_TAG_CLOSE,
                            |btn| {
                                btn.label_styled_hover_passive(
                                    "PrefCloseIcon",
                                    "✕",
                                    12.0,
                                    (Color::rgba(0.90, 0.92, 0.96, 1.0), Color::WHITE),
                                    TextAlign::Center,
                                    Style::new(),
                                );
                            },
                        )
                        .clicked()
                    {
                        actions.push(super::types::PreferencesAction::Close);
                    }
                },
            );

            // 1px Horizontal Titlebar Divider
            let card_divider = card.theme().divider;
            card.divider(card_divider);

            // 3. Body: Sidebar + Divider + Content View
            card.container_named(
                "PreferencesBody",
                Style::new().flex_row().flex_grow(1.0).height(content_h),
                |body| {
                    build_preferences_view(body, &mut params, actions, content_w, content_h);
                },
            );

            // Calculate and assign layout geometry to the entire preferences dialog hierarchy
            card.finish_layout(card_rect);
        },
    );

    let card_id = root_scope.node_id_tagged(PREF_TAG_CARD).unwrap_or_default();

    let current_scroll = root_scope.scroll_offset(PREF_TAG_CONTENT_VIEW);

    // Measure dynamic accumulated height of all tab content sections from layout pass
    let measured_total_h = root_scope
        .node_id_tagged(PREF_TAG_SCROLL_CONTAINER)
        .and_then(|id| root_scope.node_ref(id))
        .map(|node| {
            node.children
                .iter()
                .filter_map(|&cid| root_scope.node_ref(cid))
                .map(|child| child.computed_rect.bottom() + current_scroll - content_rect.y)
                .fold(0.0_f32, f32::max)
        })
        .unwrap_or(0.0);

    let total_h = measured_total_h
        .max(tab_virtual_height(*params.active_tab))
        .max(content_h);
    let max_scroll_y = (total_h - content_h).max(0.0);
    let safe_scroll = current_scroll.clamp(0.0, max_scroll_y);

    // Register measured content size and scroll boundaries on the viewport node
    root_scope.set_content_size_tagged(PREF_TAG_CONTENT_VIEW, content_w, total_h);
    root_scope.set_max_scroll(PREF_TAG_CONTENT_VIEW, max_scroll_y);
    root_scope.set_scroll_offset(PREF_TAG_CONTENT_VIEW, safe_scroll);

    // 4. Custom Scrollbar Indicator if virtual height exceeds content height
    if total_h > content_h {
        let style = preferences_scroll_style();
        if let Some(geom) =
            ScrollBarGeometry::compute_vertical(content_rect, total_h, safe_scroll, &style)
        {
            let is_scrollbar_dragging = root_scope.is_tag_captured(PREF_TAG_SCROLLBAR_THUMB);

            let mut card_scope = root_scope.child_scope(card_id);
            card_scope.scrollbar_vertical(
                geom,
                card_rect,
                is_scrollbar_dragging,
                Some(cursor_pos),
                PREF_TAG_SCROLLBAR_TRACK,
                PREF_TAG_SCROLLBAR_THUMB,
            );
        }
    }

    (card_id, card_rect, content_rect, max_scroll_y)
}

/// Renders the complete body of the preferences view (sidebar tabs, vertical divider, scrollable content area).
pub fn build_preferences_view(
    body: &mut UiScope<'_>,
    params: &mut PreferencesParams<'_>,
    actions: &mut Vec<super::types::PreferencesAction>,
    content_w: f32,
    content_h: f32,
) {
    if let Some(clicked) = body
        .sidebar_tabs_tagged(
            &SIDEBAR_TABS_TAGGED,
            params.active_tab,
            Some(PREF_TAG_CONTENT_VIEW),
        )
        .clicked_tab
    {
        actions.push(super::types::PreferencesAction::SelectTab(clicked));
    }

    let divider_color = body.theme().divider;
    body.vertical_divider(content_h, divider_color);

    let current_scroll = body.scroll_offset(PREF_TAG_CONTENT_VIEW);

    body.container_tagged(
        "PreferencesContentArea",
        Style::new()
            .width(content_w)
            .height(content_h)
            .clip_children(true)
            .scroll_offset_y(current_scroll)
            .padding_insets(Insets::new(14.0, 18.0, 14.0, 18.0)),
        WidgetRole::ScrollAreaViewport,
        PREF_TAG_CONTENT_VIEW,
        |content| {
            content.container_tagged(
                "PreferencesScrollContainer",
                Style::new().flex_col().gap(14.0),
                WidgetRole::Default,
                PREF_TAG_SCROLL_CONTAINER,
                |scroll_col| {
                    match *params.active_tab {
                        0 => build_general_tab(scroll_col, params, actions),
                        1 => build_graphics_tab(scroll_col, params),
                        2 => build_editor_tab(scroll_col, params),
                        3 => build_navigation_tab(scroll_col, params),
                        4 => build_keymap_tab(scroll_col, params),
                        5 => build_system_tab(scroll_col, params),
                        6 => build_addons_tab(scroll_col, params),
                        7 => build_input_tab(scroll_col, params),
                        8 => build_experimental_tab(scroll_col, params),
                        9 => build_modules_tab(scroll_col, params),
                        _ => {}
                    }
                    scroll_col.empty_box_passive_named(
                        "PrefScrollBottomSpacer",
                        Style::new().height(24.0),
                    );
                },
            );
        },
    );
}

impl IrisEditorOverlay {
    /// Builds the Preferences panel content when hosted as a standard dockable tab or floating window.
    ///
    /// Resolves panel rect from `scope.parent_rect()`, sets up active number input and caret blinking,
    /// renders sidebar navigation and active tab sections, and handles scrolling.
    pub(crate) fn build_preferences_panel_if_active(
        &mut self,
        scope: &mut UiScope<'_>,
        params: &OverlayUpdateParams<'_>,
    ) {
        let panel_rect = scope.parent_rect();
        if panel_rect.width <= 20.0 || panel_rect.height <= 20.0 {
            return;
        }

        let blink_caret = (self.start_time.elapsed().as_millis() % 1060) < 530;
        let mut caret_buf = String::new();
        let active_number_input =
            if let Some((t, ref s, is_all_selected)) = self.preferences.active_number_input {
                let text_with_caret = if !is_all_selected && blink_caret {
                    caret_buf.push_str(s);
                    caret_buf.push('|');
                    caret_buf.as_str()
                } else {
                    s.as_str()
                };
                Some((t, text_with_caret, is_all_selected))
            } else {
                None
            };

        let content_w = (panel_rect.width - SIDEBAR_WIDTH - 1.0).max(100.0);
        let content_h = panel_rect.height;

        let cursor_pos = self.cursor_pos();
        let current_theme = self.chrome.theme_preset;
        let mut gs = params.preferences.graphics_settings.clone();
        let mut snap = *params.preferences.snapping_settings;
        let mut cfg = params.preferences.editor_config.clone();

        let mut pref_params = PreferencesParams {
            screen_width: panel_rect.width,
            screen_height: panel_rect.height,
            window_pos: None,
            active_tab: &mut self.preferences.tab,
            collapsed_sections: &self.preferences.collapsed_sections,
            active_number_input,
            blink_caret,
            cursor_pos,
            zoom_factor: params.context.zoom_factor,
            graphics_settings: &mut gs,
            snapping_settings: &mut snap,
            editor_config: &mut cfg,
            enable_live_updates: params.context.enable_live_updates,
            enabled_modules: params.preferences.enabled_modules,
            events: &self.preferences.pending_interaction_events,
            current_theme,
        };

        let panel_bg = scope.theme().panel_bg;
        scope.container_named(
            "PreferencesPanelBody",
            Style::new()
                .flex_row()
                .width(panel_rect.width)
                .height(content_h)
                .background(panel_bg),
            |body| {
                build_preferences_view(
                    body,
                    &mut pref_params,
                    &mut self.preferences.interactions.actions,
                    content_w,
                    content_h,
                );
            },
        );
        self.preferences.pending_interaction_events.clear();

        if gs != *params.preferences.graphics_settings {
            self.pending_actions
                .push(crate::ui::types::EngineUiAction::UpdateGraphicsSettings(
                    gs.clone(),
                ));
            self.notifier
                .tag_redraw(crate::ui::panel_layout::PanelId::Preferences);
        }
        if snap != *params.preferences.snapping_settings {
            self.pending_actions
                .push(crate::ui::types::EngineUiAction::UpdateSnapSettings(snap));
            self.notifier
                .tag_redraw(crate::ui::panel_layout::PanelId::Preferences);
        }
        if cfg != *params.preferences.editor_config {
            self.pending_actions
                .push(crate::ui::types::EngineUiAction::UpdateEditorConfig(
                    cfg.clone(),
                ));
            self.notifier
                .tag_redraw(crate::ui::panel_layout::PanelId::Preferences);
        }

        let current_scroll = scope.scroll_offset(PREF_TAG_CONTENT_VIEW);
        let measured_total_h = scope
            .node_id_tagged(PREF_TAG_SCROLL_CONTAINER)
            .and_then(|id| scope.node_ref(id))
            .map(|node| {
                node.children
                    .iter()
                    .filter_map(|&cid| scope.node_ref(cid))
                    .map(|child| child.computed_rect.bottom() + current_scroll - panel_rect.y)
                    .fold(0.0_f32, f32::max)
            })
            .unwrap_or(0.0);

        let total_h = measured_total_h
            .max(tab_virtual_height(self.preferences.tab))
            .max(content_h);
        let max_scroll_y = (total_h - content_h).max(0.0);
        let safe_scroll = current_scroll.clamp(0.0, max_scroll_y);

        scope.set_content_size_tagged(PREF_TAG_CONTENT_VIEW, content_w, total_h);
        scope.set_max_scroll(PREF_TAG_CONTENT_VIEW, max_scroll_y);
        scope.set_scroll_offset(PREF_TAG_CONTENT_VIEW, safe_scroll);

        if total_h > content_h {
            let style = preferences_scroll_style();
            let content_rect = Rect::new(
                panel_rect.x + SIDEBAR_WIDTH + 1.0,
                panel_rect.y,
                content_w,
                content_h,
            );
            if let Some(geom) =
                ScrollBarGeometry::compute_vertical(content_rect, total_h, safe_scroll, &style)
            {
                let is_scrollbar_dragging = scope.is_tag_captured(PREF_TAG_SCROLLBAR_THUMB);

                scope.scrollbar_vertical(
                    geom,
                    panel_rect,
                    is_scrollbar_dragging,
                    Some(self.cursor_pos()),
                    PREF_TAG_SCROLLBAR_TRACK,
                    PREF_TAG_SCROLLBAR_THUMB,
                );
            }
        }
    }
}