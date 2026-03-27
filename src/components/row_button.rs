//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

use std::any::TypeId;

use xilem::core::{MessageCtx, Mut, View, ViewMarker, ViewPathTracker, ViewId};
use xilem::core::MessageResult;
use xilem::masonry::core::keyboard::{Key, NamedKey};
use xilem::masonry::core::PointerButton;
use xilem::masonry::accesskit::{self, Node, Role};
use xilem::masonry::imaging::Painter;
use xilem::masonry::vello::kurbo::{Point, Rect, Size};
use xilem::masonry::vello::peniko::Color;
use tracing::{Span, trace_span};

use xilem::masonry::core::{
    AccessCtx, AccessEvent, ChildrenIds, EventCtx, LayoutCtx, MeasureCtx, Modifiers, NewWidget,
    PaintCtx, PointerButtonEvent, PointerEvent, PropertiesMut, PropertiesRef, RegisterCtx,
    TextEvent, Update, UpdateCtx, Widget, WidgetId, WidgetMut, WidgetPod,
};
use xilem::masonry::layout::{LenReq, LayoutSize, SizeDef};
use xilem::masonry::properties::Background;
use xilem::masonry::vello::kurbo::Axis;
use xilem::{Pod, ViewCtx, WidgetView};

const CHILD_VIEW_ID: ViewId = ViewId::new(0);

/// Action emitted when a row button is pressed.
#[derive(PartialEq, Debug, Clone)]
pub struct RowButtonPress {
    pub button: Option<PointerButton>,
    pub click_count: u8,
    pub modifiers: Modifiers,
    pub position: Point,
}

/// A button widget designed for list/tree rows.
///
/// Key features:
/// - Content is left-aligned (not centered)
/// - No minimum height - rows are compact
/// - Full-width hover/active background highlight
/// - Stretches to fill available width from parent
pub struct RowButton {
    child: WidgetPod<dyn Widget>,
    hover_bg: Color,
    click_count: u8,
    modifiers: Modifiers,
    position: Point,
    size: Size,
}

impl RowButton {
    pub fn new(child: NewWidget<impl Widget + ?Sized>) -> Self {
        Self {
            child: child.erased().to_pod(),
            hover_bg: Color::TRANSPARENT,
            click_count: 0,
            modifiers: Modifiers::default(),
            position: Point::ZERO,
            size: Size::ZERO,
        }
    }

    pub fn with_hover_bg(mut self, color: Color) -> Self {
        self.hover_bg = color;
        self
    }

    pub fn child_mut<'t>(this: &'t mut WidgetMut<'_, Self>) -> WidgetMut<'t, dyn Widget> {
        this.ctx.get_mut(&mut this.widget.child)
    }

    pub fn set_hover_bg(this: &mut WidgetMut<'_, Self>, color: Color) {
        this.widget.hover_bg = color;
        this.ctx.request_paint_only();
    }
}

impl Widget for RowButton {
    type Action = RowButtonPress;

    fn on_pointer_event(
        &mut self,
        ctx: &mut EventCtx<'_>,
        _props: &mut PropertiesMut<'_>,
        event: &PointerEvent,
    ) {
        match event {
            PointerEvent::Down(PointerButtonEvent { state, .. }) => {
                self.click_count = state.count as u8;
                // Capture modifiers at click time (they may be released before Up)
                self.modifiers = state.modifiers;
                // Capture position in window coordinates
                self.position = Point::new(state.position.x, state.position.y);
                ctx.request_focus();
                ctx.capture_pointer();
                ctx.request_render();
            }
            PointerEvent::Up(PointerButtonEvent { button, .. }) => {
                if ctx.is_active() && ctx.is_hovered() {
                    ctx.submit_action::<Self::Action>(RowButtonPress {
                        button: *button,
                        click_count: self.click_count,
                        modifiers: self.modifiers,
                        position: self.position,
                    });
                }
                ctx.request_render();
            }
            _ => (),
        }
    }

    fn on_text_event(
        &mut self,
        ctx: &mut EventCtx<'_>,
        _props: &mut PropertiesMut<'_>,
        event: &TextEvent,
    ) {
        match event {
            TextEvent::Keyboard(event) if event.state.is_up() => {
                if matches!(&event.key, Key::Character(c) if c == " ")
                    || event.key == Key::Named(NamedKey::Enter)
                {
                    ctx.submit_action::<Self::Action>(RowButtonPress {
                        button: None,
                        click_count: 1,
                        modifiers: event.modifiers,
                        position: Point::ZERO,
                    });
                }
            }
            _ => (),
        }
    }

    fn on_access_event(
        &mut self,
        ctx: &mut EventCtx<'_>,
        _props: &mut PropertiesMut<'_>,
        event: &AccessEvent,
    ) {
        if event.action == accesskit::Action::Click {
            ctx.submit_action::<Self::Action>(RowButtonPress {
                button: None,
                click_count: 1,
                modifiers: Modifiers::default(),
                position: Point::ZERO,
            });
        }
    }

    fn update(&mut self, ctx: &mut UpdateCtx<'_>, _props: &mut PropertiesMut<'_>, event: &Update) {
        match event {
            Update::HoveredChanged(_)
            | Update::ActiveChanged(_)
            | Update::FocusChanged(_)
            | Update::DisabledChanged(_) => {
                ctx.request_render();
            }
            _ => {}
        }
    }

    fn register_children(&mut self, ctx: &mut RegisterCtx<'_>) {
        ctx.register_child(&mut self.child);
    }

    fn property_changed(&mut self, ctx: &mut UpdateCtx<'_>, property_type: TypeId) {
        if property_type == TypeId::of::<Background>() {
            ctx.request_render();
        }
    }

    fn measure(
        &mut self,
        ctx: &mut MeasureCtx<'_>,
        _props: &PropertiesRef<'_>,
        axis: Axis,
        len_req: LenReq,
        cross_length: Option<f64>,
    ) -> f64 {
        let auto_length = len_req.into();
        let context_size = LayoutSize::maybe(axis.cross(), cross_length);

        let child_length = ctx.compute_length(
            &mut self.child,
            auto_length,
            context_size,
            axis,
            cross_length,
        );

        match axis {
            Axis::Horizontal => {
                if let LenReq::FitContent(available) = len_req {
                    available
                } else {
                    child_length
                }
            }
            Axis::Vertical => child_length,
        }
    }

    fn layout(&mut self, ctx: &mut LayoutCtx<'_>, _props: &PropertiesRef<'_>, size: Size) {
        self.size = size;
        let child_size = ctx.compute_size(&mut self.child, SizeDef::fit(size), size.into());
        ctx.run_layout(&mut self.child, child_size);
        ctx.place_child(&mut self.child, Point::ORIGIN);
        ctx.derive_baselines(&self.child);
    }

    fn paint(&mut self, ctx: &mut PaintCtx<'_>, props: &PropertiesRef<'_>, painter: &mut Painter<'_>) {
        let rect = Rect::from_origin_size(Point::ZERO, self.size);
        let use_hover = ctx.is_hovered() && !ctx.is_disabled();

        if use_hover {
            if self.hover_bg != Color::TRANSPARENT {
                painter.fill(rect, self.hover_bg).draw();
            }
        } else if let Some(bg) = props.get_defined::<Background>() {
            let brush = bg.get_peniko_brush_for_rect(rect);
            painter.fill(rect, &brush).draw();
        }
    }

    fn accessibility_role(&self) -> Role {
        Role::Button
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
        true
    }

    fn accepts_focus(&self) -> bool {
        true
    }

    fn accepts_text_input(&self) -> bool {
        false
    }

    fn make_trace_span(&self, id: WidgetId) -> Span {
        trace_span!("RowButton", id = id.trace())
    }
}

/// Xilem view for a row button - left-aligned, full-width, with hover highlight.
pub struct RowButtonView<F, V> {
    child: V,
    callback: F,
    hover_bg: Color,
    disabled: bool,
}

/// Create a row button with left-aligned child content.
///
/// The button stretches to fill available width and highlights on hover.
pub fn row_button<State: 'static, Action: 'static, V: WidgetView<State, Action>>(
    child: V,
    callback: impl Fn(&mut State) -> Action + Send + Sync + 'static,
) -> RowButtonView<
    impl for<'a> Fn(&'a mut State, &RowButtonPress) -> MessageResult<Action> + Send + 'static,
    V,
> {
    RowButtonView {
        child,
        callback: move |state: &mut State, press: &RowButtonPress| {
            match press.button {
                None | Some(PointerButton::Primary) => MessageResult::Action(callback(state)),
                _ => MessageResult::Nop,
            }
        },
        hover_bg: Color::TRANSPARENT,
        disabled: false,
    }
}

/// Create a row button that receives click count (for double-click handling).
pub fn row_button_with_clicks<State: 'static, Action: 'static, V: WidgetView<State, Action>>(
    child: V,
    callback: impl Fn(&mut State, u8) -> Action + Send + Sync + 'static,
) -> RowButtonView<
    impl for<'a> Fn(&'a mut State, &RowButtonPress) -> MessageResult<Action> + Send + 'static,
    V,
> {
    RowButtonView {
        child,
        callback: move |state: &mut State, press: &RowButtonPress| {
            match press.button {
                None | Some(PointerButton::Primary) => {
                    MessageResult::Action(callback(state, press.click_count))
                }
                _ => MessageResult::Nop,
            }
        },
        hover_bg: Color::TRANSPARENT,
        disabled: false,
    }
}

/// Create a row button that receives keyboard modifiers (for Cmd+click, Shift+click, etc.).
///
/// This is useful for implementing multi-selection where:
/// - No modifiers: replace selection
/// - Cmd/Ctrl: toggle individual item
/// - Shift: extend selection range
pub fn row_button_with_modifiers<State: 'static, Action: 'static, V: WidgetView<State, Action>>(
    child: V,
    callback: impl Fn(&mut State, Modifiers) -> Action + Send + Sync + 'static,
) -> RowButtonView<
    impl for<'a> Fn(&'a mut State, &RowButtonPress) -> MessageResult<Action> + Send + 'static,
    V,
> {
    RowButtonView {
        child,
        callback: move |state: &mut State, press: &RowButtonPress| {
            match press.button {
                None | Some(PointerButton::Primary) => {
                    MessageResult::Action(callback(state, press.modifiers))
                }
                _ => MessageResult::Nop,
            }
        },
        hover_bg: Color::TRANSPARENT,
        disabled: false,
    }
}

/// Create a row button that receives full press information.
///
/// This is useful for handling different mouse buttons (right-click for context menu)
/// and getting the click position for menu placement.
pub fn row_button_with_press<State: 'static, Action: 'static, V: WidgetView<State, Action>>(
    child: V,
    callback: impl Fn(&mut State, &RowButtonPress) -> Action + Send + Sync + 'static,
) -> RowButtonView<
    impl for<'a> Fn(&'a mut State, &RowButtonPress) -> MessageResult<Action> + Send + 'static,
    V,
> {
    RowButtonView {
        child,
        callback: move |state: &mut State, press: &RowButtonPress| {
            MessageResult::Action(callback(state, press))
        },
        hover_bg: Color::TRANSPARENT,
        disabled: false,
    }
}

impl<F, V> RowButtonView<F, V> {
    /// Set the hover background color.
    pub fn hover_bg(mut self, color: Color) -> Self {
        self.hover_bg = color;
        self
    }

    /// Set the disabled state.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl<F, V> ViewMarker for RowButtonView<F, V> {}

impl<F, V, State, Action> View<State, Action, ViewCtx> for RowButtonView<F, V>
where
    V: WidgetView<State, Action>,
    F: Fn(&mut State, &RowButtonPress) -> MessageResult<Action> + Send + Sync + 'static,
    State: 'static,
    Action: 'static,
{
    type Element = Pod<RowButton>;
    type ViewState = V::ViewState;

    fn build(
        &self,
        ctx: &mut ViewCtx,
        app_state: &mut State,
    ) -> (Self::Element, Self::ViewState) {
        let (child_pod, child_state) = ctx.with_id(CHILD_VIEW_ID, |ctx| {
            self.child.build(ctx, app_state)
        });
        let pod = ctx.with_action_widget(|ctx| {
            let widget = RowButton::new(child_pod.new_widget).with_hover_bg(self.hover_bg);
            let mut pod = ctx.create_pod(widget);
            pod.new_widget.options.disabled = self.disabled;
            pod
        });
        (pod, child_state)
    }

    fn rebuild(
        &self,
        prev: &Self,
        view_state: &mut Self::ViewState,
        ctx: &mut ViewCtx,
        mut element: Mut<'_, Self::Element>,
        app_state: &mut State,
    ) {
        if prev.disabled != self.disabled {
            element.ctx.set_disabled(self.disabled);
        }
        if prev.hover_bg != self.hover_bg {
            RowButton::set_hover_bg(&mut element, self.hover_bg);
        }
        ctx.with_id(CHILD_VIEW_ID, |ctx| {
            self.child.rebuild(
                &prev.child,
                view_state,
                ctx,
                RowButton::child_mut(&mut element).downcast(),
                app_state,
            );
        });
    }

    fn teardown(
        &self,
        view_state: &mut Self::ViewState,
        ctx: &mut ViewCtx,
        mut element: Mut<'_, Self::Element>,
    ) {
        ctx.with_id(CHILD_VIEW_ID, |ctx| {
            self.child.teardown(
                view_state,
                ctx,
                RowButton::child_mut(&mut element).downcast(),
            );
        });
    }

    fn message(
        &self,
        view_state: &mut Self::ViewState,
        message: &mut MessageCtx,
        mut element: Mut<'_, Self::Element>,
        app_state: &mut State,
    ) -> MessageResult<Action> {
        match message.take_first() {
            Some(CHILD_VIEW_ID) => self.child.message(
                view_state,
                message,
                RowButton::child_mut(&mut element).downcast(),
                app_state,
            ),
            None => match message.take_message::<RowButtonPress>() {
                Some(press) => (self.callback)(app_state, &press),
                None => MessageResult::Stale,
            },
            _ => MessageResult::Stale,
        }
    }
}
