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
    // High-Performance Update: Only update entities that are currently visible to the camera
    // utilizing the VisibleEntities resource populated by the engine's culling system.
    if let Some(visible) = &visible_ents {
        for &entity in visible {
            if let Ok(mut pos) = _world.get::<&mut ae_plugin_api::Position>(entity) {
                if let Ok(vel) = _world.get::<&ae_plugin_api::Velocity>(entity) {
                    pos.x += vel.x * _dt;
                    pos.y += vel.y * _dt;
                    pos.z += vel.z * _dt;
                }
            }
        }
    } else {
        // Fallback Path: Update all entities in the world (if no visibility data exists yet)
        for (pos, vel) in
            _world.query_mut::<(&mut ae_plugin_api::Position, &ae_plugin_api::Velocity)>()
        {
            pos.x += vel.x * _dt;
            pos.y += vel.y * _dt;
            pos.z += vel.z * _dt;
        }
    }
    // ====================================
}