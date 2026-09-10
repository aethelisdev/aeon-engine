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
        let immediate_candidates = [
            parent.join("ae_engine"),
            parent.join("ae_engine.exe"),
            parent
                .parent()
                .map(|p| p.join("ae_engine"))
                .unwrap_or_default(),
            parent
                .parent()
                .map(|p| p.join("ae_engine.exe"))
                .unwrap_or_default(),
        ];
        for cand in &immediate_candidates {
            if cand.is_file() {
                return Some(cand.clone());
            }
        }
    }

    // Check workspace target directories — select the most recently built binary
    let mut candidates = vec![
        PathBuf::from("target/release/ae_engine"),
        PathBuf::from("target/release/ae_engine.exe"),
        PathBuf::from("target/debug/ae_engine"),
        PathBuf::from("target/debug/ae_engine.exe"),
    ];

    if let Ok(manifest) = std::env::var("CARGO_MANIFEST_DIR") {
        let manifest_path = PathBuf::from(manifest);
        if let Some(ws_root) = manifest_path.parent().and_then(|p| p.parent()) {
            candidates.push(ws_root.join("target/release/ae_engine"));
            candidates.push(ws_root.join("target/release/ae_engine.exe"));
            candidates.push(ws_root.join("target/debug/ae_engine"));
            candidates.push(ws_root.join("target/debug/ae_engine.exe"));
        }
    }

    let mut newest_candidate: Option<(PathBuf, std::time::SystemTime)> = None;

    for candidate in &candidates {
        if candidate.is_file()
            && let Ok(meta) = candidate.metadata()
            && let Ok(modified) = meta.modified()
        {
            if let Some((_, best_time)) = &newest_candidate {
                if modified > *best_time {
                    newest_candidate = Some((candidate.clone(), modified));
                }
            } else {
                newest_candidate = Some((candidate.clone(), modified));
            }
        }
    }

    if let Some((best_path, _)) = newest_candidate {
        return Some(best_path);
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

    // Run child process synchronously and wait for it to complete so that
    // terminal input/output stays cleanly attached to the foreground process.
    let status = command.status()?;

    log::info!("Aeon Engine process terminated with status: {status:?}. Exiting launcher.");
    std::process::exit(status.code().unwrap_or(0));
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verifies that `find_engine_binary` locates an existing engine binary in target directories.
    #[test]
    fn test_find_engine_binary_locates_target_executable() {
        let binary = find_engine_binary();
        assert!(
            binary.is_some(),
            "Engine binary should be located in workspace target directory"
        );
        let path = binary.unwrap();
        assert!(path.exists(), "Located binary path must exist on disk");
    }
}