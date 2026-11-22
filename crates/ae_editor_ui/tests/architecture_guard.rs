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
    const { assert!(MAX_PANEL_LINE_LIMIT > 0) };
}

#[test]
fn test_workbench_panels_line_limit() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let guarded_files = [
        Path::new(manifest_dir).join("src/ui/editor_ui.rs"),
        Path::new(manifest_dir).join("src/ui/panel_registry.rs"),
        Path::new(manifest_dir).join("src/ui/panel_layout.rs"),
        Path::new(manifest_dir).join("src/ui/icons.rs"),
        Path::new(manifest_dir).join("src/ui/types.rs"),
    ];

    let mut violated_files = Vec::new();

    for file in &guarded_files {
        if let Ok(content) = fs::read_to_string(file) {
            let line_count = content.lines().count();
            if line_count > MAX_PANEL_LINE_LIMIT {
                violated_files.push((file.display().to_string(), line_count));
            }
        }
    }

    assert!(
        violated_files.is_empty(),
        "Architecture Guard Violation: The following editor UI runtime files exceed {} lines limit:\n{:?}",
        MAX_PANEL_LINE_LIMIT,
        violated_files
    );
}

#[test]
fn test_workbench_no_raw_node_manipulation() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let guarded_dirs = [Path::new(manifest_dir).join("src/ui")];

    let mut violations = Vec::new();

    for dir in &guarded_dirs {
        for file in collect_rs_files(dir) {
            let file_name = file
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or_default();
            if file_name.starts_with("tests") || file_name.ends_with("_tests.rs") {
                continue;
            }
            if let Ok(content) = fs::read_to_string(&file) {
                for (line_idx, line) in content.lines().enumerate() {
                    if line.contains("set_computed_rect(") {
                        violations.push((
                            file.display().to_string(),
                            line_idx + 1,
                            "Direct coordinate assignment (set_computed_rect) is forbidden. Use declarative layout.",
                        ));
                    }
                    if line.contains(".create_node(")
                        && !file.to_string_lossy().contains("platform.rs")
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
    let guarded_dirs = [Path::new(manifest_dir).join("src/ui")];

    let mut violations = Vec::new();

    for dir in &guarded_dirs {
        for file in collect_rs_files(dir) {
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
    }

    assert!(
        violations.is_empty(),
        "Architecture Guard Violation: Detected forbidden targets struct:\n{:?}",
        violations
    );
}

#[test]
fn test_workbench_directory_and_bridge_wrappers_eliminated() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let ui_dir = Path::new(manifest_dir).join("src/ui");
    assert!(
        !ui_dir.join("workbench").exists()
            && !ui_dir.join("workbench.rs").exists()
            && !ui_dir.join("editor_sync.rs").exists()
            && !ui_dir.join("editor_actions.rs").exists()
            && !ui_dir.join("frame_data.rs").exists()
            && !ui_dir.join("panel_registry_tests.rs").exists(),
        "Faz 12 Architecture Guard Failure: Removed transitional UI glue files reappeared."
    );

    let forbidden = [
        "OverlayUpdateParams",
        "IrisEditorOverlay",
        "IrisPassParams",
        "crate::ui::workbench",
    ];
    let mut violations = Vec::new();
    for file in collect_rs_files(&ui_dir) {
        if let Ok(content) = fs::read_to_string(&file) {
            for (line_idx, line) in content.lines().enumerate() {
                for &needle in &forbidden {
                    if line.contains(needle) {
                        violations.push((file.display().to_string(), line_idx + 1, needle));
                    }
                }
            }
        }
    }
    assert!(
        violations.is_empty(),
        "Faz 11 Architecture Guard Failure: Legacy bridge wrappers reappeared:\n{:#?}",
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

#[test]
fn test_irisui_crates_io_purity_guard() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let irisui_dir = Path::new(manifest_dir)
        .parent()
        .unwrap_or_else(|| Path::new(manifest_dir))
        .join("irisui");

    if !irisui_dir.exists() {
        return;
    }

    let files = collect_rs_files(&irisui_dir);
    let mut violations = Vec::new();

    let forbidden_domain_tokens = [
        "CONSOLE_TAG_",
        "TIMELINE_TAG_",
        "ConsoleFilterLevel",
        "ConsoleFilterExt",
        "ConsoleLogCounts",
        "MediaTransportAction",
        "MediaTransportStyle",
        "TimelineKeyframeMarker",
        "TimelineRulerStyle",
        "TimelineRuler",
        "TimelineTrack",
        "TimelinePlayhead",
        "TimelineKeyframe",
        "AssetCardBuilder",
        "AssetCardPreview",
        "AssetCardBadge",
        "AssetCardStyle",
        "AssetCardFrame",
    ];

    for file in files {
        if let Ok(content) = fs::read_to_string(&file) {
            for (line_idx, line) in content.lines().enumerate() {
                let trimmed = line.trim();
                if trimmed.starts_with("//") {
                    continue;
                }
                for &token in &forbidden_domain_tokens {
                    if line.contains(token) {
                        violations.push((
                            file.display().to_string(),
                            line_idx + 1,
                            format!("Domain contamination detected in crates.io pure library: contains '{}'", token),
                        ));
                    }
                }
            }
        }
    }

    assert!(
        violations.is_empty(),
        "Architecture Guard Failure: Domain-specific concepts leaked into pure irisui library:\n{:#?}",
        violations
    );
}

fn contains_rust_let_keyword(line: &str) -> bool {
    let bytes = line.as_bytes();
    let mut in_string = false;
    let mut escape = false;
    let mut i = 0;

    while i < bytes.len() {
        let b = bytes[i];
        if in_string {
            if escape {
                escape = false;
            } else if b == b'\\' {
                escape = true;
            } else if b == b'"' {
                in_string = false;
            }
            i += 1;
            continue;
        }
        if b == b'"' {
            in_string = true;
            i += 1;
            continue;
        }
        if b == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'/' {
            break;
        }
        if bytes[i..].starts_with(b"let") {
            let left_ok = i == 0 || !(bytes[i - 1].is_ascii_alphanumeric() || bytes[i - 1] == b'_');
            let right_idx = i + 3;
            let right_ok = right_idx >= bytes.len()
                || !(bytes[right_idx].is_ascii_alphanumeric() || bytes[right_idx] == b'_');
            if left_ok && right_ok {
                return true;
            }
        }
        i += 1;
    }
    false
}

#[test]
fn test_editor_ui_zero_non_test_let_statements() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let ui_dir = Path::new(manifest_dir).join("src/ui");
    if !ui_dir.exists() {
        return;
    }

    let mut violations = Vec::new();
    for file in collect_rs_files(&ui_dir) {
        let name = file
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or_default();
        if name.starts_with("tests") || name.ends_with("_tests.rs") {
            continue;
        }
        let Ok(content) = fs::read_to_string(&file) else {
            continue;
        };
        let mut in_cfg_test = false;
        let mut entered_block = false;
        let mut brace_depth = 0usize;
        for (line_idx, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("#[cfg(test)]") {
                in_cfg_test = true;
                entered_block = line.contains('{');
                brace_depth = line
                    .matches('{')
                    .count()
                    .saturating_sub(line.matches('}').count());
                if (entered_block && brace_depth == 0) || (!entered_block && trimmed.ends_with(';'))
                {
                    in_cfg_test = false;
                }
                continue;
            }
            if in_cfg_test {
                if line.contains('{') {
                    entered_block = true;
                }
                brace_depth += line.matches('{').count();
                brace_depth = brace_depth.saturating_sub(line.matches('}').count());
                if (entered_block && brace_depth == 0) || (!entered_block && trimmed.ends_with(';'))
                {
                    in_cfg_test = false;
                }
                continue;
            }
            if contains_rust_let_keyword(line) {
                violations.push((
                    file.display().to_string(),
                    line_idx + 1,
                    trimmed.to_string(),
                ));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "Rule 1.6 Architecture Guard Violation: imperative `let` statements are forbidden in non-test crates/ae_editor_ui/src/ui/** code:\n{:#?}",
        violations
    );
}