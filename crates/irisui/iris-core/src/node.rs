// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Individual widget node representations stored within the generational arena.

use crate::color::Color;
use crate::dirty::DirtyFlags;
use crate::geometry::{Point, Rect, Size};
use crate::id::WidgetId;
use crate::style::{Style, TextAlign};

/// Type-safe host identifier for externally managed 2D textures (e.g. 3D Viewport render target).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ExternalTextureId(pub u64);

/// Semantic functional role of a UI node for layout, rendering, and occlusion management.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WidgetRole {
    /// Default general-purpose container or leaf widget.
    #[default]
    Default,
    /// Thin visual divider separator.
    Separator,
    /// Small icon indicator inside a dropdown item.
    DropdownIcon,
    /// Keyboard shortcut text label inside a dropdown item.
    DropdownShortcut,
    /// Main text label inside a dropdown item.
    DropdownLabel,
    /// Clickable menu or dropdown row item.
    DropdownItem,
    /// Floating popup or dropdown menu container.
    DropdownPopup,
    /// Modal dialog card or window (e.g. Preferences, About, Asset Preview).
    ModalWindow,
    /// Floating draggable tool window.
    FloatingWindow,
    /// Canvas for drawing real-time audio or profiler oscilloscope curves.
    OscilloscopeCanvas,
}

/// Explicit rendering and interaction stacking layer (stacking context) in the UI hierarchy.
/// Higher layers are drawn on top of lower layers and receive pointer interactions first.
/// Furthermore, opaque containers in higher layers automatically occlude content and text
/// located on lower layers, preventing visual bleeding and unwanted click-throughs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum UiLayer {
    /// Background layer for canvases, grids, and desktop wallpaper beneath all content.
    Background = 0,
    /// Standard primary UI content layer for docked panels, toolbars, buttons, and inputs.
    #[default]
    Content = 1,
    /// Floating tool windows, detached inspector palettes, and auxiliary tool dialogs.
    Floating = 2,
    /// Modal dialog windows and critical prompts that capture focus and occlude background interaction.
    Modal = 3,
    /// Floating popup elements, dropdown menus, context menus, and combo-box flyouts.
    Popup = 4,
    /// Top-most transient indicators, tooltips, drag badges, and cursor overlays.
    Tooltip = 5,
}

impl UiLayer {
    /// Returns the zero-based index of this layer, matching its numerical order (0 to 5).
    #[inline]
    pub const fn index(self) -> usize {
        self as usize
    }

    /// All layers in order from lowest (Background) to highest (Tooltip).
    pub const ALL: [UiLayer; 6] = [
        UiLayer::Background,
        UiLayer::Content,
        UiLayer::Floating,
        UiLayer::Modal,
        UiLayer::Popup,
        UiLayer::Tooltip,
    ];
}

impl WidgetRole {
    /// Returns the recommended default stacking layer for this widget role.
    #[inline]
    pub const fn default_layer(&self) -> UiLayer {
        match self {
            WidgetRole::Default | WidgetRole::Separator | WidgetRole::OscilloscopeCanvas => {
                UiLayer::Content
            }
            WidgetRole::FloatingWindow => UiLayer::Floating,
            WidgetRole::ModalWindow => UiLayer::Modal,
            WidgetRole::DropdownPopup
            | WidgetRole::DropdownItem
            | WidgetRole::DropdownIcon
            | WidgetRole::DropdownShortcut
            | WidgetRole::DropdownLabel => UiLayer::Popup,
        }
    }
}

/// A single node in the Retained-Mode UI tree stored in the central arena.
/// Each node holds hierarchical relationships (parent and children references via `WidgetId`),
/// current style parameters, fine-grained dirty flags, cached layout coordinates, and optional text payload.
#[derive(Debug, Clone)]
pub struct WidgetNode {
    /// The unique generational key of this node.
    pub id: WidgetId,
    /// Optional parent node key. `None` if this is the root node or a detached branch.
    pub parent: Option<WidgetId>,
    /// Ordered list of child node keys.
    pub children: Vec<WidgetId>,
    /// Visual and layout styling attributes.
    pub style: Style,
    /// Dirty tracking flags for selective recomputation.
    pub dirty: DirtyFlags,
    /// Cached absolute layout bounding rectangle in screen-space pixels.
    pub computed_rect: Rect,
    /// Intrinsic content size (e.g. measured text dimensions or image resolution).
    pub content_size: Size,
    /// Optional text payload displayed by this node.
    pub text: Option<String>,
    /// Optional texture UV coordinate rectangle `[min_u, min_v, max_u, max_v]` for icon/image rendering.
    pub texture_uv: Option<[f32; 4]>,
    /// Optional tint color multiplier for textured quad rendering.
    pub texture_tint: Option<Color>,
    /// Optional external texture identifier for drawing host-owned 2D images (e.g. Viewport RTT).
    pub external_texture: Option<ExternalTextureId>,
    /// Font size in pixels.
    pub font_size: f32,
    /// Line height in pixels.
    pub line_height: f32,
    /// Text RGBA foreground color.
    pub text_color: Color,
    /// Text alignment.
    pub text_align: TextAlign,
    /// Whether this node and its subtree are visible.
    pub visible: bool,
    /// Whether this node can receive mouse and keyboard interaction events.
    pub interactive: bool,
    /// Semantic functional role for layout, rendering, and occlusion management.
    pub role: WidgetRole,
    /// Explicit stacking layer for Z-ordering, occlusion culling, and hit-test priority.
    pub layer: UiLayer,
    /// Optional debug name for inspection and profiling.
    pub name: Option<String>,
}

impl WidgetNode {
    /// Default standard font size in pixels.
    pub const DEFAULT_FONT_SIZE: f32 = 14.0;
    /// Default standard line height in pixels.
    pub const DEFAULT_LINE_HEIGHT: f32 = 18.0;

    /// Creates a new `WidgetNode` with default properties and full dirty flags.
    #[inline]
    pub fn new(id: WidgetId) -> Self {
        Self {
            id,
            parent: None,
            children: Vec::new(),
            style: Style::default(),
            dirty: DirtyFlags::ALL,
            computed_rect: Rect::ZERO,
            content_size: Size::ZERO,
            text: None,
            texture_uv: None,
            texture_tint: None,
            external_texture: None,
            font_size: Self::DEFAULT_FONT_SIZE,
            line_height: Self::DEFAULT_LINE_HEIGHT,
            text_color: Color::WHITE,
            text_align: TextAlign::Left,
            visible: true,
            interactive: true,
            role: WidgetRole::Default,
            layer: UiLayer::Content,
            name: None,
        }
    }

    /// Sets the semantic functional role of the node, automatically updating the stacking layer if default.
    #[inline]
    pub fn with_role(mut self, role: WidgetRole) -> Self {
        self.role = role;
        if self.layer == UiLayer::Content {
            self.layer = role.default_layer();
        }
        self
    }

    /// Sets the semantic functional role on an existing mutable reference, automatically updating layer if default.
    #[inline]
    pub fn set_role(&mut self, role: WidgetRole) -> &mut Self {
        self.role = role;
        if self.layer == UiLayer::Content {
            self.layer = role.default_layer();
        }
        self
    }

    /// Sets the explicit stacking layer of the node (builder style).
    #[inline]
    pub fn with_layer(mut self, layer: UiLayer) -> Self {
        self.layer = layer;
        self
    }

    /// Sets the explicit stacking layer on an existing mutable reference.
    #[inline]
    pub fn set_layer(&mut self, layer: UiLayer) -> &mut Self {
        self.layer = layer;
        self
    }

    /// Sets the debug name of the node (builder style).
    #[inline]
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Sets the texture UV sub-rectangle for rendering an icon or image (builder style).
    #[inline]
    pub fn with_texture_uv(mut self, uv: [f32; 4]) -> Self {
        self.texture_uv = Some(uv);
        self
    }

    /// Sets the texture UV sub-rectangle on an existing mutable reference.
    #[inline]
    pub fn set_texture_uv(&mut self, uv: [f32; 4]) {
        self.texture_uv = Some(uv);
    }

    /// Sets the texture tint color multiplier (builder style).
    #[inline]
    pub fn with_texture_tint(mut self, tint: Color) -> Self {
        self.texture_tint = Some(tint);
        self
    }

    /// Sets the texture tint color multiplier on an existing mutable reference.
    #[inline]
    pub fn set_texture_tint(&mut self, tint: Color) {
        self.texture_tint = Some(tint);
    }

    /// Sets the external texture identifier (builder style).
    #[inline]
    pub fn with_external_texture(mut self, id: ExternalTextureId) -> Self {
        self.external_texture = Some(id);
        self
    }

    /// Sets the external texture identifier on an existing mutable reference.
    #[inline]
    pub fn set_external_texture(&mut self, id: Option<ExternalTextureId>) {
        self.external_texture = id;
    }

    /// Sets the debug name of the node on an existing mutable reference.
    #[inline]
    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = Some(name.into());
    }

    /// Sets the text content and marks `TEXT`, `LAYOUT`, and `PAINT` dirty flags.
    #[inline]
    pub fn set_text(&mut self, text: impl Into<String>) {
        let new_text = text.into();
        if self.text.as_deref() != Some(&new_text) {
            self.text = Some(new_text);
            self.dirty |= DirtyFlags::TEXT | DirtyFlags::LAYOUT | DirtyFlags::PAINT;
        }
    }

    /// Sets the text styling properties (font size, line height, color, alignment).
    #[inline]
    pub fn set_text_properties(
        &mut self,
        font_size: f32,
        line_height: f32,
        color: Color,
        align: TextAlign,
    ) {
        self.font_size = font_size;
        self.line_height = line_height;
        self.text_color = color;
        self.text_align = align;
        self.dirty |= DirtyFlags::TEXT | DirtyFlags::LAYOUT | DirtyFlags::PAINT;
    }

    /// Sets the style of the node and marks styling flags as dirty.
    #[inline]
    pub fn set_style(&mut self, style: Style) {
        if self.style != style {
            self.style = style;
            self.dirty |= DirtyFlags::STYLE | DirtyFlags::LAYOUT | DirtyFlags::PAINT;
        }
    }

    /// Marks specified dirty flags on this node.
    #[inline]
    pub fn mark_dirty(&mut self, flags: DirtyFlags) {
        self.dirty |= flags;
    }

    /// Clears specified dirty flags after processing.
    #[inline]
    pub fn clear_dirty(&mut self, flags: DirtyFlags) {
        self.dirty.remove(flags);
    }

    /// Tests if a given screen-space point falls within this node's cached bounding box.
    #[inline]
    pub fn hit_test(&self, point: Point) -> bool {
        self.visible && self.interactive && self.computed_rect.contains_point(point)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_widget_role_default_and_mutation() {
        let dummy_id = WidgetId::default();
        let mut node = WidgetNode::new(dummy_id);
        assert_eq!(node.role, WidgetRole::Default);

        node.set_role(WidgetRole::DropdownPopup);
        assert_eq!(node.role, WidgetRole::DropdownPopup);

        let modal_node = WidgetNode::new(dummy_id).with_role(WidgetRole::ModalWindow);
        assert_eq!(modal_node.role, WidgetRole::ModalWindow);

        let canvas_node = WidgetNode::new(dummy_id).with_role(WidgetRole::OscilloscopeCanvas);
        assert_eq!(canvas_node.role, WidgetRole::OscilloscopeCanvas);
    }

    #[test]
    fn test_ui_layer_ordering_and_defaults() {
        assert!(UiLayer::Background < UiLayer::Content);
        assert!(UiLayer::Content < UiLayer::Floating);
        assert!(UiLayer::Floating < UiLayer::Modal);
        assert!(UiLayer::Modal < UiLayer::Popup);
        assert!(UiLayer::Popup < UiLayer::Tooltip);

        let dummy_id = WidgetId::default();
        let default_node = WidgetNode::new(dummy_id);
        assert_eq!(default_node.layer, UiLayer::Content);

        let modal_node = WidgetNode::new(dummy_id).with_role(WidgetRole::ModalWindow);
        assert_eq!(modal_node.layer, UiLayer::Modal);

        let popup_node = WidgetNode::new(dummy_id).with_role(WidgetRole::DropdownPopup);
        assert_eq!(popup_node.layer, UiLayer::Popup);

        let floating_node = WidgetNode::new(dummy_id).with_role(WidgetRole::FloatingWindow);
        assert_eq!(floating_node.layer, UiLayer::Floating);

        let custom_layer_node = WidgetNode::new(dummy_id).with_layer(UiLayer::Tooltip);
        assert_eq!(custom_layer_node.layer, UiLayer::Tooltip);
    }
}