//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! Synth-style single radio widget.

use xilem::masonry::accesskit::{self, Node, Role, Toggled};
use xilem::masonry::core::{
    AccessCtx, AccessEvent, ChildrenIds, EventCtx, LayoutCtx, MeasureCtx, PaintCtx,
    PointerButtonEvent, PointerEvent, PropertiesMut, PropertiesRef, RegisterCtx, TextEvent, Update,
    UpdateCtx, Widget, WidgetId, WidgetMut,
};
use xilem::masonry::imaging::Painter;
use xilem::masonry::kurbo::{Axis, Circle, Point, Rect, RoundedRect, Size, Stroke};
use xilem::masonry::layout::{LenReq, Length};
use xilem::masonry::peniko::{Color, Fill};

use tracing::{Span, trace_span};

const FRAME_W: f64 = 12.0;
const FRAME_H: f64 = 12.0;
const FRAME_R: f64 = 6.0;
const DOT_RADIUS: f64 = 4.0;

const FRAME_FILL: Color = Color::from_rgb8(0x2A, 0x2A, 0x2A);
const FRAME_BORDER: Color = Color::from_rgb8(0x55, 0x55, 0x55);
const DEFAULT_TINT: Color = Color::from_rgb8(0xEE, 0xE6, 0xD8);

/// Action emitted by [`RadioWidget`] on click.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RadioToggled(pub bool);

/// Synth-styled single radio button widget.
pub struct RadioWidget {
    selected: bool,
    tint: Color,
}

impl RadioWidget {
    pub fn new(selected: bool) -> Self {
        Self {
            selected,
            tint: DEFAULT_TINT,
        }
    }

    pub fn with_tint(mut self, color: Color) -> Self {
        self.tint = color;
        self
    }

    pub fn set_selected(this: &mut WidgetMut<'_, Self>, selected: bool) {
        if this.widget.selected != selected {
            this.widget.selected = selected;
            this.ctx.request_render();
        }
    }

    pub fn set_tint(this: &mut WidgetMut<'_, Self>, color: Color) {
        this.widget.tint = color;
        this.ctx.request_render();
    }
}

impl Widget for RadioWidget {
    type Action = RadioToggled;

    fn on_pointer_event(
        &mut self,
        ctx: &mut EventCtx<'_>,
        _props: &mut PropertiesMut<'_>,
        event: &PointerEvent,
    ) {
        if ctx.is_disabled() {
            return;
        }
        match event {
            PointerEvent::Down(_) => {
                ctx.capture_pointer();
                ctx.request_render();
            }
            PointerEvent::Up(PointerButtonEvent { .. }) => {
                if ctx.is_active() && ctx.is_hovered() {
                    let new_selected = !self.selected;
                    self.selected = new_selected;
                    ctx.submit_action::<Self::Action>(RadioToggled(new_selected));
                    ctx.request_render();
                }
            }
            _ => {}
        }
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
        ctx: &mut EventCtx<'_>,
        _props: &mut PropertiesMut<'_>,
        event: &AccessEvent,
    ) {
        if event.action == accesskit::Action::Click {
            let new_selected = !self.selected;
            self.selected = new_selected;
            ctx.submit_action::<Self::Action>(RadioToggled(new_selected));
            ctx.request_render();
        }
    }

    fn update(&mut self, ctx: &mut UpdateCtx<'_>, _props: &mut PropertiesMut<'_>, event: &Update) {
        match event {
            Update::HoveredChanged(_)
            | Update::ActiveChanged(_)
            | Update::FocusChanged(_)
            | Update::DisabledChanged(_) => ctx.request_render(),
            _ => {}
        }
    }

    fn register_children(&mut self, _ctx: &mut RegisterCtx<'_>) {}

    fn accepts_pointer_interaction(&self) -> bool {
        true
    }

    fn accepts_focus(&self) -> bool {
        true
    }

    fn measure(
        &mut self,
        _ctx: &mut MeasureCtx<'_>,
        _props: &PropertiesRef<'_>,
        axis: Axis,
        _len_req: LenReq,
        _cross_length: Option<Length>,
    ) -> Length {
        match axis {
            Axis::Horizontal => Length::px(FRAME_W),
            Axis::Vertical => Length::px(FRAME_H),
        }
    }

    fn layout(&mut self, _ctx: &mut LayoutCtx<'_>, _props: &PropertiesRef<'_>, _size: Size) {}

    fn paint(
        &mut self,
        ctx: &mut PaintCtx<'_>,
        _props: &PropertiesRef<'_>,
        painter: &mut Painter<'_>,
    ) {
        let size = ctx.content_box().size();
        let frame_x = (size.width - FRAME_W) / 2.0;
        let frame_y = (size.height - FRAME_H) / 2.0;
        let frame = Rect::new(frame_x, frame_y, frame_x + FRAME_W, frame_y + FRAME_H);
        let pill = RoundedRect::from_rect(frame, FRAME_R);
        painter
            .fill(pill, FRAME_FILL)
            .fill_rule(Fill::NonZero)
            .draw();
        painter.stroke(pill, &Stroke::new(0.5), FRAME_BORDER).draw();

        if self.selected {
            let cx = frame_x + FRAME_W / 2.0;
            let cy = frame_y + FRAME_H / 2.0;
            let dot = Circle::new(Point::new(cx, cy), DOT_RADIUS);
            painter.fill(dot, self.tint).fill_rule(Fill::NonZero).draw();
        }
    }

    fn accessibility_role(&self) -> Role {
        Role::RadioButton
    }

    fn accessibility(
        &mut self,
        _ctx: &mut AccessCtx<'_>,
        _props: &PropertiesRef<'_>,
        node: &mut Node,
    ) {
        node.set_toggled(if self.selected {
            Toggled::True
        } else {
            Toggled::False
        });
        node.add_action(accesskit::Action::Click);
    }

    fn children_ids(&self) -> ChildrenIds {
        ChildrenIds::new()
    }

    fn make_trace_span(&self, id: WidgetId) -> Span {
        trace_span!("Radio", id = id.trace())
    }
}
