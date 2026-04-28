//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! Sheet widget that displays modal content with a backdrop.

use std::any::TypeId;

use tracing::{Span, trace_span};
use xilem::masonry::accesskit::{Node, Role};
use xilem::masonry::imaging::Painter;
use xilem::masonry::kurbo::{Rect, RoundedRect, Stroke};
use xilem::masonry::peniko::Color;

use xilem::masonry::core::{
    AccessCtx, AccessEvent, ChildrenIds, EventCtx, LayoutCtx, MeasureCtx, NewWidget, PaintCtx,
    PointerButtonEvent, PointerEvent, PropertiesMut, PropertiesRef, RegisterCtx, TextEvent,
    Update, UpdateCtx, Widget, WidgetId, WidgetMut, WidgetPod,
};
use xilem::masonry::kurbo::{Axis, Point, Size};
use xilem::masonry::layout::{LayoutSize, LenReq};

/// Default backdrop color (semi-transparent black).
const BACKDROP_COLOR: Color = Color::from_rgba8(0x00, 0x00, 0x00, 0x80);

/// Action emitted when the sheet is dismissed.
#[derive(PartialEq, Debug, Clone)]
pub enum SheetAction {
    /// The sheet was dismissed (via backdrop click or ESC).
    Dismissed,
}

/// A widget that displays content in a modal sheet/overlay.
///
/// The sheet appears as a floating centered panel with a semi-transparent backdrop.
/// Clicking the backdrop dismisses the sheet.
///
/// The sheet sizes to its content plus padding and centers on screen.
/// For proper behavior, place the sheet at the root level of your view tree
/// (e.g., using zstack or conditional rendering at the top level).
pub struct SheetWidget {
    /// The content to display in the sheet.
    child: WidgetPod<dyn Widget>,
    /// Whether the sheet is currently visible.
    is_presented: bool,
    /// Background color for the sheet content area.
    bg_color: Color,
    /// Corner radius for the sheet content area.
    corner_radius: f64,
    /// Padding around the content.
    padding: f64,
    /// Content area rect (calculated in layout).
    content_rect: Rect,
    /// Tracked layer ID (for Layer-based dismiss, if used).
    pub(crate) sheet_layer_id: Option<WidgetId>,
}

impl SheetWidget {
    /// Creates a new sheet widget.
    ///
    /// - `child`: the content widget to display in the sheet.
    /// - `is_presented`: whether the sheet should be initially visible.
    pub fn new(child: NewWidget<impl Widget + ?Sized>, is_presented: bool) -> Self {
        Self {
            child: child.erased().to_pod(),
            is_presented,
            bg_color: Color::WHITE,
            corner_radius: 12.0,
            padding: 24.0,
            content_rect: Rect::ZERO,
            sheet_layer_id: None,
        }
    }

    /// Sets the background color of the sheet content area.
    pub fn with_bg_color(mut self, color: Color) -> Self {
        self.bg_color = color;
        self
    }

    /// Sets the corner radius of the sheet content area.
    pub fn with_corner_radius(mut self, radius: f64) -> Self {
        self.corner_radius = radius;
        self
    }

    /// Sets the padding around the content.
    pub fn with_padding(mut self, padding: f64) -> Self {
        self.padding = padding;
        self
    }
}

impl SheetWidget {
    /// Returns a mutable reference to the child widget.
    pub fn child_mut<'t>(this: &'t mut WidgetMut<'_, Self>) -> WidgetMut<'t, dyn Widget> {
        this.ctx.get_mut(&mut this.widget.child)
    }

    /// Sets whether the sheet is presented.
    pub fn set_presented(this: &mut WidgetMut<'_, Self>, presented: bool) {
        if this.widget.is_presented != presented {
            this.widget.is_presented = presented;
            this.ctx.request_layout();
        }
    }
}

impl Widget for SheetWidget {
    type Action = SheetAction;

    fn on_pointer_event(
        &mut self,
        ctx: &mut EventCtx<'_>,
        _props: &mut PropertiesMut<'_>,
        event: &PointerEvent,
    ) {
        // Check for click outside content area (on backdrop) to dismiss
        if let PointerEvent::Down(PointerButtonEvent { state, .. }) = event {
            let local_pos = ctx.local_position(state.position);
            if !self.content_rect.contains(local_pos) {
                ctx.submit_action::<SheetAction>(SheetAction::Dismissed);
            }
        }
    }

    fn on_text_event(
        &mut self,
        ctx: &mut EventCtx<'_>,
        _props: &mut PropertiesMut<'_>,
        event: &TextEvent,
    ) {
        // Dismiss on window focus loss
        if let TextEvent::WindowFocusChange(false) = event {
            ctx.submit_action::<SheetAction>(SheetAction::Dismissed);
        }
    }

    fn on_access_event(
        &mut self,
        _ctx: &mut EventCtx<'_>,
        _props: &mut PropertiesMut<'_>,
        _event: &AccessEvent,
    ) {
    }

    fn update(&mut self, ctx: &mut UpdateCtx<'_>, _props: &mut PropertiesMut<'_>, event: &Update) {
        if let Update::WidgetAdded = event {
            if self.is_presented {
                ctx.request_layout();
            }
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
        // Sheet fills available space (should be given full window)
        // Measure child to determine content size
        let auto_length = len_req.into();
        let context_size = LayoutSize::maybe(axis.cross(), cross_length);
        let _child_len =
            ctx.compute_length(&mut self.child, auto_length, context_size, axis, cross_length);

        // Return available space (sheet fills parent)
        match len_req {
            LenReq::FitContent(space) => space,
            LenReq::MinContent => self.padding * 2.0 + _child_len,
            LenReq::MaxContent => match axis {
                Axis::Horizontal => 800.0,
                Axis::Vertical => 600.0,
            },
        }
    }

    fn layout(&mut self, ctx: &mut LayoutCtx<'_>, _props: &PropertiesRef<'_>, size: Size) {
        // Measure child to get its MINIMUM/NATURAL size (not expanded)
        let max_child_width = (size.width * 0.9) - self.padding * 2.0;
        let max_child_height = (size.height * 0.9) - self.padding * 2.0;

        // Use MinContent for both axes with ZERO context to force shrink-wrap
        use xilem::masonry::layout::{LenDef, LayoutSize as LS, SizeDef};
        let size_def = SizeDef::new(LenDef::MinContent, LenDef::MinContent);
        // Don't tell the child about available space - force it to report true minimum
        let context_size = LS::new(0.0, 0.0);

        let child_size = ctx.compute_size(&mut self.child, size_def, context_size);

        // Clamp to max bounds
        let child_width = child_size.width.min(max_child_width);
        let child_height = child_size.height.min(max_child_height);
        let child_size = Size::new(child_width, child_height);

        // Content rect is child size plus padding
        let content_width = child_size.width + self.padding * 2.0;
        let content_height = child_size.height + self.padding * 2.0;

        // Center the content rect
        let x = (size.width - content_width) / 2.0;
        let y = (size.height - content_height) / 2.0;

        self.content_rect = Rect::new(x, y, x + content_width, y + content_height);

        // Layout and place child at its natural size
        ctx.run_layout(&mut self.child, child_size);
        ctx.place_child(
            &mut self.child,
            Point::new(x + self.padding, y + self.padding),
        );

        ctx.clear_baselines();
    }

    fn paint(
        &mut self,
        ctx: &mut PaintCtx<'_>,
        _props: &PropertiesRef<'_>,
        painter: &mut Painter<'_>,
    ) {
        let size = ctx.border_box_size();

        // Draw semi-transparent backdrop
        let backdrop_rect = Rect::new(0.0, 0.0, size.width, size.height);
        painter.fill(backdrop_rect, BACKDROP_COLOR).draw();

        // Draw content area with rounded corners
        let rounded = RoundedRect::from_rect(self.content_rect, self.corner_radius);
        painter.fill(rounded, self.bg_color).draw();

        // Draw subtle border
        let border_color = Color::from_rgba8(0x80, 0x80, 0x80, 0x40);
        painter.stroke(rounded, &Stroke::new(1.0), border_color).draw();
    }

    fn accessibility_role(&self) -> Role {
        Role::Dialog
    }

    fn accessibility(
        &mut self,
        _ctx: &mut AccessCtx<'_>,
        _props: &PropertiesRef<'_>,
        _node: &mut Node,
    ) {
    }

    fn children_ids(&self) -> ChildrenIds {
        ChildrenIds::from_slice(&[self.child.id()])
    }

    fn accepts_focus(&self) -> bool {
        true
    }

    fn accepts_text_input(&self) -> bool {
        false
    }

    fn make_trace_span(&self, id: WidgetId) -> Span {
        trace_span!("SheetWidget", id = id.trace())
    }
}
