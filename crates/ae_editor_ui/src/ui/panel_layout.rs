// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
/// Modular panel identification, tree docking state (`iris_dock`),
/// and layout persistence for the Aeon Engine editor interface.
use irisui::dock::{DockState, DockTree};
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
    /// 2D Canvas & In-Game UI Designer panel.
    UiDesigner,
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
            Self::UiDesigner => "UI Designer",
        }
    }

    /// Returns the unicode icon glyph associated with this panel.
    pub fn icon(&self) -> &'static str {
        match self {
            Self::Viewport => "🎥",
            Self::Hierarchy => "🏗️",
            Self::Stats => "📊",
            Self::Inspector => "⚙️",
            Self::MaterialEditor => "🌐",
            Self::Assets => "📂",
            Self::Console => "📜",
            Self::AnimationTimeline => "🎬",
            Self::UiDesigner => "📐",
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
            Self::UiDesigner,
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
            Self::UiDesigner,
        ]
    }
}

/// Constructs the default tree layout using `iris_dock`.
/// **Layout Topology:**
/// - Center: `[Viewport, UiDesigner]` (Tabbed 3D & 2D workspace)
/// - Left Split (15%): `[Hierarchy, Stats]`
/// - Right Split (19%): `[Inspector, MaterialEditor]`
/// - Bottom Split (19%): `[Assets, Console, AnimationTimeline]`
pub fn create_default_dock_state() -> DockState<PanelId> {
    let mut tree = DockTree::new();
    let center = tree.create_leaf(vec![PanelId::Viewport, PanelId::UiDesigner]);
    tree.set_root(center);

    // 1. Split Left: Hierarchy (active) + Stats at ratio 0.14 (14% width)
    let (_left_leaf, center_right) = tree
        .split_ordered(
            center,
            irisui::dock::SplitDirection::Horizontal,
            0.14,
            vec![PanelId::Hierarchy, PanelId::Stats],
            true,
        )
        .expect("Left split succeeds");

    // 2. Split Right: Inspector (active) + Material Editor at ratio 0.82 (18% of remainder, ~15% screen width)
    let (center_leaf, _right_leaf) = tree
        .split_ordered(
            center_right,
            irisui::dock::SplitDirection::Horizontal,
            0.82,
            vec![PanelId::Inspector, PanelId::MaterialEditor],
            false,
        )
        .expect("Right split succeeds");

    // 3. Split Below Center: Assets (active) + Console + Animation Timeline at ratio 0.80 (20% of center, ~18% screen height)
    let (_viewport_leaf, _bottom_leaf) = tree
        .split_ordered(
            center_leaf,
            irisui::dock::SplitDirection::Vertical,
            0.80,
            vec![
                PanelId::Assets,
                PanelId::Console,
                PanelId::AnimationTimeline,
            ],
            false,
        )
        .expect("Bottom split succeeds");

    DockState::new(tree).with_min_pane_size(60.0)
}

/// Persistent layout state managing the `iris_dock` tree across the editor.
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
        self.dock_state.tree.find_tab(&panel).is_some()
    }

    /// Focuses an existing panel tab in the tree or opens it in a focused leaf.
    pub fn activate_or_open(&mut self, panel: PanelId) {
        if let Some((leaf, idx)) = self.dock_state.tree.find_tab(&panel) {
            let _ = self.dock_state.tree.set_active_tab(leaf, idx);
            self.dock_state.tree.set_focused_leaf(Some(leaf));
        } else {
            let _ = self.dock_state.tree.push_to_focused_leaf(panel);
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
        if let Some((leaf, idx)) = layout.dock_state.tree.find_tab(&PanelId::Stats) {
            let _ = layout.dock_state.tree.remove_tab(leaf, idx);
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
        if let Some((leaf, idx)) = layout.dock_state.tree.find_tab(&PanelId::Console) {
            let _ = layout.dock_state.tree.remove_tab(leaf, idx);
        }
        assert!(!layout.is_panel_visible(PanelId::Console));

        // Re-open Console
        layout.activate_or_open(PanelId::Console);
        assert!(layout.is_panel_visible(PanelId::Console));
    }
}