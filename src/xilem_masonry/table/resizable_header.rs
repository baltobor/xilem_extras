//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! Xilem view for the resizable header widget.

use std::sync::Arc;

use xilem::core::MessageResult;
use xilem::core::{MessageCtx, Mut, View, ViewId, ViewMarker, ViewPathTracker};
use xilem::{Pod, ViewCtx, WidgetView};

use crate::masonry::table::resizable_header::{ColumnResizeAction, ResizableHeader};

/// Xilem view for a resizable header row.
pub struct ResizableHeaderView<F, State, Action, V> {
    column_keys: Vec<Arc<str>>,
    column_widths: Vec<f64>,
    children: Vec<V>,
    callback: F,
    _phantom: std::marker::PhantomData<fn(&mut State) -> Action>,
}

/// Creates a resizable header view.
pub fn resizable_header<State: 'static, Action: 'static, V, F>(
    columns: &[(&str, f64)],
    children: Vec<V>,
    callback: F,
) -> ResizableHeaderView<F, State, Action, V>
where
    V: WidgetView<State, Action>,
    F: Fn(&mut State, Arc<str>, f64) -> Action + Send + Sync + 'static,
{
    ResizableHeaderView {
        column_keys: columns.iter().map(|(k, _)| Arc::from(*k)).collect(),
        column_widths: columns.iter().map(|(_, w)| *w).collect(),
        children,
        callback,
        _phantom: std::marker::PhantomData,
    }
}

impl<F, State, Action, V> ViewMarker for ResizableHeaderView<F, State, Action, V> {}

impl<F, State, Action, V> View<State, Action, ViewCtx> for ResizableHeaderView<F, State, Action, V>
where
    V: WidgetView<State, Action>,
    F: Fn(&mut State, Arc<str>, f64) -> Action + Send + Sync + 'static,
    State: 'static,
    Action: 'static,
{
    type Element = Pod<ResizableHeader>;
    type ViewState = Vec<V::ViewState>;

    fn build(&self, ctx: &mut ViewCtx, app_state: &mut State) -> (Self::Element, Self::ViewState) {
        let mut child_pods = Vec::new();
        let mut child_states = Vec::new();

        for (i, child) in self.children.iter().enumerate() {
            let (pod, state) =
                ctx.with_id(ViewId::new(i as u64), |ctx| child.build(ctx, app_state));
            child_pods.push(pod.new_widget.erased());
            child_states.push(state);
        }

        let pod = ctx.with_action_widget(|ctx| {
            let widget = ResizableHeader::new(
                child_pods,
                self.column_keys.clone(),
                self.column_widths.clone(),
            );
            ctx.create_pod(widget)
        });

        (pod, child_states)
    }

    fn rebuild(
        &self,
        prev: &Self,
        view_state: &mut Self::ViewState,
        ctx: &mut ViewCtx,
        mut element: Mut<'_, Self::Element>,
        app_state: &mut State,
    ) {
        if prev.column_widths != self.column_widths {
            ResizableHeader::set_column_widths(&mut element, self.column_widths.clone());
        }

        for (i, (child, prev_child)) in self.children.iter().zip(prev.children.iter()).enumerate() {
            if let Some(state) = view_state.get_mut(i) {
                if let Some(mut child_element) = ResizableHeader::child_mut(&mut element, i) {
                    ctx.with_id(ViewId::new(i as u64), |ctx| {
                        child.rebuild(prev_child, state, ctx, child_element.downcast(), app_state);
                    });
                }
            }
        }
    }

    fn teardown(
        &self,
        view_state: &mut Self::ViewState,
        ctx: &mut ViewCtx,
        mut element: Mut<'_, Self::Element>,
    ) {
        for (i, child) in self.children.iter().enumerate() {
            if let Some(state) = view_state.get_mut(i) {
                if let Some(mut child_element) = ResizableHeader::child_mut(&mut element, i) {
                    ctx.with_id(ViewId::new(i as u64), |ctx| {
                        child.teardown(state, ctx, child_element.downcast());
                    });
                }
            }
        }
    }

    fn message(
        &self,
        view_state: &mut Self::ViewState,
        message: &mut MessageCtx,
        mut element: Mut<'_, Self::Element>,
        app_state: &mut State,
    ) -> MessageResult<Action> {
        match message.take_first() {
            Some(id) => {
                let idx = id.routing_id() as usize;
                if let (Some(child), Some(state)) =
                    (self.children.get(idx), view_state.get_mut(idx))
                {
                    if let Some(mut child_element) = ResizableHeader::child_mut(&mut element, idx) {
                        return child.message(state, message, child_element.downcast(), app_state);
                    }
                }
                MessageResult::Stale
            }
            None => match message.take_message::<ColumnResizeAction>() {
                Some(action) => MessageResult::Action((self.callback)(
                    app_state,
                    action.column_key,
                    action.new_width,
                )),
                None => MessageResult::Stale,
            },
        }
    }
}
