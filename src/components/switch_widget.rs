//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! A compact on/off switch styled in the spirit of
//! xilem_synth_widgets — small pill track, soft thumb, no
//! accent-blue. Used by `styled_switch` instead of masonry's
//! default Switch widget so that form rows look "decent" rather
//! than dialog-bright.

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

/// Track width in logical pixels.
const TRACK_W: f64 = 22.0;
/// Track height (also pill diameter).
const TRACK_H: f64 = 12.0;
/// Thumb radius. Slightly smaller than track height/2 so it sits
/// inside the pill with a hairline gap.
const THUMB_R: f64 = 4.5;
/// Padding between thumb and track edge.
const THUMB_INSET: f64 = (TRACK_H / 2.0) - THUMB_R;

/// Off-state track fill — flat dark, matches form chrome.
const OFF_TRACK: Color = Color::from_rgb8(0x2A, 0x28, 0x25);
/// On-state track fill — muted bronze, derived from the synth
/// widgets' orange tint but desaturated so it reads as "active"
/// without shouting.
const ON_TRACK: Color = Color::from_rgb8(0x6A, 0x4E, 0x2A);
/// Track border — subtle so the pill stands out on dark
/// backgrounds without feeling outlined.
const TRACK_BORDER: Color = Color::from_rgb8(0x4A, 0x46, 0x40);
/// Thumb fill when off — light gray.
const THUMB_OFF: Color = Color::from_rgb8(0xC2, 0xBE, 0xB6);
/// Thumb fill when on — slightly warmer near-white.
const THUMB_ON: Color = Color::from_rgb8(0xEE, 0xE6, 0xD8);

// MARK: - Widget

/// Action emitted by [`SwitchWidget`] when its state flips.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SwitchToggled(pub bool);

/// Compact on/off switch widget. Self-contained — does not rely
/// on the masonry default styling, so its appearance is stable
/// regardless of the application's theme.
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
        let size = ctx.content_box_size();
        // Centre the track inside whatever space we got.
        let track_x = (size.width - TRACK_W) / 2.0;
        let track_y = (size.height - TRACK_H) / 2.0;
        let track = Rect::new(track_x, track_y, track_x + TRACK_W, track_y + TRACK_H);
        let pill = RoundedRect::from_rect(track, TRACK_H / 2.0);

        let track_fill = if self.on { ON_TRACK } else { OFF_TRACK };
        painter.fill(pill, track_fill).fill_rule(Fill::NonZero).draw();
        painter.stroke(pill, &Stroke::new(0.5), TRACK_BORDER).draw();

        // Thumb position — left when off, right when on.
        let thumb_y = track_y + TRACK_H / 2.0;
        let thumb_x = if self.on {
            track_x + TRACK_W - THUMB_INSET - THUMB_R
        } else {
            track_x + THUMB_INSET + THUMB_R
        };
        let thumb_color = if self.on { THUMB_ON } else { THUMB_OFF };
        let thumb = Circle::new(Point::new(thumb_x, thumb_y), THUMB_R);
        painter.fill(thumb, thumb_color).fill_rule(Fill::NonZero).draw();
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
        node.set_toggled(if self.on { Toggled::True } else { Toggled::False });
        node.add_action(accesskit::Action::Click);
    }

    fn children_ids(&self) -> ChildrenIds {
        ChildrenIds::new()
    }

    fn make_trace_span(&self, id: WidgetId) -> Span {
        trace_span!("Switch", id = id.trace())
    }
}

// MARK: - View

/// Compact synth-styled on/off switch. Drop-in replacement for
/// `xilem::view::switch`, with tighter dimensions and a muted
/// palette (no accent-blue toggled state).
pub fn synth_switch<F, State, Action>(on: bool, callback: F) -> SynthSwitch<State, Action, F>
where
    F: Fn(&mut State, bool) -> Action + Send + Sync + 'static,
    State: 'static,
{
    SynthSwitch {
        on,
        callback,
        disabled: false,
        phantom: PhantomData,
    }
}

#[must_use = "View values do nothing unless provided to Xilem."]
pub struct SynthSwitch<State, Action, F> {
    on: bool,
    callback: F,
    disabled: bool,
    phantom: PhantomData<fn(State) -> Action>,
}

impl<State, Action, F> SynthSwitch<State, Action, F> {
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl<State, Action, F> ViewMarker for SynthSwitch<State, Action, F> {}

impl<F, State, Action> View<State, Action, ViewCtx> for SynthSwitch<State, Action, F>
where
    State: 'static,
    Action: 'static,
    F: Fn(&mut State, bool) -> Action + Send + Sync + 'static,
{
    type Element = Pod<SwitchWidget>;
    type ViewState = ();

    fn build(&self, ctx: &mut ViewCtx, _: &mut State) -> (Self::Element, Self::ViewState) {
        let element = ctx.with_action_widget(|ctx| {
            let mut pod = ctx.create_pod(SwitchWidget::new(self.on));
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
        if prev.on != self.on {
            SwitchWidget::set_on(&mut element, self.on);
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
        match message.take_message::<SwitchToggled>() {
            Some(toggled) => MessageResult::Action((self.callback)(app_state, toggled.0)),
            None => MessageResult::Stale,
        }
    }
}
