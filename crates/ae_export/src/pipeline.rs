// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! High-level export orchestrator coordinating preflight checks, dependency scanning,
//! parallel cooking/packing, standalone runtime compilation, and live progress streaming.

use crate::cooker::{CookCompressionProfile, CookSummary, cook_and_pack_project};
use crate::dependency_graph::collect_project_dependencies;
use crate::error::{ExportError, ExportResult};
use crate::targets::linux::stage_linux_bundle;
use crate::targets::windows::stage_windows_bundle;
use crate::targets::{ExportTarget, preflight_check};
use ae_world::open_pak_file;
use std::path::{Path, PathBuf};
use std::sync::mpsc;

/// Coarse execution stage reported during standalone project export.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportStage {
    /// Checking compiler target stdlib and system linker availability.
    Preflight,
    /// Scanning worlds, chunks, `.aemeta` GUIDs, and referenced assets.
    ScanningDependencies,
    /// Cooking raw assets into `Intermediate/` and packing `game.aepak`.
    CookingAndPacking,
    /// Compiling `ae_runtime --release --features static-game-logic`.
    CompilingRuntime,
    /// Copying executable, `game.aepak`, and launch scripts into output directory.
    StagingOutput,
    /// Export completed and output archive passed verification.
    Completed,
    /// Export aborted with an error diagnostic.
    Failed,
}

/// Real-time progress snapshot emitted to CLI or Editor UI progress bars.
#[derive(Debug, Clone)]
pub struct ExportProgress {
    /// Current pipeline stage.
    pub stage: ExportStage,
    /// Normalized completion ratio in `[0.0, 1.0]`.
    pub fraction: f32,
    /// Human-readable status text describing the current operation.
    pub message: String,
}

/// Asynchronous event streamed by [`spawn_export_pipeline`].
#[derive(Debug, Clone)]
pub enum ExportPipelineEvent {
    /// Incremental progress update.
    Progress(ExportProgress),
    /// Pipeline completed successfully with final bundle summary.
    Finished(ExportOutcome),
    /// Pipeline failed with a descriptive error message.
    Error(String),
}

/// Input parameters configuring a standalone project export run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportRequest {
    /// Project workspace root containing `aeon_project.json`, `Content/`, and `Worlds/`.
    pub project_root: PathBuf,
    /// Additional `.aeworld` paths to bundle alongside the `aeon_project.json` entry world.
    pub selected_worlds: Vec<PathBuf>,
    /// Target desktop platform.
    pub target: ExportTarget,
    /// Block compression profile for `game.aepak`.
    pub compression: CookCompressionProfile,
    /// Destination directory where `game.aepak` and runtime executable are staged.
    pub output_dir: PathBuf,
    /// When `true`, compiles `ae_runtime --release --features static-game-logic` for `target`.
    pub build_binary: bool,
    /// Optional workspace root containing `Cargo.toml` (defaults to current working directory).
    pub workspace_root: Option<PathBuf>,
}

/// Final artifact summary produced by a successful [`run_export_pipeline`] execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportOutcome {
    /// Destination bundle directory.
    pub output_dir: PathBuf,
    /// Path to generated `game.aepak`.
    pub pak_path: PathBuf,
    /// Path to staged runtime executable when `build_binary` was `true`.
    pub executable_path: Option<PathBuf>,
    /// Cooking and packing statistics.
    pub summary: CookSummary,
    /// Number of worlds bundled into `game.aepak`.
    pub worlds_packed: usize,
    /// Non-fatal dependency warnings encountered during scanning.
    pub warnings: Vec<String>,
}

/// Executes the full export pipeline synchronously, forwarding progress updates to `progress_cb`.
///
/// # Errors
/// Returns [`ExportError`] if preflight checks (when `build_binary` is `true`), dependency
/// scanning, asset cooking, runtime compilation, or output verification fail.
pub fn run_export_pipeline(
    req: &ExportRequest,
    mut progress_cb: impl FnMut(ExportProgress),
) -> ExportResult<ExportOutcome> {
    progress_cb(ExportProgress {
        stage: ExportStage::Preflight,
        fraction: 0.05,
        message: format!("Checking toolchain for {}...", req.target.display_name()),
    });

    if req.build_binary {
        let report = preflight_check(req.target);
        if !report.is_ready() {
            return Err(ExportError::ToolchainMissing(
                report.missing_hints.join("; "),
            ));
        }
    }

    progress_cb(ExportProgress {
        stage: ExportStage::ScanningDependencies,
        fraction: 0.12,
        message: "Scanning worlds and asset dependencies...".to_string(),
    });
    let graph = collect_project_dependencies(&req.project_root, &req.selected_worlds)?;

    std::fs::create_dir_all(&req.output_dir)?;
    let pak_path = req.output_dir.join("game.aepak");
    let summary = cook_and_pack_project(
        &graph,
        &pak_path,
        req.target.texture_profile(),
        req.compression,
        &mut |cook_frac, msg| {
            progress_cb(ExportProgress {
                stage: ExportStage::CookingAndPacking,
                fraction: 0.15 + cook_frac.clamp(0.0, 1.0) * 0.45,
                message: msg.to_string(),
            });
        },
    )?;

    if req.build_binary {
        progress_cb(ExportProgress {
            stage: ExportStage::CompilingRuntime,
            fraction: 0.65,
            message: format!(
                "Compiling ae_runtime (--release --target {})...",
                req.target.cargo_triple()
            ),
        });
    }

    let workspace_root_owned = resolve_workspace_root(req.workspace_root.as_deref());
    let workspace_root = workspace_root_owned.as_path();

    let executable_path = match req.target {
        ExportTarget::LinuxX86_64 => {
            stage_linux_bundle(workspace_root, &req.output_dir, &pak_path, req.build_binary)?
        }
        ExportTarget::WindowsX86_64Gnu | ExportTarget::WindowsX86_64Msvc => stage_windows_bundle(
            workspace_root,
            &req.output_dir,
            &pak_path,
            req.target,
            req.build_binary,
        )?,
    };

    progress_cb(ExportProgress {
        stage: ExportStage::StagingOutput,
        fraction: 0.92,
        message: "Verifying exported game.aepak archive...".to_string(),
    });

    let reader = open_pak_file(&pak_path)?;
    if reader.index().entries.is_empty() {
        return Err(ExportError::ProjectNotFound(
            "Exported game.aepak contains zero entries".to_string(),
        ));
    }

    let outcome = ExportOutcome {
        output_dir: req.output_dir.clone(),
        pak_path,
        executable_path,
        summary,
        worlds_packed: graph.worlds.len(),
        warnings: graph.warnings,
    };

    progress_cb(ExportProgress {
        stage: ExportStage::Completed,
        fraction: 1.0,
        message: format!(
            "Export complete ({} worlds, {} assets, {:.1} KB).",
            outcome.worlds_packed,
            outcome.summary.total_assets,
            outcome.summary.pak_bytes as f64 / 1024.0
        ),
    });

    Ok(outcome)
}

fn resolve_workspace_root(explicit: Option<&Path>) -> PathBuf {
    if let Some(root) = explicit {
        return root.to_path_buf();
    }
    if Path::new("crates/ae_runtime/Cargo.toml").is_file() {
        return PathBuf::from(".");
    }
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .map_or_else(|| PathBuf::from("."), Path::to_path_buf)
}

/// Launches [`run_export_pipeline`] on a background worker thread and streams [`ExportPipelineEvent`]
/// updates over a non-blocking `mpsc::Receiver`.
#[must_use]
pub fn spawn_export_pipeline(req: ExportRequest) -> mpsc::Receiver<ExportPipelineEvent> {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let result = run_export_pipeline(&req, |prog| {
            let _ = tx.send(ExportPipelineEvent::Progress(prog));
        });
        match result {
            Ok(outcome) => {
                let _ = tx.send(ExportPipelineEvent::Finished(outcome));
            }
            Err(err) => {
                let _ = tx.send(ExportPipelineEvent::Error(err.to_string()));
            }
        }
    });
    rx
}

#[cfg(test)]
mod tests {
    use super::*;
    use ae_world::{EntityRecord, GuidGenerator, ProjectWorkspaceLayout, WorldDocument};

    #[test]
    fn test_export_pipeline_stages_linux_and_windows_bundles_without_cargo_build() {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos());
        let root = std::env::temp_dir().join(format!("ae_export_pipe_{nanos}"));
        let layout = ProjectWorkspaceLayout::new(&root);
        layout.ensure_directories().expect("ensure dirs");

        std::fs::write(
            root.join("aeon_project.json"),
            br#"{"name":"PipeTest","entry_scene":"Worlds/MainWorld/world.aeworld","is_2d":false}"#,
        )
        .expect("write project json");

        let mut ids = GuidGenerator::from_seed(99);
        let mut doc = WorldDocument::new(ids.next_entity_guid(), false, 128.0);
        doc.insert_entity_at(
            [0.0, 0.0, 0.0],
            EntityRecord::new(ids.next_entity_guid(), None),
        );
        doc.save_partitioned_dir(&layout.worlds_dir().join("MainWorld"), None)
            .expect("save world");

        let linux_out = root.join("dist_linux");
        let linux_req = ExportRequest {
            project_root: root.clone(),
            selected_worlds: Vec::new(),
            target: ExportTarget::LinuxX86_64,
            compression: CookCompressionProfile::FastLz4,
            output_dir: linux_out.clone(),
            build_binary: false,
            workspace_root: None,
        };
        let mut stages = Vec::new();
        let outcome = run_export_pipeline(&linux_req, |p| stages.push(p.stage))
            .expect("linux package export must succeed");
        assert!(outcome.pak_path.is_file());
        assert!(linux_out.join("run_game.sh").is_file());
        assert_eq!(stages.first().copied(), Some(ExportStage::Preflight));
        assert_eq!(stages.last().copied(), Some(ExportStage::Completed));

        let win_out = root.join("dist_win");
        let win_req = ExportRequest {
            target: ExportTarget::WindowsX86_64Gnu,
            output_dir: win_out.clone(),
            ..linux_req
        };
        let win_outcome =
            run_export_pipeline(&win_req, |_| {}).expect("windows package export must succeed");
        assert!(win_outcome.pak_path.is_file());
        assert!(win_out.join("run_game.bat").is_file());

        let _ = std::fs::remove_dir_all(&root);
    }
}