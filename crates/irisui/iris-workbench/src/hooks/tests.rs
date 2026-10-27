// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

use super::*;
use iris_core::{Color, Rect, UiTree};
use iris_widgets::canvas::ImmediateCanvas;

#[derive(Debug, Clone, PartialEq, Eq)]
struct TestAssetPayload {
    asset_id: u32,
    relative_path: String,
}

#[test]
fn test_external_texture_hook_aspect_ratio_fitting() {
    // 1. Without aspect ratio constraint: returns exact container rect
    let container = Rect::new(50.0, 50.0, 1000.0, 500.0);
    let hook = ExternalTextureHook::new(101, container);
    assert_eq!(hook.fit_aspect_ratio(), container);
    assert!(hook.is_valid());

    // 2. Wide container (1000x500 = 2.0) with 16:9 ratio (1.7778): pillarbox
    // Container is wider than 16:9 -> fits height (500), width = 500 * (16/9) = 888.8889
    let hook_16_9 = hook.with_aspect_ratio(16.0 / 9.0);
    let fitted_wide = hook_16_9.fit_aspect_ratio();
    assert_eq!(fitted_wide.height, 500.0);
    let expected_width = 500.0 * (16.0 / 9.0);
    assert!((fitted_wide.width - expected_width).abs() < 1e-3);
    let expected_x = 50.0 + (1000.0 - expected_width) * 0.5;
    assert!((fitted_wide.x - expected_x).abs() < 1e-3);
    assert_eq!(fitted_wide.y, 50.0);

    // 3. Tall container (400x800 = 0.5) with 16:9 ratio (1.7778): letterbox
    // Container is taller than 16:9 -> fits width (400), height = 400 / (16/9) = 225.0
    let tall_container = Rect::new(10.0, 20.0, 400.0, 800.0);
    let hook_tall = ExternalTextureHook::new(102, tall_container).with_aspect_ratio(16.0 / 9.0);
    let fitted_tall = hook_tall.fit_aspect_ratio();
    assert_eq!(fitted_tall.width, 400.0);
    let expected_height = 400.0 / (16.0 / 9.0);
    assert!((fitted_tall.height - expected_height).abs() < 1e-3);
    let expected_y = 20.0 + (800.0 - expected_height) * 0.5;
    assert!((fitted_tall.y - expected_y).abs() < 1e-3);
    assert_eq!(fitted_tall.x, 10.0);

    // 4. Zero and negative edge cases
    let zero_hook = ExternalTextureHook::new(103, Rect::ZERO).with_aspect_ratio(16.0 / 9.0);
    assert_eq!(zero_hook.fit_aspect_ratio(), Rect::ZERO);
    assert!(!zero_hook.is_valid());

    let neg_ratio_hook = ExternalTextureHook::new(104, container).with_aspect_ratio(-1.0);
    assert_eq!(neg_ratio_hook.fit_aspect_ratio(), container);
}

#[test]
fn test_drag_drop_payload_dynamic_and_typed() {
    let original_data = TestAssetPayload {
        asset_id: 42,
        relative_path: "models/player.gltf".to_string(),
    };

    // 1. Create erased payload
    let mut payload = DragDropPayload::new(original_data.clone(), "Player Model", "AssetBrowser");

    assert_eq!(payload.preview_label(), "Player Model");
    assert_eq!(payload.source_panel(), "AssetBrowser");
    assert!(payload.is::<TestAssetPayload>());
    assert!(!payload.is::<String>());
    assert!(!payload.is::<u32>());

    // 2. Downcast reference
    let borrowed = payload
        .downcast_ref::<TestAssetPayload>()
        .expect("Must borrow");
    assert_eq!(borrowed.asset_id, 42);
    assert_eq!(borrowed.relative_path, "models/player.gltf");

    // 3. Mutable downcast reference
    if let Some(mut_borrowed) = payload.downcast_mut::<TestAssetPayload>() {
        mut_borrowed.asset_id = 99;
    }
    assert_eq!(
        payload.downcast_ref::<TestAssetPayload>().unwrap().asset_id,
        99
    );

    // 4. Downcast value
    let unboxed = payload
        .downcast::<TestAssetPayload>()
        .expect("Downcast must succeed");
    assert_eq!(unboxed.asset_id, 99);

    // 5. TypedDragDropPayload roundtrip
    let typed = TypedDragDropPayload::new(
        TestAssetPayload {
            asset_id: 777,
            relative_path: "textures/albedo.png".to_string(),
        },
        "Albedo Texture",
        "Inspector",
    );
    let erased = typed.into_erased();
    assert!(erased.is::<TestAssetPayload>());

    let restored =
        TypedDragDropPayload::<TestAssetPayload>::try_from_erased(erased).expect("Must restore");
    assert_eq!(restored.data.asset_id, 777);
    assert_eq!(restored.preview_label, "Albedo Texture");
    assert_eq!(restored.source_panel, "Inspector");
}

#[test]
fn test_drag_drop_payload_downcast_failure_preserves_carrier() {
    let payload = DragDropPayload::new(12345_u32, "Count", "Metrics");
    let failed = payload.downcast::<String>();
    assert!(failed.is_err());
    let recovered = failed.unwrap_err();
    assert_eq!(recovered.preview_label(), "Count");
    assert_eq!(*recovered.downcast_ref::<u32>().unwrap(), 12345);
}

#[test]
fn test_custom_overlay_hook_execution() {
    let mut tree = UiTree::new();
    let node_id = tree.create_node();
    let bounds = Rect::new(10.0, 10.0, 200.0, 100.0);
    let mut canvas = ImmediateCanvas::new(&mut tree, node_id, bounds);

    // 1. Closure hook
    let mut invoked = false;
    let mut closure_hook = |c: &mut ImmediateCanvas<'_>, b: Rect| {
        c.fill_rect(b, Color::WHITE, 0.0);
        invoked = true;
    };

    closure_hook.render_overlay(&mut canvas, bounds);
    assert!(invoked);

    // 2. Boxed trait object hook
    struct TelemetryOverlay {
        sample_count: usize,
    }

    impl CustomOverlayHook for TelemetryOverlay {
        fn render_overlay(&mut self, canvas: &mut ImmediateCanvas<'_>, bounds: Rect) {
            self.sample_count += 1;
            canvas.draw_line(
                iris_core::Point::new(bounds.x, bounds.y),
                iris_core::Point::new(bounds.x + 50.0, bounds.y + 50.0),
                Color::BLACK,
                2.0,
            );
        }
    }

    let mut boxed_hook: Box<dyn CustomOverlayHook> = Box::new(TelemetryOverlay { sample_count: 0 });
    boxed_hook.render_overlay(&mut canvas, bounds);
    // Tree has recorded volatile canvas primitives
    assert!(tree.volatile_canvas_commands.contains_key(&node_id));
}

#[test]
fn test_external_texture_hook_render() {
    let mut tree = UiTree::new();
    let root = tree.create_node();
    let mut scope = iris_widgets::declarative::UiScope::new(&mut tree, root);

    let hook = ExternalTextureHook::new(42, Rect::new(10.0, 20.0, 800.0, 600.0));
    hook.render(&mut scope);

    assert_eq!(tree.get(root).unwrap().children.len(), 1);
    let child_id = tree.get(root).unwrap().children[0];
    let child = tree.get(child_id).unwrap();
    assert_eq!(
        child.external_texture,
        Some(iris_core::ExternalTextureId(42))
    );
}

#[test]
fn test_workbench_ui_scope_ext_viewport_surface_and_overlay() {
    let mut tree = UiTree::new();
    let root = tree.create_node();
    let mut scope = iris_widgets::declarative::UiScope::new(&mut tree, root);

    let hook = ExternalTextureHook::new(88, Rect::new(0.0, 0.0, 1280.0, 720.0));
    let mut overlay_rendered = false;

    // Test declarative viewport surface with child volatile overlay
    scope.viewport_surface(Some(hook), Some("Fallback"), |vp| {
        vp.volatile_overlay(
            "TestOverlay",
            1001,
            Rect::new(10.0, 10.0, 100.0, 100.0),
            |canvas| {
                overlay_rendered = true;
                canvas.draw_line(
                    iris_core::Point::new(0.0, 0.0),
                    iris_core::Point::new(10.0, 10.0),
                    Color::WHITE,
                    1.0,
                );
            },
        );
    });

    assert!(overlay_rendered, "Volatile overlay closure must execute");
    assert_eq!(tree.get(root).unwrap().children.len(), 1);
}