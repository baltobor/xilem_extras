//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).
//!
//! Synth-style single radio. Dark capsule + glowing dot, drawn
//! the same way `param_selector` draws its rows so a labeled
//! radio in a form row reads consistently with the rest of the
//! synth-styled boolean controls. Sized to match `SwitchWidget`
//! so radios and switches sit at the same row height.

use std::marker::PhantomData;

use xilem::core::{MessageCtx, MessageResult, Mut, View, ViewMarker};
use xilem::masonry::accesskit::{self, Node, Role, Toggled};
use xilem::masonry::core::{
    AccessCtx, AccessEvent, ChildrenIds, EventCtx, LayoutCtx, MeasureCtx, PaintCtx,
    PointerButtonEvent, PointerEvent, PropertiesMut, PropertiesRef, RegisterCtx, TextEvent,
    Update, UpdateCtx, Widget, WidgetId, WidgetMut,
};
use xilem::masonry::imaging::Painter;
use xilem::masonry::kurbo::{Axis, Circle, Point, Rect, RoundedRect, Size, Stroke};
use xilem::masonry::layout::{LenReq, Length};
use xilem::masonry::peniko::{Color, Fill};
use xilem::{Pod, ViewCtx};

use tracing::{Span, trace_span};

/// A single radio is a round 12×12 well — same edge radius as
/// the switch's pill (`SWITCH_H / 2 = 6`) so all the synth-style
/// controls share one corner radius.
const FRAME_W: f64 = 12.0;
const FRAME_H: f64 = 12.0;
/// Edge radius shared with the switch and param-selector capsule.
const FRAME_R: f64 = 6.0;
/// Selected dot radius.
const DOT_RADIUS: f64 = 4.0;

/// Well fill — matches the `param_selector` capsule.
const FRAME_FILL: Color = Color::from_rgb8(0x2A, 0x2A, 0x2A);
/// Well stroke.
const FRAME_BORDER: Color = Color::from_rgb8(0x55, 0x55, 0x55);
/// Default dot colour — warm off-white, matching the on/off
/// switch's thumb so all "knobs" read the same.
const DEFAULT_TINT: Color = Color::from_rgb8(0xEE, 0xE6, 0xD8);

/// Action emitted by [`RadioWidget`] on click. Carries the new
/// state so callers can mirror selection in their model.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RadioToggled(pub bool);

/// Synth-styled single radio button. No internal label — pair
/// with a `label(...)` in a `flex_row` for the row title.
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
        let size = ctx.content_box_size();
        let frame_x = (size.width - FRAME_W) / 2.0;
        let frame_y = (size.height - FRAME_H) / 2.0;
        let frame = Rect::new(
            frame_x,
            frame_y,
            frame_x + FRAME_W,
            frame_y + FRAME_H,
        );
        let pill = RoundedRect::from_rect(frame, FRAME_R);
        painter.fill(pill, FRAME_FILL).fill_rule(Fill::NonZero).draw();
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
        node.set_toggled(if self.selected { Toggled::True } else { Toggled::False });
        node.add_action(accesskit::Action::Click);
    }

    fn children_ids(&self) -> ChildrenIds {
        ChildrenIds::new()
    }

    fn make_trace_span(&self, id: WidgetId) -> Span {
        trace_span!("Radio", id = id.trace())
    }
}

// MARK: - View

/// Synth-styled single radio button view.
pub fn synth_radio<F, State, Action>(
    selected: bool,
    callback: F,
) -> SynthRadio<State, Action, F>
where
    F: Fn(&mut State, bool) -> Action + Send + Sync + 'static,
    State: 'static,
{
    SynthRadio {
        selected,
        callback,
        tint: None,
        disabled: false,
        phantom: PhantomData,
    }
}

#[must_use = "View values do nothing unless provided to Xilem."]
pub struct SynthRadio<State, Action, F> {
    selected: bool,
    callback: F,
    tint: Option<Color>,
    disabled: bool,
    phantom: PhantomData<fn(State) -> Action>,
}

impl<State, Action, F> SynthRadio<State, Action, F> {
    pub fn tint(mut self, color: Color) -> Self {
        self.tint = Some(color);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl<State, Action, F> ViewMarker for SynthRadio<State, Action, F> {}

impl<F, State, Action> View<State, Action, ViewCtx> for SynthRadio<State, Action, F>
where
    State: 'static,
    Action: 'static,
    F: Fn(&mut State, bool) -> Action + Send + Sync + 'static,
{
    type Element = Pod<RadioWidget>;
    type ViewState = ();

    fn build(&self, ctx: &mut ViewCtx, _: &mut State) -> (Self::Element, Self::ViewState) {
        let mut w = RadioWidget::new(self.selected);
        if let Some(c) = self.tint {
            w = w.with_tint(c);
        }
        let element = ctx.with_action_widget(|ctx| {
            let mut pod = ctx.create_pod(w);
            pod.new_widget.options.disabled = self.disabled;
            pod
        });
        (element, ())
    }

    fn rebuild(
        &self,
        prev: &Self,
        _: &mut Self::ViewState,
        _ctx: &mut ViewCtx,
        mut element: Mut<'_, Self::Element>,
        _: &mut State,
    ) {
        if prev.disabled != self.disabled {
            element.ctx.set_disabled(self.disabled);
        }
        if prev.selected != self.selected {
            RadioWidget::set_selected(&mut element, self.selected);
        }
        if prev.tint != self.tint {
            if let Some(c) = self.tint {
                RadioWidget::set_tint(&mut element, c);
            }
        }
    }

    fn teardown(
        &self,
        _: &mut Self::ViewState,
        ctx: &mut ViewCtx,
        element: Mut<'_, Self::Element>,
    ) {
        ctx.teardown_action_source(element);
    }

    fn message(
        &self,
        _: &mut Self::ViewState,
        message: &mut MessageCtx,
        _element: Mut<'_, Self::Element>,
        app_state: &mut State,
    ) -> MessageResult<Action> {
        match message.take_message::<RadioToggled>() {
            Some(toggled) => MessageResult::Action((self.callback)(app_state, toggled.0)),
            None => MessageResult::Stale,
        }
    }
}
