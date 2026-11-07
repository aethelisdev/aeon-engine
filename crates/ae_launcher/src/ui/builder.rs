// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Launcher Declarative UI Builder
//!
//! Assembles the top-level desktop launcher layout using pure declarative [`UiScope`]
//! primitives, coordinating the navigation sidebar and active workspace views.

use glam::Vec2;
use irisui::prelude::*;

use crate::project::ProjectRegistry;
use crate::ui::new_project::build_new_project_view;
use crate::ui::recent_projects::build_recent_projects_view;
use crate::ui::sidebar::build_sidebar;
use crate::ui::types::{LauncherAction, LauncherTab, LauncherUiState};

/// Fixed width of the left navigation sidebar in logical pixels.
pub const LAUNCHER_SIDEBAR_WIDTH: f32 = 220.0;

/// Builds the complete launcher user interface layout into the provided [`UiTree`].
///
/// Constructs a full-window declarative horizontal flex layout hosting the navigation sidebar
/// and the active tab content area ([`build_recent_projects_view`] or [`build_new_project_view`]).
pub fn build_launcher_ui(
    tree: &mut UiTree,
    state: &mut LauncherUiState,
    registry: &ProjectRegistry,
    screen_size: Vec2,
    _cursor_pos: Point,
) -> LauncherAction {
    UiScope::for_root(tree).map_or(LauncherAction::None, |mut scope| {
        build_launcher_workspace(&mut scope, state, registry, screen_size)
    })
}

/// Emits the root horizontal split workspace container.
fn build_launcher_workspace(
    scope: &mut UiScope<'_>,
    state: &mut LauncherUiState,
    registry: &ProjectRegistry,
    screen_size: Vec2,
) -> LauncherAction {
    scope
        .container_named(
            "LauncherWorkspace",
            Style::new()
                .flex_row()
                .width(screen_size.x)
                .height(screen_size.y)
                .background(Color::hex("#0e1016")),
            |root_scope| {
                render_sidebar_section(root_scope, state, screen_size.y);
                render_content_section(root_scope, state, registry, screen_size)
                    .or_else(|| crate::ui::sidebar::eval_sidebar_action(root_scope))
            },
        )
        .inner
        .unwrap_or(LauncherAction::None)
}

/// Renders the left sidebar container section into the workspace.
fn render_sidebar_section(scope: &mut UiScope<'_>, state: &LauncherUiState, height: f32) {
    scope.container_named(
        "LauncherSidebarContainer",
        Style::new()
            .width(LAUNCHER_SIDEBAR_WIDTH)
            .height(height)
            .flex_grow(0.0)
            .flex_shrink(0.0),
        |sidebar_scope| build_sidebar(sidebar_scope, state),
    );
}

/// Renders the right content workspace section according to the active tab.
fn render_content_section(
    scope: &mut UiScope<'_>,
    state: &mut LauncherUiState,
    registry: &ProjectRegistry,
    screen_size: Vec2,
) -> Option<LauncherAction> {
    scope
        .container_named(
            "LauncherContentArea",
            Style::new()
                .width(screen_size.x - LAUNCHER_SIDEBAR_WIDTH)
                .height(screen_size.y),
            |content_scope| match state.active_tab {
                LauncherTab::RecentProjects => build_recent_projects_view(content_scope, registry),
                LauncherTab::NewProject => build_new_project_view(content_scope, state),
            },
        )
        .inner
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_launcher_ui_creates_declarative_workspace_and_containers() {
        let mut tree = UiTree::new();
        let mut state = LauncherUiState::default();
        let registry = ProjectRegistry::default();
        let screen_size = Vec2::new(1024.0, 768.0);
        let cursor_pos = Point::new(0.0, 0.0);

        let action = build_launcher_ui(&mut tree, &mut state, &registry, screen_size, cursor_pos);

        assert_eq!(action, LauncherAction::None);

        let has_workspace = tree
            .iter()
            .any(|(_, n)| n.name.as_deref() == Some("LauncherWorkspace"));
        let has_sidebar = tree
            .iter()
            .any(|(_, n)| n.name.as_deref() == Some("LauncherSidebarContainer"));
        let has_content = tree
            .iter()
            .any(|(_, n)| n.name.as_deref() == Some("LauncherContentArea"));

        assert!(
            has_workspace,
            "LauncherWorkspace root container must be present"
        );
        assert!(has_sidebar, "LauncherSidebarContainer must be present");
        assert!(has_content, "LauncherContentArea must be present");
    }
}