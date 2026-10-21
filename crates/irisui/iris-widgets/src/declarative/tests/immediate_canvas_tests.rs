// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

use crate::declarative::UiScope;
use iris_core::canvas::CanvasPrimitive;
use iris_core::{Color, DirtyFlags, Point, Rect, Size, UiTree, WidgetRole};

#[test]
fn test_immediate_canvas_zero_persistent_child_allocation() {
    let mut tree = UiTree::new();
    let root = tree.create_root().unwrap();

    let canvas_resp = {
        let mut ui = UiScope::new(&mut tree, root);
        ui.immediate_canvas(Size::new(200.0, 100.0), |canvas| {
            // High-frequency per-frame operations: 50 lines, 20 rects
            for i in 0..50 {
                let x = i as f32 * 4.0;
                canvas.draw_line(Point::new(x, 0.0), Point::new(x, 100.0), Color::RED, 1.0);
            }
            for i in 0..20 {
                let y = i as f32 * 5.0;
                canvas.fill_rect(Rect::new(10.0, y, 30.0, 4.0), Color::BLUE, 1.0);
            }
        })
    };

    let canvas_node = tree.get(canvas_resp.id).expect("Canvas node must exist");
    assert_eq!(canvas_node.role, WidgetRole::ImmediateCanvas);

    // Guaranteed ZERO persistent child nodes in UiTree
    assert!(
        canvas_node.children.is_empty(),
        "ImmediateCanvas must not allocate child nodes in UiTree for drawn primitives"
    );

    // Total nodes in arena must strictly be 2 (root + canvas container)
    assert_eq!(
        tree.len(),
        2,
        "Total tree node count must remain 2 despite 70 drawing commands"
    );

    // Volatile primitives must be recorded in UiTree buffer
    let primitives = tree
        .canvas_primitives(canvas_resp.id)
        .expect("Primitives should be recorded");
    assert_eq!(primitives.len(), 70);
}

#[test]
fn test_immediate_canvas_zero_layout_dirty_thrashing() {
    let mut tree = UiTree::new();
    let root = tree.create_root().unwrap();

    let canvas_resp = {
        let mut ui = UiScope::new(&mut tree, root);
        ui.immediate_canvas(Size::new(300.0, 150.0), |canvas| {
            canvas.fill_rect(Rect::new(0.0, 0.0, 100.0, 50.0), Color::WHITE, 0.0);
        })
    };

    // Simulate resolved layout and clean tree
    if let Some(node) = tree.get_mut(canvas_resp.id) {
        node.computed_rect = Rect::new(0.0, 0.0, 300.0, 150.0);
        node.dirty = DirtyFlags::empty();
    }
    if let Some(r) = tree.get_mut(root) {
        r.dirty = DirtyFlags::empty();
    }

    // Re-emit volatile primitives inside immediate canvas for next frame
    let prev_dirty = tree.get(canvas_resp.id).unwrap().dirty;
    assert_eq!(prev_dirty, DirtyFlags::empty());

    // Populate new primitives through ImmediateCanvas directly
    let bounds = Rect::new(0.0, 0.0, 300.0, 150.0);
    tree.clear_canvas_primitives_for(canvas_resp.id);
    let mut canvas = crate::canvas::ImmediateCanvas::new(&mut tree, canvas_resp.id, bounds);
    canvas.stroke_rect(Rect::new(10.0, 10.0, 80.0, 40.0), Color::GREEN, 2.0, 4.0);
    canvas.draw_line(
        Point::new(0.0, 0.0),
        Point::new(100.0, 100.0),
        Color::RED,
        1.5,
    );

    // Node dirty flags must STILL be empty (zero layout invalidation / thrashing)
    assert_eq!(
        tree.get(canvas_resp.id).unwrap().dirty,
        DirtyFlags::empty(),
        "Adding immediate canvas primitives must not mark the node dirty for Taffy layout"
    );

    // Primitives correctly updated
    let primitives = tree.canvas_primitives(canvas_resp.id).unwrap();
    assert_eq!(primitives.len(), 2);
    assert!(matches!(primitives[0], CanvasPrimitive::Border { .. }));
    assert!(matches!(primitives[1], CanvasPrimitive::Line { .. }));
}

#[test]
fn test_immediate_canvas_polyline_and_chart_polyline() {
    let mut tree = UiTree::new();
    let root = tree.create_root().unwrap();

    let canvas_resp = {
        let mut ui = UiScope::new(&mut tree, root);
        ui.immediate_canvas(Size::new(200.0, 80.0), |canvas| {
            // Polyline
            let points = [
                Point::new(0.0, 0.0),
                Point::new(10.0, 20.0),
                Point::new(20.0, 10.0),
                Point::new(30.0, 30.0),
            ];
            canvas.draw_polyline(&points, Color::WHITE, 2.0);

            // Chart curve
            let sample_data = [10.0, 20.0, 15.0, 40.0, 35.0, 50.0];
            canvas.draw_chart_polyline(
                |idx| sample_data.get(idx).copied(),
                sample_data.len(),
                60.0,
                10,
                |_val| Color::CYAN,
            );
        })
    };

    let primitives = tree
        .canvas_primitives(canvas_resp.id)
        .expect("Primitives must exist");

    // Polyline with 4 points generates 3 line segments
    let line_segments = primitives
        .iter()
        .filter(|p| matches!(p, CanvasPrimitive::Line { .. }))
        .count();
    assert!(line_segments >= 3);
}