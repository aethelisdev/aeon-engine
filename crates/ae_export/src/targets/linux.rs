// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Linux (`x86_64-unknown-linux-gnu`) standalone bundle staging and launcher script generator.

use super::{ExportTarget, build_runtime_binary};
use crate::error::ExportResult;
use std::path::{Path, PathBuf};

/// Stages a standalone Linux release bundle (`ae_runtime`, `game.aepak`, and `run_game.sh`)
/// into `output_dir`.
///
/// When `build_binary` is `true`, builds `ae_runtime --release --features static-game-logic`
/// for `x86_64-unknown-linux-gnu` before copying the executable alongside `pak_path`.
///
/// # Errors
/// Returns [`crate::ExportError`] on build or filesystem staging failures.
pub fn stage_linux_bundle(
    workspace_root: &Path,
    output_dir: &Path,
    pak_path: &Path,
    build_binary: bool,
) -> ExportResult<Option<PathBuf>> {
    std::fs::create_dir_all(output_dir)?;

    let staged_pak = output_dir.join("game.aepak");
    if pak_path != staged_pak {
        std::fs::copy(pak_path, &staged_pak)?;
    }

    let launcher_script = output_dir.join("run_game.sh");
    std::fs::write(
        &launcher_script,
        "#!/usr/bin/env bash\nset -euo pipefail\nDIR=\"$(cd \"$(dirname \"${BASH_SOURCE[0]}\")\" && pwd)\"\nexec \"$DIR/ae_runtime\" --pak \"$DIR/game.aepak\" \"$@\"\n",
    )?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&launcher_script)?.permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&launcher_script, perms)?;
    }

    if !build_binary {
        return Ok(None);
    }

    let built = build_runtime_binary(workspace_root, ExportTarget::LinuxX86_64)?;
    let dest_bin = output_dir.join(ExportTarget::LinuxX86_64.binary_filename());
    std::fs::copy(built, &dest_bin)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&dest_bin)?.permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&dest_bin, perms)?;
    }

    Ok(Some(dest_bin))
}