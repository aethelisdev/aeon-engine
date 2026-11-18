// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Standalone project export pipeline: dependency graph scanning, incremental parallel
//! `Intermediate/` asset cooking, `16`-byte aligned `.aepak` packaging, and Linux/Windows
//! `ae_runtime` bundle generation.

pub mod cooker;
pub mod dependency_graph;
pub mod error;
pub mod pipeline;
pub mod targets;

pub use cooker::{CookCompressionProfile, CookSummary, cook_and_pack_project};
pub use dependency_graph::{
    DependencyGraph, DiscoveredAsset, DiscoveredWorld, ExportProjectManifest,
    collect_project_dependencies,
};
pub use error::{ExportError, ExportResult};
pub use pipeline::{
    ExportOutcome, ExportPipelineEvent, ExportProgress, ExportRequest, ExportStage,
    run_export_pipeline, spawn_export_pipeline,
};
pub use targets::{ExportTarget, PreflightReport, preflight_check};