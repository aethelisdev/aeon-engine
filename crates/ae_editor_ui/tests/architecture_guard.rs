// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Architecture & Quality Guard Tests
//!
//! Automated verification suite enforcing file size limits, absence of raw
//! pixel coordinate hacks, and preventing bloated boilerplate in editor UI panels.

use std::fs;
use std::path::{Path, PathBuf};

/// Maximum allowed lines for a single declarative UI panel module.
const MAX_PANEL_LINE_LIMIT: usize = 450;

/// Collects all `.rs` files recursively in a directory.
fn collect_rs_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                files.extend(collect_rs_files(&path));
            } else if path.extension().is_some_and(|ext| ext == "rs") {
                files.push(path);
            }
        }
    }
    files
}

#[test]
fn test_architecture_guard_active() {
    // Verifies that architecture guard test runner is operational
    assert!(MAX_PANEL_LINE_LIMIT > 0);
}

#[test]
fn test_workbench_panels_line_limit() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let workbench_dir = Path::new(manifest_dir).join("src/ui/workbench");

    if !workbench_dir.exists() {
        return;
    }

    let files = collect_rs_files(&workbench_dir);
    let mut violated_files = Vec::new();

    for file in files {
        if let Ok(content) = fs::read_to_string(&file) {
            let line_count = content.lines().count();
            if line_count > MAX_PANEL_LINE_LIMIT {
                violated_files.push((file.display().to_string(), line_count));
            }
        }
    }

    assert!(
        violated_files.is_empty(),
        "Architecture Guard Violation: The following workbench files exceed {} lines limit:\n{:?}",
        MAX_PANEL_LINE_LIMIT,
        violated_files
    );
}

#[test]
fn test_workbench_no_raw_node_manipulation() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let workbench_dir = Path::new(manifest_dir).join("src/ui/workbench");

    if !workbench_dir.exists() {
        return;
    }

    let files = collect_rs_files(&workbench_dir);
    let mut violations = Vec::new();

    for file in files {
        if let Ok(content) = fs::read_to_string(&file) {
            for (line_idx, line) in content.lines().enumerate() {
                // Check for manual coordinate manipulation
                if line.contains("set_computed_rect(") {
                    violations.push((
                        file.display().to_string(),
                        line_idx + 1,
                        "Direct coordinate assignment (set_computed_rect) is forbidden. Use declarative layout.",
                    ));
                }
                // Check for raw node creation in panels
                if line.contains(".create_node(") && !file.to_string_lossy().contains("platform.rs")
                {
                    violations.push((
                        file.display().to_string(),
                        line_idx + 1,
                        "Raw tree.create_node() in panels is forbidden. Use declarative widgets (Row, Col, Box).",
                    ));
                }
            }
        }
    }

    assert!(
        violations.is_empty(),
        "Architecture Guard Violation: Detected forbidden manual node/coordinate manipulation:\n{:?}",
        violations
    );
}

#[test]
fn test_workbench_no_targets_spaghetti() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let workbench_dir = Path::new(manifest_dir).join("src/ui/workbench");

    if !workbench_dir.exists() {
        return;
    }

    let files = collect_rs_files(&workbench_dir);
    let mut violations = Vec::new();

    for file in files {
        if let Ok(content) = fs::read_to_string(&file) {
            for (line_idx, line) in content.lines().enumerate() {
                if line.contains("struct Target ") || line.contains("struct Targets ") {
                    violations.push((
                        file.display().to_string(),
                        line_idx + 1,
                        "Parallel pixel target structs are forbidden. Use native UI event dispatching.",
                    ));
                }
            }
        }
    }

    assert!(
        violations.is_empty(),
        "Architecture Guard Violation: Detected forbidden targets struct:\n{:?}",
        violations
    );
}

#[test]
fn test_strict_zero_allow_attributes() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let crates_dir = Path::new(manifest_dir)
        .parent()
        .unwrap_or_else(|| Path::new(manifest_dir));

    let files = collect_rs_files(crates_dir);
    let mut violations = Vec::new();

    for file in files {
        if file.ends_with("architecture_guard.rs") {
            continue;
        }
        if let Ok(content) = fs::read_to_string(&file) {
            for (line_idx, line) in content.lines().enumerate() {
                let trimmed = line.trim();
                if trimmed.starts_with("//") {
                    continue;
                }
                if line.contains("#[allow(") || line.contains("#![allow(") {
                    violations.push((
                        file.display().to_string(),
                        line_idx + 1,
                        "Rule 11.1 Violation: #[allow(...)] attribute detected! Linters must never be silenced; fix the root cause.",
                    ));
                }
            }
        }
    }

    assert!(
        violations.is_empty(),
        "Architecture Guard Failure: Clippy suppressions (#[allow]) are universally forbidden:\n{:?}",
        violations
    );
}

#[test]
fn test_no_empty_closure_hacks() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let ui_dir = Path::new(manifest_dir).join("src/ui");

    if !ui_dir.exists() {
        return;
    }

    let files = collect_rs_files(&ui_dir);
    let mut violations = Vec::new();

    for file in files {
        if let Ok(content) = fs::read_to_string(&file) {
            for (line_idx, line) in content.lines().enumerate() {
                let trimmed = line.trim();
                if trimmed.starts_with("//") {
                    continue;
                }
                // Check for dummy closure patterns used to bypass declarative UI
                if trimmed.contains("|_| {}")
                    || trimmed.contains("|_| ()")
                    || trimmed.contains("|_scope| {}")
                    || trimmed.contains("|_s| {}")
                {
                    violations.push((
                        file.display().to_string(),
                        line_idx + 1,
                        "Architecture Guard Violation: Dummy empty closure hack detected. You must write genuine declarative UI.",
                    ));
                }
            }
        }
    }

    assert!(
        violations.is_empty(),
        "Architecture Guard Violation: Detected dummy closure hacks:\n{:?}",
        violations
    );
}