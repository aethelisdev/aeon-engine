// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
/// Modular panel identification, tree docking state (`irisui_dock`),
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
mod tests {
    use super::*;

    #[test]
    fn test_default_layout_integrity() {
        let dock_state = create_default_dock_state();
        for &panel in PanelId::all_default_layout() {
            assert!(
                dock_state.contains_tab(&panel),
                "Panel {:?} must be visible in default layout",
                panel
            );
        }
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