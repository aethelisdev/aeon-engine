// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Error types returned by the project export, asset cooking, and packaging pipeline.

use ae_world::WorldIoError;
use thiserror::Error;

/// Unified error type emitted by `ae_export` dependency discovery, cooking, and target builds.
#[derive(Debug, Error)]
pub enum ExportError {
    /// Filesystem I/O failure while reading source assets or staging export output.
    #[error("Export I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Binary world, chunk, `.aemeta`, or `.aepak` serialization failure.
    #[error("World/Pak error: {0}")]
    WorldIo(#[from] WorldIoError),

    /// Project root or entry world could not be located or parsed.
    #[error("Project configuration error: {0}")]
    ProjectNotFound(String),

    /// Required cross-compilation target or system linker is missing.
    #[error("Missing build toolchain: {0}")]
    ToolchainMissing(String),

    /// Cargo runtime compilation exited with a non-zero status code.
    #[error("Runtime build failed: {0}")]
    CargoBuildFailed(String),

    /// Raw model, texture, or audio asset failed to decode or cook.
    #[error("Asset cooking failed for '{asset}': {reason}")]
    AssetCookFailed {
        /// Virtual or filesystem path of the failing asset.
        asset: String,
        /// Underlying decoder or encoder diagnostic.
        reason: String,
    },
}

/// Convenience `Result` alias for `ae_export` operations.
pub type ExportResult<T> = Result<T, ExportError>;