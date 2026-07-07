//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! Xilem view for the clipped widget.

use xilem::core::{MessageCtx, MessageResult, Mut, View, ViewId, ViewMarker, ViewPathTracker};
use xilem::{Pod, ViewCtx, WidgetView};

use crate::masonry::components::clipped::ClippedWidget;

const CHILD_VIEW_ID: ViewId = ViewId::new(0);

/// Xilem view for clipping content.
pub struct ClippedView<V> {
    child: V,
}

impl<V> ClippedView<V> {
    pub fn new(child: V) -> Self {
        Self { child }
    }
}

/// Wrap a widget to clip its content to its bounds.
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
        let (child_pod, child_state) =
            ctx.with_id(CHILD_VIEW_ID, |ctx| self.child.build(ctx, app_state));
        let pod =
            ctx.with_action_widget(|ctx| ctx.create_pod(ClippedWidget::new(child_pod.new_widget)));
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
            self.child.teardown(
                view_state,
                ctx,
                ClippedWidget::child_mut(&mut element).downcast(),
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
                ClippedWidget::child_mut(&mut element).downcast(),
                app_state,
            ),
            _ => MessageResult::Stale,
        }
    }
}
