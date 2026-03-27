//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! Xilem view for the context menu widget.

use std::marker::PhantomData;
use std::sync::Arc;

use xilem::core::{MessageCtx, MessageResult, Mut, View, ViewId, ViewMarker, ViewPathTracker};
use xilem::{Pod, ViewCtx, WidgetView};

use super::widget::{ContextMenuAction, ContextMenuWidget};
use crate::menu_items::{BoxedMenuEntry, MenuItems};

/// Randomly generated view ID for the child content.
const CHILD_VIEW_ID: ViewId = ViewId::new(0xc0_7e_47_01);

/// Xilem view that wraps content and shows a context menu on right-click.
///
/// Created via the [`context_menu`] function. Each menu item carries its own
/// action callback, eliminating index matching errors.
///
/// # Example
///
/// ```ignore
/// use xilem_extras::{context_menu, menu_item, separator};
///
/// context_menu(
///     label("Right-click me"),
///     (
///         menu_item("Cut", |state: &mut AppState| state.cut()),
///         menu_item("Copy", |state| state.copy()),
///         separator(),
///         menu_item("Paste", |state| state.paste()),
///     ),
/// )
/// ```
#[must_use = "View values do nothing unless provided to Xilem."]
pub struct ContextMenuView<State, Action, V, I> {
    child: V,
    items: I,
    phantom: PhantomData<fn(&mut State) -> Action>,
}

/// Creates a context menu view that wraps content.
///
/// Each menu item carries its own action callback, eliminating index matching errors.
///
/// - `child`: the content view to wrap (e.g. a label, button, or any widget).
/// - `items`: tuple of menu items created with [`menu_item`](crate::menu_item) and [`separator`](crate::separator).
///
/// # Example
///
/// ```ignore
/// use xilem::view::label;
/// use xilem_extras::{context_menu, menu_item, separator};
///
/// context_menu(
///     label("Right-click me"),
///     (
///         menu_item("Open", |state: &mut AppState| state.open()),
///         menu_item("Delete", |state| state.delete()),
///         separator(),
///         menu_item("Rename", |state| state.rename()),
///     ),
/// )
/// ```
pub fn context_menu<State, Action, V, I>(
    child: V,
    items: I,
) -> ContextMenuView<State, Action, V, I>
where
    State: 'static,
    Action: 'static,
    V: WidgetView<State, Action>,
    I: MenuItems<State, Action>,
{
    ContextMenuView {
        child,
        items,
        phantom: PhantomData,
    }
}

/// View state for ContextMenuView, storing the collected menu entries.
pub struct ContextMenuViewState<State: 'static, Action: 'static, V: WidgetView<State, Action>> {
    child_state: V::ViewState,
    entries: Arc<Vec<BoxedMenuEntry<State, Action>>>,
}

impl<State, Action, V, I> ViewMarker for ContextMenuView<State, Action, V, I> {}

impl<State, Action, V, I> View<State, Action, ViewCtx> for ContextMenuView<State, Action, V, I>
where
    State: 'static,
    Action: 'static,
    V: WidgetView<State, Action>,
    I: MenuItems<State, Action> + Clone,
{
    type Element = Pod<ContextMenuWidget>;
    type ViewState = ContextMenuViewState<State, Action, V>;

    fn build(
        &self,
        ctx: &mut ViewCtx,
        app_state: &mut State,
    ) -> (Self::Element, Self::ViewState) {
        let entries: Vec<BoxedMenuEntry<State, Action>> = self.items.clone().collect_entries();

        // Build labels for the widget
        let labels: Vec<String> = entries
            .iter()
            .map(|e| e.label().unwrap_or("---").to_string())
            .collect();

        let (child, child_state) = ctx.with_id(CHILD_VIEW_ID, |ctx| {
            self.child.build(ctx, app_state)
        });

        let pod = ctx.with_action_widget(|ctx| {
            ctx.create_pod(ContextMenuWidget::new(child.new_widget, labels))
        });

        let view_state = ContextMenuViewState {
            child_state,
            entries: Arc::new(entries),
        };

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
        // Rebuild entries
        let entries: Vec<BoxedMenuEntry<State, Action>> = self.items.clone().collect_entries();
        let labels: Vec<String> = entries
            .iter()
            .map(|e| e.label().unwrap_or("---").to_string())
            .collect();

        let prev_labels: Vec<String> = view_state.entries
            .iter()
            .map(|e| e.label().unwrap_or("---").to_string())
            .collect();

        if prev_labels != labels {
            ContextMenuWidget::set_items(&mut element, labels);
        }
        view_state.entries = Arc::new(entries);

        ctx.with_id(CHILD_VIEW_ID, |ctx| {
            self.child.rebuild(
                &prev.child,
                &mut view_state.child_state,
                ctx,
                ContextMenuWidget::child_mut(&mut element).downcast(),
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
                ContextMenuWidget::child_mut(&mut element).downcast(),
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
                ContextMenuWidget::child_mut(&mut element).downcast(),
                app_state,
            ),
            None => match message.take_message::<ContextMenuAction>() {
                Some(boxed) => match *boxed {
                    ContextMenuAction::ItemSelected(index) => {
                        // Execute the action from the stored entry
                        if let Some(entry) = view_state.entries.get(index) {
                            if let Some(action) = entry.execute(app_state) {
                                return MessageResult::Action(action);
                            }
                        }
                        MessageResult::Nop
                    }
                },
                None => MessageResult::Stale,
            },
            _ => MessageResult::Stale,
        }
    }
}
