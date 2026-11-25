// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! External Asset Ingestion & Dependency Packaging.
//!
//! Copies external assets, directories, and glTF external buffer/texture URIs into
//! a project's asset directory so imported scenes remain self-contained.

use std::path::{Path, PathBuf};

/// Recursively copies a directory tree from `src` to `dst`.
fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let from = entry.path();
        let to = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_all(&from, &to)?;
        } else {
            std::fs::copy(&from, &to)?;
        }
    }
    Ok(())
}

/// Decodes URL percent-encoded sequences in external glTF URIs (e.g. `%20` -> space).
pub fn percent_decode(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut bytes = input.bytes();
    while let Some(b) = bytes.next() {
        if b == b'%' {
            let h1 = bytes.next().unwrap_or(b'0');
            let h2 = bytes.next().unwrap_or(b'0');
            let v1 = decode_hex_nibble(h1);
            let v2 = decode_hex_nibble(h2);
            if let (Some(n1), Some(n2)) = (v1, v2) {
                out.push(((n1 << 4) | n2) as char);
                continue;
            }
        }
        out.push(b as char);
    }
    out
}

#[inline]
fn decode_hex_nibble(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn copy_gltf_uri(uri: &str, parent_dir: &Path, dest_folder: &Path, label: &str) {
    if uri.starts_with("data:") {
        return;
    }
    let clean_uri = percent_decode(uri);
    let src_dep = parent_dir.join(&clean_uri);
    let dst_dep = dest_folder.join(&clean_uri);
    if let Some(p) = dst_dep.parent() {
        let _ = std::fs::create_dir_all(p);
    }
    if src_dep.exists() {
        let _ = std::fs::copy(&src_dep, &dst_dep);
        log::info!("📁 Copied glTF {label} {:?} -> {:?}", src_dep, dst_dep);
    }
}

/// Copies an external asset into the target project folder, resolving external dependencies
/// (`.bin` vertex/index buffers, textures, license files) for glTF models to guarantee self-containment.
pub fn import_asset_path(src: &Path, target_dir: &Path) -> PathBuf {
    if src.starts_with(target_dir) {
        return src.to_path_buf();
    }

    if src.is_dir() {
        let folder_name = src
            .file_name()
            .unwrap_or(std::ffi::OsStr::new("model_folder"));
        let dest_folder = target_dir.join(folder_name);
        if let Err(e) = copy_dir_all(src, &dest_folder) {
            log::warn!("Failed to copy dropped directory {:?}: {}", src, e);
            return src.to_path_buf();
        }
        log::info!("📁 Copied dropped directory {:?} -> {:?}", src, dest_folder);

        if dest_folder.join("scene.gltf").exists() {
            return dest_folder.join("scene.gltf");
        }
        if dest_folder.join("scene.glb").exists() {
            return dest_folder.join("scene.glb");
        }
        if let Ok(entries) = std::fs::read_dir(&dest_folder) {
            for entry in entries.flatten() {
                let p = entry.path();
                let ext = p
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("")
                    .to_ascii_lowercase();
                if matches!(ext.as_str(), "glb" | "gltf" | "obj" | "fbx") {
                    return p;
                }
            }
        }
        return dest_folder;
    }

    let ext = src
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();

    if ext == "gltf" {
        let file_name = src
            .file_name()
            .unwrap_or(std::ffi::OsStr::new("scene.gltf"));
        let is_generic_scene = file_name
            .to_string_lossy()
            .eq_ignore_ascii_case("scene.gltf");

        let dest_folder = if is_generic_scene {
            let folder_name = src
                .parent()
                .and_then(|p| p.file_name())
                .unwrap_or(std::ffi::OsStr::new("model"));
            target_dir.join(folder_name)
        } else {
            let stem = src.file_stem().unwrap_or(std::ffi::OsStr::new("model"));
            target_dir.join(stem)
        };

        if let Err(e) = std::fs::create_dir_all(&dest_folder) {
            log::warn!(
                "Failed to create destination directory {:?}: {}",
                dest_folder,
                e
            );
            return src.to_path_buf();
        }

        let dest_gltf = dest_folder.join(file_name);
        if let Err(e) = std::fs::copy(src, &dest_gltf) {
            log::warn!("Failed to copy gltf file {:?}: {}", src, e);
            return src.to_path_buf();
        }
        log::info!("📁 Copied glTF file {:?} -> {:?}", src, dest_gltf);

        let parent_dir = src.parent().unwrap_or(Path::new(""));
        if let Some(uris) = ae_mesh::collect_gltf_external_uris(src) {
            for uri in &uris.buffers {
                copy_gltf_uri(uri, parent_dir, &dest_folder, "buffer");
            }
            for uri in &uris.images {
                copy_gltf_uri(uri, parent_dir, &dest_folder, "texture");
            }
        }

        let bin_candidate = src.with_extension("bin");
        if bin_candidate.exists()
            && let Some(bin_name) = bin_candidate.file_name()
        {
            let dst_bin = dest_folder.join(bin_name);
            if !dst_bin.exists() {
                let _ = std::fs::copy(&bin_candidate, &dst_bin);
            }
        }

        let lic_candidate = parent_dir.join("license.txt");
        if lic_candidate.exists() {
            let _ = std::fs::copy(&lic_candidate, dest_folder.join("license.txt"));
        }

        return dest_gltf;
    }

    if let Some(file_name) = src.file_name() {
        let dest = target_dir.join(file_name);
        match std::fs::copy(src, &dest) {
            Ok(_) => {
                log::info!("📁 Copied dropped asset {:?} -> {:?}", src, dest);
                dest
            }
            Err(err) => {
                log::warn!("Could not copy asset to {:?}: {}", dest, err);
                src.to_path_buf()
            }
        }
    } else {
        src.to_path_buf()
    }
}

/// Ensures `target_dir` exists and copies any external files in `paths` into it,
/// returning `(resolved_paths, copied_count)`.
#[must_use]
pub fn import_dropped_paths(paths: &[PathBuf], target_dir: &Path) -> (Vec<PathBuf>, usize) {
    if !target_dir.exists() {
        let _ = std::fs::create_dir_all(target_dir);
    }

    let mut imported_paths = Vec::with_capacity(paths.len());
    let mut imported_count = 0;

    for path in paths {
        let import_path = import_asset_path(path, target_dir);
        if import_path != *path {
            imported_count += 1;
        }
        imported_paths.push(import_path);
    }

    (imported_paths, imported_count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_percent_decode() {
        assert_eq!(percent_decode("hello%20world.bin"), "hello world.bin");
        assert_eq!(percent_decode("scene.bin"), "scene.bin");
        assert_eq!(percent_decode("path/to/%41%42%43.png"), "path/to/ABC.png");
        assert_eq!(percent_decode("special%2Bchar.bin"), "special+char.bin");
    }

    #[test]
    fn test_import_asset_path_gltf_with_bin_dependencies() {
        let temp_dir = std::env::temp_dir().join(format!("ae_test_gltf_{}", std::process::id()));
        let src_model_dir = temp_dir.join("source_model");
        let target_dir = temp_dir.join("project_assets");

        let _ = std::fs::create_dir_all(&src_model_dir);
        let _ = std::fs::create_dir_all(&target_dir);

        let src_gltf = src_model_dir.join("scene.gltf");
        let src_bin = src_model_dir.join("scene.bin");
        let src_lic = src_model_dir.join("license.txt");

        let gltf_content = r#"{
            "asset": { "version": "2.0" },
            "buffers": [ { "byteLength": 16, "uri": "scene.bin" } ]
        }"#;
        std::fs::write(&src_gltf, gltf_content).expect("Failed to write test gltf");
        std::fs::write(&src_bin, b"0123456789abcdef").expect("Failed to write test bin");
        std::fs::write(&src_lic, "MIT License").expect("Failed to write test license");

        let imported = import_asset_path(&src_gltf, &target_dir);

        assert!(imported.ends_with("source_model/scene.gltf"));
        assert!(imported.exists());

        let imported_parent = imported.parent().expect("imported parent");
        assert!(imported_parent.join("scene.bin").exists());
        assert_eq!(
            std::fs::read(imported_parent.join("scene.bin")).expect("read scene.bin"),
            b"0123456789abcdef"
        );
        assert!(imported_parent.join("license.txt").exists());

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}