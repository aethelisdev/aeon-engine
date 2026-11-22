// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Unit Tests for `PanZoomViewport`
//!
//! Verifies aspect-fit math, screen/virtual coordinate roundtrips, wheel zooming,
//! middle-mouse drag panning, `draggable_box` hit/drag isolation, `pin_guide` rendering,
//! and degenerate zero-size safety guards.

use super::{PanZoomBoxSpec, PanZoomSpec, PanZoomTransform, PinGuideSpec, snap_scalar};
use crate::declarative::scope::UiScope;
use irisui_core::{MouseButton, Point, Rect, Size, UiEvent, UiTag, UiTree};

#[test]
fn test_pan_zoom_transform_roundtrip_and_zero_guards() {
    let vp = Rect::new(50.0, 84.0, 800.0, 566.0);
    let transform =
        PanZoomTransform::compute(vp, Size::new(1920.0, 1080.0), 32.0, 1.5, [20.0, -10.0]);

    assert!(transform.base_scale > 0.0);
    assert!(transform.canvas_rect.width > 0.0);
    assert!(transform.canvas_rect.height > 0.0);

    let virtual_pt = Point::new(640.0, 360.0);
    let screen_pt = transform.virtual_to_screen(virtual_pt);
    let back = transform.screen_to_virtual(screen_pt);
    assert!((back.x - virtual_pt.x).abs() < 1e-2);
    assert!((back.y - virtual_pt.y).abs() < 1e-2);

    // Zero / non-finite bounds must never produce NaN or panic
    let degenerate = PanZoomTransform::compute(
        Rect::ZERO,
        Size::new(0.0, -5.0),
        f32::NAN,
        f32::INFINITY,
        [f32::NAN, 0.0],
    );
    assert!(degenerate.base_scale.is_finite() && degenerate.base_scale > 0.0);
    assert_eq!(
        PanZoomTransform::default().screen_to_virtual(Point::new(10.0, 20.0)),
        Point::ZERO
    );
    assert_eq!(snap_scalar(13.0, Some(8.0)), 16.0);
    assert_eq!(snap_scalar(13.0, Some(0.0)), 13.0);
    assert_eq!(snap_scalar(13.0, None), 13.0);
}

#[test]
fn test_pan_zoom_viewport_wheel_zoom_and_middle_pan() {
    const VP_TAG: u64 = UiTag::from_str("test.pan_zoom.vp").raw();
    const CANVAS_TAG: u64 = UiTag::from_str("test.pan_zoom.canvas").raw();

    let mut tree = UiTree::new();
    let root = tree.create_root().expect("root");
    let panel_rect = Rect::new(0.0, 0.0, 800.0, 600.0);
    let mut zoom = 1.0;
    let mut pan_offset = [0.0, 0.0];

    {
        let mut scope = UiScope::new(&mut tree, root);
        scope.panel("Host").bounds(panel_rect).show(|p| {
            p.pan_zoom_viewport(
                PanZoomSpec::new(
                    VP_TAG,
                    CANVAS_TAG,
                    Size::new(1920.0, 1080.0),
                    &mut zoom,
                    &mut pan_offset,
                ),
                |_pz| {},
            );
        });
        scope.finish_layout(panel_rect);
    }

    // 1. Scroll up inside viewport zooms in cleanly without mutating scroll_offset_y
    let vp_rect = tree.tag_rects.get(&VP_TAG).copied().expect("vp rect");
    let center = Point::new(
        vp_rect.x + vp_rect.width * 0.5,
        vp_rect.y + vp_rect.height * 0.5,
    );
    let scroll_res = tree.dispatch_event(&UiEvent::MouseScroll {
        delta_x: 0.0,
        delta_y: 1.0,
        point: center,
    });
    assert!(scroll_res.consumed);
    assert_eq!(tree.event_state.frame_scrolled_tag, Some(VP_TAG));

    let _ = tree.clear_children(root);
    let zoomed = {
        let mut scope = UiScope::new(&mut tree, root);
        let r = scope.panel("Host").bounds(panel_rect).show(|p| {
            p.pan_zoom_viewport(
                PanZoomSpec::new(
                    VP_TAG,
                    CANVAS_TAG,
                    Size::new(1920.0, 1080.0),
                    &mut zoom,
                    &mut pan_offset,
                ),
                |_pz| {},
            )
            .view_changed
        });
        scope.finish_layout(panel_rect);
        r.inner
    };
    assert!(zoomed, "Wheel scroll must flag view_changed");
    assert!((zoom - 1.1).abs() < 1e-4, "Zoom should increase by 0.1");

    // 2. Middle-mouse drag pans viewport
    tree.clear_frame_clicks();
    let _ = tree.dispatch_event(&UiEvent::MouseDown {
        button: MouseButton::Middle,
        point: center,
    });
    let _ = tree.dispatch_event(&UiEvent::MouseMove {
        point: Point::new(center.x + 25.0, center.y - 15.0),
    });

    let _ = tree.clear_children(root);
    let panned = {
        let mut scope = UiScope::new(&mut tree, root);
        let r = scope.panel("Host").bounds(panel_rect).show(|p| {
            p.pan_zoom_viewport(
                PanZoomSpec::new(
                    VP_TAG,
                    CANVAS_TAG,
                    Size::new(1920.0, 1080.0),
                    &mut zoom,
                    &mut pan_offset,
                ),
                |_pz| {},
            )
            .view_changed
        });
        scope.finish_layout(panel_rect);
        r.inner
    };
    assert!(panned);
    assert!((pan_offset[0] - 25.0).abs() < 1e-3);
    assert!((pan_offset[1] + 15.0).abs() < 1e-3);

    let _ = tree.dispatch_event(&UiEvent::MouseUp {
        button: MouseButton::Middle,
        point: Point::new(center.x + 25.0, center.y - 15.0),
    });
}

#[test]
fn test_pan_zoom_draggable_box_and_pin_guide() {
    const VP_TAG: u64 = UiTag::from_str("test.pan_zoom.box_vp").raw();
    const CANVAS_TAG: u64 = UiTag::from_str("test.pan_zoom.box_canvas").raw();
    const ELEM_TAG: u64 = UiTag::from_str("test.pan_zoom.elem").raw();

    let mut tree = UiTree::new();
    let root = tree.create_root().expect("root");
    let panel_rect = Rect::new(0.0, 0.0, 1200.0, 800.0);
    let mut zoom = 1.0;
    let mut pan_offset = [0.0, 0.0];

    {
        let mut scope = UiScope::new(&mut tree, root);
        scope.panel("Host").bounds(panel_rect).show(|p| {
            p.pan_zoom_viewport(
                PanZoomSpec::new(
                    VP_TAG,
                    CANVAS_TAG,
                    Size::new(1920.0, 1080.0),
                    &mut zoom,
                    &mut pan_offset,
                ),
                |pz| {
                    let box_resp = pz.draggable_box(PanZoomBoxSpec::new(
                        "CanvasBox",
                        ELEM_TAG,
                        [200.0, 100.0, 400.0, 200.0],
                        true,
                    ));
                    pz.pin_guide(PinGuideSpec::new(
                        Point::new(0.0, 0.0),
                        box_resp.local_rect,
                        "TopLeft",
                    ));
                },
            );
        });
        scope.finish_layout(panel_rect);
    }

    // Center of element must hit ELEM_TAG without being occluded by passive outline or pin guide
    let elem_rect = tree.tag_rects.get(&ELEM_TAG).copied().expect("elem rect");
    let hit = tree.hit_test_target(Point::new(
        elem_rect.x + elem_rect.width * 0.5,
        elem_rect.y + elem_rect.height * 0.5,
    ));
    assert_eq!(hit.map(|r| r.tag), Some(ELEM_TAG));
    assert!(
        tree.find_node_by_name("PanZoomGridCanvas").is_some(),
        "Single ImmediateCanvas grid node must be emitted"
    );
    assert!(
        tree.find_node_by_name("PanZoomAnchorPin").is_some(),
        "Anchor pin guide must be emitted when pin_guide is called"
    );
}