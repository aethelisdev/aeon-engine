// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

/// Project configuration file stored in the root of every Aeon Engine project as `aeon_project.json`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProjectConfig {
    /// Human-readable project display name.
    pub name: String,

    /// Absolute filesystem path to the project root directory.
    pub path: String,

    /// Active dimension mode: `"2D"` or `"3D"`.
    pub dimension_mode: String,

    /// Engine version compatibility identifier (e.g., `"0.9.0"`).
    pub engine_version: String,

    /// Initial scene path relative to the project root (e.g., `"scenes/main.aee"`).
    pub entry_scene: String,

    /// UNIX timestamp of the most recent access in seconds.
    pub last_opened: u64,
}

impl Default for ProjectConfig {
    fn default() -> Self {
        Self {
            name: "New Aeon Project".to_string(),
            path: String::new(),
            dimension_mode: "3D".to_string(),
            engine_version: "0.9.0".to_string(),
            entry_scene: "scenes/main.aee".to_string(),
            last_opened: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
        }
    }
}

/// Registry managing recent projects and disk persistence.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProjectRegistry {
    /// List of known recent projects, sorted by most recently opened.
    pub recent_projects: Vec<ProjectConfig>,
}

impl ProjectRegistry {
    /// Canonical storage path for recent projects registry: `~/.config/aeon_engine/recent_projects.json`.
    pub fn config_path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home)
            .join(".config")
            .join("aeon_engine")
            .join("recent_projects.json")
    }

    /// Loads the project registry from disk or returns an empty default if absent.
    pub fn load_from_disk() -> Self {
        let path = Self::config_path();
        if let Ok(content) = fs::read_to_string(&path)
            && let Ok(registry) = serde_json::from_str::<Self>(&content)
        {
            return registry;
        }
        Self::default()
    }

    /// Persists the project registry to disk.
    pub fn save_to_disk(&self) {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = fs::write(path, json);
        }
    }

    /// Adds a project or updates its last opened timestamp, keeping the list sorted.
    pub fn add_or_touch(&mut self, mut config: ProjectConfig) {
        config.last_opened = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        self.recent_projects.retain(|p| p.path != config.path);
        self.recent_projects.insert(0, config);
        self.save_to_disk();
    }

    /// Removes a project from the recent projects list by path.
    pub fn remove(&mut self, path: &str) {
        self.recent_projects.retain(|p| p.path != path);
        self.save_to_disk();
    }

    /// Initializes a new project directory structure on disk with `aeon_project.json`.
    /// Creates `assets/`, `scenes/`, and an initial empty `scenes/main.aee` file.
    pub fn create_project(
        name: &str,
        parent_dir: &Path,
        dimension_mode: &str,
    ) -> std::io::Result<ProjectConfig> {
        let sanitized_name = name.trim();
        let project_dir = parent_dir.join(sanitized_name);
        fs::create_dir_all(project_dir.join("assets"))?;
        fs::create_dir_all(project_dir.join("scenes"))?;

        // Write a valid empty initial scene
        let scene_path = project_dir.join("scenes").join("main.aee");
        if !scene_path.exists() {
            let initial_scene_json = r#"{"name":"Main Scene","entities":[]}"#;
            fs::write(&scene_path, initial_scene_json)?;
        }

        let config = ProjectConfig {
            name: sanitized_name.to_string(),
            path: project_dir.to_string_lossy().to_string(),
            dimension_mode: dimension_mode.to_string(),
            engine_version: "0.9.0".to_string(),
            entry_scene: "scenes/main.aee".to_string(),
            last_opened: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
        };

        let config_json = serde_json::to_string_pretty(&config)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        fs::write(project_dir.join("aeon_project.json"), config_json)?;

        Ok(config)
    }
}