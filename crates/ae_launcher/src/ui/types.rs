// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

use std::path::PathBuf;

/// Primary active navigation tab in the launcher sidebar.
///
/// Controls whether the launcher presents the list of recently opened projects
/// or the project creation wizard with template settings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LauncherTab {
    /// List of recently opened projects.
    #[default]
    RecentProjects,

    /// Project creation wizard with dimension mode template selection.
    NewProject,
}

/// User interaction outcome produced during a launcher frame.
///
/// Encapsulates discrete actions dispatched by buttons, cards, or inputs
/// for consumption by the launcher application event loop.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LauncherAction {
    /// No state-changing action triggered this frame.
    None,

    /// Switches the active launcher tab.
    SwitchTab(LauncherTab),

    /// Requests native filesystem browser dialog to pick parent directory.
    BrowseDirectory,

    /// Launches an existing project and exits the launcher process.
    OpenProject(PathBuf, String),

    /// Creates a new project from template, launches it, and exits the launcher.
    CreateAndLaunch {
        /// Display name of the new project directory.
        name: String,
        /// Parent directory path where project will be generated.
        parent_dir: PathBuf,
        /// Active dimension mode (`"3D"`).
        mode: String,
    },

    /// Removes a project path from the recent list.
    RemoveRecent(String),

    /// Sets or clears keyboard focus for the project name input text box.
    FocusNameInput(bool),

    /// Selects active dimension mode and optionally provides transient status feedback.
    SelectDimensionMode {
        /// Selected dimension mode (`"2D"` or `"3D"`).
        mode: String,
        /// Optional transient status message.
        status_message: Option<String>,
    },
}

/// Mutable interactive state of the Launcher UI.
///
/// Maintains user input buffer, active tab selection, focus state,
/// text caret blink phase, and transient status messages.
pub struct LauncherUiState {
    /// Active sidebar navigation tab.
    pub active_tab: LauncherTab,

    /// Name text entered in the new project wizard.
    pub new_project_name: String,

    /// Directory path entered in the new project wizard.
    pub new_project_dir: String,

    /// Selected dimension mode (`"3D"`).
    pub selected_mode: String,

    /// Current search filter query for recent projects list.
    pub search_query: String,

    /// Whether the project name input box currently has active focus.
    pub is_name_focused: bool,

    /// Blink phase state for drawing the vertical text cursor.
    pub cursor_blink_visible: bool,

    /// Status message or error notice to display in the footer.
    pub status_message: Option<String>,
}

impl Default for LauncherUiState {
    fn default() -> Self {
        let default_dir = std::env::var("HOME")
            .map(|h| format!("{}/projects", h))
            .unwrap_or_else(|_| ".".to_string());

        Self {
            active_tab: LauncherTab::RecentProjects,
            new_project_name: "MyNewGame".to_string(),
            new_project_dir: default_dir,
            selected_mode: "3D".to_string(),
            search_query: String::new(),
            is_name_focused: false,
            cursor_blink_visible: false,
            status_message: None,
        }
    }
}