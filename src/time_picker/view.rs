//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! Xilem view for the time picker widget.

use std::marker::PhantomData;

use xilem::core::{MessageCtx, MessageResult, Mut, View, ViewMarker};
use xilem::{Pod, ViewCtx};

use super::widget::{TimeAction, TimePickerWidget};

/// Xilem view that wraps a [`TimePickerWidget`].
///
/// Created via the [`time_picker`] function.
///
/// # Example
///
/// ```ignore
/// time_picker(
///     model.hour,
///     model.minute,
///     |state: &mut AppState, hour: u8, minute: u8| {
///         state.hour = hour;
///         state.minute = minute;
///     },
/// )
/// ```
#[must_use = "View values do nothing unless provided to Xilem."]
pub struct TimePickerView<State, Action, F> {
    hour: u8,
    minute: u8,
    callback: F,
    phantom: PhantomData<fn(State) -> Action>,
}

/// Creates a time picker view.
///
/// - `hour`: Current hour (0-23).
/// - `minute`: Current minute (0-59).
/// - `callback`: Called with `(state, hour, minute)` when time changes.
pub fn time_picker<State, Action, F>(
    hour: u8,
    minute: u8,
    callback: F,
) -> TimePickerView<State, Action, F>
where
    State: 'static,
    Action: 'static,
    F: Fn(&mut State, u8, u8) -> Action + Send + Sync + 'static,
{
    TimePickerView {
        hour,
        minute,
        callback,
        phantom: PhantomData,
    }
}

impl<State, Action, F> ViewMarker for TimePickerView<State, Action, F> {}

impl<State, Action, F> View<State, Action, ViewCtx> for TimePickerView<State, Action, F>
where
    State: 'static,
    Action: 'static,
    F: Fn(&mut State, u8, u8) -> Action + Send + Sync + 'static,
{
    type Element = Pod<TimePickerWidget>;
    type ViewState = ();

    fn build(
        &self,
        ctx: &mut ViewCtx,
        _app_state: &mut State,
    ) -> (Self::Element, Self::ViewState) {
        let pod = ctx.with_action_widget(|ctx| {
            ctx.create_pod(TimePickerWidget::new(self.hour, self.minute))
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
        if prev.hour != self.hour || prev.minute != self.minute {
            TimePickerWidget::set_time(&mut element, self.hour, self.minute);
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
        match message.take_message::<TimeAction>() {
            Some(boxed) => {
                MessageResult::Action((self.callback)(app_state, boxed.hour, boxed.minute))
            }
            None => MessageResult::Stale,
        }
    }
}
