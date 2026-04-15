//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! Xilem view for the pulldown menu button.

use std::marker::PhantomData;
use std::sync::Arc;

use xilem::core::{MessageCtx, MessageResult, Mut, View, ViewId, ViewMarker, ViewPathTracker};
use xilem::{Pod, ViewCtx, WidgetView};

use super::widget::{MenuButton, MenuButtonPress, MenuItemData};
use crate::menu_items::{BoxedMenuEntry, MenuItems};

/// Randomly generated view ID for the child label.
const LABEL_VIEW_ID: ViewId = ViewId::new(0xa7f3_b0d1);

/// Xilem view that wraps a [`MenuButton`] widget.
///
/// Created via the [`menu_button`] function. Each menu item carries its own
/// action callback, eliminating index matching errors.
///
/// # Example
///
/// ```ignore
/// use xilem_extras::{menu_button, menu_item, separator};
///
/// menu_button(
///     label("File"),
///     (
///         menu_item("New", |state: &mut AppState| state.new_file()),
///         menu_item("Open", |state| state.open_file()),
///         separator(),
///         menu_item("Exit", |state| state.exit()),
///     ),
/// )
/// ```
#[must_use = "View values do nothing unless provided to Xilem."]
pub struct MenuButtonView<State, Action, V, I> {
    label: V,
    items: I,
    phantom: PhantomData<fn(&mut State) -> Action>,
}

/// Creates a menu button view.
///
/// Each menu item carries its own action callback, eliminating index matching errors.
///
/// - `label`: the always-visible label view (e.g. `label("File")`).
/// - `items`: tuple of menu items created with [`menu_item`](crate::menu_item) and [`separator`](crate::separator).
///
/// # Example
///
/// ```ignore
/// use xilem::view::label;
/// use xilem_extras::{menu_button, menu_item, separator};
///
/// menu_button(
///     label("File"),
///     (
///         menu_item("New", |state: &mut AppState| state.new_file()),
///         menu_item("Open", |state| state.open_file()),
///         separator(),
///         menu_item("Exit", |state| state.exit()),
///     ),
/// )
/// ```
pub fn menu_button<State, Action, V, I>(
    label: V,
    items: I,
) -> MenuButtonView<State, Action, V, I>
where
    State: 'static,
    Action: 'static,
    V: WidgetView<State, Action>,
    I: MenuItems<State, Action>,
{
    MenuButtonView {
        label,
        items,
        phantom: PhantomData,
    }
}

/// View state for MenuButtonView, storing the collected menu entries.
pub struct MenuButtonViewState<State: 'static, Action: 'static, V: WidgetView<State, Action>> {
    label_state: V::ViewState,
    entries: Arc<Vec<BoxedMenuEntry<State, Action>>>,
}

impl<State, Action, V, I> ViewMarker for MenuButtonView<State, Action, V, I> {}

impl<State, Action, V, I> View<State, Action, ViewCtx> for MenuButtonView<State, Action, V, I>
where
    State: 'static,
    Action: 'static,
    V: WidgetView<State, Action>,
    I: MenuItems<State, Action> + Clone,
{
    type Element = Pod<MenuButton>;
    type ViewState = MenuButtonViewState<State, Action, V>;

    fn build(
        &self,
        ctx: &mut ViewCtx,
        app_state: &mut State,
    ) -> (Self::Element, Self::ViewState) {
        let entries: Vec<BoxedMenuEntry<State, Action>> = self.items.clone().collect_entries();

        // Convert entries to MenuItemData for the widget
        let item_data = Self::entries_to_item_data(&entries);

        let (child, label_state) = ctx.with_id(LABEL_VIEW_ID, |ctx| {
            self.label.build(ctx, app_state)
        });

        let pod = ctx.with_action_widget(|ctx| {
            ctx.create_pod(MenuButton::new_with_data(child.new_widget, item_data))
        });

        let view_state = MenuButtonViewState {
            label_state,
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

        // Always update item data - checkmarks may have changed
        let item_data = Self::entries_to_item_data(&entries);
        MenuButton::set_items_data(&mut element, item_data);
        view_state.entries = Arc::new(entries);

        ctx.with_id(LABEL_VIEW_ID, |ctx| {
            self.label.rebuild(
                &prev.label,
                &mut view_state.label_state,
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
                &mut view_state.label_state,
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
                &mut view_state.label_state,
                message,
                MenuButton::child_mut(&mut element).downcast(),
                app_state,
            ),
            None => match message.take_message::<MenuButtonPress>() {
                Some(press) => {
                    // Execute the action from the stored entry
                    if let Some(entry) = view_state.entries.get(press.index) {
                        if let Some(action) = entry.execute(app_state) {
                            return MessageResult::Action(action);
                        }
                    }
                    MessageResult::Nop
                }
                None => MessageResult::Stale,
            },
            _ => MessageResult::Stale,
        }
    }
}

// Helper functions for MenuButtonView
impl<State, Action, V, I> MenuButtonView<State, Action, V, I>
where
    State: 'static,
    Action: 'static,
{
    /// Convert MenuEntry items to MenuItemData for the widget.
    fn entries_to_item_data(entries: &[BoxedMenuEntry<State, Action>]) -> Vec<MenuItemData> {
        entries.iter().map(|entry| {
            if entry.is_submenu() {
                // Get submenu children
                let children = entry.submenu_items()
                    .map(|items| Self::entries_to_item_data(&items))
                    .unwrap_or_default();
                MenuItemData::Submenu {
                    label: entry.label().unwrap_or("").to_string(),
                    children,
                }
            } else if entry.is_actionable() {
                // Regular menu item
                if let Some(label) = entry.label() {
                    MenuItemData::Action {
                        label: label.to_string(),
                        checked: entry.checked(),
                    }
                } else {
                    MenuItemData::Separator
                }
            } else {
                // Separator
                MenuItemData::Separator
            }
        }).collect()
    }
}
