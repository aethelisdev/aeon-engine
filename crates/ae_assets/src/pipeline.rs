// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Asynchronous Asset Parsing, FBX Conversion & Import Validation Pipeline.
//!
//! Provides typed import format classification ([`ImportAssetKind`]), typed background
//! import completion payloads ([`AsyncImportResult`]), `rayon` glTF/`FBX2glTF` workers,
//! and path/dimension compatibility validation decoupled from UI or runtime state.

use super::scanner::is_world_file_3d;
use ae_mesh::{ParsedModelData, is_safe_path, parse_gltf_file};
use std::path::{Path, PathBuf};
use std::sync::mpsc::Receiver;

/// Authoritative file format categories supported by the unified asset import pipeline.
///
/// Replaces extension trait-object registries and string sentinel heuristics with explicit
/// compile-time variant matching.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ImportAssetKind {
    /// Raster sprite or PBR texture image (`png`, `jpg`, `jpeg`, `tga`, `bmp`).
    Texture,
    /// Binary or text glTF 2.0 3D model (`gltf`, `glb`).
    GltfModel,
    /// Autodesk FBX model converted asynchronously via `FBX2glTF` (`fbx`).
    FbxModel,
    /// WGSL runtime shader source (`wgsl`).
    WgslShader,
    /// Hierarchical `.aeprefab` entity template archive (`aeprefab`).
    Prefab,
    /// Partitioned or binary `.aeworld` scene archive (`aeworld`).
    WorldScene,
}

impl ImportAssetKind {
    /// Classifies `path` using its case-insensitive extension without heap allocation.
    #[inline]
    #[must_use]
    pub fn from_path(path: &Path) -> Option<Self> {
        path.extension()
            .and_then(|e| e.to_str())
            .and_then(Self::from_extension)
    }

    /// Classifies a bare file extension string (case-insensitive).
    #[inline]
    #[must_use]
    pub fn from_extension(ext: &str) -> Option<Self> {
        if ext.eq_ignore_ascii_case("png")
            || ext.eq_ignore_ascii_case("jpg")
            || ext.eq_ignore_ascii_case("jpeg")
            || ext.eq_ignore_ascii_case("tga")
            || ext.eq_ignore_ascii_case("bmp")
        {
            Some(Self::Texture)
        } else if ext.eq_ignore_ascii_case("gltf") || ext.eq_ignore_ascii_case("glb") {
            Some(Self::GltfModel)
        } else if ext.eq_ignore_ascii_case("fbx") {
            Some(Self::FbxModel)
        } else if ext.eq_ignore_ascii_case("wgsl") {
            Some(Self::WgslShader)
        } else if ext.eq_ignore_ascii_case("aeprefab") {
            Some(Self::Prefab)
        } else if ext.eq_ignore_ascii_case("aeworld") {
            Some(Self::WorldScene)
        } else {
            None
        }
    }
}

/// Typed completion event emitted by asynchronous dialog pickers and `rayon` import workers.
pub enum AsyncImportResult {
    /// Native OS file picker returned a selected path and target world spawn coordinate.
    PickedPath {
        /// Selected source path on disk.
        path: PathBuf,
        /// Target world-space spawn position (`[0.0; 3]` for dialogs).
        spawn_pos: [f32; 3],
    },
    /// Background `rayon` worker parsed CPU geometry/textures ready for GPU upload.
    ModelParsed {
        /// Decoded CPU geometry, skeletons, and embedded texture data.
        data: Box<ParsedModelData>,
        /// Target world-space spawn coordinates.
        spawn_pos: [f32; 3],
        /// Whether standard `-90 deg X` axis correction applies (e.g. converted FBX assets).
        apply_fbx_axis_fix: bool,
    },
    /// Asynchronous dialog, conversion, or parsing failure message.
    Error(String),
}

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
        return Err("Cannot load 2D world in 3D mode! Switch to 3D mode.");
    }

    Ok(())
}

/// Spawns a background `rayon` task to parse a `.gltf`/`.glb` file into [`AsyncImportResult::ModelParsed`].
#[must_use]
pub fn spawn_gltf_import(
    path: &Path,
    final_name: String,
    spawn_pos: [f32; 3],
) -> Receiver<AsyncImportResult> {
    let path_str = path.to_string_lossy().into_owned();
    let (tx, rx) = std::sync::mpsc::channel();
    rayon::spawn(move || {
        let msg = match parse_gltf_file(&path_str, final_name) {
            Ok(data) => AsyncImportResult::ModelParsed {
                data: Box::new(data),
                spawn_pos,
                apply_fbx_axis_fix: false,
            },
            Err(e) => AsyncImportResult::Error(e),
        };
        let _ = tx.send(msg);
    });
    rx
}

/// Spawns a background `rayon` task converting `.fbx` to `.glb` and parsing CPU geometry off the main thread.
#[must_use]
pub fn spawn_fbx_import(
    path: &Path,
    final_name: String,
    spawn_pos: [f32; 3],
) -> Receiver<AsyncImportResult> {
    let source_path = path.to_path_buf();
    let (tx, rx) = std::sync::mpsc::channel();

    rayon::spawn(move || {
        let msg = match convert_fbx_to_glb(&source_path) {
            Ok(glb_path) => {
                let glb_str = glb_path.to_string_lossy();
                match parse_gltf_file(&glb_str, final_name) {
                    Ok(data) => AsyncImportResult::ModelParsed {
                        data: Box::new(data),
                        spawn_pos,
                        apply_fbx_axis_fix: true,
                    },
                    Err(e) => AsyncImportResult::Error(format!(
                        "Converted GLB parse failed ({}): {e}",
                        glb_path.display()
                    )),
                }
            }
            Err(e) => AsyncImportResult::Error(e),
        };
        let _ = tx.send(msg);
    });

    rx
}

fn convert_fbx_to_glb(path: &Path) -> Result<PathBuf, String> {
    let tool_path = resolve_fbx2gltf_tool_path();
    if !Path::new(tool_path).exists() {
        return Err(format!(
            "FBX2glTF tool not found at '{tool_path}'. Please make sure the tools/ folder is intact."
        ));
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

    let file_stem = path
        .file_stem()
        .unwrap_or(std::ffi::OsStr::new("model"))
        .to_string_lossy();

    let parent_writable = path.parent().is_some_and(|dir| {
        let probe = dir.join(format!(".ae_probe_{}", std::process::id()));
        if std::fs::write(&probe, b"").is_ok() {
            let _ = std::fs::remove_file(&probe);
            true
        } else {
            false
        }
    });

    let (output_stem_str, output_path) = if parent_writable {
        let stem = path.with_extension("");
        let glb = path.with_extension("glb");
        (stem.to_string_lossy().into_owned(), glb)
    } else {
        let temp_dir = std::env::temp_dir();
        let temp_stem = temp_dir.join(format!("ae_{file_stem}"));
        let glb = temp_stem.with_extension("glb");
        (temp_stem.to_string_lossy().into_owned(), glb)
    };

    let path_str = path.to_string_lossy();
    let output = std::process::Command::new(tool_path)
        .arg("-i")
        .arg(path_str.as_ref())
        .arg("-o")
        .arg(&output_stem_str)
        .arg("-b")
        .arg("--pbr-metallic-roughness")
        .arg("--normalize-weights")
        .arg("1")
        .output()
        .map_err(|e| format!("Failed to execute FBX2glTF tool: {e}"))?;

    if output.status.success() && output_path.exists() {
        Ok(output_path)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("FBX conversion failed: {stderr}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn test_import_asset_kind_classifies_extensions_and_rejects_unknown() {
        assert_eq!(
            ImportAssetKind::from_path(Path::new("tex/Brick.PNG")),
            Some(ImportAssetKind::Texture)
        );
        assert_eq!(
            ImportAssetKind::from_path(Path::new("mesh/Hero.gLb")),
            Some(ImportAssetKind::GltfModel)
        );
        assert_eq!(
            ImportAssetKind::from_path(Path::new("mesh/Rig.FBX")),
            Some(ImportAssetKind::FbxModel)
        );
        assert_eq!(
            ImportAssetKind::from_path(Path::new("shaders/custom.wgsl")),
            Some(ImportAssetKind::WgslShader)
        );
        assert_eq!(
            ImportAssetKind::from_path(Path::new("prefabs/Door.aeprefab")),
            Some(ImportAssetKind::Prefab)
        );
        assert_eq!(
            ImportAssetKind::from_path(Path::new("worlds/Main.aeworld")),
            Some(ImportAssetKind::WorldScene)
        );
        assert_eq!(ImportAssetKind::from_path(Path::new("README.txt")), None);
        assert_eq!(ImportAssetKind::from_path(Path::new("")), None);
    }
}