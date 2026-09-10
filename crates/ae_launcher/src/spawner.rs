// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

use std::path::{Path, PathBuf};
use std::process::Command;

/// Locates the `ae_engine` binary in the filesystem.
/// Searches relative to the current executable directory, then checks `target/release` and `target/debug`.
pub fn find_engine_binary() -> Option<PathBuf> {
    if let Ok(current_exe) = std::env::current_exe()
        && let Some(parent) = current_exe.parent()
    {
        let candidate = parent.join("ae_engine");
        if candidate.exists() {
            return Some(candidate);
        }
        let candidate_exe = parent.join("ae_engine.exe");
        if candidate_exe.exists() {
            return Some(candidate_exe);
        }
    }

    // Check workspace target directories
    let candidates = [
        PathBuf::from("target/release/ae_engine"),
        PathBuf::from("target/release/ae_engine.exe"),
        PathBuf::from("target/debug/ae_engine"),
        PathBuf::from("target/debug/ae_engine.exe"),
    ];

    for candidate in &candidates {
        if candidate.exists() {
            return Some(candidate.clone());
        }
    }

    None
}

/// Spawns the Aeon Engine process for a project and terminates the launcher process immediately.
/// Ensures 0 bytes of launcher RAM remains allocated once the editor is launched.
pub fn launch_engine_and_exit(project_path: &Path, dimension_mode: &str) -> std::io::Result<()> {
    log::info!(
        "Launching Aeon Engine for project: {:?} in {} mode",
        project_path,
        dimension_mode
    );

    let mut command = if let Some(binary) = find_engine_binary() {
        let mut cmd = Command::new(binary);
        cmd.arg("--project").arg(project_path);
        cmd.arg("--mode").arg(dimension_mode);
        cmd
    } else {
        // Fallback: Run via cargo if binary not yet compiled directly in target
        let mut cmd = Command::new("cargo");
        cmd.arg("run")
            .arg("--bin")
            .arg("ae_engine")
            .arg("--")
            .arg("--project")
            .arg(project_path)
            .arg("--mode")
            .arg(dimension_mode);
        cmd
    };

    // Detach and spawn child process
    command.spawn()?;

    log::info!("Engine spawned successfully. Exiting launcher to release all RAM/VRAM.");
    std::process::exit(0);
}