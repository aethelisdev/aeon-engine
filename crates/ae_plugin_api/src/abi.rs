// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! ABI hash definitions and platform-specific library metadata for safe FFI boundaries.
//!

/// The compiled ABI version hash of the Aeon Engine.
pub const ENGINE_ABI_HASH: &str = if cfg!(debug_assertions) {
    "ae-abi-v0.8.0-debug"
} else {
    "ae-abi-v0.8.0-release"
};

/// Null-terminated CStr version of `ENGINE_ABI_HASH` for safe FFI export across plugin boundaries.
pub const ENGINE_ABI_HASH_C_STR: &std::ffi::CStr = if cfg!(debug_assertions) {
    c"ae-abi-v0.8.0-debug"
} else {
    c"ae-abi-v0.8.0-release"
};

/// Returns the platform-specific dynamic library extension.
pub fn platform_lib_extension() -> &'static str {
    if cfg!(target_os = "windows") {
        "dll"
    } else if cfg!(target_os = "macos") {
        "dylib"
    } else {
        "so"
    }
}

/// Returns the platform-specific shared library filename including prefix (e.g. `libgame_logic.so`).
pub fn platform_lib_filename(name: &str) -> String {
    if cfg!(target_os = "windows") {
        format!("{}.dll", name)
    } else if cfg!(target_os = "macos") {
        format!("lib{}.dylib", name)
    } else {
        format!("lib{}.so", name)
    }
}