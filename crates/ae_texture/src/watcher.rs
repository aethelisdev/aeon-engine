// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! File modification tracking and live hot-reloading utilities for texture assets.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Texture file watcher that tracks disk modification timestamps for live asset hot-reloading.
#[derive(Debug, Default)]
pub struct TextureFileWatcher {
    /// Maps canonical disk paths to their last checked modification timestamp.
    tracked_files: HashMap<PathBuf, SystemTime>,
}

impl TextureFileWatcher {
    /// Creates a new texture file watcher instance.
    pub fn new() -> Self {
        Self {
            tracked_files: HashMap::new(),
        }
    }

    /// Registers or updates a tracked texture path with an initial timestamp.
    /// Uses `entry().or_insert_with()` so existing file timestamps are preserved
    /// and not overwritten on every frame check.
    pub fn track_file(&mut self, path: PathBuf, initial_modified: Option<SystemTime>) {
        if let std::collections::hash_map::Entry::Vacant(entry) = self.tracked_files.entry(path) {
            let time = initial_modified
                .or_else(|| {
                    std::fs::metadata(entry.key())
                        .and_then(|m| m.modified())
                        .ok()
                })
                .unwrap_or_else(SystemTime::now);
            entry.insert(time);
        }
    }

    /// Manually updates the recorded modification timestamp for a tracked file.
    pub fn update_timestamp(&mut self, path: &Path, time: SystemTime) {
        if let Some(tracked) = self.tracked_files.get_mut(path) {
            *tracked = time;
        }
    }

    /// Untracks a texture file.
    pub fn untrack_file(&mut self, path: &Path) {
        self.tracked_files.remove(path);
    }

    /// Scans all tracked texture files on disk and returns a list of canonical paths
    /// that have been modified since their last recorded timestamp.
    /// Automatically updates internal timestamps for modified files.
    pub fn check_modified_files(&mut self) -> Vec<PathBuf> {
        let mut modified = Vec::new();

        for (path, last_time) in self.tracked_files.iter_mut() {
            if let Ok(meta) = std::fs::metadata(path) {
                if let Ok(current_time) = meta.modified() {
                    if current_time > *last_time {
                        *last_time = current_time;
                        modified.push(path.clone());
                    }
                }
            }
        }

        modified
    }

    /// Returns the number of files currently tracked for live hot-reloading.
    pub fn tracked_count(&self) -> usize {
        self.tracked_files.len()
    }
}