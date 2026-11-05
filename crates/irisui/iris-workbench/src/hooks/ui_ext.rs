// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Workbench UI Extensions
//!
//! Provides high-level declarative extension methods on [`UiScope`] for embedding
//! 3D viewport surfaces, aspect-ratio-fitted external GPU textures, volatile
//! immediate hardware overlays, and floating drag-and-drop preview capsules with zero boilerplate.

use iris_core::Color;
use iris_core::geometry::{Point, Rect};
use iris_core::style::{AlignItems, JustifyContent, Style, TextAlign};
use iris_widgets::canvas::ImmediateCanvas;
use iris_widgets::declarative::UiScope;

use super::drag_drop::{DragDropController, DragDropPayload, DragDropPreviewStyle};
use super::external_texture::{AspectRatio, ExternalTextureView};
use super::overlay::CustomOverlayHook;
use super::texture::ExternalTextureHook;
use crate::dialogs::{
    ConfirmDialogDescriptor, InputDialogDescriptor, ModalController, ModalDescriptor, ModalOutcome,
};

/// Extension trait providing high-level declarative workbench primitives on [`UiScope`].
///
/// Designed to eliminate boilerplate glue code on the application and engine side.
/// Instead of manually defining custom hook structs, managing absolute container styles,
/// and wiring low-level texture IDs, consumers call [`Self::viewport_surface`],
/// [`Self::volatile_overlay`], [`Self::drag_drop_preview`], [`Self::workbench_modal`],
/// [`Self::confirm_modal`], and [`Self::input_modal`] declaratively.
pub trait WorkbenchUiScopeExt {
    /// Declaratively presents an external GPU hardware texture inside the current container scope.
    ///
    /// Automatically calculates mathematical aspect ratio fitting (letterbox/pillarbox/fill/stretch)
    /// without requiring engine code to compute manual offsets or scissoring rects.
    ///
    /// # Arguments
    /// * `texture_id` - 64-bit hardware texture key recognized by the GPU renderer backend.
    /// * `width` - Source texture native width in pixels.
    /// * `height` - Source texture native height in pixels.
    /// * `aspect_ratio` - Desired [`AspectRatio`] constraint policy.
    fn external_texture(
        &mut self,
        texture_id: u64,
        width: f32,
        height: f32,
        aspect_ratio: AspectRatio,
    );

    /// Declaratively presents an external GPU texture using an [`ExternalTextureView`] component.
    fn external_texture_view(&mut self, view: ExternalTextureView);

    /// Declaratively renders a hardware-accelerated viewport surface with optional aspect ratio
    /// fitting, fallback placeholder messaging, and a nested content scope for overlays.
    ///
    /// When `texture` is `Some(hook)`, renders the GPU texture quad with aspect ratio preservation.
    /// When `texture` is `None` and `placeholder_text` is provided, displays centered fallback text.
    /// The `content_fn` closure receives an inner [`UiScope`] allowing child HUD elements,
    /// floating toolbars, and buttons to be declared on top of the viewport canvas.
    ///
    /// # Arguments
    /// * `texture` - Optional [`ExternalTextureHook`] descriptor.
    /// * `placeholder_text` - Human-readable label displayed when `texture` is `None`.
    /// * `content_fn` - Declarative closure building viewport overlays, toolbars, and controls.
    fn viewport_surface<F>(
        &mut self,
        texture: Option<ExternalTextureHook>,
        placeholder_text: Option<&str>,
        content_fn: F,
    ) where
        F: FnOnce(&mut UiScope<'_>);

    /// Declaratively embeds a volatile immediate-mode hardware overlay directly into an
    /// [`ImmediateCanvas`] without persistent child nodes in the tree.
    ///
    /// # Arguments
    /// * `name` - Diagnostic debug name for the canvas container.
    /// * `tag` - 64-bit semantic identifier for hit-testing and tag retrieval.
    /// * `rect` - Bounding rectangle of the overlay in panel coordinates.
    /// * `draw_fn` - Immediate drawing closure executed to populate per-frame volatile primitives.
    fn volatile_overlay<F>(&mut self, name: &'static str, tag: u64, rect: Rect, draw_fn: F)
    where
        F: FnOnce(&mut ImmediateCanvas<'_>);

    /// Declaratively embeds a volatile overlay using a type implementing [`CustomOverlayHook`].
    ///
    /// # Arguments
    /// * `name` - Diagnostic debug name for the canvas container.
    /// * `tag` - 64-bit semantic identifier for hit-testing and tag retrieval.
    /// * `rect` - Bounding rectangle of the overlay in panel coordinates.
    /// * `hook` - Concrete type implementing [`CustomOverlayHook`].
    fn volatile_overlay_hook<H>(&mut self, name: &'static str, tag: u64, rect: Rect, hook: H)
    where
        H: CustomOverlayHook;

    /// Declaratively renders the active floating drag-and-drop preview capsule at the specified cursor position.
    ///
    /// If `controller.is_dragging()` is true, renders an elevated floating container next to `cursor_pos`
    /// displaying the category badge (if present) and the preview label text.
    ///
    /// # Arguments
    /// * `controller` - Drag-and-drop state controller.
    /// * `cursor_pos` - Current cursor location in window coordinates.
    /// * `style` - Optional visual styling overrides; uses default studio theme if `None`.
    fn drag_drop_preview(
        &mut self,
        controller: &DragDropController,
        cursor_pos: Point,
        style: Option<&DragDropPreviewStyle>,
    );

    /// Declaratively renders a customized floating drag-and-drop preview at the cursor position.
    ///
    /// Executes the `custom_draw` closure inside a floating container positioned at the cursor,
    /// allowing applications to display custom thumbnails, 3D landing rings, or multi-item stacks.
    ///
    /// # Arguments
    /// * `controller` - Drag-and-drop state controller.
    /// * `cursor_pos` - Current cursor location in window coordinates.
    /// * `custom_draw` - Drawing closure executed with the active payload.
    fn drag_drop_preview_custom<F>(
        &mut self,
        controller: &DragDropController,
        cursor_pos: Point,
        custom_draw: F,
    ) where
        F: FnOnce(&mut UiScope<'_>, &DragDropPayload);

    /// Declaratively renders an interactive modal dialogue into this scope.
    fn workbench_modal<Action, F>(
        &mut self,
        descriptor: &ModalDescriptor,
        viewport: Rect,
        escape_pressed: bool,
        content: F,
    ) -> ModalOutcome<Action>
    where
        F: FnOnce(&mut UiScope<'_>) -> ModalOutcome<Action>;

    /// Declaratively renders a confirmation or danger dialogue card into this scope.
    fn confirm_modal(
        &mut self,
        descriptor: &ConfirmDialogDescriptor,
        viewport: Rect,
        escape_pressed: bool,
    ) -> ModalOutcome<bool>;

    /// Declaratively renders a single-line text input dialogue card into this scope.
    fn input_modal(
        &mut self,
        descriptor: &InputDialogDescriptor,
        input_buffer: &mut String,
        viewport: Rect,
        escape_pressed: bool,
    ) -> ModalOutcome<String>;

    /// Declaratively renders the entire application shell chrome (top menubar, bottom statusbar,
    /// and baseline dividers) in a single unified invocation from [`WorkbenchDescriptor`].
    ///
    /// # Arguments
    /// * `descriptor` - Configured workbench shell descriptor.
    /// * `screen_width` - Available width in physical pixels.
    /// * `screen_height` - Available window height in physical pixels.
    ///
    /// Returns the semantic interaction tag of any clicked menu action, or `None`.
    fn workbench_chrome<TabId>(
        &mut self,
        descriptor: &crate::descriptors::WorkbenchDescriptor<TabId>,
        screen_width: f32,
        screen_height: f32,
    ) -> Option<u64>;

    /// Declaratively renders the application shell chrome and dispatches any triggered menu action
    /// directly into the provided [`crate::action_bus::ActionBus`].
    ///
    /// # Type Parameters
    /// * `TabId` - Identifier type representing dock panels and tabs.
    /// * `Action` - Strongly-typed action type processed by the application event bus.
    /// * `F` - Mapping closure translating raw 64-bit tags to concrete action payloads.
    ///
    /// # Arguments
    /// * `descriptor` - Configured workbench shell descriptor.
    /// * `screen_width` - Available width in physical pixels.
    /// * `screen_height` - Available window height in physical pixels.
    /// * `action_bus` - Target event bus receiving dispatched actions.
    /// * `map_tag` - Closure converting a triggered semantic tag to an optional action.
    ///
    /// Returns the semantic interaction tag of any clicked menu action, or `None`.
    fn workbench_chrome_with_action_bus<TabId, Action, F>(
        &mut self,
        descriptor: &crate::descriptors::WorkbenchDescriptor<TabId>,
        screen_width: f32,
        screen_height: f32,
        action_bus: &mut crate::action_bus::ActionBus<Action>,
        map_tag: F,
    ) -> Option<u64>
    where
        Action: 'static + Send + Sync,
        F: FnOnce(u64) -> Option<Action>;
}

impl<'a> WorkbenchUiScopeExt for UiScope<'a> {
    fn external_texture(
        &mut self,
        texture_id: u64,
        width: f32,
        height: f32,
        aspect_ratio: AspectRatio,
    ) {
        let view =
            ExternalTextureView::new(texture_id, width, height).with_aspect_ratio(aspect_ratio);
        view.render(self);
    }

    fn external_texture_view(&mut self, view: ExternalTextureView) {
        view.render(self);
    }

    fn viewport_surface<F>(
        &mut self,
        texture: Option<ExternalTextureHook>,
        placeholder_text: Option<&str>,
        content_fn: F,
    ) where
        F: FnOnce(&mut UiScope<'_>),
    {
        let parent_rect = self.parent_rect();
        let root_style = Style::new()
            .position_absolute()
            .left(0.0)
            .top(0.0)
            .width(parent_rect.width)
            .height(parent_rect.height)
            .background(Color::TRANSPARENT)
            .clip_children(true);

        self.container_passive(root_style, |vp_scope| {
            if let Some(tex) = texture {
                tex.render(vp_scope);
            } else if let Some(placeholder) = placeholder_text {
                let ph_style = Style::new()
                    .position_absolute()
                    .left(0.0)
                    .top(0.0)
                    .width(parent_rect.width)
                    .height(parent_rect.height)
                    .justify_content(JustifyContent::Center)
                    .align_items(AlignItems::Center);
                vp_scope.container_passive(ph_style, |ph| {
                    ph.label_custom(placeholder, 14.0, Color::hex("#888888"), TextAlign::Center);
                });
            }

            content_fn(vp_scope);
        });
    }

    fn volatile_overlay<F>(&mut self, name: &'static str, tag: u64, rect: Rect, draw_fn: F)
    where
        F: FnOnce(&mut ImmediateCanvas<'_>),
    {
        let style = Style::new()
            .position_absolute()
            .left(rect.x)
            .top(rect.y)
            .width(rect.width)
            .height(rect.height);

        self.immediate_canvas_styled(name, tag, style, draw_fn);
    }

    fn volatile_overlay_hook<H>(&mut self, name: &'static str, tag: u64, rect: Rect, mut hook: H)
    where
        H: CustomOverlayHook,
    {
        let style = Style::new()
            .position_absolute()
            .left(rect.x)
            .top(rect.y)
            .width(rect.width)
            .height(rect.height);

        self.immediate_canvas_styled(name, tag, style, |canvas| {
            hook.render_overlay(canvas, rect);
        });
    }

    fn drag_drop_preview(
        &mut self,
        controller: &DragDropController,
        cursor_pos: Point,
        style: Option<&DragDropPreviewStyle>,
    ) {
        let Some(payload) = controller.active_payload() else {
            return;
        };

        let default_style = DragDropPreviewStyle::default();
        let s = style.unwrap_or(&default_style);
        let pos = Point::new(cursor_pos.x + s.offset.x, cursor_pos.y + s.offset.y);

        let container_style = Style::new()
            .position_absolute()
            .left(pos.x)
            .top(pos.y)
            .flex_row()
            .align_items(AlignItems::Center)
            .padding_insets(s.padding)
            .gap(s.gap)
            .background(s.background)
            .border(s.border_width, s.border_color)
            .border_radius(s.corner_radius);

        self.container_measured(
            "DragDropPreviewCapsule",
            container_style,
            pos,
            24.0,
            |capsule| {
                if let (Some(badge), Some(color)) = (payload.badge_text(), payload.badge_color()) {
                    capsule.colored_label(color, badge);
                }
                capsule.label_custom(
                    payload.preview_label(),
                    s.font_size,
                    s.text_color,
                    TextAlign::Left,
                );
            },
        );
    }

    fn drag_drop_preview_custom<F>(
        &mut self,
        controller: &DragDropController,
        _cursor_pos: Point,
        custom_draw: F,
    ) where
        F: FnOnce(&mut UiScope<'_>, &DragDropPayload),
    {
        let Some(payload) = controller.active_payload() else {
            return;
        };

        custom_draw(self, payload);
    }

    fn workbench_modal<Action, F>(
        &mut self,
        descriptor: &ModalDescriptor,
        viewport: Rect,
        escape_pressed: bool,
        content: F,
    ) -> ModalOutcome<Action>
    where
        F: FnOnce(&mut UiScope<'_>) -> ModalOutcome<Action>,
    {
        ModalController::render_modal(self, descriptor, viewport, escape_pressed, content)
    }

    fn confirm_modal(
        &mut self,
        descriptor: &ConfirmDialogDescriptor,
        viewport: Rect,
        escape_pressed: bool,
    ) -> ModalOutcome<bool> {
        ModalController::render_confirm_dialog(self, descriptor, viewport, escape_pressed)
    }

    fn input_modal(
        &mut self,
        descriptor: &InputDialogDescriptor,
        input_buffer: &mut String,
        viewport: Rect,
        escape_pressed: bool,
    ) -> ModalOutcome<String> {
        ModalController::render_input_dialog(
            self,
            descriptor,
            input_buffer,
            viewport,
            escape_pressed,
        )
    }

    fn workbench_chrome<TabId>(
        &mut self,
        descriptor: &crate::descriptors::WorkbenchDescriptor<TabId>,
        screen_width: f32,
        screen_height: f32,
    ) -> Option<u64> {
        descriptor.render_chrome(self, screen_width, screen_height)
    }

    fn workbench_chrome_with_action_bus<TabId, Action, F>(
        &mut self,
        descriptor: &crate::descriptors::WorkbenchDescriptor<TabId>,
        screen_width: f32,
        screen_height: f32,
        action_bus: &mut crate::action_bus::ActionBus<Action>,
        map_tag: F,
    ) -> Option<u64>
    where
        Action: 'static + Send + Sync,
        F: FnOnce(u64) -> Option<Action>,
    {
        descriptor.render_chrome_with_action_bus(
            self,
            screen_width,
            screen_height,
            action_bus,
            map_tag,
        )
    }
}