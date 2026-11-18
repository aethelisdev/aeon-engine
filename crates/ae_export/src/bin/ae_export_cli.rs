// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Command-line interface for exporting standalone Aeon Engine projects (`game.aepak` +
//! `ae_runtime` executable) for Linux and Windows targets.

use ae_export::{CookCompressionProfile, ExportRequest, ExportTarget, run_export_pipeline};
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let mut project_root = PathBuf::from(".");
    let mut output_dir = PathBuf::from("dist_export/standalone");
    let mut target = ExportTarget::LinuxX86_64;
    let mut compression = CookCompressionProfile::FastLz4;
    let mut build_binary = true;
    let mut selected_worlds = Vec::new();

    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--project" => {
                if let Some(val) = args.next() {
                    project_root = PathBuf::from(val);
                }
            }
            "--out" => {
                if let Some(val) = args.next() {
                    output_dir = PathBuf::from(val);
                }
            }
            "--target" => {
                if let Some(val) = args.next() {
                    target = match val.to_ascii_lowercase().as_str() {
                        "linux" => ExportTarget::LinuxX86_64,
                        "windows" | "windows-gnu" => ExportTarget::WindowsX86_64Gnu,
                        "windows-msvc" => ExportTarget::WindowsX86_64Msvc,
                        other => {
                            return Err(format!(
                                "Unsupported --target '{other}' (expected linux, windows-gnu, windows-msvc)"
                            )
                            .into());
                        }
                    };
                }
            }
            "--compression" => {
                if let Some(val) = args.next() {
                    compression = match val.to_ascii_lowercase().as_str() {
                        "lz4" | "fast" => CookCompressionProfile::FastLz4,
                        "raw" | "none" => CookCompressionProfile::UncompressedRaw,
                        other => {
                            return Err(format!(
                                "Unsupported --compression '{other}' (expected lz4 or raw)"
                            )
                            .into());
                        }
                    };
                }
            }
            "--world" => {
                if let Some(val) = args.next() {
                    selected_worlds.push(PathBuf::from(val));
                }
            }
            "--no-build" => build_binary = false,
            other => {
                return Err(format!("Unknown argument: {other}").into());
            }
        }
    }

    let req = ExportRequest {
        project_root,
        selected_worlds,
        target,
        compression,
        output_dir,
        build_binary,
        workspace_root: None,
    };

    let outcome = run_export_pipeline(&req, |prog| {
        log::info!("[{:>3.0}%] {}", prog.fraction * 100.0, prog.message);
    })?;

    log::info!(
        "Exported {} worlds ({} assets) to {}",
        outcome.worlds_packed,
        outcome.summary.total_assets,
        outcome.output_dir.display()
    );
    for warn in &outcome.warnings {
        log::warn!("{warn}");
    }
    Ok(())
}