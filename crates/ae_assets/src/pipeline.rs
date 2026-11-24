// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Asynchronous Asset Parsing, FBX Conversion & Import Validation Pipeline.
//!
//! Provides headless background parsing workers (`rayon`), path/dimension validation,
//! FBX2glTF conversion spawning, and mesh normalization helpers decoupled from runtime state.

use super::scanner::is_world_file_3d;
use ae_mesh::{ParsedModelData, is_safe_path, parse_gltf_file};
use std::path::{Path, PathBuf};
use std::sync::mpsc::Receiver;

/// Returns the platform-specific `FBX2glTF` binary path relative to workspace root.
#[must_use]
pub const fn resolve_fbx2gltf_tool_path() -> &'static str {
    if cfg!(target_os = "windows") {
        "tools/windows/FBX2glTF.exe"
    } else if cfg!(target_os = "macos") {
        "tools/macos/FBX2glTF_macos"
    } else {
        "tools/linux/FBX2glTF_linux"
    }
}

/// Validates security and 2D/3D dimension compatibility for a dropped or imported asset path.
///
/// Returns `Ok(())` if safe and compatible with `is_2d`, or a user-facing rejection reason.
pub fn validate_asset_drop_for_dimension(path: &Path, is_2d: bool) -> Result<(), &'static str> {
    let path_str = path.to_string_lossy();
    if !is_safe_path(&path_str) {
        return Err("Security Error: Unsafe path blocked!");
    }

    let Some(ext) = path.extension().and_then(|e| e.to_str()) else {
        return Ok(());
    };
    let ext_lower = ext.to_ascii_lowercase();

    if is_2d {
        if matches!(ext_lower.as_str(), "gltf" | "glb" | "fbx" | "obj") {
            return Err("Cannot import 3D models in 2D mode!");
        }
        if ext_lower == "aeworld" && is_world_file_3d(path) {
            return Err("Cannot load 3D world in 2D mode! Switch to 3D mode.");
        }
    } else if ext_lower == "aeworld" && !is_world_file_3d(path) {
        return Err("Cannot load 2D world in 3D mode! Switch to 2D mode.");
    }

    Ok(())
}

/// Spawns a background `rayon` task to parse a `.gltf`/`.glb` file into CPU geometry and texture payloads.
#[must_use]
pub fn spawn_gltf_parse(
    path: String,
    final_name: String,
) -> Receiver<Result<ParsedModelData, String>> {
    let (tx, rx) = std::sync::mpsc::channel();
    rayon::spawn(move || {
        let result = parse_gltf_file(&path, final_name);
        let _ = tx.send(result);
    });
    rx
}

/// Spawns a background `rayon` task that invokes `FBX2glTF` to convert an `.fbx` file into `.glb`.
#[must_use]
pub fn spawn_fbx_convert(path: &Path) -> Receiver<Result<PathBuf, String>> {
    let path_clone = path.to_path_buf();
    let path_str_clone = path.to_string_lossy().to_string();
    let (tx, rx) = std::sync::mpsc::channel();

    rayon::spawn(move || {
        let tool_path = resolve_fbx2gltf_tool_path();
        if !Path::new(tool_path).exists() {
            let _ = tx.send(Err(format!(
                "FBX2glTF tool not found at '{}'. Please make sure the tools/ folder is intact.",
                tool_path
            )));
            return;
        }

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(metadata) = std::fs::metadata(tool_path) {
                let mut perms = metadata.permissions();
                let mode = perms.mode();
                if mode & 0o111 == 0 {
                    perms.set_mode(mode | 0o755);
                    let _ = std::fs::set_permissions(tool_path, perms);
                }
            }
        }

        let file_stem = path_clone
            .file_stem()
            .unwrap_or(std::ffi::OsStr::new("model"))
            .to_string_lossy();

        let parent_writable = path_clone.parent().is_some_and(|dir| {
            let probe = dir.join(format!(".ae_probe_{}", std::process::id()));
            if std::fs::write(&probe, b"").is_ok() {
                let _ = std::fs::remove_file(&probe);
                true
            } else {
                false
            }
        });

        let (output_stem_str, output_path) = if parent_writable {
            let stem = path_clone.with_extension("");
            let glb = path_clone.with_extension("glb");
            (stem.to_string_lossy().to_string(), glb)
        } else {
            let temp_dir = std::env::temp_dir();
            let temp_stem = temp_dir.join(format!("ae_{}", file_stem));
            let glb = temp_stem.with_extension("glb");
            (temp_stem.to_string_lossy().to_string(), glb)
        };

        let output = std::process::Command::new(tool_path)
            .arg("-i")
            .arg(&path_str_clone)
            .arg("-o")
            .arg(&output_stem_str)
            .arg("-b")
            .arg("--pbr-metallic-roughness")
            .arg("--normalize-weights")
            .arg("1")
            .output();

        match output {
            Ok(out) if out.status.success() && output_path.exists() => {
                let _ = tx.send(Ok(output_path));
            }
            Ok(out) => {
                let stderr = String::from_utf8_lossy(&out.stderr);
                let _ = tx.send(Err(format!("FBX conversion failed: {}", stderr)));
            }
            Err(e) => {
                let _ = tx.send(Err(format!("Failed to execute FBX2glTF tool: {}", e)));
            }
        }
    });

    rx
}

/// Computes unit-normalization scale factor for newly imported models with miniature AABBs.
#[must_use]
pub fn compute_model_auto_scale(min: [f32; 3], max: [f32; 3]) -> f32 {
    let size_x = max[0] - min[0];
    let size_y = max[1] - min[1];
    let size_z = max[2] - min[2];
    let max_dim = size_x.max(size_y).max(size_z);
    if max_dim < 0.005 {
        1000.0
    } else if max_dim < 0.05 {
        100.0
    } else {
        1.0
    }
}

/// Returns `true` when `path_str` ends with `.fbx` (case-insensitive) for axis orientation correction.
#[must_use]
pub fn is_fbx_model_path(path_str: &str) -> bool {
    Path::new(path_str)
        .extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("fbx"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_model_auto_scale_thresholds() {
        assert_eq!(
            compute_model_auto_scale([0.0; 3], [0.001, 0.002, 0.001]),
            1000.0
        );
        assert_eq!(
            compute_model_auto_scale([0.0; 3], [0.02, 0.01, 0.01]),
            100.0
        );
        assert_eq!(compute_model_auto_scale([-1.0; 3], [1.0; 3]), 1.0);
    }

    #[test]
    fn test_is_fbx_model_path() {
        assert!(is_fbx_model_path("assets/models/character.fbx"));
        assert!(is_fbx_model_path("MODEL.FBX"));
        assert!(!is_fbx_model_path("assets/models/character.glb"));
    }

    #[test]
    fn test_validate_asset_drop_for_dimension() {
        assert!(
            validate_asset_drop_for_dimension(Path::new("../outside/evil.glb"), false).is_err()
        );
        assert!(
            validate_asset_drop_for_dimension(Path::new("assets/models/cube.glb"), true).is_err()
        );
        assert!(
            validate_asset_drop_for_dimension(Path::new("assets/models/cube.glb"), false).is_ok()
        );
        assert!(
            validate_asset_drop_for_dimension(Path::new("assets/textures/tile.png"), true).is_ok()
        );
    }
}