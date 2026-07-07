//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! Xilem view for the group box widget.

use xilem::core::MessageResult;
use xilem::core::{MessageCtx, Mut, View, ViewId, ViewMarker, ViewPathTracker};
use xilem::masonry::peniko::Color;
use xilem::{Pod, ViewCtx, WidgetView};

use crate::masonry::components::group_box::{DEFAULT_BG, GroupBox};

const CHILD_VIEW_ID: ViewId = ViewId::new(0);

/// Xilem view for a [`GroupBox`].
pub struct GroupBoxView<V> {
    label: String,
    child: V,
    tint: Option<Color>,
}

/// Wrap a child in a labeled [`GroupBox`].
pub fn group_box<State, Action, V>(label: impl Into<String>, child: V) -> GroupBoxView<V>
where
    State: 'static,
    Action: 'static,
    V: WidgetView<State, Action>,
{
    GroupBoxView {
        label: label.into(),
        child,
        tint: None,
    }
}

impl<V> GroupBoxView<V> {
    pub fn tint(mut self, color: Color) -> Self {
        self.tint = Some(color);
        self
    }
}

impl<V> ViewMarker for GroupBoxView<V> {}

impl<V, State, Action> View<State, Action, ViewCtx> for GroupBoxView<V>
where
    V: WidgetView<State, Action>,
    State: 'static,
    Action: 'static,
{
    type Element = Pod<GroupBox>;
    type ViewState = V::ViewState;

    fn build(&self, ctx: &mut ViewCtx, app_state: &mut State) -> (Self::Element, Self::ViewState) {
        let (child_pod, child_state) =
            ctx.with_id(CHILD_VIEW_ID, |ctx| self.child.build(ctx, app_state));
        let pod = ctx.with_action_widget(|ctx| {
            let mut widget = GroupBox::new(self.label.clone(), child_pod.new_widget);
            if let Some(tint) = self.tint {
                widget = widget.with_tint(tint);
            }
            ctx.create_pod(widget)
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
        if prev.label != self.label {
            GroupBox::set_label(&mut element, self.label.clone());
        }
        if prev.tint != self.tint {
            GroupBox::set_tint(&mut element, self.tint.unwrap_or(DEFAULT_BG));
        }
        ctx.with_id(CHILD_VIEW_ID, |ctx| {
            self.child.rebuild(
                &prev.child,
                view_state,
                ctx,
                GroupBox::child_mut(&mut element).downcast(),
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
                GroupBox::child_mut(&mut element).downcast(),
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
                GroupBox::child_mut(&mut element).downcast(),
                app_state,
            ),
            _ => MessageResult::Stale,
        }
    }
}
