// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Windows (`x86_64-pc-windows-gnu` / `x86_64-pc-windows-msvc`) standalone bundle staging.

use super::{ExportTarget, build_runtime_binary};
use crate::error::ExportResult;
use std::path::{Path, PathBuf};

/// Stages a standalone Windows release bundle (`ae_runtime.exe`, `game.aepak`, and `run_game.bat`)
/// into `output_dir`.
///
/// When `build_binary` is `true`, builds `ae_runtime --release --features static-game-logic`
/// for the requested Windows `target` (`WindowsX86_64Gnu` or `WindowsX86_64Msvc`).
///
/// # Errors
/// Returns [`crate::ExportError`] on missing cross-linker, compilation failure, or I/O error.
pub fn stage_windows_bundle(
    workspace_root: &Path,
    output_dir: &Path,
    pak_path: &Path,
    target: ExportTarget,
    build_binary: bool,
) -> ExportResult<Option<PathBuf>> {
    std::fs::create_dir_all(output_dir)?;

    let staged_pak = output_dir.join("game.aepak");
    if pak_path != staged_pak {
        std::fs::copy(pak_path, &staged_pak)?;
    }

    let launcher_bat = output_dir.join("run_game.bat");
    std::fs::write(
        &launcher_bat,
        "@echo off\r\n\"%~dp0ae_runtime.exe\" --pak \"%~dp0game.aepak\" %*\r\n",
    )?;

    if !build_binary {
        return Ok(None);
    }

    let built = build_runtime_binary(workspace_root, target)?;
    let dest_bin = output_dir.join(target.binary_filename());
    std::fs::copy(built, &dest_bin)?;
    Ok(Some(dest_bin))
}