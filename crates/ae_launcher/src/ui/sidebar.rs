// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Launcher Declarative Sidebar Navigation
//!
//! Renders the left navigation sidebar rail using pure declarative [`UiScope`] primitives,
//! assembling the product branding header, active tab navigation buttons, and version footer.

use irisui::prelude::*;

use crate::icons::{ICON_FOLDER, ICON_PLUS, ICON_WIREFRAME, ICON_WORLD};
use crate::ui::builder::LAUNCHER_SIDEBAR_WIDTH;
use crate::ui::types::{LauncherAction, LauncherTab, LauncherUiState};

/// Semantic tag assigned to the "Recent Projects" navigation button.
pub const TAG_NAV_RECENT_PROJECTS: u64 = 0xA101;

/// Semantic tag assigned to the "New Project" navigation button.
pub const TAG_NAV_NEW_PROJECT: u64 = 0xA102;

/// Builds the left sidebar navigation hierarchy into the declarative [`UiScope`].
///
/// Features branding logo typography, tab selector buttons with active/hover styling,
/// and engine versioning metadata, returning any tab-switch action triggered in the frame.
pub fn build_sidebar(scope: &mut UiScope<'_>, state: &LauncherUiState) -> Option<LauncherAction> {
    scope.container_named(
        "LauncherSidebar",
        Style::new()
            .flex_col()
            .justify_content(JustifyContent::SpaceBetween)
            .width(LAUNCHER_SIDEBAR_WIDTH)
            .flex_grow(1.0)
            .padding_insets(Insets::new(18.0, 12.0, 16.0, 12.0))
            .background(Color::hex("#12141a"))
            .border(1.0, Color::hex("#1e222d")),
        |sidebar| {
            sidebar.container_named(
                "SidebarTopSection",
                Style::new().flex_col().gap(18.0),
                |top| {
                    render_sidebar_header(top);
                    render_sidebar_navigation(top, state);
                },
            );
            render_sidebar_footer(sidebar);
        },
    );

    eval_sidebar_action(scope)
}

/// Renders the sidebar header badge containing the world icon and product title.
fn render_sidebar_header(scope: &mut UiScope<'_>) {
    scope.container_named(
        "SidebarHeader",
        Style::new()
            .flex_row()
            .align_items(AlignItems::Center)
            .gap(10.0)
            .padding_insets(Insets::new(0.0, 4.0, 6.0, 4.0)),
        |header| {
            header.icon_named("SidebarLogoIcon", ICON_WORLD, Color::hex("#00e5ff"), 20.0);
            header.label_styled_passive(
                "SidebarTitle",
                "AEON HUB",
                14.0,
                Color::hex("#ffffff"),
                TextAlign::Left,
                Style::new().width(150.0).height(20.0),
            );
        },
    );
}

/// Renders navigation rail items using declarative [`NavItemConfig`] builders.
fn render_sidebar_navigation(scope: &mut UiScope<'_>, state: &LauncherUiState) {
    scope.container_named("SidebarNavGroup", Style::new().flex_col().gap(6.0), |nav| {
        nav.nav_item(
            "NavRecentProjects",
            NavItemConfig::new("Recent Projects")
                .atlas_icon(ICON_FOLDER)
                .active(state.active_tab == LauncherTab::RecentProjects)
                .active_color(Color::hex("#ffd166"))
                .tag(TAG_NAV_RECENT_PROJECTS),
        );
        nav.nav_item(
            "NavNewProject",
            NavItemConfig::new("New Project")
                .atlas_icon(ICON_PLUS)
                .active(state.active_tab == LauncherTab::NewProject)
                .active_color(Color::hex("#00e5ff"))
                .tag(TAG_NAV_NEW_PROJECT),
        );
    });
}

/// Evaluates navigation button click events and maps them to semantic tab switch actions.
pub fn eval_sidebar_action(scope: &mut UiScope<'_>) -> Option<LauncherAction> {
    scope
        .map_tag_clicked(TAG_NAV_RECENT_PROJECTS, || {
            LauncherAction::SwitchTab(LauncherTab::RecentProjects)
        })
        .or_else(|| {
            scope.map_tag_clicked(TAG_NAV_NEW_PROJECT, || {
                LauncherAction::SwitchTab(LauncherTab::NewProject)
            })
        })
}

/// Renders the engine version and wireframe icon footer at the bottom of the rail.
fn render_sidebar_footer(scope: &mut UiScope<'_>) {
    scope.container_named(
        "SidebarFooter",
        Style::new()
            .flex_row()
            .align_items(AlignItems::Center)
            .gap(8.0)
            .padding_insets(Insets::new(0.0, 4.0, 0.0, 4.0)),
        |footer| {
            footer.icon_named(
                "SidebarVersionIcon",
                ICON_WIREFRAME,
                Color::hex("#525866"),
                14.0,
            );
            footer.label_styled_passive(
                "SidebarVersionText",
                "Aeon Core v0.9.0",
                10.0,
                Color::hex("#525866"),
                TextAlign::Left,
                Style::new().width(150.0).height(16.0),
            );
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_sidebar_creates_navigation_hierarchy() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("root node");
        let state = LauncherUiState::default();

        let action = build_sidebar(&mut UiScope::new(&mut tree, root), &state);

        assert_eq!(action, None);

        let has_sidebar = tree
            .iter()
            .any(|(_, n)| n.name.as_deref() == Some("LauncherSidebar"));
        let has_header = tree
            .iter()
            .any(|(_, n)| n.name.as_deref() == Some("SidebarHeader"));
        let has_title = tree
            .iter()
            .any(|(_, n)| n.text.as_deref() == Some("AEON HUB"));
        let has_recent_btn = tree
            .iter()
            .any(|(_, n)| n.name.as_deref() == Some("NavRecentProjects"));
        let has_new_btn = tree
            .iter()
            .any(|(_, n)| n.name.as_deref() == Some("NavNewProject"));
        let has_footer = tree
            .iter()
            .any(|(_, n)| n.name.as_deref() == Some("SidebarFooter"));

        assert!(has_sidebar, "LauncherSidebar container must exist");
        assert!(has_header, "SidebarHeader must exist");
        assert!(has_title, "AEON HUB title must exist");
        assert!(has_recent_btn, "NavRecentProjects button must exist");
        assert!(has_new_btn, "NavNewProject button must exist");
        assert!(has_footer, "SidebarFooter must exist");
    }

    #[test]
    fn test_build_sidebar_click_dispatches_switch_tab_action() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("root node");
        let state = LauncherUiState::default();

        tree.event_state.frame_clicked_tag = Some(TAG_NAV_NEW_PROJECT);

        let action = build_sidebar(&mut UiScope::new(&mut tree, root), &state);

        assert_eq!(
            action,
            Some(LauncherAction::SwitchTab(LauncherTab::NewProject))
        );
    }
}