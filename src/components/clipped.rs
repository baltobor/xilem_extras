//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! Clipped widget that clips its content to its bounds.
//!
//! Use this to wrap content that may overflow its allocated space,
//! such as text in resizable table cells. The content is clipped
//! at the widget boundaries.

use std::any::TypeId;

use xilem::core::{MessageCtx, MessageResult, Mut, View, ViewId, ViewMarker, ViewPathTracker};
use xilem::masonry::accesskit::{Node, Role};
use xilem::masonry::core::{
    AccessCtx, AccessEvent, ChildrenIds, EventCtx, LayoutCtx, MeasureCtx, NewWidget, PaintCtx,
    PointerEvent, PropertiesMut, PropertiesRef, RegisterCtx, TextEvent, Update, UpdateCtx, Widget,
    WidgetId, WidgetMut, WidgetPod,
};
use xilem::masonry::imaging::Painter;
use xilem::masonry::kurbo::{Axis, Point, Rect, Size};
use xilem::masonry::layout::{LenReq, LayoutSize, SizeDef};
use xilem::{Pod, ViewCtx, WidgetView};
use tracing::{trace_span, Span};

const CHILD_VIEW_ID: ViewId = ViewId::new(0);

/// A wrapper widget that clips its content to its bounds.
///
/// This widget wraps a child and sets a clip path equal to its own
/// size, preventing any content from rendering outside its boundaries.
/// Useful for table cells where text should not overflow into adjacent cells.
pub struct ClippedWidget {
    child: WidgetPod<dyn Widget>,
    size: Size,
}

impl ClippedWidget {
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

impl Widget for ClippedWidget {
    type Action = ();

    fn on_pointer_event(
        &mut self,
        _ctx: &mut EventCtx<'_>,
        _props: &mut PropertiesMut<'_>,
        _event: &PointerEvent,
    ) {
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

    fn update(
        &mut self,
        _ctx: &mut UpdateCtx<'_>,
        _props: &mut PropertiesMut<'_>,
        _event: &Update,
    ) {
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
        let auto_length = len_req.into();
        let context_size = LayoutSize::maybe(axis.cross(), cross_length);
        ctx.compute_length(&mut self.child, auto_length, context_size, axis, cross_length)
    }

    fn layout(&mut self, ctx: &mut LayoutCtx<'_>, _props: &PropertiesRef<'_>, size: Size) {
        self.size = size;

        // Set clip path to widget bounds - this clips content to our size
        let clip_rect = Rect::from_origin_size(Point::ZERO, size);
        ctx.set_clip_path(clip_rect);

        let child_size = ctx.compute_size(&mut self.child, SizeDef::fit(size), size.into());
        ctx.run_layout(&mut self.child, child_size);
        ctx.place_child(&mut self.child, Point::ORIGIN);
        ctx.derive_baselines(&self.child);
    }

    fn paint(
        &mut self,
        _ctx: &mut PaintCtx<'_>,
        _props: &PropertiesRef<'_>,
        _painter: &mut Painter<'_>,
    ) {
        // Transparent - child paints itself, clipped by the clip path set in layout
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
        trace_span!("Clipped", id = id.trace())
    }
}

/// Xilem view for clipping content.
pub struct ClippedView<V> {
    child: V,
}

impl<V> ClippedView<V> {
    /// Creates a new clipped view wrapping the given child.
    pub fn new(child: V) -> Self {
        Self { child }
    }
}

/// Wrap a widget to clip its content to its bounds.
///
/// Use this when content may overflow its allocated space, such as
/// text in resizable table cells. The wrapper prevents content from
/// rendering outside the widget boundaries.
///
/// # Example
///
/// ```ignore
/// use xilem_extras::clipped;
///
/// // In a table cell:
/// clipped(
///     label(text)
///         .text_size(13.0)
///         .padding(4.0)
/// ).width(column_width.px())
/// ```
pub fn clipped<State: 'static, Action: 'static, V: WidgetView<State, Action>>(
    child: V,
) -> ClippedView<V> {
    ClippedView { child }
}

impl<V> ViewMarker for ClippedView<V> {}

impl<V, State, Action> View<State, Action, ViewCtx> for ClippedView<V>
where
    V: WidgetView<State, Action>,
    State: 'static,
    Action: 'static,
{
    type Element = Pod<ClippedWidget>;
    type ViewState = V::ViewState;

    fn build(&self, ctx: &mut ViewCtx, app_state: &mut State) -> (Self::Element, Self::ViewState) {
        let (child_pod, child_state) = ctx.with_id(CHILD_VIEW_ID, |ctx| self.child.build(ctx, app_state));
        let pod = ctx.with_action_widget(|ctx| ctx.create_pod(ClippedWidget::new(child_pod.new_widget)));
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
                ClippedWidget::child_mut(&mut element).downcast(),
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
            self.child
                .teardown(view_state, ctx, ClippedWidget::child_mut(&mut element).downcast());
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
                ClippedWidget::child_mut(&mut element).downcast(),
                app_state,
            ),
            _ => MessageResult::Stale,
        }
    }
}
