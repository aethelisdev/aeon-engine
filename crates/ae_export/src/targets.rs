// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Target platform definitions, toolchain preflight checks, and standalone runtime bundlers
//! for Linux (`x86_64-unknown-linux-gnu`) and Windows (`x86_64-pc-windows-gnu` / `msvc`).

pub mod linux;
pub mod windows;

use crate::error::{ExportError, ExportResult};
use ae_world::TargetTextureProfile;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Supported standalone desktop export targets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ExportTarget {
    /// Native Linux ELF64 executable (`x86_64-unknown-linux-gnu`).
    #[default]
    LinuxX86_64,
    /// Windows PE64 executable cross-compiled with MinGW (`x86_64-pc-windows-gnu`).
    WindowsX86_64Gnu,
    /// Windows PE64 executable compiled with MSVC (`x86_64-pc-windows-msvc`).
    WindowsX86_64Msvc,
}

impl ExportTarget {
    /// Returns the Rust/Cargo target triple string.
    #[must_use]
    pub const fn cargo_triple(self) -> &'static str {
        match self {
            Self::LinuxX86_64 => "x86_64-unknown-linux-gnu",
            Self::WindowsX86_64Gnu => "x86_64-pc-windows-gnu",
            Self::WindowsX86_64Msvc => "x86_64-pc-windows-msvc",
        }
    }

    /// Returns the output runtime executable filename for this target.
    #[must_use]
    pub const fn binary_filename(self) -> &'static str {
        match self {
            Self::LinuxX86_64 => "ae_runtime",
            Self::WindowsX86_64Gnu | Self::WindowsX86_64Msvc => "ae_runtime.exe",
        }
    }

    /// Returns a human-readable platform label for CLI and Editor UI displays.
    #[must_use]
    pub const fn display_name(self) -> &'static str {
        match self {
            Self::LinuxX86_64 => "Linux (x86_64)",
            Self::WindowsX86_64Gnu => "Windows (x86_64 MinGW)",
            Self::WindowsX86_64Msvc => "Windows (x86_64 MSVC)",
        }
    }

    /// Returns the GPU texture compression profile matched to this target platform.
    #[must_use]
    pub const fn texture_profile(self) -> TargetTextureProfile {
        TargetTextureProfile::Desktop
    }
}

/// Result of verifying whether Rust target stdlib and system linker tools are installed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreflightReport {
    /// Target triple evaluated.
    pub target: ExportTarget,
    /// `true` when `rustup target list --installed` includes `target.cargo_triple()`.
    pub rustup_target_installed: bool,
    /// `true` when a compatible C/system linker binary is reachable in `PATH`.
    pub linker_available: bool,
    /// Actionable install commands or diagnostics when checks fail.
    pub missing_hints: Vec<String>,
}

impl PreflightReport {
    /// Returns `true` when all toolchain prerequisites are satisfied.
    #[must_use]
    pub fn is_ready(&self) -> bool {
        self.rustup_target_installed && self.linker_available
    }
}

/// Checks `rustup` target installation and linker availability for `target`.
#[must_use]
pub fn preflight_check(target: ExportTarget) -> PreflightReport {
    let triple = target.cargo_triple();
    let rustup_target_installed = Command::new("rustup")
        .args(["target", "list", "--installed"])
        .output()
        .ok()
        .filter(|out| out.status.success())
        .is_some_and(|out| {
            String::from_utf8_lossy(&out.stdout)
                .lines()
                .any(|line| line.trim() == triple)
        });

    let linker_available = match target {
        ExportTarget::LinuxX86_64 => command_succeeds("cc", &["--version"]),
        ExportTarget::WindowsX86_64Gnu => {
            command_succeeds("x86_64-w64-mingw32-gcc", &["--version"])
        }
        ExportTarget::WindowsX86_64Msvc => {
            cfg!(target_os = "windows")
                || command_succeeds("lld-link", &["--version"])
                || command_succeeds("cargo-xwin", &["--version"])
        }
    };

    let mut missing_hints = Vec::new();
    if !rustup_target_installed {
        missing_hints.push(format!("Install target stdlib: rustup target add {triple}"));
    }
    if !linker_available {
        let hint = match target {
            ExportTarget::LinuxX86_64 => "Install system C linker (e.g. gcc/clang)".to_string(),
            ExportTarget::WindowsX86_64Gnu => {
                "Install MinGW cross-linker (e.g. x86_64-w64-mingw32-gcc / mingw-w64)".to_string()
            }
            ExportTarget::WindowsX86_64Msvc => {
                "Windows MSVC target requires Windows host link.exe or lld-link / cargo-xwin"
                    .to_string()
            }
        };
        missing_hints.push(hint);
    }

    PreflightReport {
        target,
        rustup_target_installed,
        linker_available,
        missing_hints,
    }
}

fn command_succeeds(program: &str, args: &[&str]) -> bool {
    Command::new(program)
        .args(args)
        .output()
        .is_ok_and(|out| out.status.success())
}

/// Invokes `cargo build --release -p ae_runtime --features static-game-logic --target <triple>`
/// inside `workspace_root` and returns the compiled executable path.
///
/// # Errors
/// Returns [`ExportError::ToolchainMissing`] or [`ExportError::CargoBuildFailed`] on build errors.
pub(crate) fn build_runtime_binary(
    workspace_root: &Path,
    target: ExportTarget,
) -> ExportResult<PathBuf> {
    let report = preflight_check(target);
    if !report.is_ready() {
        return Err(ExportError::ToolchainMissing(
            report.missing_hints.join("; "),
        ));
    }

    let triple = target.cargo_triple();
    let status = Command::new("cargo")
        .current_dir(workspace_root)
        .args([
            "build",
            "--release",
            "-p",
            "ae_runtime",
            "--features",
            "static-game-logic",
            "--target",
            triple,
        ])
        .status()?;

    if !status.success() {
        return Err(ExportError::CargoBuildFailed(format!(
            "cargo build --release -p ae_runtime --target {triple} exited with {status}"
        )));
    }

    let built_bin = workspace_root
        .join("target")
        .join(triple)
        .join("release")
        .join(target.binary_filename());
    if !built_bin.is_file() {
        return Err(ExportError::CargoBuildFailed(format!(
            "Expected runtime binary not found at {}",
            built_bin.display()
        )));
    }

    Ok(built_bin)
}