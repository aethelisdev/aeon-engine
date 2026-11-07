// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Launcher New Project Declarative View
//!
//! Renders the project creation wizard using pure declarative [`UiScope`] primitives,
//! providing project name input, filesystem locator, dimension mode cards, and launch actions.
//! Employs framework-level [`LabeledInputConfig`], [`SelectionCardConfig`], and [`PrimaryButtonConfig`]
//! to ensure zero-`let` ergonomics, theme conformance, and flat declarative closures.

use std::path::PathBuf;

use irisui::prelude::*;

use crate::icons::{ICON_CUBE, ICON_FOLDER, ICON_PLUS};
use crate::ui::types::{LauncherAction, LauncherUiState};

/// Semantic tag assigned to the project name input text box.
pub const TAG_NAME_INPUT: u64 = 0xD001;

/// Semantic tag assigned to the project location input and folder browse button.
pub const TAG_BROWSE_DIR: u64 = 0xD002;

/// Semantic tag assigned to the 2D dimension mode card (Closed Beta locked).
pub const TAG_MODE_2D: u64 = 0xD003;

/// Semantic tag assigned to the 3D dimension mode card.
pub const TAG_MODE_3D: u64 = 0xD004;

/// Semantic tag assigned to the "Create & Launch Project" confirmation button.
pub const TAG_CREATE_PROJECT: u64 = 0xD005;

/// Builds the New Project wizard tab view into the declarative [`UiScope`].
pub fn build_new_project_view(
    scope: &mut UiScope<'_>,
    state: &mut LauncherUiState,
) -> Option<LauncherAction> {
    scope.container_named(
        "NewProjectContent",
        Style::new()
            .flex_col()
            .padding_insets(Insets::new(24.0, 32.0, 24.0, 32.0))
            .gap(18.0),
        |view| {
            render_new_project_header(view);
            render_name_input_section(view, state);
            render_location_input_section(view, state);
            render_dimension_mode_section(view, state);
            render_create_button_section(view, state);
        },
    );

    eval_new_project_action(scope, state)
}

/// Renders the wizard header title and description typography.
fn render_new_project_header(scope: &mut UiScope<'_>) {
    scope.container_named(
        "NewProjectHeaderGroup",
        Style::new().flex_col().gap(6.0),
        |header| {
            header.label_styled_passive(
                "NewProjectTitle",
                "Create New Project",
                20.0,
                header.theme().text_primary,
                TextAlign::Left,
                Style::new().width(300.0).height(28.0),
            );
            header.label_styled_passive(
                "NewProjectSubtitle",
                "Configure your project name, location, and engine dimension mode.",
                11.5,
                header.theme().text_secondary,
                TextAlign::Left,
                Style::new().width(500.0).height(20.0),
            );
        },
    );
}

/// Renders the project name input row using Iris's native editable text input widget.
fn render_name_input_section(scope: &mut UiScope<'_>, state: &mut LauncherUiState) {
    scope.labeled_text_input(
        "ProjectNameField",
        "PROJECT NAME",
        &mut state.new_project_name,
        TAG_NAME_INPUT,
        420.0,
    );
}

/// Renders the project location input and folder browser picker button.
fn render_location_input_section(scope: &mut UiScope<'_>, state: &LauncherUiState) {
    scope.labeled_input(
        "ProjectLocationField",
        LabeledInputConfig::new("PROJECT LOCATION", &state.new_project_dir)
            .tag(TAG_BROWSE_DIR)
            .width(420.0)
            .trailing_button(
                LabeledInputButton::new(ICON_FOLDER, TAG_BROWSE_DIR)
                    .icon_tint(scope.theme().warning),
            ),
    );
}

/// Renders the dimension mode template selection group hosting 2D Beta and 3D cards.
fn render_dimension_mode_section(scope: &mut UiScope<'_>, state: &LauncherUiState) {
    scope.container_named(
        "DimensionModeSection",
        Style::new().flex_col().gap(6.0),
        |col| {
            col.label_styled_passive(
                "ModeLabel",
                "DIMENSION MODE",
                10.0,
                col.theme().text_muted,
                TextAlign::Left,
                Style::new().width(200.0).height(16.0),
            );
            col.container_named("ModeCardRow", Style::new().flex_row().gap(16.0), |row| {
                render_mode_2d_card(row);
                render_mode_3d_card(row, state.selected_mode == "3D");
            });
        },
    );
}

/// Renders the 2D Sprite Engine mode card with locked Closed Beta status badge.
fn render_mode_2d_card(scope: &mut UiScope<'_>) {
    scope.selection_card(
        "ModeCard2D",
        SelectionCardConfig::new(
            "2D Sprite Engine",
            "Under active development.\nAvailable in future releases.",
        )
        .atlas_icon(ICON_FOLDER)
        .icon_tint(scope.theme().text_secondary)
        .badge("BETA")
        .tag(TAG_MODE_2D)
        .width(204.0)
        .height(84.0),
    );
}

/// Renders the 3D Spatial Engine mode card with active selection styling.
fn render_mode_3d_card(scope: &mut UiScope<'_>, is_selected: bool) {
    scope.selection_card(
        "ModeCard3D",
        SelectionCardConfig::new(
            "3D Spatial Engine",
            "Full PBR forward passes,\nCSM shadows & skeletal mesh.",
        )
        .atlas_icon(ICON_CUBE)
        .icon_tint(scope.theme().accent)
        .selected(is_selected)
        .tag(TAG_MODE_3D)
        .width(204.0)
        .height(84.0),
    );
}

/// Renders the primary "Create & Launch Project" button and status feedback label.
fn render_create_button_section(scope: &mut UiScope<'_>, state: &LauncherUiState) {
    scope.primary_button(
        "CreateProjectButton",
        PrimaryButtonConfig::new("Create & Launch Project")
            .atlas_icon(ICON_PLUS)
            .tag(TAG_CREATE_PROJECT)
            .width(420.0)
            .height(42.0)
            .status_message(state.status_message.as_deref()),
    );
}

/// Evaluates button clicks across the project creation wizard form.
pub fn eval_new_project_action(
    scope: &mut UiScope<'_>,
    state: &LauncherUiState,
) -> Option<LauncherAction> {
    scope
        .map_tag_clicked(TAG_CREATE_PROJECT, || LauncherAction::CreateAndLaunch {
            name: state.new_project_name.clone(),
            parent_dir: PathBuf::from(&state.new_project_dir),
            mode: state.selected_mode.clone(),
        })
        .or_else(|| scope.map_tag_clicked(TAG_BROWSE_DIR, || LauncherAction::BrowseDirectory))
        .or_else(|| scope.map_tag_clicked(TAG_NAME_INPUT, || LauncherAction::FocusNameInput(true)))
        .or_else(|| {
            scope.map_tag_clicked(TAG_MODE_2D, || LauncherAction::SelectDimensionMode {
                mode: "3D".to_string(),
                status_message: Some(
                    "2D Sprite Engine is currently in Closed Beta. Only 3D Spatial Engine is available in this release."
                        .to_string(),
                ),
            })
        })
        .or_else(|| {
            scope.map_tag_clicked(TAG_MODE_3D, || LauncherAction::SelectDimensionMode {
                mode: "3D".to_string(),
                status_message: None,
            })
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_new_project_view_creates_form_hierarchy() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("root node");
        let mut state = LauncherUiState::default();

        let action = build_new_project_view(&mut UiScope::new(&mut tree, root), &mut state);

        assert_eq!(action, None);

        let has_content = tree
            .iter()
            .any(|(_, n)| n.name.as_deref() == Some("NewProjectContent"));
        let has_name_box = tree
            .iter()
            .any(|(_, n)| n.name.as_deref() == Some("ProjectNameField"));
        let has_location_box = tree
            .iter()
            .any(|(_, n)| n.name.as_deref() == Some("ProjectLocationField"));
        let has_mode_2d = tree
            .iter()
            .any(|(_, n)| n.name.as_deref() == Some("ModeCard2D"));
        let has_mode_3d = tree
            .iter()
            .any(|(_, n)| n.name.as_deref() == Some("ModeCard3D"));
        let has_create_btn = tree
            .iter()
            .any(|(_, n)| n.name.as_deref() == Some("CreateProjectButton"));

        assert!(has_content, "NewProjectContent container must exist");
        assert!(has_name_box, "ProjectNameField must exist");
        assert!(has_location_box, "ProjectLocationField must exist");
        assert!(has_mode_2d, "ModeCard2D must exist");
        assert!(has_mode_3d, "ModeCard3D must exist");
        assert!(has_create_btn, "CreateProjectButton must exist");
    }

    #[test]
    fn test_eval_new_project_action_click_dispatches_create_action() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("root node");
        let mut state = LauncherUiState {
            new_project_name: "MyNewGame".to_string(),
            new_project_dir: "/workspace".to_string(),
            selected_mode: "3D".to_string(),
            ..Default::default()
        };

        tree.event_state.frame_clicked_tag = Some(TAG_CREATE_PROJECT);

        let action = build_new_project_view(&mut UiScope::new(&mut tree, root), &mut state);

        assert_eq!(
            action,
            Some(LauncherAction::CreateAndLaunch {
                name: "MyNewGame".to_string(),
                parent_dir: PathBuf::from("/workspace"),
                mode: "3D".to_string(),
            })
        );
    }
}