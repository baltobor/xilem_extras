//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! A single item inside a pulldown menu dropdown.

use std::any::TypeId;
use std::sync::Arc;

use xilem::masonry::accesskit::{self, Node, Role};
use tracing::{Span, trace_span};
use xilem::masonry::imaging::Painter;
use xilem::masonry::kurbo::{Rect, RoundedRect};
use xilem::masonry::peniko::Color;

use xilem::masonry::core::{
    AccessCtx, AccessEvent, ChildrenIds, EventCtx, LayoutCtx, MeasureCtx, PaintCtx,
    PointerEvent, PropertiesMut, PropertiesRef, RegisterCtx, TextEvent, Update, UpdateCtx,
    Widget, WidgetId, WidgetMut, WidgetPod,
};
use xilem::masonry::kurbo::{Axis, Point, Size};
use xilem::masonry::layout::{LayoutSize, LenReq, SizeDef};
use xilem::masonry::core::StyleProperty;
use xilem::masonry::widgets::Label;

/// Default hover background color.
const BG_HOVER: Color = Color::from_rgba8(0x50, 0x50, 0x60, 0xFF);

/// Default menu item height (matches macOS standard).
pub const DEFAULT_ITEM_HEIGHT: f64 = 30.0;

/// Font size derived from item height.
const TEXT_SIZE: f32 = (DEFAULT_ITEM_HEIGHT * 0.43) as f32;

/// Horizontal padding.
const ITEM_PADDING_H: f64 = 12.0;

/// Checkmark width for items with checked state.
const CHECKMARK_WIDTH: f64 = 20.0;

/// A single item inside a [`MenuDropdown`](super::MenuDropdown).
///
/// Renders a styled label with hover highlight. The parent dropdown
/// handles the actual click-to-select logic via `ctx.target()`.
pub struct PulldownMenuItem {
    child: WidgetPod<Label>,
    size: Size,
    hover_bg: Color,
    /// Checked state: None = no checkmark area, Some(true) = checkmark, Some(false) = empty space
    checked: Option<bool>,
}

impl PulldownMenuItem {
    /// Creates a new menu item with the given label text.
    pub fn new(text: impl Into<Arc<str>>) -> Self {
        let label = Label::new(text).with_style(StyleProperty::FontSize(TEXT_SIZE));
        Self {
            child: WidgetPod::new(label),
            size: Size::ZERO,
            hover_bg: BG_HOVER,
            checked: None,
        }
    }

    /// Sets the hover background color.
    pub fn with_hover_bg(mut self, color: Color) -> Self {
        self.hover_bg = color;
        self
    }

    /// Sets the checked state for this menu item.
    ///
    /// - `None`: No checkmark area (default)
    /// - `Some(true)`: Shows a checkmark
    /// - `Some(false)`: Shows empty space (for alignment)
    pub fn with_checked(mut self, checked: Option<bool>) -> Self {
        self.checked = checked;
        self
    }
}

impl PulldownMenuItem {
    /// Returns a mutable reference to the inner label.
    pub fn label_mut<'t>(this: &'t mut WidgetMut<'_, Self>) -> WidgetMut<'t, Label> {
        this.ctx.get_mut(&mut this.widget.child)
    }

    /// Sets the hover background color.
    pub fn set_hover_bg(this: &mut WidgetMut<'_, Self>, color: Color) {
        this.widget.hover_bg = color;
        this.ctx.request_paint_only();
    }
}

impl Widget for PulldownMenuItem {
    type Action = ();

    fn on_pointer_event(
        &mut self,
        _ctx: &mut EventCtx<'_>,
        _props: &mut PropertiesMut<'_>,
        _event: &PointerEvent,
    ) {
        // Click handling is done by the parent MenuDropdown.
    }

    fn on_text_event(
        &mut self,
        _ctx: &mut EventCtx<'_>,
        _props: &mut PropertiesMut<'_>,
        _event: &TextEvent,
    ) {
    }

    fn on_access_event(
        &mut self,
        _ctx: &mut EventCtx<'_>,
        _props: &mut PropertiesMut<'_>,
        _event: &AccessEvent,
    ) {
    }

    fn update(&mut self, ctx: &mut UpdateCtx<'_>, _props: &mut PropertiesMut<'_>, event: &Update) {
        match event {
            Update::HoveredChanged(_) => ctx.request_paint_only(),
            _ => {}
        }
    }

    fn register_children(&mut self, ctx: &mut RegisterCtx<'_>) {
        ctx.register_child(&mut self.child);
    }

    fn property_changed(&mut self, _ctx: &mut UpdateCtx<'_>, _property_type: TypeId) {}

    fn measure(
        &mut self,
        ctx: &mut MeasureCtx<'_>,
        _props: &PropertiesRef<'_>,
        axis: Axis,
        len_req: LenReq,
        cross_length: Option<f64>,
    ) -> f64 {
        match axis {
            Axis::Horizontal => {
                let auto_length = len_req.into();
                let context_size = LayoutSize::maybe(axis.cross(), cross_length);
                let child_length =
                    ctx.compute_length(&mut self.child, auto_length, context_size, axis, cross_length);
                let checkmark_space = if self.checked.is_some() { CHECKMARK_WIDTH } else { 0.0 };
                child_length + 2.0 * ITEM_PADDING_H + checkmark_space
            }
            Axis::Vertical => DEFAULT_ITEM_HEIGHT,
        }
    }

    fn layout(&mut self, ctx: &mut LayoutCtx<'_>, _props: &PropertiesRef<'_>, size: Size) {
        self.size = size;
        let checkmark_space = if self.checked.is_some() { CHECKMARK_WIDTH } else { 0.0 };
        let inner = Size::new(
            (size.width - 2.0 * ITEM_PADDING_H - checkmark_space).max(0.0),
            size.height,
        );
        let child_size = ctx.compute_size(&mut self.child, SizeDef::fit(inner), inner.into());
        ctx.run_layout(&mut self.child, child_size);

        // Left-align (after checkmark space), vertically center.
        let x = ITEM_PADDING_H + checkmark_space;
        let y = ((size.height - child_size.height) * 0.5).max(0.0);
        ctx.place_child(&mut self.child, (x, y).into());
        ctx.derive_baselines(&self.child);
    }

    fn paint(&mut self, ctx: &mut PaintCtx<'_>, _props: &PropertiesRef<'_>, painter: &mut Painter<'_>) {
        if ctx.is_hovered() {
            let rect = Rect::from_origin_size(Point::ZERO, self.size);
            let rounded = RoundedRect::from_rect(rect, 3.0);
            painter.fill(rounded, self.hover_bg).draw();
        }

        // Draw checkmark if checked
        if let Some(true) = self.checked {
            use xilem::masonry::kurbo::{BezPath, Stroke};

            let check_x = ITEM_PADDING_H + 2.0;
            let check_y = self.size.height / 2.0;

            // Simple checkmark path
            let mut path = BezPath::new();
            path.move_to((check_x, check_y));
            path.line_to((check_x + 4.0, check_y + 4.0));
            path.line_to((check_x + 12.0, check_y - 4.0));

            let stroke_color = Color::from_rgba8(0xE0, 0xE0, 0xE0, 0xFF);
            painter.stroke(&path, &Stroke::new(2.0), stroke_color).draw();
        }
    }

    fn accessibility_role(&self) -> Role {
        Role::MenuItem
    }

    fn accessibility(
        &mut self,
        _ctx: &mut AccessCtx<'_>,
        _props: &PropertiesRef<'_>,
        node: &mut Node,
    ) {
        node.add_action(accesskit::Action::Click);
    }

    fn children_ids(&self) -> ChildrenIds {
        ChildrenIds::from_slice(&[self.child.id()])
    }

    fn propagates_pointer_interaction(&self) -> bool {
        false
    }

    fn make_trace_span(&self, id: WidgetId) -> Span {
        trace_span!("PulldownMenuItem", id = id.trace())
    }
}
