//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! Xilem view for the sheet widget.

use std::marker::PhantomData;

use xilem::core::{MessageCtx, MessageResult, Mut, View, ViewId, ViewMarker, ViewPathTracker};
use xilem::{Pod, ViewCtx, WidgetView};

use super::widget::{SheetAction, SheetWidget};

/// Randomly generated view ID for the child content.
const CHILD_VIEW_ID: ViewId = ViewId::new(0x5e_e7_a0_01);

/// Xilem view that displays content in a modal sheet.
///
/// Created via the [`sheet`] function. The sheet appears as a floating layer
/// with a semi-transparent backdrop. Clicking the backdrop or pressing ESC
/// dismisses the sheet.
///
/// # Example
///
/// ```ignore
/// use xilem::view::{flex_col, label, button};
/// use xilem_extras::sheet;
///
/// if model.show_modal {
///     sheet(
///         flex_col((
///             label("Modal Title"),
///             label("Modal content here"),
///             button("Close", |state: &mut AppState| {
///                 state.show_modal = false;
///             }),
///         )),
///         |state: &mut AppState| {
///             state.show_modal = false;
///         },
///     )
/// }
/// ```
#[must_use = "View values do nothing unless provided to Xilem."]
pub struct SheetView<State, Action, V, F> {
    child: V,
    on_dismiss: F,
    phantom: PhantomData<fn(&mut State) -> Action>,
}

/// Creates a sheet view that displays content in a modal overlay.
///
/// The sheet appears as a floating layer with a semi-transparent backdrop.
/// Clicking the backdrop dismisses the sheet.
///
/// - `child`: the content view to display in the sheet.
/// - `on_dismiss`: callback invoked when the sheet is dismissed.
///
/// # Example
///
/// ```ignore
/// use xilem::view::{flex_col, label, button};
/// use xilem_extras::sheet;
///
/// sheet(
///     flex_col((
///         label("Are you sure?"),
///         button("Yes", |state: &mut AppState| state.confirm()),
///         button("No", |state: &mut AppState| state.cancel()),
///     )),
///     |state: &mut AppState| {
///         state.show_dialog = false;
///     },
/// )
/// ```
pub fn sheet<State, Action, V, F>(
    child: V,
    on_dismiss: F,
) -> SheetView<State, Action, V, F>
where
    State: 'static,
    Action: 'static,
    V: WidgetView<State, Action>,
    F: Fn(&mut State) -> Action + Send + Sync + 'static,
{
    SheetView {
        child,
        on_dismiss,
        phantom: PhantomData,
    }
}

/// View state for SheetView.
pub struct SheetViewState<State: 'static, Action: 'static, V: WidgetView<State, Action>> {
    child_state: V::ViewState,
}

impl<State, Action, V, F> ViewMarker for SheetView<State, Action, V, F> {}

impl<State, Action, V, F> View<State, Action, ViewCtx> for SheetView<State, Action, V, F>
where
    State: 'static,
    Action: 'static,
    V: WidgetView<State, Action>,
    F: Fn(&mut State) -> Action + Send + Sync + 'static,
{
    type Element = Pod<SheetWidget>;
    type ViewState = SheetViewState<State, Action, V>;

    fn build(
        &self,
        ctx: &mut ViewCtx,
        app_state: &mut State,
    ) -> (Self::Element, Self::ViewState) {
        let (child, child_state) = ctx.with_id(CHILD_VIEW_ID, |ctx| {
            self.child.build(ctx, app_state)
        });

        let pod = ctx.with_action_widget(|ctx| {
            ctx.create_pod(SheetWidget::new(child.new_widget, true))
        });

        let view_state = SheetViewState { child_state };

        (pod, view_state)
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
                &mut view_state.child_state,
                ctx,
                SheetWidget::child_mut(&mut element).downcast(),
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
                &mut view_state.child_state,
                ctx,
                SheetWidget::child_mut(&mut element).downcast(),
            );
        });
        ctx.teardown_action_source(element);
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
                &mut view_state.child_state,
                message,
                SheetWidget::child_mut(&mut element).downcast(),
                app_state,
            ),
            None => match message.take_message::<SheetAction>() {
                Some(boxed) => match *boxed {
                    SheetAction::Dismissed => {
                        MessageResult::Action((self.on_dismiss)(app_state))
                    }
                },
                None => MessageResult::Stale,
            },
            _ => MessageResult::Stale,
        }
    }
}
