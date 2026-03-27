//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! Xilem view for the dropdown select widget.

use std::marker::PhantomData;

use xilem::core::{MessageCtx, MessageResult, Mut, View, ViewMarker};
use xilem::{Pod, ViewCtx};

use super::widget::{DropdownSelect, DropdownSelectAction};

/// Xilem view that wraps a [`DropdownSelect`] widget.
///
/// Created via the [`dropdown_select`] function. The widget shows the
/// currently selected value and opens a dropdown with options when clicked.
///
/// # Example
///
/// ```ignore
/// dropdown_select(
///     model.selected_index,
///     vec!["Option A", "Option B", "Option C"],
///     |state: &mut AppState, value: &str, index: usize| {
///         state.selected_value = value.to_string();
///         state.selected_index = index;
///     },
/// )
/// ```
#[must_use = "View values do nothing unless provided to Xilem."]
pub struct DropdownSelectView<State, Action, F> {
    options: Vec<String>,
    selected_index: usize,
    callback: F,
    phantom: PhantomData<fn(State) -> Action>,
}

/// Creates a dropdown select view.
///
/// - `selected_index`: index of the currently selected option.
/// - `options`: the list of option labels.
/// - `callback`: called with `(state, selected_value, selected_index)` when an option is picked.
///
/// # Example
///
/// ```ignore
/// use xilem_extras::dropdown_select;
///
/// // Create a dropdown select with 3 options
/// dropdown_select(
///     model.selected_index,
///     vec!["Small".to_string(), "Medium".to_string(), "Large".to_string()],
///     |state: &mut AppState, value: &str, index: usize| {
///         state.size = value.to_string();
///         state.size_index = index;
///     },
/// )
/// ```
pub fn dropdown_select<State, Action, F>(
    selected_index: usize,
    options: Vec<String>,
    callback: F,
) -> DropdownSelectView<State, Action, F>
where
    State: 'static,
    Action: 'static,
    F: Fn(&mut State, &str, usize) -> Action + Send + Sync + 'static,
{
    DropdownSelectView {
        options,
        selected_index,
        callback,
        phantom: PhantomData,
    }
}

impl<State, Action, F> ViewMarker for DropdownSelectView<State, Action, F> {}

impl<State, Action, F> View<State, Action, ViewCtx> for DropdownSelectView<State, Action, F>
where
    State: 'static,
    Action: 'static,
    F: Fn(&mut State, &str, usize) -> Action + Send + Sync + 'static,
{
    type Element = Pod<DropdownSelect>;
    type ViewState = ();

    fn build(
        &self,
        ctx: &mut ViewCtx,
        _app_state: &mut State,
    ) -> (Self::Element, Self::ViewState) {
        let pod = ctx.with_action_widget(|ctx| {
            ctx.create_pod(DropdownSelect::new(self.options.clone(), self.selected_index))
        });
        (pod, ())
    }

    fn rebuild(
        &self,
        prev: &Self,
        _view_state: &mut Self::ViewState,
        _ctx: &mut ViewCtx,
        mut element: Mut<'_, Self::Element>,
        _app_state: &mut State,
    ) {
        if prev.options != self.options {
            DropdownSelect::set_options(&mut element, self.options.clone());
        }
        if prev.selected_index != self.selected_index {
            DropdownSelect::set_selected_index(&mut element, self.selected_index);
        }
    }

    fn teardown(
        &self,
        _view_state: &mut Self::ViewState,
        ctx: &mut ViewCtx,
        element: Mut<'_, Self::Element>,
    ) {
        ctx.teardown_action_source(element);
    }

    fn message(
        &self,
        _view_state: &mut Self::ViewState,
        message: &mut MessageCtx,
        _element: Mut<'_, Self::Element>,
        app_state: &mut State,
    ) -> MessageResult<Action> {
        match message.take_message::<DropdownSelectAction>() {
            Some(action) => {
                MessageResult::Action((self.callback)(app_state, &action.value, action.index))
            }
            None => MessageResult::Stale,
        }
    }
}
