// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Dynamic Reflection Component Card Builder
//!
//! Automatically inspects and renders cards for any ECS components registered
//! in `ComponentRegistry` that lack dedicated static `ComponentInspectorHandler` cards.

use super::components::physics::helpers::render_component_header;
use super::registry::{ComponentRenderContext, InspectorRegistry};
use irisui::prelude::*;

/// Renders dynamic reflection cards for all attached components from `ComponentRegistry`
/// that lack a dedicated static card handler in `InspectorRegistry`.
pub fn render_dynamic_component_cards(
    tree: &mut UiTree,
    parent_id: WidgetId,
    ctx: &mut ComponentRenderContext<'_>,
) -> f32 {
    let inspector_registry = InspectorRegistry::global();
    let handled_names: std::collections::HashSet<_> = inspector_registry
        .handlers()
        .iter()
        .map(|h| h.component_name())
        .collect();

    let comp_registry = ae_core::registry::ComponentRegistry::global();
    let mut total_added_h = 0.0;
    let card_gap = 6.0;
    let padding = 8.0;

    for handler in comp_registry.handlers() {
        let type_name = handler.type_name();
        if handled_names.contains(type_name)
            || crate::ui::panels::inspector::dynamic_reflection::is_internal_or_specialized(
                type_name,
            )
            || !handler.has_component(ctx.world, ctx.entity)
        {
            continue;
        }

        // Card header + content height
        let card_h = 24.0 + padding * 2.0 + 24.0;
        let card_rect = Rect::new(ctx.base_x, ctx.base_y, ctx.card_w, card_h);

        let card_id = tree.create_node();
        if let Some(node) = tree.get_mut(card_id) {
            node.set_name(format!("DynamicCard_{}", type_name));
            node.computed_rect = card_rect;
            node.style = Style::new()
                .background(Color::rgba(0.090, 0.094, 0.110, 0.98))
                .border(1.0, Color::rgba(0.133, 0.141, 0.165, 0.85))
                .border_radius(6.0);
        }
        let _ = tree.add_child(parent_id, card_id);

        render_component_header(
            tree,
            card_id,
            ctx,
            "🧩",
            type_name,
            Color::rgba(0.40, 0.80, 0.90, 1.0),
            type_name,
        );

        // Body: show JSON serialized representation or marker info
        let desc_id = tree.create_node();
        if let Some(node) = tree.get_mut(desc_id) {
            node.set_name("DynamicCardSummary");
            let summary = if let Some(bytes) = handler.capture(ctx.world, ctx.entity) {
                if let Ok(val) = serde_json::from_slice::<serde_json::Value>(&bytes) {
                    match val {
                        serde_json::Value::Object(map) if map.is_empty() => {
                            "Marker Component (No data fields)".to_string()
                        }
                        serde_json::Value::Object(map) => {
                            let parts: Vec<String> =
                                map.iter().map(|(k, v)| format!("{}: {}", k, v)).collect();
                            parts.join("  |  ")
                        }
                        other => other.to_string(),
                    }
                } else {
                    "Dynamic ECS Component".to_string()
                }
            } else {
                "Dynamic ECS Component".to_string()
            };

            node.set_text(summary);
            node.font_size = 10.5;
            node.line_height = 20.0;
            node.text_color = Color::rgba(0.620, 0.635, 0.678, 1.0);
            node.computed_rect = Rect::new(
                ctx.base_x + padding,
                ctx.base_y + padding + 24.0 + 2.0,
                ctx.card_w - padding * 2.0,
                20.0,
            );
        }
        let _ = tree.add_child(card_id, desc_id);

        let step_h = card_h + card_gap;
        ctx.base_y += step_h;
        total_added_h += step_h;
    }

    total_added_h
}