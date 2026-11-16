// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Error types for the layout engine and Taffy integration.

use thiserror::Error;

/// Layout computation and tree synchronization errors.
#[derive(Debug, Error)]
pub enum LayoutError {
    /// Underlying Taffy layout calculation or node resolution failed.
    #[error("Taffy layout error: {0}")]
    Taffy(String),
}