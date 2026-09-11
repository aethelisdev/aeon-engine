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

    /// Returns the optional texture array layer coordinates for panels with a dedicated GPU atlas icon.
    pub fn atlas_icon(&self) -> Option<[f32; 4]> {
        match self {
            Self::Assets => Some(crate::ui::iris_bridge::icons::ICON_FOLDER),
            _ => None,
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
    /// Monotonically increasing revision counter incremented on layout mutations.
    #[serde(default)]
    pub revision: u64,
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
            revision: 0,
        }
    }

    /// Monotonically increments the layout mutation revision counter.
    #[inline]
    pub fn bump_revision(&mut self) {
        self.revision = self.revision.wrapping_add(1);
    }

    /// Resets all docking surfaces, splits, and tabs to the factory default configuration.
    pub fn reset_to_default(&mut self) {
        self.dock_state = create_default_dock_state();
        self.bump_revision();
    }

    /// Checks if a panel currently exists anywhere in the docking tree.
    pub fn is_panel_visible(&self, panel: PanelId) -> bool {
        self.dock_state.tree.find_tab(&panel).is_some()
    }

    /// Closes a tab at the specified leaf and index, collapsing any emptied leaf containers.
    pub fn close_tab(&mut self, leaf: irisui::dock::DockNodeId, tab_idx: usize) {
        let _ = self.dock_state.tree.remove_tab(leaf, tab_idx);
        self.dock_state.tree.collapse_empty_leaves();
        self.bump_revision();
    }

    /// Focuses an existing panel tab in the tree or opens it relative to its canonical partner/anchor.
    pub fn activate_or_open(&mut self, panel: PanelId) {
        if let Some((leaf, idx)) = self.dock_state.tree.find_tab(&panel) {
            let _ = self.dock_state.tree.set_active_tab(leaf, idx);
            self.bump_revision();
            self.dock_state.tree.set_focused_leaf(Some(leaf));
            return;
        }

        let ideal_partner = match panel {
            PanelId::Hierarchy => PanelId::Stats,
            PanelId::Stats => PanelId::Hierarchy,
            PanelId::Inspector => PanelId::MaterialEditor,
            PanelId::MaterialEditor => PanelId::Inspector,
            PanelId::Assets => PanelId::Console,
            PanelId::Console | PanelId::AnimationTimeline => PanelId::Assets,
            PanelId::Viewport => PanelId::UiDesigner,
            PanelId::UiDesigner => PanelId::Viewport,
        };

        // 1. Try docking as a tab alongside ideal partner leaf
        if let Some((partner_leaf, _)) = self.dock_state.tree.find_tab(&ideal_partner)
            && let Ok(idx) = self.dock_state.tree.add_tab(partner_leaf, panel)
        {
            let _ = self.dock_state.tree.set_active_tab(partner_leaf, idx);
            self.bump_revision();
            self.dock_state.tree.set_focused_leaf(Some(partner_leaf));
            return;
        }

        // 2. Try docking relative to Viewport (the central anchor of the editor)
        if let Some((viewport_leaf, _)) = self.dock_state.tree.find_tab(&PanelId::Viewport) {
            let res = match panel {
                PanelId::Hierarchy | PanelId::Stats => {
                    self.dock_state.tree.split_left(viewport_leaf, panel)
                }
                PanelId::Inspector | PanelId::MaterialEditor => {
                    self.dock_state.tree.split_right(viewport_leaf, panel)
                }
                PanelId::Assets | PanelId::Console | PanelId::AnimationTimeline => {
                    self.dock_state.tree.split_below(viewport_leaf, panel)
                }
                PanelId::Viewport | PanelId::UiDesigner => self
                    .dock_state
                    .tree
                    .add_tab(viewport_leaf, panel)
                    .map(|_| viewport_leaf),
            };
            if let Ok(new_leaf) = res {
                self.bump_revision();
                self.dock_state.tree.set_focused_leaf(Some(new_leaf));
                return;
            }
        }

        // 3. Fallback to focused leaf or root
        let _ = self.dock_state.tree.push_to_focused_leaf(panel);
        self.bump_revision();
    }

    /// Clamps all floating windows to ensure their title bars and content remain accessible within the workspace bounds.
    /// Constrains vertical coordinates so floating panel title bars never get pushed underneath
    /// the top menubar (`min_y`) or off the bottom of the screen. Horizontally, ensures at least
    /// a visible grab margin remains accessible.
    pub fn clamp_floating_windows(
        &mut self,
        screen_w: f32,
        screen_h: f32,
        min_y: f32,
        status_bar_h: f32,
    ) {
        const TAB_BAR_H: f32 = 26.0;
        let available_h = (screen_h - min_y - status_bar_h).max(TAB_BAR_H);
        let max_y = (screen_h - status_bar_h - TAB_BAR_H).max(min_y);

        for win in &mut self.dock_state.floating_windows {
            if screen_w > 100.0 {
                win.rect.width = win.rect.width.clamp(220.0, screen_w);
            }
            if available_h > TAB_BAR_H {
                win.rect.height = win.rect.height.clamp(140.0, available_h);
            }

            win.rect.y = win.rect.y.clamp(min_y, max_y);

            let max_x = (screen_w - 60.0).max(0.0);
            let min_x = (60.0 - win.rect.width).min(0.0);
            win.rect.x = win.rect.x.clamp(min_x, max_x);
        }
    }

    /// Intelligently docks a floating window back into the main docking layout near its canonical partner.
    pub fn smart_dock_back_panel(&mut self, win_id: u64) {
        let Some(win) = self
            .dock_state
            .floating_windows
            .iter()
            .find(|w| w.id == win_id)
        else {
            return;
        };

        let Some(panel) = win.tree.iter().find_map(|(_, node)| match node {
            irisui::dock::DockNode::Leaf { tabs, active_tab } => tabs.get(*active_tab).copied(),
            _ => None,
        }) else {
            return;
        };

        let ideal_partner = match panel {
            PanelId::Hierarchy => PanelId::Stats,
            PanelId::Stats => PanelId::Hierarchy,
            PanelId::Inspector => PanelId::MaterialEditor,
            PanelId::MaterialEditor => PanelId::Inspector,
            PanelId::Assets => PanelId::Console,
            PanelId::Console | PanelId::AnimationTimeline => PanelId::Assets,
            PanelId::Viewport => PanelId::UiDesigner,
            PanelId::UiDesigner => PanelId::Viewport,
        };

        // 1. Try docking as a tab alongside ideal partner leaf
        if let Some((partner_leaf, _)) = self.dock_state.tree.find_tab(&ideal_partner) {
            let _ = self.dock_state.dock_floating_window(
                win_id,
                partner_leaf,
                irisui::dock::DropZone::Center,
            );
            self.bump_revision();
            return;
        }

        // 2. Try docking relative to Viewport (the central anchor of the editor)
        if let Some((viewport_leaf, _)) = self.dock_state.tree.find_tab(&PanelId::Viewport) {
            let target_zone = match panel {
                PanelId::Hierarchy | PanelId::Stats => irisui::dock::DropZone::Left,
                PanelId::Inspector | PanelId::MaterialEditor => irisui::dock::DropZone::Right,
                PanelId::Assets | PanelId::Console | PanelId::AnimationTimeline => {
                    irisui::dock::DropZone::Bottom
                }
                PanelId::Viewport | PanelId::UiDesigner => irisui::dock::DropZone::Center,
            };
            let _ = self
                .dock_state
                .dock_floating_window(win_id, viewport_leaf, target_zone);
            self.bump_revision();
            return;
        }

        // 3. Fallback to any existing leaf in the tree
        if let Some(fallback_leaf) = self.dock_state.tree.find_first_leaf() {
            let _ = self.dock_state.dock_floating_window(
                win_id,
                fallback_leaf,
                irisui::dock::DropZone::Center,
            );
            self.bump_revision();
        }
    }
}

/// Tab viewer implementation for [`PanelId`] used by Iris docking layout computations.
#[derive(Debug, Clone, Copy, Default)]
pub struct PanelTabViewer;

impl irisui::dock::TabViewer<PanelId> for PanelTabViewer {
    fn title(&self, tab: &PanelId) -> String {
        format!("{} {}", tab.icon(), tab.title())
    }

    fn closeable(&self, tab: &PanelId) -> bool {
        *tab != PanelId::Viewport
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

    #[test]
    fn test_close_tab_collapses_empty_leaf() {
        let mut layout = PanelLayoutState::new_default();
        // Remove both Inspector and MaterialEditor from the right leaf
        let (leaf, idx) = layout
            .dock_state
            .tree
            .find_tab(&PanelId::Inspector)
            .unwrap();
        layout.close_tab(leaf, idx);
        assert!(!layout.is_panel_visible(PanelId::Inspector));
        // Leaf still exists because MaterialEditor is present
        assert!(layout.dock_state.tree.iter().any(|(id, _)| id == leaf));

        let (leaf2, idx2) = layout
            .dock_state
            .tree
            .find_tab(&PanelId::MaterialEditor)
            .unwrap();
        assert_eq!(leaf, leaf2);
        layout.close_tab(leaf2, idx2);
        assert!(!layout.is_panel_visible(PanelId::MaterialEditor));

        // Now that all tabs in the leaf were closed, the leaf is collapsed and pruned from tree
        assert!(layout.dock_state.tree.iter().all(|(id, _)| id != leaf));
    }

    #[test]
    fn test_clamp_floating_windows_bounds() {
        use irisui::dock::FloatingWindow;
        use irisui::prelude::Rect;

        let mut layout = PanelLayoutState::new_default();
        // Add a floating window positioned way off-screen (above menubar, negative coordinates)
        let off_screen_win = FloatingWindow::new(
            99,
            "Offscreen Hierarchy",
            Rect::new(-500.0, -100.0, 300.0, 400.0),
            vec![PanelId::Hierarchy],
        );
        layout.dock_state.floating_windows.push(off_screen_win);

        // Add a floating window positioned way below the screen
        let too_low_win = FloatingWindow::new(
            100,
            "Too Low Window",
            Rect::new(2500.0, 3000.0, 300.0, 400.0),
            vec![PanelId::Stats],
        );
        layout.dock_state.floating_windows.push(too_low_win);

        let screen_w = 1920.0;
        let screen_h = 1080.0;
        let min_y = 34.0; // Menubar height
        let status_bar_h = 24.0;

        layout.clamp_floating_windows(screen_w, screen_h, min_y, status_bar_h);

        let win1 = layout
            .dock_state
            .floating_windows
            .iter()
            .find(|w| w.id == 99)
            .unwrap();
        // Title bar Y must be clamped to min_y
        assert_eq!(win1.rect.y, min_y);
        // Left coordinate clamped so at least 60px remains visible
        assert_eq!(win1.rect.x, 60.0 - win1.rect.width);

        let win2 = layout
            .dock_state
            .floating_windows
            .iter()
            .find(|w| w.id == 100)
            .unwrap();
        // Title bar Y clamped so title bar does not sink below status bar
        assert_eq!(win2.rect.y, screen_h - status_bar_h - 26.0);
        // Right coordinate clamped so at least 60px remains visible on screen
        assert_eq!(win2.rect.x, screen_w - 60.0);
    }

    #[test]
    fn test_panel_layout_revision_tracking() {
        let mut layout = PanelLayoutState::new_default();
        assert_eq!(layout.revision, 0);

        layout.bump_revision();
        assert_eq!(layout.revision, 1);

        layout.reset_to_default();
        assert_eq!(layout.revision, 2);

        // Closing a tab should increment revision
        if let Some((leaf, _)) = layout.dock_state.tree.find_tab(&PanelId::Console) {
            layout.close_tab(leaf, 1);
            assert_eq!(layout.revision, 3);
        }

        // Activating or opening a panel should increment revision
        layout.activate_or_open(PanelId::Console);
        assert_eq!(layout.revision, 4);
    }
}