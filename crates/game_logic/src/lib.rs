// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
/// Aeon Engine — Example Game Logic Plugin
/// This is a minimal example of a hot-reloadable gameplay plugin.
/// The plugin is compiled as a `cdylib` and dynamically loaded by the engine.
/// # Rules
/// - Plugin must be STATELESS — all state lives in the ECS World
/// - No GPU resources (wgpu device) inside the plugin
/// - Export functions with `#[no_mangle]` and `extern "C"`
/// # Safety
/// This plugin contains ZERO `unsafe` blocks. The FFI boundary uses safe
/// reference wrappers (`&mut PluginContextFFI`), and all world access is safe.
use ae_plugin_api::PluginContextFFI;

/// Returns the compile-time ABI hash that this plugin was built with.
/// The host engine queries this symbol upon library load to verify
/// that the compiled configurations (version and profile) match exactly,
/// preventing memory corruption due to Rust ABI mismatch.
#[unsafe(no_mangle)]
pub extern "C" fn plugin_abi_hash() -> *const std::ffi::c_char {
    if cfg!(debug_assertions) {
        "ae-abi-v0.7.0-debug\0".as_ptr() as *const std::ffi::c_char
    } else {
        "ae-abi-v0.7.0-release\0".as_ptr() as *const std::ffi::c_char
    }
}

/// Main update function called by the engine every frame.
/// This function is resolved by name ("plugin_update") via libloading.
/// Modify this function and rebuild (`cargo build -p game_logic`) to see
/// hot reload in action — no engine restart needed!
#[unsafe(no_mangle)]
pub extern "C" fn plugin_update(ctx: &mut PluginContextFFI<'_>) {
    let _dt = ctx.delta_time;

    // Get visible entities from resources if available (cloned to release the borrow on ctx)
    let visible_ents = ctx
        .get_resources()
        .and_then(|r| r.get::<ae_plugin_api::VisibleEntities>())
        .map(|v| v.entities.clone());

    // Get safe mutable reference to the World via the API helper
    let _world = match ctx.get_world() {
        Some(w) => w,
        None => return,
    };

    // ===== GAMEPLAY CODE GOES HERE =====
    // Note: Standard linear velocity integration (pos += vel * dt) is performed
    // by EcsManager::update in the engine core. This plugin implements custom,
    // hot-reloadable gameplay logic (e.g., custom rotation or gameplay behavior).
    if let Some(visible) = &visible_ents {
        for &entity in visible {
            if let Ok(mut rot) = _world.get::<&mut ae_plugin_api::Rotation>(entity) {
                if _world.get::<&ae_plugin_api::PlayerTag>(entity).is_ok() {
                    // Example hot-reloadable gameplay logic: smooth Y-axis spin for player entities
                    let half_angle = 0.5 * _dt;
                    let (sin, cos) = half_angle.sin_cos();
                    let (w, y) = (rot.w, rot.y);
                    rot.w = w * cos - y * sin;
                    rot.y = y * cos + w * sin;
                }
            }
        }
    }
    // ====================================
}