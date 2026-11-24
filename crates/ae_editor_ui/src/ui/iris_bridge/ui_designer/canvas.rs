// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # 2D Visual UI Designer Virtual Canvas (`PanZoomViewport`-Backed)
//!
//! Delegates letterbox aspect-fitting, zoom/pan navigation, coordinate transforms, grid drawing,
//! draggable element boxes, and anchor pin guide overlays cleanly to
//! [`irisui::prelude::UiScope::pan_zoom_viewport`] without local layout arithmetic or `let` plumbing.

use super::types::{
    UI_DESIGNER_TAG_CANVAS_BOARD, UI_DESIGNER_TAG_LETTERBOX, UiDesignerState, UiDragState,
    encode_element_tag,
};
use crate::ui::panel_layout::PanelId;
use crate::ui::types::EngineUiAction;
use ae_plugin_api::{UiAnchor, UiElement};
use ae_ui::{UiDrawCommand, UiLayoutResolver, UiTextAlignment};
use irisui::dock::DeferredNotifier;
use irisui::prelude::*;
use irisui::workbench::ActionBus;

/// Maps an in-game [`UiTextAlignment`] into an Iris UI [`TextAlign`] token.
#[inline]
#[must_use]
const fn map_text_align(align: UiTextAlignment) -> TextAlign {
    match align {
        UiTextAlignment::Left => TextAlign::Left,
        UiTextAlignment::Center => TextAlign::Center,
        UiTextAlignment::Right => TextAlign::Right,
    }
}

/// Returns the human-readable anchor name shown next to active anchor pins.
#[inline]
#[must_use]
const fn anchor_label(anchor: UiAnchor) -> &'static str {
    match anchor {
        UiAnchor::TopLeft => "TopLeft",
        UiAnchor::TopCenter => "TopCenter",
        UiAnchor::TopRight => "TopRight",
        UiAnchor::CenterLeft => "CenterLeft",
        UiAnchor::Center => "Center",
        UiAnchor::CenterRight => "CenterRight",
        UiAnchor::BottomLeft => "BottomLeft",
        UiAnchor::BottomCenter => "BottomCenter",
        UiAnchor::BottomRight => "BottomRight",
    }
}

struct DragEmitCtx<'a> {
    drag_state: &'a mut Option<UiDragState>,
    snap_grid: Option<f32>,
    action_bus: &'a mut ActionBus<EngineUiAction>,
    notifier: &'a mut DeferredNotifier<PanelId>,
}

/// Translates an active drag gesture on `elem` into an ECS [`EngineUiAction::modify_component`] mutation.
fn apply_element_drag(
    entity: ae_ecs::Entity,
    elem: &UiElement,
    total_virtual_delta: Point,
    virtual_size: Size,
    ctx: &mut DragEmitCtx<'_>,
) {
    ctx.action_bus.emit(EngineUiAction::modify_component(
        entity,
        "UiElement",
        &UiElement {
            offset: [
                snap_scalar(
                    ctx.drag_state
                        .get_or_insert_with(|| UiDragState {
                            entity,
                            anchor_origin: elem
                                .anchor
                                .compute_origin(virtual_size.width, virtual_size.height),
                            drag_start_mouse_canvas: [0.0, 0.0],
                            initial_offset: elem.offset,
                        })
                        .initial_offset[0]
                        + total_virtual_delta.x,
                    ctx.snap_grid,
                ),
                snap_scalar(
                    ctx.drag_state
                        .filter(|d| d.entity == entity)
                        .map_or(elem.offset[1], |d| d.initial_offset[1])
                        + total_virtual_delta.y,
                    ctx.snap_grid,
                ),
            ],
            ..*elem
        },
    ));
    ctx.notifier.tag_redraw(PanelId::UiDesigner);
}

/// Helper building the configured [`PanZoomBoxSpec`] for `elem` cleanly without local `let` bindings.
fn build_element_box_spec(
    elem: &UiElement,
    ent: ae_ecs::Entity,
    virtual_size: Size,
    is_selected: bool,
    show_anchor_guides: bool,
) -> PanZoomBoxSpec {
    PanZoomBoxSpec::new(
        "UiCanvasElement",
        encode_element_tag(ent),
        [
            elem.compute_rect(virtual_size.width, virtual_size.height)
                .min_x,
            elem.compute_rect(virtual_size.width, virtual_size.height)
                .min_y,
            elem.compute_rect(virtual_size.width, virtual_size.height)
                .max_x,
            elem.compute_rect(virtual_size.width, virtual_size.height)
                .max_y,
        ],
        is_selected,
    )
    .with_anchor_guide(
        is_selected && show_anchor_guides,
        Point::new(
            elem.anchor
                .compute_origin(virtual_size.width, virtual_size.height)[0],
            elem.anchor
                .compute_origin(virtual_size.width, virtual_size.height)[1],
        ),
        anchor_label(elem.anchor),
    )
}

/// Renders one resolved in-game [`UiDrawCommand`] inside `pz` without intermediate bindings.
fn emit_draw_command(pz: &mut PanZoomScope<'_, '_>, cmd: &UiDrawCommand) {
    match cmd {
        UiDrawCommand::Rect {
            rect,
            fill_color,
            border_color,
            border_width,
            border_radius,
            ..
        } => {
            pz.draw_filled_rect_virtual(
                [rect.min_x, rect.min_y, rect.max_x, rect.max_y],
                Color::rgba(fill_color[0], fill_color[1], fill_color[2], fill_color[3]),
                (border_color[3] > 0.01 && *border_width > 0.0).then_some(Color::rgba(
                    border_color[0],
                    border_color[1],
                    border_color[2],
                    border_color[3],
                )),
                *border_width,
                *border_radius,
            );
        }
        UiDrawCommand::Text {
            pos,
            text,
            font_size,
            color,
            alignment,
            ..
        } => {
            pz.draw_text_virtual(
                Point::new(pos[0], pos[1]),
                text.as_str(),
                *font_size,
                Color::rgba(color[0], color[1], color[2], color[3]),
                map_text_align(*alignment),
            );
        }
        UiDrawCommand::Image { rect, tint, .. } => {
            pz.draw_image_rect_virtual(
                [rect.min_x, rect.min_y, rect.max_x, rect.max_y],
                Color::rgba(tint[0], tint[1], tint[2], tint[3]),
            );
        }
    }
}

/// Emits interactive hitboxes, drag handlers, selection outlines, and anchor guides for all visible [`UiElement`]s.
fn emit_canvas_elements(
    pz: &mut PanZoomScope<'_, '_>,
    world: &ae_ecs::World,
    selected_entity: Option<ae_ecs::Entity>,
    show_anchor_guides: bool,
    ctx: &mut DragEmitCtx<'_>,
) {
    for (ent, elem) in world.query::<(ae_ecs::Entity, &UiElement)>().iter() {
        if !elem.visible {
            continue;
        }
        pz.draggable_box(build_element_box_spec(
            elem,
            ent,
            pz.transform().virtual_size,
            selected_entity == Some(ent),
            show_anchor_guides,
        ))
        .on_click(|| {
            *ctx.drag_state = None;
            ctx.action_bus.emit(EngineUiAction::SelectEntity(Some(ent)));
            ctx.notifier.tag_redraw(PanelId::UiDesigner);
        })
        .on_drag_total_virtual(|total_v_delta| {
            if ctx.drag_state.is_none_or(|d| d.entity != ent) {
                *ctx.drag_state = Some(UiDragState {
                    entity: ent,
                    anchor_origin: elem.anchor.compute_origin(
                        pz.transform().virtual_size.width,
                        pz.transform().virtual_size.height,
                    ),
                    drag_start_mouse_canvas: [0.0, 0.0],
                    initial_offset: elem.offset,
                });
                ctx.action_bus.emit(EngineUiAction::SelectEntity(Some(ent)));
            }
            apply_element_drag(ent, elem, total_v_delta, pz.transform().virtual_size, ctx);
        })
        .on_drag_end(|| {
            *ctx.drag_state = None;
        });
    }
}

/// Builds the [`PanZoomViewport`](irisui::prelude::UiScope::pan_zoom_viewport)-backed virtual
/// canvas surface, element hierarchy projection, and interactive drag/selection handlers.
pub fn build_designer_canvas(
    scope: &mut UiScope<'_>,
    world: &ae_ecs::World,
    selected_entity: Option<ae_ecs::Entity>,
    state: &mut UiDesignerState,
    action_bus: &mut ActionBus<EngineUiAction>,
    notifier: &mut DeferredNotifier<PanelId>,
) -> PanZoomTransform {
    scope
        .pan_zoom_viewport(
            PanZoomSpec::new(
                UI_DESIGNER_TAG_LETTERBOX,
                UI_DESIGNER_TAG_CANVAS_BOARD,
                Size::new(
                    state.aspect_ratio.resolution()[0],
                    state.aspect_ratio.resolution()[1],
                ),
                &mut state.zoom,
                &mut state.pan_offset,
            )
            .with_names("UiDesignerLetterbox", "UiDesignerCanvasSurface")
            .grid(state.show_grid, 64.0),
            |pz| {
                for cmd in UiLayoutResolver::resolve_draw_commands(
                    world,
                    pz.transform().virtual_size.width,
                    pz.transform().virtual_size.height,
                    pz.pointer_virtual_pos().map(|p| [p.x, p.y]),
                    false,
                )
                .iter()
                {
                    emit_draw_command(pz, cmd);
                }

                emit_canvas_elements(
                    pz,
                    world,
                    selected_entity,
                    state.show_anchor_guides,
                    &mut DragEmitCtx {
                        drag_state: &mut state.drag_state,
                        snap_grid: state.snap_grid,
                        action_bus,
                        notifier,
                    },
                );
            },
        )
        .on_view_changed(|| notifier.tag_redraw(PanelId::UiDesigner))
        .on_background_click(|| {
            state.drag_state = None;
            action_bus.emit(EngineUiAction::SelectEntity(None));
            notifier.tag_redraw(PanelId::UiDesigner);
        })
        .transform
}