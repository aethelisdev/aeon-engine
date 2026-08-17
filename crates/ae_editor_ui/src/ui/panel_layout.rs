// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
/// Modular panel identification, tree docking state (`egui_dock`),
/// and layout persistence for the Aeon Engine editor interface.
use egui_dock::DockState;
use serde::{Deserialize, Serialize};

/// Unique identifier for each editor tool panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PanelId {
    /// 3D Viewport Scene View.
    Viewport,
    /// Scene Hierarchy (Outliner) panel.
    Hierarchy,
    /// CPU and Render Profiler / Engine Stats panel.
    Stats,
    /// Entity Component Inspector panel.
    Inspector,
    /// Material & Texture Editor panel.
    MaterialEditor,
    /// 3D Models & 2D Textures Asset Browser panel.
    Assets,
    /// Developer Log & Diagnostics Console panel.
    Console,
    /// Skeletal Animation Player & Timeline Scrubbing panel.
    AnimationTimeline,
}

impl PanelId {
    /// Returns the human-readable display title for this panel.
    pub fn title(&self) -> &'static str {
        match self {
            Self::Viewport => "Viewport",
            Self::Hierarchy => "Hierarchy",
            Self::Stats => "Stats",
            Self::Inspector => "Inspector",
            Self::MaterialEditor => "Material Editor",
            Self::Assets => "Assets",
            Self::Console => "Console",
            Self::AnimationTimeline => "Timeline",
        }
    }

    /// Returns the unicode icon glyph associated with this panel.
    pub fn icon(&self) -> &'static str {
        match self {
            Self::Viewport => "🎥",
            Self::Hierarchy => "🏗️",
            Self::Stats => "📊",
            Self::Inspector => "⚙️",
            Self::MaterialEditor => "🎨",
            Self::Assets => "📂",
            Self::Console => "📜",
            Self::AnimationTimeline => "🎬",
        }
    }

    /// Returns an immutable slice of all standard dockable tool panels (excluding main viewport).
    pub fn all_tool_panels() -> &'static [Self] {
        &[
            Self::Hierarchy,
            Self::Stats,
            Self::Inspector,
            Self::MaterialEditor,
            Self::Assets,
            Self::Console,
            Self::AnimationTimeline,
        ]
    }

    /// Returns an immutable slice of all panel IDs including Viewport.
    pub fn all() -> &'static [Self] {
        &[
            Self::Viewport,
            Self::Hierarchy,
            Self::Stats,
            Self::Inspector,
            Self::MaterialEditor,
            Self::Assets,
            Self::Console,
            Self::AnimationTimeline,
        ]
    }
}

/// Constructs the default tree layout using `egui_dock`.
/// **Layout Topology:**
/// - Left Split (15%): `[Hierarchy, Stats]`
/// - Right Split (19%): `[Inspector, MaterialEditor]`
/// - Bottom Split (19%): `[Assets, Console, AnimationTimeline]`
/// - Center Viewport: Spacious ~66% Width × ~81% Height 3D Workspace
pub fn create_default_dock_state() -> DockState<PanelId> {
    let mut dock_state = DockState::new(vec![PanelId::Viewport]);

    // Split Left: Hierarchy & Stats (15% width)
    let [center, _left] = dock_state.main_surface_mut().split_left(
        egui_dock::NodeIndex::root(),
        0.15,
        vec![PanelId::Hierarchy, PanelId::Stats],
    );

    // Split Right: Inspector & Material Editor (19% width)
    let [center, _right] = dock_state.main_surface_mut().split_right(
        center,
        0.81,
        vec![PanelId::Inspector, PanelId::MaterialEditor],
    );

    // Split Below Center: Assets, Console, Animation Timeline (19% height)
    let [_center, _bottom] = dock_state.main_surface_mut().split_below(
        center,
        0.81,
        vec![
            PanelId::Assets,
            PanelId::Console,
            PanelId::AnimationTimeline,
        ],
    );

    dock_state
}

/// Persistent layout state managing the `egui_dock` tree across the editor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanelLayoutState {
    /// Full tree docking state.
    pub dock_state: DockState<PanelId>,
}

impl Default for PanelLayoutState {
    fn default() -> Self {
        Self::new_default()
    }
}

impl PanelLayoutState {
    /// Constructs the default panel layout state with pre-configured tree splits.
    pub fn new_default() -> Self {
        Self {
            dock_state: create_default_dock_state(),
        }
    }

    /// Resets all docking surfaces, splits, and tabs to the factory default configuration.
    pub fn reset_to_default(&mut self) {
        self.dock_state = create_default_dock_state();
    }

    /// Checks if a panel currently exists anywhere in the docking tree.
    pub fn is_panel_visible(&self, panel: PanelId) -> bool {
        self.dock_state.find_tab(&panel).is_some()
    }

    /// Focuses an existing panel tab in the tree or opens it in a focused leaf.
    pub fn activate_or_open(&mut self, panel: PanelId) {
        if let Some(location) = self.dock_state.find_tab(&panel) {
            let _ = self.dock_state.set_active_tab(location);
        } else {
            self.dock_state.push_to_focused_leaf(panel);
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_default_layout_integrity() {
        let layout = PanelLayoutState::new_default();
        for &panel in PanelId::all() {
            assert!(
                layout.is_panel_visible(panel),
                "Panel {:?} must be visible in default layout",
                panel
            );
        }
    }

    #[test]
    fn test_reset_to_default() {
        let mut layout = PanelLayoutState::new_default();
        // Remove a tab
        if let Some(loc) = layout.dock_state.find_tab(&PanelId::Stats) {
            layout.dock_state.remove_tab(loc);
        }
        assert!(!layout.is_panel_visible(PanelId::Stats));

        // Reset
        layout.reset_to_default();
        assert!(layout.is_panel_visible(PanelId::Stats));
    }

    #[test]
    fn test_activate_or_open_tab() {
        let mut layout = PanelLayoutState::new_default();
        // Remove Console
        if let Some(loc) = layout.dock_state.find_tab(&PanelId::Console) {
            layout.dock_state.remove_tab(loc);
        }
        assert!(!layout.is_panel_visible(PanelId::Console));

        // Re-open Console
        layout.activate_or_open(PanelId::Console);
        assert!(layout.is_panel_visible(PanelId::Console));
    }
}