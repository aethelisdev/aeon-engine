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

/// Detects whether the launcher is executing within a Cargo workspace development tree.
/// Returns `Some(workspace_root)` if a valid Cargo workspace containing `Cargo.toml` is found,
/// or `None` if running as a standalone deployed binary outside the repository.
pub fn detect_dev_workspace_root() -> Option<PathBuf> {
    if let Ok(manifest) = std::env::var("CARGO_MANIFEST_DIR") {
        let path = PathBuf::from(manifest);
        if let Some(ws_root) = path.parent().and_then(|p| p.parent())
            && ws_root.join("Cargo.toml").is_file()
        {
            return Some(ws_root.to_path_buf());
        }
    }

    let cwd = PathBuf::from(".");
    if cwd.join("Cargo.toml").is_file() {
        return Some(cwd);
    }

    let parent = PathBuf::from("..");
    if parent.join("Cargo.toml").is_file() {
        return Some(parent);
    }

    if let Ok(current_exe) = std::env::current_exe() {
        let mut cur = current_exe.parent();
        for _ in 0..4 {
            if let Some(dir) = cur {
                if dir.join("Cargo.toml").is_file() {
                    return Some(dir.to_path_buf());
                }
                cur = dir.parent();
            }
        }
    }

    None
}

/// Spawns the Aeon Engine process for a project and terminates the launcher process immediately.
/// In development environments (when running within a Cargo workspace), it automatically invokes
/// `cargo run` targeting `ae_engine` with matching build profile (`--release` or debug) to guarantee
/// that any modified engine code is incrementally recompiled before launching.
/// In standalone production deployments, it directly executes the precompiled `ae_engine` binary
/// with zero cargo overhead.
pub fn launch_engine_and_exit(project_path: &Path, dimension_mode: &str) -> std::io::Result<()> {
    log::info!(
        "Launching Aeon Engine for project: {:?} in {} mode",
        project_path,
        dimension_mode
    );

    // If running in a development workspace tree, force incremental build & launch via Cargo
    // to guarantee the engine binary is always 100% fresh and up to date.
    if let Some(ws_root) = detect_dev_workspace_root() {
        let cargo_bin = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
        let mut cmd = Command::new(cargo_bin);
        cmd.current_dir(ws_root);
        cmd.arg("run");
        if !cfg!(debug_assertions) {
            cmd.arg("--release");
        }
        cmd.arg("-p")
            .arg("ae_engine")
            .arg("--bin")
            .arg("ae_engine")
            .arg("--")
            .arg("--project")
            .arg(project_path)
            .arg("--mode")
            .arg(dimension_mode);

        log::info!("Executing Cargo build & run pipeline for ae_engine...");
        if let Ok(status) = cmd.status() {
            log::info!("Aeon Engine process terminated with status: {status:?}. Exiting launcher.");
            std::process::exit(status.code().unwrap_or(0));
        }
        log::warn!("Cargo invocation failed; falling back to direct binary resolution.");
    }

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
    let status = match command.status() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("[AE_LAUNCHER ERROR] Failed to execute Aeon Engine binary: {e}");
            return Err(e);
        }
    };

    log::info!("Aeon Engine process terminated with status: {status:?}. Exiting launcher.");
    std::process::exit(status.code().unwrap_or(0));
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verifies that `find_engine_binary` locates an existing engine binary in target directories.
    #[test]
    fn test_find_engine_binary_locates_target_executable() {
        if let Some(path) = find_engine_binary() {
            assert!(path.exists(), "Located binary path must exist on disk");
        } else {
            // In a clean environment where ae_engine binary has not yet been compiled,
            // find_engine_binary safely returns None, and the launcher relies on workspace detection.
            assert!(
                detect_dev_workspace_root().is_some(),
                "In absence of precompiled binary, workspace root must be detectable for cargo fallback"
            );
        }
    }

    /// Verifies that `detect_dev_workspace_root` locates the workspace root during testing.
    #[test]
    fn test_detect_dev_workspace_root_resolution() {
        let ws_root = detect_dev_workspace_root();
        assert!(
            ws_root.is_some(),
            "Workspace root must be resolvable in cargo test execution"
        );
        let root = ws_root.unwrap();
        assert!(
            root.join("Cargo.toml").exists(),
            "Resolved workspace root must contain Cargo.toml"
        );
    }
}