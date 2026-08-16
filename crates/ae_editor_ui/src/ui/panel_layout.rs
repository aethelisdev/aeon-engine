// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
/// Modular panel identification, docking zones, drag-and-drop state,
/// and layout persistence for the Aeon Engine editor interface.
use serde::{Deserialize, Serialize};

/// Unique identifier for each editor tool panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PanelId {
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
            Self::Hierarchy => "Hierarchy",
            Self::Stats => "Stats",
            Self::Inspector => "Inspector",
            Self::MaterialEditor => "Material Editor",
            Self::Assets => "Assets",
            Self::Console => "Console",
            Self::AnimationTimeline => "Animation Timeline",
        }
    }

    /// Returns the unicode icon glyph associated with this panel.
    pub fn icon(&self) -> &'static str {
        match self {
            Self::Hierarchy => "🏗️",
            Self::Stats => "📊",
            Self::Inspector => "⚙️",
            Self::MaterialEditor => "🎨",
            Self::Assets => "📂",
            Self::Console => "📜",
            Self::AnimationTimeline => "🎬",
        }
    }

    /// Returns the default docking zone for this panel.
    pub fn default_zone(&self) -> PanelZone {
        match self {
            Self::Hierarchy | Self::Stats => PanelZone::Left,
            Self::Inspector | Self::MaterialEditor => PanelZone::Right,
            Self::Assets | Self::Console | Self::AnimationTimeline => PanelZone::Bottom,
        }
    }

    /// Returns an immutable slice of all standard built-in panel IDs.
    pub fn all() -> &'static [Self] {
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
}

/// Docking zone regions where tool panels can reside.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PanelZone {
    /// Left dock panel (typically outliner, hierarchy, stats).
    Left,
    /// Right dock panel (typically inspector, properties, material).
    Right,
    /// Bottom workspace panel (typically assets, console, timeline).
    Bottom,
}

impl PanelZone {
    /// Returns the human-readable display name of the docking zone.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Left => "Left Panel",
            Self::Right => "Right Panel",
            Self::Bottom => "Bottom Workspace",
        }
    }
}

/// Transient state tracking an active mouse drag-and-drop tab interaction.
#[derive(Debug, Clone, Copy)]
pub struct TabDragState {
    /// The panel ID of the tab being dragged.
    pub panel_id: PanelId,
    /// The docking zone the tab originated from.
    pub source_zone: PanelZone,
    /// The zero-based index of the tab in its source zone.
    pub source_index: usize,
    /// Screen-space position where the drag began.
    pub drag_origin: egui::Pos2,
    /// The docking zone currently hovered by the pointer (if any).
    pub hovered_zone: Option<PanelZone>,
    /// The slot index in the target zone where the tab would be inserted.
    pub hovered_index: usize,
    /// Whether the tab has been dragged outside its source tab bar (detached mode).
    pub is_detached: bool,
}

impl TabDragState {
    /// Creates a new tab drag state instance.
    pub fn new(
        panel_id: PanelId,
        source_zone: PanelZone,
        source_index: usize,
        drag_origin: egui::Pos2,
    ) -> Self {
        Self {
            panel_id,
            source_zone,
            source_index,
            drag_origin,
            hovered_zone: Some(source_zone),
            hovered_index: source_index,
            is_detached: false,
        }
    }
}

/// Persistent layout state managing all tabs across the editor docking zones.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanelLayoutState {
    /// Tabs currently docked in the left panel.
    pub left_tabs: Vec<PanelId>,
    /// Currently active tab in the left panel.
    pub active_left_tab: Option<PanelId>,
    /// Visibility toggle for the left panel.
    pub show_left_panel: bool,

    /// Tabs currently docked in the right panel.
    pub right_tabs: Vec<PanelId>,
    /// Currently active tab in the right panel.
    pub active_right_tab: Option<PanelId>,
    /// Visibility toggle for the right panel.
    pub show_right_panel: bool,

    /// Tabs currently docked in the bottom panel.
    pub bottom_tabs: Vec<PanelId>,
    /// Currently active tab in the bottom panel.
    pub active_bottom_tab: Option<PanelId>,
    /// Visibility toggle for the bottom panel.
    pub show_bottom_panel: bool,
}

impl Default for PanelLayoutState {
    fn default() -> Self {
        Self::new_default()
    }
}

impl PanelLayoutState {
    /// Constructs the default  engine editor panel layout.
    pub fn new_default() -> Self {
        Self {
            left_tabs: vec![PanelId::Hierarchy, PanelId::Stats],
            active_left_tab: Some(PanelId::Hierarchy),
            show_left_panel: true,

            right_tabs: vec![PanelId::Inspector, PanelId::MaterialEditor],
            active_right_tab: Some(PanelId::Inspector),
            show_right_panel: true,

            bottom_tabs: vec![
                PanelId::Assets,
                PanelId::Console,
                PanelId::AnimationTimeline,
            ],
            active_bottom_tab: Some(PanelId::Console),
            show_bottom_panel: true,
        }
    }

    /// Resets all docking zones and tabs to the factory default configuration.
    pub fn reset_to_default(&mut self) {
        *self = Self::new_default();
    }

    /// Returns an immutable reference to the tab list for a given docking zone.
    pub fn get_zone_tabs(&self, zone: PanelZone) -> &[PanelId] {
        match zone {
            PanelZone::Left => &self.left_tabs,
            PanelZone::Right => &self.right_tabs,
            PanelZone::Bottom => &self.bottom_tabs,
        }
    }

    /// Returns a mutable reference to the tab list for a given docking zone.
    pub fn get_zone_tabs_mut(&mut self, zone: PanelZone) -> &mut Vec<PanelId> {
        match zone {
            PanelZone::Left => &mut self.left_tabs,
            PanelZone::Right => &mut self.right_tabs,
            PanelZone::Bottom => &mut self.bottom_tabs,
        }
    }

    /// Returns the currently active tab ID for a given docking zone.
    pub fn get_active_tab(&self, zone: PanelZone) -> Option<PanelId> {
        match zone {
            PanelZone::Left => self.active_left_tab,
            PanelZone::Right => self.active_right_tab,
            PanelZone::Bottom => self.active_bottom_tab,
        }
    }

    /// Sets the active tab ID for a given docking zone.
    pub fn set_active_tab(&mut self, zone: PanelZone, panel: PanelId) {
        match zone {
            PanelZone::Left => {
                if self.left_tabs.contains(&panel) {
                    self.active_left_tab = Some(panel);
                    self.show_left_panel = true;
                }
            }
            PanelZone::Right => {
                if self.right_tabs.contains(&panel) {
                    self.active_right_tab = Some(panel);
                    self.show_right_panel = true;
                }
            }
            PanelZone::Bottom => {
                if self.bottom_tabs.contains(&panel) {
                    self.active_bottom_tab = Some(panel);
                    self.show_bottom_panel = true;
                }
            }
        }
    }

    /// Locates which zone and slot index a given panel currently resides in.
    pub fn find_panel_zone(&self, panel: PanelId) -> Option<(PanelZone, usize)> {
        if let Some(idx) = self.left_tabs.iter().position(|&p| p == panel) {
            return Some((PanelZone::Left, idx));
        }
        if let Some(idx) = self.right_tabs.iter().position(|&p| p == panel) {
            return Some((PanelZone::Right, idx));
        }
        if let Some(idx) = self.bottom_tabs.iter().position(|&p| p == panel) {
            return Some((PanelZone::Bottom, idx));
        }
        None
    }

    /// Moves a panel tab to a target docking zone at a specified index.
    /// Handles both intra-zone reordering (e.g. moving a tab from slot 2 to slot 0)
    /// and cross-zone relocation (e.g. moving Stats from Left to Bottom).
    pub fn move_tab(&mut self, panel: PanelId, to_zone: PanelZone, target_index: usize) {
        // 1. Remove panel from its current location
        let mut source_zone_opt = None;

        if let Some(pos) = self.left_tabs.iter().position(|&p| p == panel) {
            self.left_tabs.remove(pos);
            let source_was_active = self.active_left_tab == Some(panel);
            source_zone_opt = Some(PanelZone::Left);
            if source_was_active {
                self.active_left_tab = self.left_tabs.first().copied();
            }
        } else if let Some(pos) = self.right_tabs.iter().position(|&p| p == panel) {
            self.right_tabs.remove(pos);
            let source_was_active = self.active_right_tab == Some(panel);
            source_zone_opt = Some(PanelZone::Right);
            if source_was_active {
                self.active_right_tab = self.right_tabs.first().copied();
            }
        } else if let Some(pos) = self.bottom_tabs.iter().position(|&p| p == panel) {
            self.bottom_tabs.remove(pos);
            let source_was_active = self.active_bottom_tab == Some(panel);
            source_zone_opt = Some(PanelZone::Bottom);
            if source_was_active {
                self.active_bottom_tab = self.bottom_tabs.first().copied();
            }
        }

        // 2. Insert panel into the target zone
        let target_tabs = self.get_zone_tabs_mut(to_zone);
        let safe_index = target_index.min(target_tabs.len());
        target_tabs.insert(safe_index, panel);

        // 3. Always activate the newly moved/reordered tab in the destination zone
        match to_zone {
            PanelZone::Left => {
                self.active_left_tab = Some(panel);
                self.show_left_panel = true;
            }
            PanelZone::Right => {
                self.active_right_tab = Some(panel);
                self.show_right_panel = true;
            }
            PanelZone::Bottom => {
                self.active_bottom_tab = Some(panel);
                self.show_bottom_panel = true;
            }
        }

        // 4. If source zone became empty and was not the target zone, keep state clean
        if let Some(source_zone) = source_zone_opt
            && source_zone != to_zone
        {
            let source_empty = self.get_zone_tabs(source_zone).is_empty();
            if source_empty {
                match source_zone {
                    PanelZone::Left => {
                        self.active_left_tab = None;
                    }
                    PanelZone::Right => {
                        self.active_right_tab = None;
                    }
                    PanelZone::Bottom => {
                        self.active_bottom_tab = None;
                    }
                }
            }
        }
    }

    /// Activates a panel and opens its host docking zone if closed.
    /// If the panel is not currently docked in any zone, it restores it into its default zone.
    pub fn activate_or_open(&mut self, panel: PanelId) {
        if let Some((zone, _)) = self.find_panel_zone(panel) {
            self.set_active_tab(zone, panel);
        } else {
            let default_zone = panel.default_zone();
            let len = self.get_zone_tabs(default_zone).len();
            self.move_tab(panel, default_zone, len);
        }
    }

    /// Checks whether a panel is currently docked and visible (active and panel open).
    pub fn is_panel_visible(&self, panel: PanelId) -> bool {
        if let Some((zone, _)) = self.find_panel_zone(panel) {
            match zone {
                PanelZone::Left => self.show_left_panel && self.active_left_tab == Some(panel),
                PanelZone::Right => self.show_right_panel && self.active_right_tab == Some(panel),
                PanelZone::Bottom => {
                    self.show_bottom_panel && self.active_bottom_tab == Some(panel)
                }
            }
        } else {
            false
        }
    }

    /// Toggles a panel's visibility. If open and active, hides it; otherwise opens and focuses it.
    pub fn toggle_panel(&mut self, panel: PanelId) {
        if let Some((zone, _)) = self.find_panel_zone(panel) {
            let is_active = self.get_active_tab(zone) == Some(panel);
            let is_zone_shown = match zone {
                PanelZone::Left => self.show_left_panel,
                PanelZone::Right => self.show_right_panel,
                PanelZone::Bottom => self.show_bottom_panel,
            };

            if is_zone_shown && is_active {
                // If it is already the active shown tab, hide the zone
                match zone {
                    PanelZone::Left => self.show_left_panel = false,
                    PanelZone::Right => self.show_right_panel = false,
                    PanelZone::Bottom => self.show_bottom_panel = false,
                }
            } else {
                // Focus and show
                self.activate_or_open(panel);
            }
        } else {
            self.activate_or_open(panel);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_layout_integrity() {
        let layout = PanelLayoutState::new_default();
        assert_eq!(layout.left_tabs.len(), 2);
        assert_eq!(layout.right_tabs.len(), 2);
        assert_eq!(layout.bottom_tabs.len(), 3);
        assert_eq!(layout.active_left_tab, Some(PanelId::Hierarchy));
        assert_eq!(layout.active_right_tab, Some(PanelId::Inspector));
        assert_eq!(layout.active_bottom_tab, Some(PanelId::Console));
    }

    #[test]
    fn test_tab_reordering_within_zone() {
        let mut layout = PanelLayoutState::new_default();
        // Move AnimationTimeline (index 2) to front (index 0) in Bottom zone
        layout.move_tab(PanelId::AnimationTimeline, PanelZone::Bottom, 0);

        assert_eq!(layout.bottom_tabs[0], PanelId::AnimationTimeline);
        assert_eq!(layout.bottom_tabs[1], PanelId::Assets);
        assert_eq!(layout.bottom_tabs[2], PanelId::Console);
        assert_eq!(layout.active_bottom_tab, Some(PanelId::AnimationTimeline));
    }

    #[test]
    fn test_tab_move_cross_zone() {
        let mut layout = PanelLayoutState::new_default();
        // Move Stats from Left to Right at index 0
        layout.move_tab(PanelId::Stats, PanelZone::Right, 0);

        assert_eq!(layout.left_tabs.len(), 1);
        assert_eq!(layout.left_tabs[0], PanelId::Hierarchy);
        assert_eq!(layout.right_tabs.len(), 3);
        assert_eq!(layout.right_tabs[0], PanelId::Stats);
        assert_eq!(layout.right_tabs[1], PanelId::Inspector);
        assert_eq!(layout.right_tabs[2], PanelId::MaterialEditor);
        assert_eq!(layout.active_right_tab, Some(PanelId::Stats));
    }

    #[test]
    fn test_reset_to_default() {
        let mut layout = PanelLayoutState::new_default();
        layout.move_tab(PanelId::Stats, PanelZone::Right, 0);
        layout.reset_to_default();

        assert_eq!(layout.left_tabs.len(), 2);
        assert_eq!(layout.left_tabs[1], PanelId::Stats);
        assert_eq!(layout.right_tabs.len(), 2);
    }
}