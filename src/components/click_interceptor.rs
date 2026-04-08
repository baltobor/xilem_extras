//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! Click interceptor widget that marks pointer events as handled.
//!
//! Use this to wrap interactive widgets (buttons, text inputs) inside
//! tree rows or other clickable containers. The interceptor marks
//! pointer events as handled, preventing parent widgets from also
//! responding to the click.

use std::any::TypeId;

use xilem::core::{MessageCtx, Mut, View, ViewMarker, ViewPathTracker, ViewId};
use xilem::core::MessageResult;
use xilem::masonry::accesskit::{Node, Role};
use xilem::masonry::imaging::Painter;
use xilem::masonry::kurbo::{Point, Size};
use tracing::{Span, trace_span};

use xilem::masonry::core::{
    AccessCtx, AccessEvent, ChildrenIds, EventCtx, LayoutCtx, MeasureCtx, NewWidget,
    PaintCtx, PointerEvent, PropertiesMut, PropertiesRef, RegisterCtx,
    TextEvent, Update, UpdateCtx, Widget, WidgetId, WidgetMut, WidgetPod,
};
use xilem::masonry::layout::{LenReq, LayoutSize, SizeDef};
use xilem::masonry::kurbo::Axis;
use xilem::{Pod, ViewCtx, WidgetView};

const CHILD_VIEW_ID: ViewId = ViewId::new(0);

/// A transparent wrapper that marks pointer events as handled.
///
/// This widget wraps a child and intercepts pointer Down events,
/// marking them as handled so that parent widgets (like RowButton)
/// don't also process the click.
///
/// The child widget still receives and handles all events normally.
pub struct ClickInterceptorWidget {
    child: WidgetPod<dyn Widget>,
    size: Size,
}

impl ClickInterceptorWidget {
    pub fn new(child: NewWidget<impl Widget + ?Sized>) -> Self {
        Self {
            child: child.erased().to_pod(),
            size: Size::ZERO,
        }
    }

    pub fn child_mut<'t>(this: &'t mut WidgetMut<'_, Self>) -> WidgetMut<'t, dyn Widget> {
        this.ctx.get_mut(&mut this.widget.child)
    }
}

impl Widget for ClickInterceptorWidget {
    type Action = ();

    fn on_pointer_event(
        &mut self,
        ctx: &mut EventCtx<'_>,
        _props: &mut PropertiesMut<'_>,
        event: &PointerEvent,
    ) {
        // Mark pointer down as handled so parent containers (RowButton)
        // don't also try to handle this click
        if let PointerEvent::Down(_) = event {
            ctx.set_handled();
        }
    }

    fn on_text_event(
        &mut self,
        _ctx: &mut EventCtx<'_>,
        _props: &mut PropertiesMut<'_>,
        _event: &TextEvent,
    ) {
        // Let child handle text events
    }

    fn on_access_event(
        &mut self,
        _ctx: &mut EventCtx<'_>,
        _props: &mut PropertiesMut<'_>,
        _event: &AccessEvent,
    ) {
        // Let child handle accessibility events
    }

    fn update(&mut self, _ctx: &mut UpdateCtx<'_>, _props: &mut PropertiesMut<'_>, _event: &Update) {
    }

    fn register_children(&mut self, ctx: &mut RegisterCtx<'_>) {
        ctx.register_child(&mut self.child);
    }

    fn property_changed(&mut self, _ctx: &mut UpdateCtx<'_>, _property_type: TypeId) {
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
        ctx.compute_length(&mut self.child, auto_length, context_size, axis, cross_length)
    }

    fn layout(&mut self, ctx: &mut LayoutCtx<'_>, _props: &PropertiesRef<'_>, size: Size) {
        self.size = size;
        let child_size = ctx.compute_size(&mut self.child, SizeDef::fit(size), size.into());
        ctx.run_layout(&mut self.child, child_size);
        ctx.place_child(&mut self.child, Point::ORIGIN);
        ctx.derive_baselines(&self.child);
    }

    fn paint(&mut self, _ctx: &mut PaintCtx<'_>, _props: &PropertiesRef<'_>, _painter: &mut Painter<'_>) {
        // Transparent - no painting, child paints itself
    }

    fn accessibility_role(&self) -> Role {
        Role::GenericContainer
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

    fn propagates_pointer_interaction(&self) -> bool {
        true
    }

    fn accepts_focus(&self) -> bool {
        false
    }

    fn accepts_text_input(&self) -> bool {
        false
    }

    fn make_trace_span(&self, id: WidgetId) -> Span {
        trace_span!("ClickInterceptor", id = id.trace())
    }
}

/// Xilem view for click interceptor.
pub struct ClickInterceptorView<V> {
    child: V,
}

/// Wrap a widget to intercept clicks inside a clickable container.
///
/// Use this when placing buttons, text inputs, or other interactive
/// widgets inside tree rows or other clickable areas. The interceptor
/// marks pointer events as handled, preventing the parent from also
/// responding to the click.
///
/// # Example
///
/// ```ignore
/// use xilem_extras::click_interceptor;
///
/// // Inside a tree row builder:
/// if is_editing {
///     flex_row((
///         icon,
///         click_interceptor(text_input(value, on_change)),
///         click_interceptor(button(cancel_icon, on_cancel)),
///     ))
/// }
/// ```
pub fn click_interceptor<State: 'static, Action: 'static, V: WidgetView<State, Action>>(
    child: V,
) -> ClickInterceptorView<V> {
    ClickInterceptorView { child }
}

impl<V> ViewMarker for ClickInterceptorView<V> {}

impl<V, State, Action> View<State, Action, ViewCtx> for ClickInterceptorView<V>
where
    V: WidgetView<State, Action>,
    State: 'static,
    Action: 'static,
{
    type Element = Pod<ClickInterceptorWidget>;
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
            ctx.create_pod(ClickInterceptorWidget::new(child_pod.new_widget))
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
        ctx.with_id(CHILD_VIEW_ID, |ctx| {
            self.child.rebuild(
                &prev.child,
                view_state,
                ctx,
                ClickInterceptorWidget::child_mut(&mut element).downcast(),
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
                ClickInterceptorWidget::child_mut(&mut element).downcast(),
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
                ClickInterceptorWidget::child_mut(&mut element).downcast(),
                app_state,
            ),
            _ => MessageResult::Stale,
        }
    }
}
