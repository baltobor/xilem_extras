//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! A visual separator inside a pulldown menu.

use std::any::TypeId;

use xilem::masonry::accesskit::{Node, Role};
use tracing::{Span, trace_span};
use xilem::masonry::imaging::Painter;
use xilem::masonry::vello::kurbo::Rect;
use xilem::masonry::vello::peniko::Color;

use xilem::masonry::core::{
    AccessCtx, AccessEvent, ChildrenIds, EventCtx, LayoutCtx, MeasureCtx, PaintCtx,
    PointerEvent, PropertiesMut, PropertiesRef, RegisterCtx, TextEvent, Update, UpdateCtx,
    Widget, WidgetId,
};
use xilem::masonry::kurbo::{Axis, Size};
use xilem::masonry::layout::LenReq;

/// Separator line color (subtle gray).
const SEPARATOR_COLOR: Color = Color::from_rgba8(0x58, 0x56, 0x52, 0xFF);
const SEPARATOR_HEIGHT: f64 = 1.0;
const SEPARATOR_MARGIN_V: f64 = 6.0;

/// A visual separator line for use inside a [`MenuDropdown`](super::MenuDropdown).
///
/// Renders as a thin horizontal line with vertical margins. Used to group
/// related menu items visually.
pub struct MenuSeparator {
    size: Size,
    color: Color,
}

impl MenuSeparator {
    /// Creates a new menu separator.
    pub fn new() -> Self {
        Self {
            size: Size::ZERO,
            color: SEPARATOR_COLOR,
        }
    }

    /// Sets the separator line color.
    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }
}

impl Default for MenuSeparator {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for MenuSeparator {
    type Action = ();

    fn on_pointer_event(
        &mut self,
        _ctx: &mut EventCtx<'_>,
        _props: &mut PropertiesMut<'_>,
        _event: &PointerEvent,
    ) {
        // Separators don't handle pointer events.
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

    fn update(&mut self, _ctx: &mut UpdateCtx<'_>, _props: &mut PropertiesMut<'_>, _event: &Update) {
    }

    fn register_children(&mut self, _ctx: &mut RegisterCtx<'_>) {
        // No children.
    }

    fn property_changed(&mut self, _ctx: &mut UpdateCtx<'_>, _property_type: TypeId) {}

    fn measure(
        &mut self,
        _ctx: &mut MeasureCtx<'_>,
        _props: &PropertiesRef<'_>,
        axis: Axis,
        len_req: LenReq,
        cross_length: Option<f64>,
    ) -> f64 {
        match axis {
            Axis::Horizontal => {
                // Use available width from FitContent, or cross_length, or a minimum
                match len_req {
                    LenReq::FitContent(available) => available,
                    _ => cross_length.unwrap_or(100.0),
                }
            }
            Axis::Vertical => SEPARATOR_HEIGHT + 2.0 * SEPARATOR_MARGIN_V,
        }
    }

    fn layout(&mut self, _ctx: &mut LayoutCtx<'_>, _props: &PropertiesRef<'_>, size: Size) {
        self.size = size;
    }

    fn paint(&mut self, _ctx: &mut PaintCtx<'_>, _props: &PropertiesRef<'_>, painter: &mut Painter<'_>) {
        let y = self.size.height / 2.0;
        // Full width line - spans the entire allocated width
        let rect = Rect::new(
            0.0,
            y - SEPARATOR_HEIGHT / 2.0,
            self.size.width,
            y + SEPARATOR_HEIGHT / 2.0,
        );
        painter.fill(rect, self.color).draw();
    }

    fn accessibility_role(&self) -> Role {
        Role::Splitter
    }

    fn accessibility(
        &mut self,
        _ctx: &mut AccessCtx<'_>,
        _props: &PropertiesRef<'_>,
        _node: &mut Node,
    ) {
    }

    fn children_ids(&self) -> ChildrenIds {
        ChildrenIds::from_slice(&[])
    }

    fn propagates_pointer_interaction(&self) -> bool {
        false
    }

    fn make_trace_span(&self, id: WidgetId) -> Span {
        trace_span!("MenuSeparator", id = id.trace())
    }
}
