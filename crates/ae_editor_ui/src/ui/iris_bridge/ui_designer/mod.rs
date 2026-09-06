// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # 2D Visual UI Designer & HUD Studio Native Iris Bridge
//!
//! Provides the complete 100% native Iris UI GPU SDF implementation of the
//! 2D in-game UI Designer and interactive WYSIWYG canvas studio.
//!

pub mod anchors;
pub mod canvas;
pub mod events;
pub mod panel;
pub mod popups;
pub mod toolbar;
pub mod types;

pub use events::{
    UiDesignerClickResult, handle_ui_designer_click, handle_ui_designer_drag,
    handle_ui_designer_scroll,
};
pub use panel::build_ui_designer_panel;
pub use toolbar::UI_DESIGNER_TOOLBAR_HEIGHT;
pub use types::{
    CanvasAspectRatio, UiDesignerAction, UiDesignerPanelParams, UiDesignerPanelTargets,
    UiDesignerState, UiDragState, UiElementType,
};

#[cfg(test)]
mod tests {
    use super::*;
    use ae_core::ecs::{UiAnchor, UiButton, UiElement};
    use irisui::prelude::*;

    #[test]
    fn test_ui_designer_panel_build() {
        let mut tree = UiTree::new();
        let root = tree.create_node();
        let world = hecs::World::new();
        let state = UiDesignerState::default();
        let panel_rect = Rect::new(50.0, 50.0, 800.0, 600.0);

        let params = UiDesignerPanelParams {
            panel_rect,
            world: &world,
            selected_entity: None,
            cursor_pos: Point::new(0.0, 0.0),
            state: &state,
            is_aspect_dropdown_open: false,
            is_add_menu_open: false,
        };

        let targets = build_ui_designer_panel(&mut tree, root, &params);

        assert_eq!(targets.panel_rect, panel_rect);
        assert!(targets.btn_aspect.is_some());
        assert!(targets.btn_zoom_in.is_some());
        assert!(targets.btn_zoom_out.is_some());
        assert!(targets.btn_zoom_reset.is_some());
        assert!(targets.btn_snap.is_some());
        assert!(targets.btn_anchors.is_some());
        assert!(targets.btn_grid.is_some());
        assert!(targets.btn_add_element.is_some());
        assert!(targets.canvas_rect.width > 0.0);
        assert!(targets.canvas_rect.height > 0.0);
        assert!(targets.base_scale > 0.0);
    }

    #[test]
    fn test_ui_designer_canvas_projection() {
        let mut tree = UiTree::new();
        let root = tree.create_node();
        let mut world = hecs::World::new();

        let ent = world.spawn((
            UiElement {
                anchor: UiAnchor::Center,
                offset: [0.0, 0.0],
                size: [200.0, 50.0],
                pivot: [0.5, 0.5],
                visible: true,
                z_index: 0,
                alpha: 1.0,
            },
            UiButton {
                text: "Start Game".to_string(),
                ..Default::default()
            },
        ));

        let state = UiDesignerState::default();
        let panel_rect = Rect::new(0.0, 0.0, 1024.0, 768.0);

        let params = UiDesignerPanelParams {
            panel_rect,
            world: &world,
            selected_entity: Some(ent),
            cursor_pos: Point::new(512.0, 384.0),
            state: &state,
            is_aspect_dropdown_open: false,
            is_add_menu_open: false,
        };

        let targets = build_ui_designer_panel(&mut tree, root, &params);
        assert_eq!(targets.element_rects.len(), 1);
        let (found_ent, found_rect) = targets.element_rects[0];
        assert_eq!(found_ent, ent);
        assert!(found_rect.width > 0.0);
        assert!(found_rect.height > 0.0);
    }

    #[test]
    fn test_ui_designer_click_hit_testing() {
        let mut tree = UiTree::new();
        let root = tree.create_node();
        let mut world = hecs::World::new();
        let ent = world.spawn((UiElement {
            anchor: UiAnchor::TopLeft,
            offset: [100.0, 100.0],
            size: [150.0, 40.0],
            pivot: [0.0, 0.0],
            visible: true,
            z_index: 0,
            alpha: 1.0,
        },));

        let state = UiDesignerState::default();
        let panel_rect = Rect::new(0.0, 0.0, 1000.0, 800.0);

        let params = UiDesignerPanelParams {
            panel_rect,
            world: &world,
            selected_entity: None,
            cursor_pos: Point::new(0.0, 0.0),
            state: &state,
            is_aspect_dropdown_open: false,
            is_add_menu_open: false,
        };

        let targets = build_ui_designer_panel(&mut tree, root, &params);

        // Click on aspect ratio button
        let aspect_btn_p = Point::new(
            targets.btn_aspect.map_or(0.0, |r| r.x + 5.0),
            targets.btn_aspect.map_or(0.0, |r| r.y + 5.0),
        );
        let click_res = handle_ui_designer_click(aspect_btn_p, &targets, false, false);
        assert_eq!(
            click_res.action,
            Some(UiDesignerAction::ToggleAspectDropdown)
        );

        // Click on element
        let elem_rect = targets.element_rects[0].1;
        let elem_p = Point::new(elem_rect.x + 10.0, elem_rect.y + 10.0);
        let elem_click = handle_ui_designer_click(elem_p, &targets, false, false);
        assert_eq!(
            elem_click.action,
            Some(UiDesignerAction::SelectEntity(Some(ent)))
        );
        assert!(elem_click.start_element_drag.is_some());
    }

    #[test]
    fn test_ui_designer_element_drag() {
        let targets = UiDesignerPanelTargets {
            panel_rect: Rect::new(0.0, 0.0, 800.0, 600.0),
            canvas_rect: Rect::new(100.0, 100.0, 600.0, 400.0),
            resolution: [1920.0, 1080.0],
            snap_grid: Some(16.0),
            current_zoom: 1.0,
            ..Default::default()
        };

        let mut world = hecs::World::new();
        let ent = world.spawn(());

        let drag_state = UiDragState {
            entity: ent,
            anchor_origin: [0.0, 0.0],
            drag_start_mouse_canvas: [500.0, 500.0],
            initial_offset: [100.0, 100.0],
        };

        // Move cursor 100px to the right in canvas space
        // Screen coords: canvas_rect.x + (600.0 / 1920.0) * canvas_rect.width
        let cursor_x = targets.canvas_rect.x + (600.0 / 1920.0) * targets.canvas_rect.width;
        let cursor_y = targets.canvas_rect.y + (500.0 / 1080.0) * targets.canvas_rect.height;

        let action = handle_ui_designer_drag(
            Point::new(cursor_x, cursor_y),
            [0.0, 0.0],
            Some(&drag_state),
            false,
            &targets,
        );

        match action {
            Some(UiDesignerAction::UpdateElementOffset { entity, offset }) => {
                assert_eq!(entity, ent);
                // initial 100 + delta 100 = 200, snapped to 16 -> (200 / 16).round() * 16 = 13 * 16 = 208 or 192 or 200
                assert!((offset[0] - 200.0).abs() < 17.0);
            }
            _ => panic!("Expected UpdateElementOffset action"),
        }
    }

    #[test]
    fn test_ui_designer_toolbar_layout() {
        let mut tree = UiTree::new();
        let root = tree.create_node();
        let world = hecs::World::new();
        let state = UiDesignerState::default();
        let panel_rect = Rect::new(0.0, 0.0, 1200.0, 800.0);

        let params = UiDesignerPanelParams {
            panel_rect,
            world: &world,
            selected_entity: None,
            cursor_pos: Point::new(0.0, 0.0),
            state: &state,
            is_aspect_dropdown_open: false,
            is_add_menu_open: false,
        };

        let targets = build_ui_designer_panel(&mut tree, root, &params);

        let aspect = targets.btn_aspect.expect("Aspect button target must exist");
        assert!(
            aspect.width >= 140.0,
            "Aspect button width must accommodate full label without clipping"
        );
        assert!(aspect.height >= 24.0);

        let zoom_out = targets.btn_zoom_out.expect("Zoom out button must exist");
        let zoom_reset = targets
            .btn_zoom_reset
            .expect("Zoom reset button must exist");
        let zoom_in = targets.btn_zoom_in.expect("Zoom in button must exist");
        assert!(zoom_out.x < zoom_reset.x && zoom_reset.x < zoom_in.x);
        assert!(zoom_reset.width >= 50.0);

        let snap = targets.btn_snap.expect("Snap button must exist");
        assert!(snap.width >= 80.0);

        let anchors = targets.btn_anchors.expect("Anchors button must exist");
        assert!(anchors.width >= 78.0);

        let grid = targets.btn_grid.expect("Grid button must exist");
        assert!(grid.width >= 60.0);

        let add = targets.btn_add_element.expect("Add button must exist");
        assert!(add.width >= 110.0);
    }
}