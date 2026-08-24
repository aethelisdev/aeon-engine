// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Dynamic WGSL Shader Module Loader & GPU Resource Compiler.
//!
//! Provides path-safe loading, deduplication, and compilation of `.wgsl` shader files
//! into the central `AssetManager.shaders` storage.
//!

use crate::asset::{AssetHandle, AssetManager, ShaderAsset, is_safe_path};
use crate::render::RenderState;

impl RenderState {
    /// Compiles a WGSL shader from raw source code and registers it into the `AssetManager`.
    /// Validates the shader module on the WGPU device and stores both the compiled
    /// `wgpu::ShaderModule` and the raw source text for runtime inspection and pipeline linking.
    pub fn load_shader_from_memory(
        &self,
        assets: &mut AssetManager,
        name: &str,
        source: &str,
        path: Option<&str>,
    ) -> Result<AssetHandle, String> {
        let shader_module = self
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some(name),
                source: wgpu::ShaderSource::Wgsl(source.into()),
            });

        let source_path_str = path.unwrap_or("").to_string();
        let canonical_path = if !source_path_str.is_empty() {
            std::fs::canonicalize(&source_path_str).ok()
        } else {
            None
        };

        let asset = ShaderAsset {
            module: shader_module,
            source_code: source.to_string(),
            name: name.to_string(),
            source_path: source_path_str,
        };

        let handle = assets.shaders.insert(asset);

        if let Some(cp) = canonical_path {
            assets.shader_path_map.insert(cp, handle);
        }

        log::info!(
            "⚡ [Shader Loader] Successfully compiled and registered shader: '{}'",
            name
        );
        Ok(handle)
    }

    /// Loads a `.wgsl` shader file from disk, verifies path security, deduplicates,
    /// and compiles it into the `AssetManager`.
    pub fn load_shader(
        &self,
        assets: &mut AssetManager,
        path: &str,
    ) -> Result<AssetHandle, String> {
        if !is_safe_path(path) {
            core::hint::cold_path();
            let err_msg = format!("[SECURITY ERROR] Blocked unsafe shader load path: {}", path);
            log::error!("{}", err_msg);
            return Err(err_msg);
        }

        let canonical_path = match std::fs::canonicalize(path) {
            Ok(p) => p,
            Err(e) => {
                core::hint::cold_path();
                let err_msg = format!(
                    "[ERROR] Failed to canonicalize shader path '{}': {}",
                    path, e
                );
                log::error!("{}", err_msg);
                return Err(err_msg);
            }
        };

        // Path-based deduplication
        if let Some(&handle) = assets
            .shader_path_map
            .get(&canonical_path)
            .filter(|&&h| assets.shaders.get(h).is_some())
        {
            return Ok(handle);
        }

        let source = match std::fs::read_to_string(&canonical_path) {
            Ok(s) => s,
            Err(e) => {
                core::hint::cold_path();
                let err_msg = format!("[ERROR] Failed to read shader file '{}': {}", path, e);
                log::error!("{}", err_msg);
                return Err(err_msg);
            }
        };

        let file_name = std::path::Path::new(path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unnamed_shader");

        self.load_shader_from_memory(assets, file_name, &source, Some(path))
    }
}