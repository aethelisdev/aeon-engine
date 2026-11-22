// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
/// Modular panel identification, tree docking state (`irisui_dock`),
/// and layout persistence for the Aeon Engine editor interface.
use super::iris_bridge::{
    assets::ASSETS_TAG_PANEL_ROOT, console::CONSOLE_TAG_PANEL_ROOT,
    hierarchy::HIERARCHY_TAG_PANEL_ROOT, inspector::TAG_INSPECTOR_PANEL_ROOT,
    stats::STATS_TAG_PANEL_ROOT, timeline::TIMELINE_TAG_PANEL_ROOT,
    ui_designer::UI_DESIGNER_TAG_PANEL_ROOT,
};
use irisui::core::Color;
use irisui::dock::{DockLayoutNode, DockState};
use irisui::workbench::{
    LiveRefreshInterval, PanelBackground, PanelBorder, PanelMetadata, PanelSurfaceStyle,
};
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

    /// Returns the unified declarative [`irisui::prelude::UiIcon`] associated with this panel.
    #[must_use]
    pub fn ui_icon(&self) -> irisui::prelude::UiIcon {
        match self {
            Self::Viewport => irisui::prelude::UiIcon::Glyph("🎥"),
            Self::Hierarchy => irisui::prelude::UiIcon::Glyph("🏗️"),
            Self::Stats => irisui::prelude::UiIcon::Glyph("📊"),
            Self::Inspector => irisui::prelude::UiIcon::Glyph("⚙️"),
            Self::MaterialEditor => irisui::prelude::UiIcon::Glyph("🌐"),
            Self::Assets => {
                irisui::prelude::UiIcon::Atlas(crate::ui::iris_bridge::icons::ICON_FOLDER)
            }
            Self::Console => irisui::prelude::UiIcon::Glyph("📜"),
            Self::AnimationTimeline => irisui::prelude::UiIcon::Glyph("🎬"),
            Self::UiDesigner => irisui::prelude::UiIcon::Glyph("📐"),
            Self::Preferences => irisui::prelude::UiIcon::Glyph("⚙️"),
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

    /// Builds the optional declarative [`PanelSurfaceStyle`] wrapping this panel's leaf content.
    ///
    /// [`Self::Viewport`] returns `None` so its WGPU render target compositing and viewport HUD
    /// manage their own pass-through surface directly.
    #[must_use]
    pub const fn surface(self) -> Option<PanelSurfaceStyle> {
        match self {
            Self::Viewport => None,
            Self::Hierarchy => Some(
                PanelSurfaceStyle::new("HierarchyPanelRoot")
                    .with_tag(HIERARCHY_TAG_PANEL_ROOT.raw()),
            ),
            Self::Stats => Some(
                PanelSurfaceStyle::new("StatsPanelRoot")
                    .with_tag(STATS_TAG_PANEL_ROOT)
                    .with_background(PanelBackground::Color(Color::rgba(0.06, 0.07, 0.09, 1.0))),
            ),
            Self::Inspector => Some(
                PanelSurfaceStyle::new("InspectorPanelRoot").with_tag(TAG_INSPECTOR_PANEL_ROOT),
            ),
            Self::MaterialEditor => Some(PanelSurfaceStyle::new("MaterialPanelRoot")),
            Self::Assets => {
                Some(PanelSurfaceStyle::new("AssetsPanelRoot").with_tag(ASSETS_TAG_PANEL_ROOT))
            }
            Self::Console => Some(
                PanelSurfaceStyle::new("ConsolePanelRoot")
                    .with_tag(CONSOLE_TAG_PANEL_ROOT)
                    .with_background(PanelBackground::Color(Color::rgba(0.05, 0.06, 0.08, 1.0))),
            ),
            Self::AnimationTimeline => Some(
                PanelSurfaceStyle::new("AnimationTimelinePanelRoot")
                    .with_tag(TIMELINE_TAG_PANEL_ROOT)
                    .with_border(PanelBorder::ThemeWidth(1.0)),
            ),
            Self::UiDesigner => Some(
                PanelSurfaceStyle::new("UiDesignerPanelRoot")
                    .with_tag(UI_DESIGNER_TAG_PANEL_ROOT)
                    .with_background(PanelBackground::Color(Color::rgba(
                        0.065, 0.070, 0.082, 1.0,
                    )))
                    .with_border(PanelBorder::Custom(
                        1.0,
                        Color::rgba(0.14, 0.15, 0.18, 0.70),
                    )),
            ),
            Self::Preferences => Some(
                PanelSurfaceStyle::new("Preferences")
                    .row()
                    .with_border(PanelBorder::None),
            ),
        }
    }

    /// Builds declarative [`PanelMetadata`] for persistent registration in `ScopedPanelRegistry`.
    #[must_use]
    pub fn metadata(self, live_refresh: LiveRefreshInterval) -> PanelMetadata<Self> {
        PanelMetadata::new(self, self.title())
            .with_optional_icon(Some(self.ui_icon()))
            .with_domain_bound(matches!(
                self,
                Self::Hierarchy | Self::Inspector | Self::MaterialEditor | Self::AnimationTimeline
            ))
            .with_closeable(!matches!(self, Self::Viewport))
            .with_clear_background(matches!(self, Self::Viewport))
            .with_placement(self.placement())
            .with_live_refresh(live_refresh)
            .with_skip_collapsed(true)
            .with_optional_surface(self.surface())
    }
}

/// Constructs the default tree layout using `irisui_dock`'s declarative [`DockLayoutNode`].
///
/// **Layout Topology:**
/// - Center: `[Viewport, UiDesigner]` (Tabbed 3D & 2D workspace)
/// - Left Split (14%): `[Hierarchy, Stats]`
/// - Right Split (18% of remaining): `[Inspector, MaterialEditor]`
/// - Bottom Split (20% of center): `[Assets, Console, AnimationTimeline]`
pub fn create_default_dock_state() -> DockState<PanelId> {
    DockState::from_layout(DockLayoutNode::split_horizontal(
        0.14,
        DockLayoutNode::tabs(vec![PanelId::Hierarchy, PanelId::Stats]),
        DockLayoutNode::split_horizontal(
            0.82,
            DockLayoutNode::split_vertical(
                0.80,
                DockLayoutNode::tabs(vec![PanelId::Viewport, PanelId::UiDesigner]),
                DockLayoutNode::tabs(vec![
                    PanelId::Assets,
                    PanelId::Console,
                    PanelId::AnimationTimeline,
                ]),
            ),
            DockLayoutNode::tabs(vec![PanelId::Inspector, PanelId::MaterialEditor]),
        ),
    ))
    .with_min_pane_size(60.0)
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