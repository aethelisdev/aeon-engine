// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Regression tests ensuring popups and comboboxes align across displays exceeding 1920x1080.

use irisui_core::{Rect, Style, WidgetRole};
use irisui_dock::{DockState, DockTree, SplitDirection};
use irisui_widgets::declarative::PopupPlacement;

use super::Workbench;
use super::frame::{WorkbenchExternalContext, WorkbenchFrameUpdateParams};
use crate::descriptors::PanelMetadata;
use crate::descriptors::panel_registry::WorkbenchPanelRegistry;
use crate::descriptors::workbench::WorkbenchDescriptor;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum PopupTestTab {
    Viewport,
    Inspector,
}

fn run_inspector_popup_scenario(screen_w: f32, screen_h: f32) -> (Rect, Rect, Rect, Rect, Rect) {
    const FOOTER_ANCHOR_TAG: u64 = 0xBEEF_4401;
    const POPUP_NAME: &str = "InspectorFooterMenu";

    let mut dock_tree = DockTree::new();
    let left_leaf = dock_tree.create_leaf(vec![PopupTestTab::Viewport]);
    dock_tree.set_root(left_leaf);
    let _ = dock_tree.split(
        left_leaf,
        SplitDirection::Horizontal,
        0.82,
        vec![PopupTestTab::Inspector],
    );
    let dock = DockState::new(dock_tree);

    let mut wb = Workbench::<PopupTestTab, (), ()>::new(WorkbenchDescriptor::new(
        "PopupCoordSuite",
        dock.clone(),
    ))
    .with_menubar_height(36.0)
    .with_statusbar_height(22.0)
    .with_screen_size((screen_w, screen_h));

    wb.panel_registry
        .register_metadata(PanelMetadata::new(PopupTestTab::Viewport, "Viewport"));
    wb.panel_registry
        .register_metadata(PanelMetadata::new(PopupTestTab::Inspector, "Inspector"));

    let mut registry = WorkbenchPanelRegistry::<PopupTestTab, bool>::new();
    registry.register_domain_panel(PopupTestTab::Viewport, |scope, _| {
        scope.label("Viewport");
    });
    registry.register_domain_panel(PopupTestTab::Inspector, |scope, open| {
        let panel_rect = scope.parent_rect();
        scope
            .panel("InspectorPanelRoot")
            .bounds(panel_rect)
            .show(|p| {
                p.scroll_area("InspectorCardsContainer", 0xCAFE_2000, None, |cards| {
                    let mut idx = 0usize;
                    cards.prop_combobox("Shape", &mut idx, &["Box", "Sphere", "Capsule"]);
                });
                p.row_with().show(|r| {
                    r.container_tagged_popup_toggle(
                        "AddComponentBtn",
                        Style::new().width(140.0).height(24.0),
                        WidgetRole::Button,
                        FOOTER_ANCHOR_TAG,
                        FOOTER_ANCHOR_TAG,
                        |b, _| {
                            b.label("Add Component");
                        },
                    );
                });
                if *open {
                    p.scoped_popup_anchored_to_tag(
                        POPUP_NAME,
                        FOOTER_ANCHOR_TAG,
                        Rect::ZERO,
                        140.0,
                        PopupPlacement::Above,
                        |m| {
                            m.label("Physics");
                        },
                    );
                }
            });
    });

    let mut is_open = false;
    wb.update_frame_live_with_registry(
        WorkbenchFrameUpdateParams {
            dock: &dock,
            scale_factor: 1.0,
            selection_id: None,
            domain_revision: 0,
            is_cursor_occluded: false,
        },
        WorkbenchExternalContext {
            registry: &mut registry,
            context: &mut is_open,
        },
        |i| i,
        |_s, _c, _sz| {},
        |_s, _c| {},
    )
    .expect("frame 0");

    let root_id = wb.tree.root().expect("root node");
    let root_rect = wb.tree.get(root_id).expect("root node").computed_rect;

    let combo_tag = wb
        .tree
        .iter()
        .find(|(_, n)| n.name.as_deref() == Some("ComboPill"))
        .map(|(_, n)| n.tag)
        .expect("ComboPill tag");
    let combo_anchor = wb.tree.tag_rect(combo_tag).expect("ComboPill tag_rect");
    let footer_anchor = wb
        .tree
        .resolve_anchor_rect(FOOTER_ANCHOR_TAG)
        .expect("footer anchor rect");

    is_open = true;
    wb.tree.active_combobox_tag = Some(combo_tag);
    wb.tree.open_popup(FOOTER_ANCHOR_TAG);
    wb.notifier.tag_redraw(PopupTestTab::Inspector);

    wb.update_frame_live_with_registry(
        WorkbenchFrameUpdateParams {
            dock: &dock,
            scale_factor: 1.0,
            selection_id: None,
            domain_revision: 0,
            is_cursor_occluded: false,
        },
        WorkbenchExternalContext {
            registry: &mut registry,
            context: &mut is_open,
        },
        |i| i.with_popup_active(true),
        |_s, _c, _sz| {},
        |_s, _c| {},
    )
    .expect("frame 1");

    let combo_popup_rect = wb
        .tree
        .node_rect("ComboBoxPopup")
        .expect("ComboBoxPopup must exist");
    let footer_popup_rect = wb
        .tree
        .node_rect(POPUP_NAME)
        .expect("footer popup must exist");

    (
        root_rect,
        combo_anchor,
        combo_popup_rect,
        footer_anchor,
        footer_popup_rect,
    )
}

#[test]
fn test_qhd_right_dock_panel_popups_align_beyond_1920x1080() {
    let (root_rect, combo_anchor, combo_popup_rect, footer_anchor, footer_popup_rect) =
        run_inspector_popup_scenario(2516.0, 1440.0);

    assert_eq!(root_rect, Rect::new(0.0, 0.0, 2516.0, 1440.0));
    assert!(
        combo_anchor.x > 2000.0 && footer_anchor.x > 2000.0,
        "Inspector triggers must sit beyond x=2000 on 2516px display: combo={combo_anchor:?}, footer={footer_anchor:?}"
    );
    assert!(
        (combo_popup_rect.x - combo_anchor.x).abs() < 2.0,
        "ComboBoxPopup must align with trigger past 1920px: anchor={combo_anchor:?}, popup={combo_popup_rect:?}"
    );
    assert!(
        (footer_popup_rect.x - footer_anchor.x).abs() < 2.0
            && footer_popup_rect.bottom() <= footer_anchor.y
            && (footer_anchor.y - footer_popup_rect.bottom()) <= 6.0,
        "Footer popup must sit directly above trigger past 1080px: anchor={footer_anchor:?}, popup={footer_popup_rect:?}"
    );
}

#[test]
fn test_tall_portrait_footer_popup_stays_directly_above_anchor() {
    let (root_rect, _combo_anchor, _combo_popup_rect, footer_anchor, footer_popup_rect) =
        run_inspector_popup_scenario(900.0, 1600.0);

    assert_eq!(root_rect, Rect::new(0.0, 0.0, 900.0, 1600.0));
    assert!(
        footer_anchor.y > 1400.0,
        "Footer anchor must sit near bottom of 1600px viewport: {footer_anchor:?}"
    );
    assert!(
        footer_popup_rect.bottom() <= footer_anchor.y
            && (footer_anchor.y - footer_popup_rect.bottom()) <= 6.0,
        "Footer popup must sit directly above trigger rather than clamping to 1080px: anchor={footer_anchor:?}, popup={footer_popup_rect:?}"
    );
}