// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
use ae_plugin_api::{
    DynamicEventBus, PluginContext, PluginContextFFI, Resources, ScriptingBackend,
};
/// AE Plugin Host — Dynamic Plugin Loader with Hot Reload
/// This crate implements the native plugin backend that loads `.dll`/`.so`/`.dylib`
/// files at runtime using `libloading`. It supports hot reload by watching file
/// timestamps and performing safe unload/load cycles with versioned filenames.
/// # Safety Boundary
/// This crate contains the ONLY `unsafe` blocks in the entire Aeon Engine codebase.
/// They are isolated in two functions: `load_library()` and `call_plugin_update()`.
/// All unsafe is required for FFI interop via `libloading` and is well-documented.
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Native plugin backend that loads compiled Rust cdylib plugins.
/// Implements the `ScriptingBackend` trait for hot-reloadable native code.
/// Uses versioned file copies to avoid OS library caching issues.
pub struct NativePluginBackend {
    /// Path to the source compiled library (e.g., `target/debug/game_logic.dll`).
    source_path: PathBuf,
    /// Directory where versioned copies are stored for loading.
    staging_dir: PathBuf,
    /// The currently loaded library handle (managed by libloading).
    library: Option<libloading::Library>,
    /// Monotonically increasing version counter for unique filenames.
    version: u64,
    /// Last known modification timestamp of the source file.
    last_modified: Option<SystemTime>,
}

impl NativePluginBackend {
    /// Creates a new NativePluginBackend for the given source library path.
    /// # Arguments
    /// * `source_path` - Path to the compiled cdylib (e.g., `target/debug/game_logic.dll`)
    /// * `staging_dir` - Directory for versioned copies (e.g., `target/plugins/`)
    pub fn new(source_path: PathBuf, staging_dir: PathBuf) -> Self {
        Self {
            source_path,
            staging_dir,
            library: None,
            version: 0,
            last_modified: None,
        }
    }

    /// Returns the versioned filename for the current version counter.
    /// Example: `game_logic_v3.dll`
    fn versioned_filename(&self) -> String {
        let ext = ae_plugin_api::platform_lib_extension();
        let stem = self
            .source_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("plugin");
        format!("{}_v{}.{}", stem, self.version, ext)
    }

    /// Copies the source library to a versioned staging path.
    /// This prevents OS-level DLL caching issues where the old library
    /// cannot be unloaded because the file is still locked.
    fn copy_to_staging(&self) -> Result<PathBuf, String> {
        std::fs::create_dir_all(&self.staging_dir)
            .map_err(|e| format!("Failed to create staging dir: {}", e))?;

        let dest = self.staging_dir.join(self.versioned_filename());
        std::fs::copy(&self.source_path, &dest)
            .map_err(|e| format!("Failed to copy plugin to staging: {}", e))?;

        log::info!(
            "[PluginHost] Copied {} → {}",
            self.source_path.display(),
            dest.display()
        );
        Ok(dest)
    }

    /// Returns the last modification time of the source library file.
    fn get_source_modified(&self) -> Option<SystemTime> {
        std::fs::metadata(&self.source_path)
            .ok()
            .and_then(|m| m.modified().ok())
    }

    /// Loads a library from the given path.
    /// # Safety Boundary (1 of 2)
    /// Contains `unsafe` block for `libloading::Library::new()` which calls
    /// the OS-level `LoadLibraryW` (Windows) / `dlopen` (Unix).
    /// This is safe because the caller guarantees `lib_path` points to a valid
    /// cdylib compiled from the same Rust toolchain with matching ABI.
    fn load_library(&mut self, lib_path: &Path) -> Result<(), String> {
        // SAFETY: libloading::Library::new loads a shared library from disk.
        // The path points to a versioned staging copy that we just created.
        // The library is compiled from ae_plugin_api ensuring ABI compatibility.
        let lib = unsafe {
            libloading::Library::new(lib_path)
                .map_err(|e| format!("Failed to load library '{}': {}", lib_path.display(), e))?
        };

        // SAFETY: lib.get resolves the 'plugin_abi_hash' symbol from the loaded plugin library.
        // The symbol returns a pointer to a null-terminated static string containing the compile-time ABI hash.
        let abi_hash_fn = unsafe {
            lib.get::<unsafe extern "C" fn() -> *const std::ffi::c_char>(b"plugin_abi_hash\0")
                .map_err(|e| format!("Failed to find 'plugin_abi_hash' symbol: {}", e))?
        };

        // SAFETY: We invoke the looked-up function pointer to retrieve the static raw pointer.
        // The pointer is guaranteed valid because the static string literal has 'static lifetime.
        let raw_hash = unsafe { abi_hash_fn() };
        if raw_hash.is_null() {
            return Err("Plugin ABI hash is null!".to_string());
        }

        // SAFETY: Standard conversion of raw C string pointer to a safe Rust &str.
        // The pointer is non-null and points to a valid null-terminated ASCII string.
        let plugin_hash_str = unsafe { std::ffi::CStr::from_ptr(raw_hash) }
            .to_str()
            .map_err(|e| format!("Failed to parse Plugin ABI hash as UTF-8: {}", e))?;

        let engine_hash = ae_plugin_api::ENGINE_ABI_HASH;

        if plugin_hash_str != engine_hash {
            return Err(format!(
                "ABI MISMATCH: Host Engine is '{}' but Plugin is '{}'. Rebuild the plugin with the correct profile.",
                engine_hash, plugin_hash_str
            ));
        }

        self.library = Some(lib);
        self.last_modified = self.get_source_modified();

        log::info!(
            "[PluginHost] Successfully loaded plugin from: {} (ABI: {})",
            lib_path.display(),
            plugin_hash_str
        );
        Ok(())
    }

    /// Calls the plugin_update function from the loaded library.
    /// # Safety Boundary (2 of 2)
    /// Contains `unsafe` blocks for:
    /// - `libloading::Library::get()` — resolves the "plugin_update" symbol
    /// - Calling the resolved FFI function pointer
    /// Both are safe because the plugin is compiled with the same `ae_plugin_api`
    /// crate ensuring type-level ABI compatibility.
    fn call_plugin_update(&self, ctx_ffi: &mut PluginContextFFI<'_>) -> Result<(), String> {
        let lib = self
            .library
            .as_ref()
            .ok_or_else(|| "No library loaded".to_string())?;

        // SAFETY: Library::get resolves a symbol by name from the loaded library.
        // The type parameter PluginUpdateFn ensures the function pointer type matches
        // the exported function in the plugin (both use ae_plugin_api types).
        let func = unsafe {
            lib.get::<ae_plugin_api::PluginUpdateFn>(b"plugin_update\0")
                .map_err(|e| format!("Failed to find 'plugin_update' symbol: {}", e))?
        };

        // SAFETY: We call the plugin function with a valid, non-null mutable reference.
        // The PluginContextFFI is stack-allocated and lives for the duration of this call.
        unsafe {
            func(ctx_ffi);
        }

        Ok(())
    }
}

impl ScriptingBackend for NativePluginBackend {
    fn name(&self) -> &str {
        "Native (cdylib)"
    }

    fn load(&mut self, path: &Path) -> Result<(), String> {
        self.source_path = path.to_path_buf();
        self.version += 1;

        let staged_path = self.copy_to_staging()?;
        self.load_library(&staged_path)
    }

    fn unload(&mut self) -> Result<(), String> {
        // Drop library handle (calls FreeLibrary/dlclose internally).
        self.library = None;
        log::info!("[PluginHost] Plugin unloaded.");
        Ok(())
    }

    fn call_update(&self, ctx: &mut PluginContext) -> Result<(), String> {
        // Convert safe PluginContext to FFI-compatible PluginContextFFI
        let mut ctx_ffi = PluginContextFFI {
            world: Some(ctx.world),
            resources: Some(ctx.resources),
            event_bus: Some(ctx.event_bus),
            delta_time: ctx.delta_time,
        };

        self.call_plugin_update(&mut ctx_ffi)
    }

    fn needs_reload(&self) -> bool {
        let current = self.get_source_modified();
        match (&self.last_modified, &current) {
            (Some(old), Some(new)) => new > old,
            (None, Some(_)) => true,
            _ => false,
        }
    }

    fn reload(&mut self) -> Result<(), String> {
        log::info!(
            "[PluginHost] Hot reload triggered for: {}",
            self.source_path.display()
        );

        // 1. Unload old library (drops handle, OS releases file lock)
        self.unload()?;

        // 2. Increment version for unique staging filename
        self.version += 1;

        // 3. Copy source to versioned staging path
        let staged_path = self.copy_to_staging()?;

        // 4. Load new library and validate symbol
        self.load_library(&staged_path)?;

        log::info!("[PluginHost] Hot reload complete (v{}).", self.version);
        Ok(())
    }
}

/// Manages multiple scripting backends and orchestrates plugin updates.
/// The PluginManager is the main entry point for the engine to interact
/// with the plugin system. It holds a list of backends and calls their
/// update functions each frame.
pub struct PluginManager {
    /// List of active scripting backends.
    backends: Vec<Box<dyn ScriptingBackend>>,
    /// Whether hot reload checking is enabled.
    pub hot_reload_enabled: bool,
    /// Frame counter for throttling reload checks.
    frame_counter: u64,
    /// How often to check for file changes (in frames). Default: every 60 frames.
    pub reload_check_interval: u64,
}

impl PluginManager {
    /// Creates a new PluginManager with no backends loaded.
    pub fn new() -> Self {
        Self {
            backends: Vec::new(),
            hot_reload_enabled: true,
            frame_counter: 0,
            reload_check_interval: 60,
        }
    }

    /// Registers a new scripting backend.
    pub fn add_backend(&mut self, backend: Box<dyn ScriptingBackend>) {
        log::info!("[PluginManager] Registered backend: {}", backend.name());
        self.backends.push(backend);
    }

    /// Loads a native plugin from the given source path.
    /// Convenience method that creates a `NativePluginBackend` and loads it.
    pub fn load_native_plugin(
        &mut self,
        source_path: PathBuf,
        staging_dir: PathBuf,
    ) -> Result<(), String> {
        let mut backend = NativePluginBackend::new(source_path.clone(), staging_dir);
        backend.load(&source_path)?;
        self.add_backend(Box::new(backend));
        Ok(())
    }

    /// Called each frame by the engine. Checks for hot reload and calls plugin updates.
    pub fn tick(
        &mut self,
        world: &mut hecs::World,
        resources: &mut Resources,
        event_bus: &mut DynamicEventBus,
        delta_time: f32,
    ) {
        self.frame_counter = self.frame_counter.wrapping_add(1);

        // Throttled hot reload check
        if self.hot_reload_enabled && self.frame_counter % self.reload_check_interval == 0 {
            for backend in &mut self.backends {
                if backend.needs_reload() {
                    match backend.reload() {
                        Ok(()) => log::info!("[PluginManager] Reloaded: {}", backend.name()),
                        Err(e) => log::error!(
                            "[PluginManager] Reload failed for {}: {}",
                            backend.name(),
                            e
                        ),
                    }
                }
            }
        }

        // Call update on all loaded backends
        let mut ctx = PluginContext::new(world, resources, event_bus, delta_time);
        for backend in &self.backends {
            if let Err(e) = backend.call_update(&mut ctx) {
                log::error!("[PluginManager] Update error in {}: {}", backend.name(), e);
            }
        }
    }

    /// Returns the number of registered backends.
    pub fn backend_count(&self) -> usize {
        self.backends.len()
    }

    /// Returns a list of backend names (for UI display).
    pub fn backend_names(&self) -> Vec<&str> {
        self.backends.iter().map(|b| b.name()).collect()
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}