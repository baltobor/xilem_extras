//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! Sheet layer widget (kept for backward compatibility).
//!
//! Note: The primary sheet implementation now uses SheetWidget directly
//! without a separate layer. This module is kept for potential future use
//! or alternative implementations.

use tracing::{Span, trace_span};
use xilem::masonry::accesskit::{Node, Role};
use xilem::masonry::imaging::Painter;
use xilem::masonry::kurbo::{Rect, RoundedRect, Stroke};
use xilem::masonry::peniko::Color;

use xilem::masonry::core::{
    AccessCtx, AccessEvent, ChildrenIds, ComposeCtx, EventCtx, Layer, LayoutCtx, MeasureCtx,
    PaintCtx, PointerButtonEvent, PointerEvent, PropertiesMut, PropertiesRef, RegisterCtx,
    TextEvent, Update, UpdateCtx, Widget, WidgetId,
};
use xilem::masonry::kurbo::{Axis, Size};
use xilem::masonry::layout::LenReq;

use super::widget::{SheetAction, SheetWidget};

/// Default backdrop color (semi-transparent black).
const BACKDROP_COLOR: Color = Color::from_rgba8(0x00, 0x00, 0x00, 0x80);

/// A floating layer that displays a sheet modal backdrop.
///
/// Note: This is kept for backward compatibility. The primary sheet
/// implementation now uses SheetWidget directly.
pub struct SheetLayer {
    /// The creator widget ID (SheetWidget).
    creator: WidgetId,
    /// Background color for the content area.
    bg_color: Color,
    /// Corner radius for the content area.
    corner_radius: f64,
    /// Padding around the content.
    #[allow(dead_code)]
    padding: f64,
    /// Content area rect (calculated in layout).
    content_rect: Rect,
}

impl SheetLayer {
    /// Creates a new sheet layer.
    pub fn new(
        creator: WidgetId,
        bg_color: Color,
        corner_radius: f64,
        padding: f64,
    ) -> Self {
        Self {
            creator,
            bg_color,
            corner_radius,
            padding,
            content_rect: Rect::ZERO,
        }
    }

    /// Dismiss this sheet and notify the creator.
    fn dismiss(&mut self, ctx: &mut EventCtx<'_>) {
        let self_id = ctx.widget_id();
        ctx.remove_layer(self_id);
        ctx.mutate_later(self.creator, move |mut sheet| {
            let mut sheet = sheet.downcast::<SheetWidget>();
            sheet.widget.sheet_layer_id = None;
            sheet.ctx.submit_action::<SheetAction>(SheetAction::Dismissed);
        });
    }
}

impl Widget for SheetLayer {
    type Action = ();

    fn on_pointer_event(
        &mut self,
        ctx: &mut EventCtx<'_>,
        _props: &mut PropertiesMut<'_>,
        event: &PointerEvent,
    ) {
        // Handle click on content area (absorb the event)
        if let PointerEvent::Down(PointerButtonEvent { state, .. }) = event {
            let local_pos = ctx.local_position(state.position);
            if self.content_rect.contains(local_pos) {
                // Click inside content - absorb but don't dismiss
                return;
            }
        }
    }

    fn on_text_event(
        &mut self,
        ctx: &mut EventCtx<'_>,
        _props: &mut PropertiesMut<'_>,
        event: &TextEvent,
    ) {
        // Handle window focus loss
        if let TextEvent::WindowFocusChange(false) = event {
            self.dismiss(ctx);
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
            // Register this layer's ID with the parent SheetWidget
            let id = ctx.widget_id();
            ctx.mutate_later(self.creator, move |mut sheet| {
                let sheet = sheet.downcast::<SheetWidget>();
                sheet.widget.sheet_layer_id = Some(id);
            });
        }
    }

    fn register_children(&mut self, _ctx: &mut RegisterCtx<'_>) {
        // No children
    }

    fn measure(
        &mut self,
        _ctx: &mut MeasureCtx<'_>,
        _props: &PropertiesRef<'_>,
        axis: Axis,
        len_req: LenReq,
        _cross_length: Option<f64>,
    ) -> f64 {
        // Sheet layer fills available space
        match len_req {
            LenReq::FitContent(space) => space,
            LenReq::MinContent | LenReq::MaxContent => match axis {
                Axis::Horizontal => 800.0,
                Axis::Vertical => 600.0,
            },
        }
    }

    fn layout(&mut self, ctx: &mut LayoutCtx<'_>, _props: &PropertiesRef<'_>, size: Size) {
        // Calculate centered content rect (placeholder size)
        let max_width = (size.width * 0.8).min(600.0);
        let max_height = (size.height * 0.8).min(400.0);

        let x = (size.width - max_width) / 2.0;
        let y = (size.height - max_height) / 2.0;

        self.content_rect = Rect::new(x, y, x + max_width, y + max_height);
        ctx.clear_baselines();
    }

    fn compose(&mut self, _ctx: &mut ComposeCtx<'_>) {}

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
        ChildrenIds::from_slice(&[])
    }

    fn as_layer(&mut self) -> Option<&mut dyn Layer> {
        Some(self)
    }

    fn make_trace_span(&self, id: WidgetId) -> Span {
        trace_span!("SheetLayer", id = id.trace())
    }
}

impl Layer for SheetLayer {
    fn capture_pointer_event(
        &mut self,
        ctx: &mut EventCtx<'_>,
        _props: &mut PropertiesMut<'_>,
        event: &PointerEvent,
    ) {
        match event {
            PointerEvent::Down(PointerButtonEvent { state, .. }) => {
                // Check if click is outside the content area (on backdrop)
                let local_pos = ctx.local_position(state.position);
                if !self.content_rect.contains(local_pos) {
                    self.dismiss(ctx);
                }
            }
            PointerEvent::Cancel(..) => {
                self.dismiss(ctx);
            }
            _ => {}
        }
    }
}
