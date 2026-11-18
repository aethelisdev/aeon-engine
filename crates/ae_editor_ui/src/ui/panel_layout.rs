// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
/// Modular panel identification, tree docking state (`irisui_dock`),
/// and layout persistence for the Aeon Engine editor interface.
use irisui::dock::{DockState, DockTree, FloatingWindowClampBounds};
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
    /// Engine & Editor Global Configuration and Preferences panel.
    Preferences,
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
            Self::Preferences => "Preferences",
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
            Self::Preferences => "⚙️",
        }
    }

    /// Returns the optional texture array layer coordinates for panels with a dedicated GPU atlas icon.
    pub fn atlas_icon(&self) -> Option<[f32; 4]> {
        match self {
            Self::Assets => Some(crate::ui::iris_bridge::icons::ICON_FOLDER),
            _ => None,
        }
    }

    /// Returns the canonical alphanumeric string identifier for this panel used in registries.
    ///
    /// This string identifier matches [`irisui_dock::DockPanel::id`] for registry lookups.
    pub fn id_str(&self) -> &'static str {
        match self {
            Self::Viewport => "viewport",
            Self::Hierarchy => "hierarchy",
            Self::Stats => "stats",
            Self::Inspector => "inspector",
            Self::MaterialEditor => "material_editor",
            Self::Assets => "assets",
            Self::Console => "console",
            Self::AnimationTimeline => "animation_timeline",
            Self::UiDesigner => "ui_designer",
            Self::Preferences => "preferences",
        }
    }

    /// Returns the declarative library docking/floating placement rule for this panel.
    #[inline]
    #[must_use]
    pub const fn placement(self) -> irisui::dock::TabPlacement<Self> {
        match self {
            Self::Preferences => irisui::dock::TabPlacement::FloatingCentered {
                size: irisui::prelude::Size::new(760.0, 540.0),
            },
            Self::Hierarchy | Self::Stats => irisui::dock::TabPlacement::Docked {
                partner: Some(ideal_partner_for(self)),
                anchor: Some(Self::Viewport),
                zone: irisui::dock::DropZone::Left,
            },
            Self::Inspector | Self::MaterialEditor => irisui::dock::TabPlacement::Docked {
                partner: Some(ideal_partner_for(self)),
                anchor: Some(Self::Viewport),
                zone: irisui::dock::DropZone::Right,
            },
            Self::Assets | Self::Console | Self::AnimationTimeline => {
                irisui::dock::TabPlacement::Docked {
                    partner: Some(ideal_partner_for(self)),
                    anchor: Some(Self::Viewport),
                    zone: irisui::dock::DropZone::Bottom,
                }
            }
            Self::Viewport | Self::UiDesigner => irisui::dock::TabPlacement::Docked {
                partner: Some(ideal_partner_for(self)),
                anchor: Some(Self::Viewport),
                zone: irisui::dock::DropZone::Center,
            },
        }
    }

    /// Parses a string identifier into its corresponding [`PanelId`] if recognized.
    ///
    /// Returns `None` if the provided string does not match any built-in panel identifier.
    pub fn from_id_str(s: &str) -> Option<Self> {
        match s {
            "viewport" => Some(Self::Viewport),
            "hierarchy" => Some(Self::Hierarchy),
            "stats" => Some(Self::Stats),
            "inspector" => Some(Self::Inspector),
            "material_editor" => Some(Self::MaterialEditor),
            "assets" => Some(Self::Assets),
            "console" => Some(Self::Console),
            "animation_timeline" => Some(Self::AnimationTimeline),
            "ui_designer" => Some(Self::UiDesigner),
            "preferences" => Some(Self::Preferences),
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
            Self::Preferences,
        ]
    }

    /// Returns an immutable slice of all panels pre-docked in the factory default layout.
    pub fn all_default_layout() -> &'static [Self] {
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

    /// Returns an immutable slice of all panel IDs including Viewport and on-demand panels.
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
            Self::Preferences,
        ]
    }
}

/// Constructs the default tree layout using `irisui_dock`.
///
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

/// Persistent layout state managing the `irisui_dock` tree across the editor.
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

    /// Checks if a panel currently exists anywhere in the docking tree or in an active floating window.
    pub fn is_panel_visible(&self, panel: PanelId) -> bool {
        self.dock_state.contains_tab(&panel)
    }

    /// Closes a tab at the specified leaf and index, collapsing any emptied leaf containers.
    pub fn close_tab(&mut self, leaf: irisui::dock::DockNodeId, tab_idx: usize) {
        let _ = self.dock_state.close_tab(leaf, tab_idx);
    }

    /// Closes a panel whether it is currently docked in the workspace tree or hosted in a detached floating window.
    ///
    /// Returns `true` if the panel was found and closed, or `false` if it was not open.
    pub fn close_panel(&mut self, panel: PanelId) -> bool {
        self.dock_state.close_tab_by_id(&panel)
    }

    /// Toggles a panel's visibility, closing it if currently visible (docked or floating),
    /// or activating/opening it if hidden.
    ///
    /// If the panel is [`PanelId::Preferences`], opens it as a detached floating window.
    pub fn toggle_panel(&mut self, panel: PanelId) {
        self.dock_state.toggle_tab(
            panel,
            panel.title(),
            &panel.placement(),
            Some(irisui::prelude::Size::new(1920.0, 1080.0)),
        );
    }

    /// Focuses an existing panel tab in the tree or opens it relative to its canonical partner/anchor.
    ///
    /// If the panel is already open in a detached floating window, brings that floating window to the front.
    /// If the panel is [`PanelId::Preferences`], opens it as a detached floating window centered on screen.
    pub fn activate_or_open(&mut self, panel: PanelId) {
        self.dock_state.open_or_focus_tab(
            panel,
            panel.title(),
            &panel.placement(),
            Some(irisui::prelude::Size::new(1920.0, 1080.0)),
        );
    }

    /// Opens the Preferences panel as a detached floating window or brings it to the front.
    pub fn open_preferences_floating(&mut self, screen_width: f32, screen_height: f32) {
        self.dock_state.open_or_focus_centered_floating(
            PanelId::Preferences,
            PanelId::Preferences.title(),
            irisui::prelude::Size::new(760.0, 540.0),
            Some(irisui::prelude::Size::new(screen_width, screen_height)),
        );
    }

    /// Constrains all detached floating windows to stay within user-accessible viewport boundaries.
    #[inline]
    pub fn clamp_floating_windows(&mut self, bounds: &FloatingWindowClampBounds) {
        self.dock_state.clamp_floating_windows(bounds);
    }

    /// Docks a floating window back to its canonical home leaf in the tree.
    pub fn smart_dock_back_panel(&mut self, win_id: u64) {
        self.dock_state
            .dock_floating_by_placement(win_id, |panel| panel.placement());
    }
}

/// Returns the canonical sibling/partner panel paired with `panel` for opening or docking back.
#[inline]
#[must_use]
pub const fn ideal_partner_for(panel: PanelId) -> PanelId {
    match panel {
        PanelId::Hierarchy => PanelId::Stats,
        PanelId::Stats => PanelId::Hierarchy,
        PanelId::Inspector => PanelId::MaterialEditor,
        PanelId::MaterialEditor => PanelId::Inspector,
        PanelId::Assets => PanelId::Console,
        PanelId::Console | PanelId::AnimationTimeline => PanelId::Assets,
        PanelId::Viewport => PanelId::UiDesigner,
        PanelId::UiDesigner => PanelId::Viewport,
        PanelId::Preferences => PanelId::Inspector,
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_default_layout_integrity() {
        let layout = PanelLayoutState::new_default();
        for &panel in PanelId::all_default_layout() {
            assert!(
                layout.is_panel_visible(panel),
                "Panel {:?} must be visible in default layout",
                panel
            );
        }
    }

    #[test]
    fn test_open_preferences_floating() {
        let mut layout = PanelLayoutState::new_default();
        assert!(!layout.is_panel_visible(PanelId::Preferences));

        // 1. Open Preferences floating window
        layout.open_preferences_floating(1920.0, 1080.0);
        assert_eq!(layout.dock_state.floating_windows.len(), 1);
        assert!(
            layout
                .dock_state
                .is_tab_in_floating_window(&PanelId::Preferences)
        );
        assert!(layout.is_panel_visible(PanelId::Preferences));

        // 2. Re-invoking should not duplicate the floating window
        layout.open_preferences_floating(1920.0, 1080.0);
        assert_eq!(layout.dock_state.floating_windows.len(), 1);
    }

    #[test]
    fn test_activate_or_open_preferences_floating() {
        let mut layout = PanelLayoutState::new_default();
        assert!(!layout.is_panel_visible(PanelId::Preferences));

        // 1. Opening Preferences via activate_or_open creates floating window
        layout.activate_or_open(PanelId::Preferences);
        assert!(layout.is_panel_visible(PanelId::Preferences));
        assert!(
            layout
                .dock_state
                .is_tab_in_floating_window(&PanelId::Preferences)
        );
        assert_eq!(layout.dock_state.floating_windows.len(), 1);

        // 2. Calling activate_or_open again preserves single floating window
        layout.activate_or_open(PanelId::Preferences);
        assert_eq!(layout.dock_state.floating_windows.len(), 1);
    }

    #[test]
    fn test_toggle_preferences_panel() {
        let mut layout = PanelLayoutState::new_default();
        assert!(!layout.is_panel_visible(PanelId::Preferences));

        // 1. Toggle opens Preferences floating window
        layout.toggle_panel(PanelId::Preferences);
        assert!(layout.is_panel_visible(PanelId::Preferences));
        assert!(
            layout
                .dock_state
                .is_tab_in_floating_window(&PanelId::Preferences)
        );
        assert_eq!(layout.dock_state.floating_windows.len(), 1);

        // 2. Toggle closes the floating Preferences window
        layout.toggle_panel(PanelId::Preferences);
        assert!(!layout.is_panel_visible(PanelId::Preferences));
        assert!(
            !layout
                .dock_state
                .is_tab_in_floating_window(&PanelId::Preferences)
        );
        assert_eq!(layout.dock_state.floating_windows.len(), 0);
    }

    #[test]
    fn test_preferences_floating_window_events_and_redock() {
        use irisui::dock::floating::FloatingWindowClampBounds;
        use irisui::dock::{TabViewer, handle_floating_window_event};
        use irisui::prelude::{MouseButton, Point, UiEvent, UiScope};

        struct DummyViewer;
        impl TabViewer<PanelId> for DummyViewer {
            fn title(&self, tab: &PanelId) -> String {
                format!("{:?}", tab)
            }
            fn ui(&mut self, _scope: &mut UiScope<'_>, _tab: &PanelId) {}
        }

        let mut layout = PanelLayoutState::new_default();
        layout.open_preferences_floating(1920.0, 1080.0);
        assert_eq!(layout.dock_state.floating_windows.len(), 1);

        let initial_rect = layout.dock_state.floating_windows[0].rect;
        let bounds = FloatingWindowClampBounds::new(1920.0, 1080.0)
            .with_min_y(30.0)
            .with_bottom_inset(24.0);
        let viewer = DummyViewer;

        // 1. Drag titlebar: MouseDown on titlebar, MouseMove by (50, 30), MouseUp
        let down_ev = UiEvent::MouseDown {
            point: Point::new(initial_rect.x + 300.0, initial_rect.y + 12.0),
            button: MouseButton::Left,
        };
        let res_down =
            handle_floating_window_event(&mut layout.dock_state, &viewer, &down_ev, bounds);
        assert!(res_down.consumed);
        assert!(layout.dock_state.has_active_floating_drag());

        let move_ev = UiEvent::MouseMove {
            point: Point::new(initial_rect.x + 350.0, initial_rect.y + 42.0),
        };
        let res_move =
            handle_floating_window_event(&mut layout.dock_state, &viewer, &move_ev, bounds);
        assert!(res_move.consumed);
        assert_eq!(
            layout.dock_state.floating_windows[0].rect.x,
            initial_rect.x + 50.0
        );
        assert_eq!(
            layout.dock_state.floating_windows[0].rect.y,
            initial_rect.y + 30.0
        );

        let up_ev = UiEvent::MouseUp {
            point: Point::new(initial_rect.x + 350.0, initial_rect.y + 42.0),
            button: MouseButton::Left,
        };
        let res_up = handle_floating_window_event(&mut layout.dock_state, &viewer, &up_ev, bounds);
        assert!(res_up.consumed);
        assert!(!layout.dock_state.has_active_floating_drag());

        // 2. Dock back to tree via smart_dock_back_panel
        let win_id = layout.dock_state.floating_windows[0].id;
        layout.smart_dock_back_panel(win_id);
        assert_eq!(layout.dock_state.floating_windows.len(), 0);
        assert!(layout.is_panel_visible(PanelId::Preferences));
        assert!(
            !layout
                .dock_state
                .is_tab_in_floating_window(&PanelId::Preferences)
        );
        assert!(
            layout
                .dock_state
                .tree
                .find_tab(&PanelId::Preferences)
                .is_some()
        );
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

        let bounds = FloatingWindowClampBounds::new(screen_w, screen_h)
            .with_min_y(min_y)
            .with_bottom_inset(status_bar_h);

        layout.clamp_floating_windows(&bounds);

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
    fn test_panel_id_string_roundtrip() {
        for &panel in PanelId::all() {
            let s = panel.id_str();
            assert!(!s.is_empty());
            let parsed = PanelId::from_id_str(s);
            assert_eq!(parsed, Some(panel));
        }
        assert_eq!(PanelId::from_id_str("unknown_panel"), None);
    }
}