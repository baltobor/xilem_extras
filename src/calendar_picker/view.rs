//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! Xilem view for the calendar picker widget.

use std::marker::PhantomData;

use chrono::NaiveDate;
use xilem::core::{MessageCtx, MessageResult, Mut, View, ViewMarker};
use xilem::{Pod, ViewCtx};

use super::widget::{CalendarAction, CalendarPickerWidget};

/// Xilem view that wraps a [`CalendarPickerWidget`].
///
/// Created via the [`calendar_picker`] function.
///
/// Note: This widget-based view does not render text.
/// For proper text rendering, build your calendar UI using
/// xilem's label and flex views directly.
///
/// # Example
///
/// ```ignore
/// calendar_picker(
///     model.selected_date,
///     |state: &mut AppState, date: NaiveDate| {
///         state.selected_date = Some(date);
///     },
/// )
/// ```
#[must_use = "View values do nothing unless provided to Xilem."]
pub struct CalendarPickerView<State, Action, F> {
    selected_date: Option<NaiveDate>,
    callback: F,
    phantom: PhantomData<fn(State) -> Action>,
}

/// Creates a calendar picker view (widget-based).
///
/// Note: This widget draws backgrounds and arrows but not text.
/// For proper text rendering, build your calendar UI using
/// xilem's label and flex views directly.
pub fn calendar_picker<State, Action, F>(
    selected_date: Option<NaiveDate>,
    callback: F,
) -> CalendarPickerView<State, Action, F>
where
    State: 'static,
    Action: 'static,
    F: Fn(&mut State, NaiveDate) -> Action + Send + Sync + 'static,
{
    CalendarPickerView {
        selected_date,
        callback,
        phantom: PhantomData,
    }
}

impl<State, Action, F> ViewMarker for CalendarPickerView<State, Action, F> {}

impl<State, Action, F> View<State, Action, ViewCtx> for CalendarPickerView<State, Action, F>
where
    State: 'static,
    Action: 'static,
    F: Fn(&mut State, NaiveDate) -> Action + Send + Sync + 'static,
{
    type Element = Pod<CalendarPickerWidget>;
    type ViewState = ();

    fn build(
        &self,
        ctx: &mut ViewCtx,
        _app_state: &mut State,
    ) -> (Self::Element, Self::ViewState) {
        let pod = ctx.with_action_widget(|ctx| {
            ctx.create_pod(CalendarPickerWidget::new(self.selected_date))
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
        if prev.selected_date != self.selected_date {
            CalendarPickerWidget::set_selected_date(&mut element, self.selected_date);
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
        match message.take_message::<CalendarAction>() {
            Some(boxed) => match *boxed {
                CalendarAction::DateSelected(date) => {
                    MessageResult::Action((self.callback)(app_state, date))
                }
                CalendarAction::MonthChanged(_) => {
                    MessageResult::Nop
                }
            },
            None => MessageResult::Stale,
        }
    }
}
