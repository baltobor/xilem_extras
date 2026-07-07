//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! Compact on/off switch widget.

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

const TRACK_W: f64 = 22.0;
const TRACK_H: f64 = 12.0;
const THUMB_R: f64 = 4.5;
const THUMB_INSET: f64 = (TRACK_H / 2.0) - THUMB_R;

const OFF_TRACK: Color = Color::from_rgb8(0x2A, 0x28, 0x25);
const ON_TRACK: Color = Color::from_rgb8(0x6A, 0x4E, 0x2A);
const TRACK_BORDER: Color = Color::from_rgb8(0x4A, 0x46, 0x40);
const THUMB_OFF: Color = Color::from_rgb8(0xC2, 0xBE, 0xB6);
const THUMB_ON: Color = Color::from_rgb8(0xEE, 0xE6, 0xD8);

/// Action emitted by [`SwitchWidget`] when its state flips.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SwitchToggled(pub bool);

/// Compact on/off switch widget.
pub struct SwitchWidget {
    on: bool,
}

impl SwitchWidget {
    pub fn new(on: bool) -> Self {
        Self { on }
    }

    pub fn set_on(this: &mut WidgetMut<'_, Self>, on: bool) {
        if this.widget.on != on {
            this.widget.on = on;
            this.ctx.request_render();
        }
    }
}

impl Widget for SwitchWidget {
    type Action = SwitchToggled;

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
                    let new_on = !self.on;
                    self.on = new_on;
                    ctx.submit_action::<Self::Action>(SwitchToggled(new_on));
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
            let new_on = !self.on;
            self.on = new_on;
            ctx.submit_action::<Self::Action>(SwitchToggled(new_on));
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
            Axis::Horizontal => Length::px(TRACK_W),
            Axis::Vertical => Length::px(TRACK_H),
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
        let track_x = (size.width - TRACK_W) / 2.0;
        let track_y = (size.height - TRACK_H) / 2.0;
        let track = Rect::new(track_x, track_y, track_x + TRACK_W, track_y + TRACK_H);
        let pill = RoundedRect::from_rect(track, TRACK_H / 2.0);

        let track_fill = if self.on { ON_TRACK } else { OFF_TRACK };
        painter
            .fill(pill, track_fill)
            .fill_rule(Fill::NonZero)
            .draw();
        painter.stroke(pill, &Stroke::new(0.5), TRACK_BORDER).draw();

        let thumb_y = track_y + TRACK_H / 2.0;
        let thumb_x = if self.on {
            track_x + TRACK_W - THUMB_INSET - THUMB_R
        } else {
            track_x + THUMB_INSET + THUMB_R
        };
        let thumb_color = if self.on { THUMB_ON } else { THUMB_OFF };
        let thumb = Circle::new(Point::new(thumb_x, thumb_y), THUMB_R);
        painter
            .fill(thumb, thumb_color)
            .fill_rule(Fill::NonZero)
            .draw();
    }

    fn accessibility_role(&self) -> Role {
        Role::Switch
    }

    fn accessibility(
        &mut self,
        _ctx: &mut AccessCtx<'_>,
        _props: &PropertiesRef<'_>,
        node: &mut Node,
    ) {
        node.set_toggled(if self.on {
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
        trace_span!("Switch", id = id.trace())
    }
}
