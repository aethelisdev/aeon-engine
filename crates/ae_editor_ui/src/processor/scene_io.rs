// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

use super::UiContext;

/// Handles triggering model import file dialog.
pub fn handle_open_model_dialog(ctx: &mut UiContext) {
    let (tx, rx) = std::sync::mpsc::channel();
    ctx.dialog_receivers.push(rx);
    std::thread::spawn(move || {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter(
                "3D Models & Textures",
                &["gltf", "glb", "obj", "fbx", "png", "jpg", "jpeg"],
            )
            .pick_file()
        {
            let _ = tx.send(path);
        }
    });
}

/// Helper to recursively copy directories when importing folder trees.
fn copy_dir_all(src: &std::path::Path, dst: &std::path::Path) -> std::io::Result<()> {
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

/// Helper to decode URL percent-encoded characters in glTF URIs (e.g. `%20` -> space).
fn percent_decode(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut bytes = input.bytes().peekable();
    while let Some(b) = bytes.next() {
        if b == b'%' {
            let h1 = bytes.next().unwrap_or(b'0');
            let h2 = bytes.next().unwrap_or(b'0');
            let v1 = match h1 {
                b'0'..=b'9' => Some(h1 - b'0'),
                b'a'..=b'f' => Some(h1 - b'a' + 10),
                b'A'..=b'F' => Some(h1 - b'A' + 10),
                _ => None,
            };
            let v2 = match h2 {
                b'0'..=b'9' => Some(h2 - b'0'),
                b'a'..=b'f' => Some(h2 - b'a' + 10),
                b'A'..=b'F' => Some(h2 - b'A' + 10),
                _ => None,
            };
            if let (Some(n1), Some(n2)) = (v1, v2) {
                out.push((n1 << 4 | n2) as char);
                continue;
            }
        }
        out.push(b as char);
    }
    out
}

/// Copies an external asset into the target project folder, resolving external dependencies
/// (`.bin` vertex/index buffers, textures, materials) for glTF models to guarantee self-containment.
fn import_asset_path(src: &std::path::Path, target_dir: &std::path::Path) -> std::path::PathBuf {
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

        let parent_dir = src.parent().unwrap_or(std::path::Path::new(""));
        if let Ok(gltf) = gltf::Gltf::open(src) {
            for buffer in gltf.buffers() {
                if let gltf::buffer::Source::Uri(uri) = buffer.source()
                    && !uri.starts_with("data:")
                {
                    let clean_uri = percent_decode(uri);
                    let src_buf = parent_dir.join(&clean_uri);
                    let dst_buf = dest_folder.join(&clean_uri);
                    if let Some(p) = dst_buf.parent() {
                        let _ = std::fs::create_dir_all(p);
                    }
                    if src_buf.exists() {
                        let _ = std::fs::copy(&src_buf, &dst_buf);
                        log::info!("📁 Copied glTF buffer {:?} -> {:?}", src_buf, dst_buf);
                    }
                }
            }
            for image in gltf.images() {
                if let gltf::image::Source::Uri { uri, .. } = image.source()
                    && !uri.starts_with("data:")
                {
                    let clean_uri = percent_decode(uri);
                    let src_img = parent_dir.join(&clean_uri);
                    let dst_img = dest_folder.join(&clean_uri);
                    if let Some(p) = dst_img.parent() {
                        let _ = std::fs::create_dir_all(p);
                    }
                    if src_img.exists() {
                        let _ = std::fs::copy(&src_img, &dst_img);
                        log::info!("📁 Copied glTF texture {:?} -> {:?}", src_img, dst_img);
                    }
                }
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

/// Handles ingesting dropped external files into the asset pipeline.
///
/// Follows industry-standard DCC / game engine conventions:
/// - If dropped onto the 3D Viewport: copies the asset (and glTF external dependencies)
///   to the project directory, computes ground plane raycast intersection from the cursor,
///   and spawns the entity directly at the pointed world position.
/// - If dropped onto the Asset Browser: imports the asset into the active folder
///   and refreshes the content browser without dirtying the 3D scene.
pub fn handle_import_dropped_files(
    ctx: &mut UiContext,
    paths: Vec<std::path::PathBuf>,
    screen_pos: Option<[f32; 2]>,
) {
    let viewport_rect = ctx.ui.last_viewport_rect;
    let is_over_viewport = screen_pos.is_some_and(|pos| {
        viewport_rect.contains_point(irisui::prelude::Point::new(pos[0], pos[1]))
    });

    let current_folder = ctx.ui.asset_browser.current_folder.clone();
    let target_dir = if is_over_viewport && current_folder == std::path::Path::new("assets") {
        current_folder.join("models")
    } else {
        current_folder
    };

    if !target_dir.exists() {
        let _ = std::fs::create_dir_all(&target_dir);
    }

    let mut imported_paths = Vec::with_capacity(paths.len());
    let mut imported_count = 0;

    for path in paths {
        let before_path = path.clone();
        let import_path = import_asset_path(&path, &target_dir);
        if import_path != before_path {
            imported_count += 1;
        }
        imported_paths.push(import_path);
    }

    if imported_count > 0 {
        ctx.ui.asset_browser.cached_items.clear();
        ctx.ui
            .iris_overlay
            .notifier
            .tag_redraw(crate::ui::panel_layout::PanelId::Assets);
        ctx.ui.iris_overlay.chrome.needs_layout_rebuild = true;
    }

    if is_over_viewport {
        // Industry Standard Viewport Drop: raycast to ground plane at mouse cursor
        let world_pos = if let Some(pos) = screen_pos {
            crate::assets::drag_drop::compute_ground_intersection(pos, viewport_rect, ctx.camera)
                .unwrap_or([0.0, 0.0, 0.0])
        } else {
            [0.0, 0.0, 0.0]
        };

        for path in imported_paths {
            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_ascii_lowercase();

            match ext.as_str() {
                "gltf" | "glb" => {
                    super::spawning::handle_spawn_model_path_at(ctx, path.clone(), world_pos);
                    ctx.ui.set_status_message(
                        format!(
                            "Spawned 3D model at ({:.1}, {:.1}, {:.1})",
                            world_pos[0], world_pos[1], world_pos[2]
                        ),
                        irisui::prelude::Color::rgb(0.0, 0.898, 1.0),
                    );
                }
                "png" | "jpg" | "jpeg" => {
                    super::spawning::handle_spawn_sprite_path_at(ctx, path.clone(), world_pos);
                    ctx.ui.set_status_message(
                        format!(
                            "Spawned sprite at ({:.1}, {:.1}, {:.1})",
                            world_pos[0], world_pos[1], world_pos[2]
                        ),
                        irisui::prelude::Color::rgb(0.0, 0.898, 1.0),
                    );
                }
                _ => {
                    // Non-immediate formats (FBX, scenes, etc.) route to engine async importer
                    let (tx, rx) = std::sync::mpsc::channel();
                    let _ = tx.send(path);
                    ctx.dialog_receivers.push(rx);
                }
            }
        }
    } else {
        // Dropped onto Asset Browser or other panels: import to library only
        ctx.ui.set_status_message(
            format!(
                "Imported {} file(s) into project assets",
                imported_paths.len()
            ),
            irisui::prelude::Color::rgb(0.0, 0.898, 1.0),
        );
    }
}

/// Handles triggering prefab load file dialog.
pub fn handle_open_load_prefab_dialog(ctx: &mut UiContext) {
    let (tx, rx) = std::sync::mpsc::channel();
    ctx.dialog_receivers.push(rx);
    std::thread::spawn(move || {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Aeon Prefab (*.prefab, *.json)", &["prefab", "json"])
            .pick_file()
        {
            let _ = tx.send(path);
        }
    });
}

/// Handles triggering save scene file dialog.
pub fn handle_open_save_scene_dialog(ctx: &mut UiContext) {
    let (tx, rx) = std::sync::mpsc::channel();
    ctx.ui.scene_dialog_receivers.push(rx);
    let is_2d = ctx.editor.is_2d;
    std::thread::spawn(move || {
        let (filter_name, extensions, default_name) = if is_2d {
            ("Aeon 2D Scene (*.ae2d)", &["ae2d", "aee"][..], "scene.ae2d")
        } else {
            ("Aeon 3D Scene (*.ae3d)", &["ae3d", "aee"][..], "scene.ae3d")
        };
        if let Some(path) = rfd::FileDialog::new()
            .add_filter(filter_name, extensions)
            .set_file_name(default_name)
            .save_file()
        {
            let _ = tx.send(crate::ui::SceneDialogAction::SaveTo(path));
        }
    });
}

/// Handles triggering load scene file dialog.
pub fn handle_open_load_scene_dialog(ctx: &mut UiContext) {
    let (tx, rx) = std::sync::mpsc::channel();
    ctx.ui.scene_dialog_receivers.push(rx);
    let is_2d = ctx.editor.is_2d;
    std::thread::spawn(move || {
        let (filter_name, extensions) = if is_2d {
            ("Aeon 2D Scene (*.ae2d)", &["ae2d", "aee"][..])
        } else {
            ("Aeon 3D Scene (*.ae3d)", &["ae3d", "aee"][..])
        };
        if let Some(path) = rfd::FileDialog::new()
            .add_filter(filter_name, extensions)
            .pick_file()
        {
            let _ = tx.send(crate::ui::SceneDialogAction::LoadFrom(path));
        }
    });
}

/// Handles saving scene via file dialog.
pub fn handle_save_scene(ctx: &mut UiContext) {
    handle_open_save_scene_dialog(ctx);
}

/// Handles loading scene via file dialog.
pub fn handle_load_scene(ctx: &mut UiContext) {
    handle_open_load_scene_dialog(ctx);
}

/// Handles saving scene directly to specified path.
pub fn handle_save_scene_to_path(ctx: &mut UiContext, path: std::path::PathBuf) {
    ctx.ui.active_scene_path = path.to_string_lossy().to_string();
    ctx.ui.pending_save_path = Some(path.clone());
    ctx.ui.should_save_scene = true;
    ctx.editor.active_scene_path = Some(path.clone());
    log::info!("💾 Scene save request queued for path {:?}", path);
}

/// Handles loading scene directly from specified path.
pub fn handle_load_scene_from_path(ctx: &mut UiContext, path: std::path::PathBuf) {
    ctx.ui.active_scene_path = path.to_string_lossy().to_string();
    ctx.ui.pending_load_path = Some(path.clone());
    ctx.ui.should_load_scene = true;
    ctx.ui.is_loading_assets = true;
    ctx.editor.active_scene_path = Some(path.clone());
    log::info!("📂 Scene load request queued for path {:?}", path);
}

/// Handles saving an entity as a prefab asset to specified path.
pub fn handle_save_entity_as_prefab(
    ctx: &mut UiContext,
    entity: hecs::Entity,
    path: std::path::PathBuf,
) {
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
        && let Err(e) = std::fs::create_dir_all(parent)
    {
        log::error!("Failed to create prefab directory {:?}: {}", parent, e);
        ctx.ui.set_status_message(
            format!("Failed to create directory: {}", e),
            irisui::prelude::Color::rgba(0.95, 0.35, 0.35, 1.0),
        );
        return;
    }
    let prefab = ae_editor::prefab::Prefab::create_from_entity(ctx.world, entity);
    if let Err(e) = prefab.save_to_file(&path) {
        log::error!("Failed to save prefab to {:?}: {}", path, e);
        ctx.ui.set_status_message(
            format!("Failed to save prefab: {}", e),
            irisui::prelude::Color::rgba(0.95, 0.35, 0.35, 1.0),
        );
    } else {
        log::info!("📦 Prefab successfully saved to {:?}", path);
        ctx.ui.set_status_message(
            format!("Saved prefab to {}", path.display()),
            irisui::prelude::Color::rgba(0.25, 0.85, 0.45, 1.0),
        );
    }
}

/// Handles instantiating a prefab asset from specified path into the ECS world.
pub fn handle_instantiate_prefab(ctx: &mut UiContext, path: std::path::PathBuf) {
    match ae_editor::prefab::Prefab::load_from_file(&path) {
        Ok(prefab) => {
            let new_entity = prefab.instantiate(ctx.world, None);
            ctx.editor.selected_entities.clear();
            ctx.editor.selected_entities_set.clear();
            ctx.editor.selected_entities.push(new_entity);
            ctx.editor.selected_entities_set.insert(new_entity);
            ctx.ui.selected_entity = Some(new_entity);
            log::info!("✨ Instantiated prefab from {:?}", path);
        }
        Err(e) => {
            log::error!("Failed to instantiate prefab from {:?}: {}", path, e);
        }
    }
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

        // Write minimal dummy gltf JSON referencing scene.bin
        let gltf_content = r#"{
            "asset": { "version": "2.0" },
            "buffers": [ { "byteLength": 16, "uri": "scene.bin" } ]
        }"#;
        std::fs::write(&src_gltf, gltf_content).expect("Failed to write test gltf");
        std::fs::write(&src_bin, b"0123456789abcdef").expect("Failed to write test bin");
        std::fs::write(&src_lic, "MIT License").expect("Failed to write test license");

        let imported = import_asset_path(&src_gltf, &target_dir);

        // Should be placed inside subfolder named after the source model dir ("source_model")
        assert!(imported.ends_with("source_model/scene.gltf"));
        assert!(imported.exists());

        let imported_parent = imported.parent().unwrap();
        assert!(imported_parent.join("scene.bin").exists());
        assert_eq!(
            std::fs::read(imported_parent.join("scene.bin")).unwrap(),
            b"0123456789abcdef"
        );
        assert!(imported_parent.join("license.txt").exists());

        // Cleanup
        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}