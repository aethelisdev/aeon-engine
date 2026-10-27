// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Workbench Error Types
//!
//! Provides the primary error enumeration and result alias for the Iris Workbench framework.
//! Covers layout failures, panel lifecycle mismatches, and tree synchronization faults.

use thiserror::Error;

/// Result type alias for operations throughout the Iris Workbench crate.
pub type WorkbenchResult<T> = Result<T, WorkbenchError>;

/// Represents catastrophic or recoverable error conditions within the Iris Workbench application shell.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum WorkbenchError {
    /// Indicates that layout calculation or bounding box resolution failed for a UI subtree or dock node.
    #[error("Subtree layout calculation failed: {0}")]
    LayoutFailed(String),

    /// Indicates an attempt to access, activate, or rebuild a panel that has not been registered.
    #[error("Panel not found in registry: {0}")]
    PanelNotFound(String),

    /// Indicates that a panel identifier was registered more than once without an explicit override.
    #[error("Panel already registered in workbench: {0}")]
    PanelAlreadyRegistered(String),

    /// Indicates that a tree structure invariant, docking hierarchy, or parent-child link was violated.
    #[error("Invalid workbench node hierarchy or tree structure: {0}")]
    InvalidHierarchy(String),

    /// Generic error describing external hook failures, modal dismissal issues, or unexpected workbench states.
    #[error("Workbench operation failed: {0}")]
    Custom(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workbench_error_display_formatting() {
        let layout_err = WorkbenchError::LayoutFailed("Constraint solver overflow".to_string());
        assert_eq!(
            layout_err.to_string(),
            "Subtree layout calculation failed: Constraint solver overflow"
        );

        let not_found_err = WorkbenchError::PanelNotFound("Inspector".to_string());
        assert_eq!(
            not_found_err.to_string(),
            "Panel not found in registry: Inspector"
        );

        let dup_err = WorkbenchError::PanelAlreadyRegistered("Hierarchy".to_string());
        assert_eq!(
            dup_err.to_string(),
            "Panel already registered in workbench: Hierarchy"
        );

        let hierarchy_err = WorkbenchError::InvalidHierarchy("Orphan dock leaf".to_string());
        assert_eq!(
            hierarchy_err.to_string(),
            "Invalid workbench node hierarchy or tree structure: Orphan dock leaf"
        );

        let custom_err = WorkbenchError::Custom("Texture hook timeout".to_string());
        assert_eq!(
            custom_err.to_string(),
            "Workbench operation failed: Texture hook timeout"
        );
    }

    #[test]
    fn test_workbench_error_equality() {
        let err1 = WorkbenchError::PanelNotFound("Assets".to_string());
        let err2 = WorkbenchError::PanelNotFound("Assets".to_string());
        let err3 = WorkbenchError::PanelNotFound("Console".to_string());

        assert_eq!(err1, err2);
        assert_ne!(err1, err3);
    }
}