//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! Xilem view for the pulldown menu button.

use std::marker::PhantomData;

use xilem::core::{MessageCtx, MessageResult, Mut, View, ViewId, ViewMarker, ViewPathTracker};
use xilem::{Pod, ViewCtx, WidgetView};

use super::widget::{MenuButton, MenuButtonPress};

/// Randomly generated view ID for the child label.
const LABEL_VIEW_ID: ViewId = ViewId::new(0xa7f3_b0d1);

/// Xilem view that wraps a [`MenuButton`] widget.
///
/// Created via the [`menu_button`] function. The button shows a label
/// and opens a dropdown with the given item labels when clicked.
/// The callback receives the index of the selected item.
///
/// # Example
///
/// ```ignore
/// menu_button(
///     label("File"),
///     vec!["Import...", "Export...", "Exit"],
///     |state: &mut AppState, index: usize| {
///         match index {
///             0 => state.import(),
///             1 => state.export(),
///             2 => state.exit(),
///             _ => {}
///         }
///     },
/// )
/// ```
#[must_use = "View values do nothing unless provided to Xilem."]
pub struct MenuButtonView<State, Action, F, V> {
    label: V,
    items: Vec<String>,
    callback: F,
    phantom: PhantomData<fn(State) -> Action>,
}

/// Creates a menu button view.
///
/// - `label`: the always-visible label view (e.g. `label("File")`).
/// - `items`: the dropdown menu item labels.
/// - `callback`: called with `(state, selected_index)` when an item is picked.
///
/// # Example
///
/// ```ignore
/// use xilem::view::label;
/// use xilem_extras::menu_button;
///
/// // Create a File menu button
/// menu_button(
///     label("File"),
///     vec!["New".to_string(), "Open".to_string(), "Save".to_string()],
///     |state: &mut AppState, index: usize| {
///         match index {
///             0 => state.new_file(),
///             1 => state.open_file(),
///             2 => state.save_file(),
///             _ => {}
///         }
///     },
/// )
/// ```
pub fn menu_button<State, Action, V, F>(
    label: V,
    items: Vec<String>,
    callback: F,
) -> MenuButtonView<State, Action, F, V>
where
    State: 'static,
    Action: 'static,
    V: WidgetView<State, Action>,
    F: Fn(&mut State, usize) -> Action + Send + Sync + 'static,
{
    MenuButtonView {
        label,
        items,
        callback,
        phantom: PhantomData,
    }
}

impl<State, Action, F, V> ViewMarker for MenuButtonView<State, Action, F, V> {}

impl<State, Action, F, V> View<State, Action, ViewCtx> for MenuButtonView<State, Action, F, V>
where
    State: 'static,
    Action: 'static,
    V: WidgetView<State, Action>,
    F: Fn(&mut State, usize) -> Action + Send + Sync + 'static,
{
    type Element = Pod<MenuButton>;
    type ViewState = V::ViewState;

    fn build(
        &self,
        ctx: &mut ViewCtx,
        app_state: &mut State,
    ) -> (Self::Element, Self::ViewState) {
        let (child, child_state) = ctx.with_id(LABEL_VIEW_ID, |ctx| {
            self.label.build(ctx, app_state)
        });
        let pod = ctx.with_action_widget(|ctx| {
            ctx.create_pod(MenuButton::new(child.new_widget, self.items.clone()))
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
        if prev.items != self.items {
            MenuButton::set_items(&mut element, self.items.clone());
        }
        ctx.with_id(LABEL_VIEW_ID, |ctx| {
            self.label.rebuild(
                &prev.label,
                view_state,
                ctx,
                MenuButton::child_mut(&mut element).downcast(),
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
        ctx.with_id(LABEL_VIEW_ID, |ctx| {
            self.label.teardown(
                view_state,
                ctx,
                MenuButton::child_mut(&mut element).downcast(),
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
            Some(LABEL_VIEW_ID) => self.label.message(
                view_state,
                message,
                MenuButton::child_mut(&mut element).downcast(),
                app_state,
            ),
            None => match message.take_message::<MenuButtonPress>() {
                Some(press) => MessageResult::Action((self.callback)(app_state, press.index)),
                None => MessageResult::Stale,
            },
            _ => MessageResult::Stale,
        }
    }
}
