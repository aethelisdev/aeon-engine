// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
/// Gizmo coordinate space — determines whether gizmo axes align with
/// world axes (identity orientation) or the selected entity's local rotation.
/// Used by the gizmo system to orient its visual handles and to compute
/// drag deltas in the correct coordinate frame. Toggled via the viewport
/// HUD button next to the W/E/R gizmo mode selector.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum GizmoSpace {
    /// Gizmo axes are always aligned to the world X/Y/Z directions,
    /// regardless of entity rotation. This is the default mode.
    World,
    /// Gizmo axes are rotated to match the selected entity's local
    /// orientation (its `Rotation` quaternion). Scale, translate, and
    /// rotate deltas are computed along the entity's own axes.
    Local,
}

impl GizmoSpace {
    /// Toggles between `World` and `Local`.
    pub fn toggle(self) -> Self {
        match self {
            Self::World => Self::Local,
            Self::Local => Self::World,
        }
    }
}