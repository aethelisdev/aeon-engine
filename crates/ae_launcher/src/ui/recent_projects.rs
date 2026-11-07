// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Launcher Recent Projects Declarative View
//!
//! Renders the list of recently opened projects using pure declarative [`UiScope`] primitives,
//! offering dimension indicators, project directory paths, and launch interaction triggers.

use std::path::PathBuf;

use irisui::prelude::*;

use crate::icons::{ICON_CUBE, ICON_FOLDER, ICON_SELECT};
use crate::project::{ProjectConfig, ProjectRegistry};
use crate::ui::types::LauncherAction;

/// Semantic tag base offset for project card click triggers.
pub const TAG_PROJECT_CARD_BASE: u64 = 0xB000;

/// Semantic tag base offset for explicit project open button click triggers.
pub const TAG_PROJECT_OPEN_BASE: u64 = 0xC000;

/// Builds the Recent Projects tab content view into the declarative [`UiScope`].
pub fn build_recent_projects_view(
    scope: &mut UiScope<'_>,
    registry: &ProjectRegistry,
) -> Option<LauncherAction> {
    scope.container_named(
        "RecentProjectsContent",
        Style::new()
            .flex_col()
            .padding_insets(Insets::new(24.0, 32.0, 24.0, 32.0))
            .gap(16.0),
        |view| {
            render_projects_header(view);
            render_projects_list(view, registry);
        },
    );

    eval_projects_action(scope, registry)
}

/// Renders section heading and description typography.
fn render_projects_header(scope: &mut UiScope<'_>) {
    scope.container_named(
        "ProjectsHeaderGroup",
        Style::new().flex_col().gap(6.0),
        |header| {
            header.label_styled_passive(
                "ProjectsTitle",
                "Projects",
                20.0,
                header.theme().text_primary,
                TextAlign::Left,
                Style::new().width(200.0).height(28.0),
            );
            header.label_styled_passive(
                "ProjectsSubtitle",
                "Select a recent project to launch the editor.",
                11.5,
                header.theme().text_secondary,
                TextAlign::Left,
                Style::new().width(400.0).height(20.0),
            );
        },
    );
}

/// Renders the list of recent project cards or an empty state placeholder.
fn render_projects_list(scope: &mut UiScope<'_>, registry: &ProjectRegistry) {
    if registry.recent_projects.is_empty() {
        render_empty_state(scope);
        return;
    }

    scope.container_named(
        "ProjectsCardList",
        Style::new().flex_col().gap(10.0),
        |list| {
            for (idx, project) in registry.recent_projects.iter().take(10).enumerate() {
                render_project_card(list, idx, project);
            }
        },
    );
}

/// Renders the placeholder label when no recent projects are registered.
fn render_empty_state(scope: &mut UiScope<'_>) {
    scope.label_styled_passive(
        "EmptyProjectsLabel",
        "No recent projects found. Click 'New Project' to create one.",
        12.0,
        scope.theme().text_muted,
        TextAlign::Left,
        Style::new().width(500.0).height(24.0),
    );
}

/// Renders an individual project card using the declarative [`ActionCardConfig`] primitive.
fn render_project_card(scope: &mut UiScope<'_>, idx: usize, project: &ProjectConfig) {
    scope.action_card(
        "ProjectCard",
        ActionCardConfig::new(&project.name)
            .subtitle(format_project_detail(project))
            .atlas_icon(project_mode_icon(project))
            .icon_tint(project_mode_tint(project, &scope.theme()))
            .tag(TAG_PROJECT_CARD_BASE + idx as u64)
            .action_button("Open", ICON_SELECT, TAG_PROJECT_OPEN_BASE + idx as u64),
    );
}

/// Resolves the GPU SDF atlas icon coordinates matching the project dimension mode.
#[inline]
fn project_mode_icon(project: &ProjectConfig) -> [f32; 4] {
    if project.dimension_mode.eq_ignore_ascii_case("2D") {
        ICON_FOLDER
    } else {
        ICON_CUBE
    }
}

/// Resolves the visual indicator tint color matching the project dimension mode.
#[inline]
fn project_mode_tint(project: &ProjectConfig, theme: &Theme) -> Color {
    if project.dimension_mode.eq_ignore_ascii_case("2D") {
        Color::hex("#2ecc71")
    } else {
        theme.accent
    }
}

/// Evaluates card and open button clicks across the recent projects list.
pub fn eval_projects_action(
    scope: &mut UiScope<'_>,
    registry: &ProjectRegistry,
) -> Option<LauncherAction> {
    registry
        .recent_projects
        .iter()
        .take(10)
        .enumerate()
        .find_map(|(idx, project)| {
            (scope.tag_clicked(TAG_PROJECT_CARD_BASE + idx as u64)
                || scope.tag_clicked(TAG_PROJECT_OPEN_BASE + idx as u64))
            .then(|| {
                LauncherAction::OpenProject(
                    PathBuf::from(&project.path),
                    project.dimension_mode.clone(),
                )
            })
        })
}

/// Formats the project detail description string with dimension tag and absolute path.
fn format_project_detail(project: &ProjectConfig) -> String {
    if project.dimension_mode.eq_ignore_ascii_case("2D") {
        format!("[2D Sprite] • {}", project.path)
    } else {
        format!("[3D Spatial] • {}", project.path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_recent_projects_view_renders_cards() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("root node");
        let mut registry = ProjectRegistry::default();
        registry.recent_projects.push(ProjectConfig {
            name: "AlphaProject".to_string(),
            path: "/tmp/alpha".to_string(),
            dimension_mode: "3D".to_string(),
            last_opened: 100,
            ..Default::default()
        });

        let action = build_recent_projects_view(&mut UiScope::new(&mut tree, root), &registry);

        assert_eq!(action, None);

        let has_content = tree
            .iter()
            .any(|(_, n)| n.name.as_deref() == Some("RecentProjectsContent"));
        let has_card = tree
            .iter()
            .any(|(_, n)| n.name.as_deref() == Some("ProjectCard"));
        let has_name = tree
            .iter()
            .any(|(_, n)| n.text.as_deref() == Some("AlphaProject"));

        assert!(has_content, "RecentProjectsContent container must exist");
        assert!(has_card, "ProjectCard container must exist");
        assert!(has_name, "AlphaProject name label must exist");
    }

    #[test]
    fn test_build_recent_projects_view_empty_state() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("root node");
        let registry = ProjectRegistry::default();

        let action = build_recent_projects_view(&mut UiScope::new(&mut tree, root), &registry);

        assert_eq!(action, None);

        let has_empty = tree
            .iter()
            .any(|(_, n)| n.name.as_deref() == Some("EmptyProjectsLabel"));
        assert!(has_empty, "EmptyProjectsLabel must exist when empty");
    }

    #[test]
    fn test_eval_projects_action_click_dispatches_open_project() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("root node");
        let mut registry = ProjectRegistry::default();
        registry.recent_projects.push(ProjectConfig {
            name: "BetaProject".to_string(),
            path: "/tmp/beta".to_string(),
            dimension_mode: "3D".to_string(),
            last_opened: 200,
            ..Default::default()
        });

        tree.event_state.frame_clicked_tag = Some(TAG_PROJECT_OPEN_BASE);

        let action = build_recent_projects_view(&mut UiScope::new(&mut tree, root), &registry);

        assert_eq!(
            action,
            Some(LauncherAction::OpenProject(
                PathBuf::from("/tmp/beta"),
                "3D".to_string(),
            ))
        );
    }
}