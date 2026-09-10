// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

use std::fs;
use std::path::PathBuf;

use crate::project::{ProjectConfig, ProjectRegistry};
use crate::ui::{LauncherTab, LauncherUiState};

#[test]
fn test_project_config_serialization_roundtrip() {
    let config = ProjectConfig {
        name: "Test2DGame".to_string(),
        path: "/tmp/test_project".to_string(),
        dimension_mode: "2D".to_string(),
        engine_version: "0.9.0".to_string(),
        entry_scene: "scenes/main.aee".to_string(),
        last_opened: 1726000000,
    };

    let json = serde_json::to_string(&config).expect("Serialize failed");
    let deserialized: ProjectConfig = serde_json::from_str(&json).expect("Deserialize failed");

    assert_eq!(config, deserialized);
}

#[test]
fn test_project_registry_add_and_touch() {
    let mut registry = ProjectRegistry::default();
    assert!(registry.recent_projects.is_empty());

    let p1 = ProjectConfig {
        name: "ProjectAlpha".to_string(),
        path: "/tmp/alpha".to_string(),
        dimension_mode: "2D".to_string(),
        ..Default::default()
    };

    let p2 = ProjectConfig {
        name: "ProjectBeta".to_string(),
        path: "/tmp/beta".to_string(),
        dimension_mode: "3D".to_string(),
        ..Default::default()
    };

    registry.add_or_touch(p1.clone());
    registry.add_or_touch(p2.clone());

    assert_eq!(registry.recent_projects.len(), 2);
    assert_eq!(registry.recent_projects[0].path, "/tmp/beta");

    // Touching p1 moves it to the top
    registry.add_or_touch(p1);
    assert_eq!(registry.recent_projects[0].path, "/tmp/alpha");
    assert_eq!(registry.recent_projects.len(), 2);

    registry.remove("/tmp/alpha");
    assert_eq!(registry.recent_projects.len(), 1);
    assert_eq!(registry.recent_projects[0].path, "/tmp/beta");
}

#[test]
fn test_create_project_on_disk() {
    let temp_dir = std::env::temp_dir().join(format!("ae_test_proj_{}", std::process::id()));
    let _ = fs::remove_dir_all(&temp_dir);
    fs::create_dir_all(&temp_dir).unwrap();

    let config = ProjectRegistry::create_project("Sandbox2D", &temp_dir, "2D")
        .expect("Project creation failed");

    assert_eq!(config.name, "Sandbox2D");
    assert_eq!(config.dimension_mode, "2D");

    let proj_path = PathBuf::from(&config.path);
    assert!(proj_path.join("assets").is_dir());
    assert!(proj_path.join("scenes").is_dir());
    assert!(proj_path.join("scenes/main.aee").is_file());
    assert!(proj_path.join("aeon_project.json").is_file());

    // Clean up
    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_launcher_ui_state_defaults() {
    let state = LauncherUiState::default();
    assert_eq!(state.active_tab, LauncherTab::RecentProjects);
    assert_eq!(state.selected_mode, "3D");
    assert_eq!(state.new_project_name, "MyNewGame");
    assert!(!state.is_name_focused);
    assert!(!state.cursor_blink_visible);
    assert!(state.status_message.is_none());
}

#[test]
fn test_launcher_focus_and_cursor_blink() {
    let mut state = LauncherUiState::default();
    assert!(!state.is_name_focused);

    // Focusing enables typing and blink
    state.is_name_focused = true;
    state.cursor_blink_visible = true;
    assert!(state.is_name_focused && state.cursor_blink_visible);

    // Unfocusing disables both
    state.is_name_focused = false;
    state.cursor_blink_visible = false;
    assert!(!state.is_name_focused);
}

#[test]
fn test_launcher_icon_loading() {
    const AEON_ICON_BYTES: &[u8] = include_bytes!("../../ae_engine/assets/icon/aeicon.png");
    let icon = crate::icon::load_icon_from_memory(AEON_ICON_BYTES);
    assert!(icon.is_some(), "Aeon Engine icon must decode successfully");
}