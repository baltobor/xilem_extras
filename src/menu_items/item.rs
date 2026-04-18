//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! Clickable menu item with action callback.

use std::marker::PhantomData;
use std::sync::Arc;

use xilem::masonry::core::{NewWidget, Widget};

use super::entry::MenuEntry;
use crate::menu_button::PulldownMenuItem;

/// A clickable menu item with label and action callback.
///
/// Created via [`menu_item`]. When selected, executes the action callback
/// with mutable access to the application state.
///
/// The action is stored in an `Arc` to enable cloning, which is required
/// for the menu system to work with xilem's view diffing.
///
/// # Editable Items
///
/// Use `.is_editable(true)` for items that trigger inline editing (like Rename).
/// When selected, the action is called which should set appropriate state to
/// enable edit mode in the tree row builder.
///
/// ```ignore
/// menu_item("Rename", move |state: &mut AppState| {
///     state.editing_node = Some(node_id.clone());
/// }).is_editable(true)
/// ```
///
/// # Checkmark Items
///
/// Use `.checked(true)` to display a checkmark prefix for toggleable items.
///
/// ```ignore
/// menu_item("Dark Mode", move |state: &mut AppState| {
///     state.dark_mode = !state.dark_mode;
/// }).checked(state.dark_mode)
/// ```
pub struct MenuItem<State, Action> {
    label: String,
    action: Arc<dyn Fn(&mut State) -> Action + Send + Sync>,
    editable: bool,
    checked: Option<bool>,
    _phantom: PhantomData<fn(&mut State) -> Action>,
}

impl<State, Action> Clone for MenuItem<State, Action> {
    fn clone(&self) -> Self {
        Self {
            label: self.label.clone(),
            action: Arc::clone(&self.action),
            editable: self.editable,
            checked: self.checked,
            _phantom: PhantomData,
        }
    }
}

impl<State, Action> MenuItem<State, Action> {
    /// Creates a new menu item with the given label and action.
    pub fn new<F>(label: impl Into<String>, action: F) -> Self
    where
        F: Fn(&mut State) -> Action + Send + Sync + 'static,
    {
        Self {
            label: label.into(),
            action: Arc::new(action),
            editable: false,
            checked: None,
            _phantom: PhantomData,
        }
    }

    /// Sets whether this menu item triggers inline editing mode.
    ///
    /// When `true`, selecting this item should enable edit mode for the
    /// associated tree row (e.g., for rename operations). The action callback
    /// should set appropriate state to enable editing.
    ///
    /// # Example
    ///
    /// ```ignore
    /// menu_item("Rename", move |state: &mut AppState| {
    ///     state.editing_node = Some(node_id.clone());
    /// }).is_editable(true)
    /// ```
    pub fn is_editable(mut self, editable: bool) -> Self {
        self.editable = editable;
        self
    }

    /// Returns whether this menu item is editable.
    pub fn editable(&self) -> bool {
        self.editable
    }

    /// Sets whether this menu item displays a checkmark.
    ///
    /// When `Some(true)`, displays a checkmark prefix.
    /// When `Some(false)`, displays empty space (for alignment with checked items).
    /// When `None`, no checkmark area is shown (default).
    ///
    /// # Example
    ///
    /// ```ignore
    /// menu_item("Dark Mode", move |state: &mut AppState| {
    ///     state.dark_mode = !state.dark_mode;
    /// }).checked(state.dark_mode)
    /// ```
    pub fn checked(mut self, is_checked: bool) -> Self {
        self.checked = Some(is_checked);
        self
    }

    /// Returns the checked state of this menu item.
    pub fn is_checked(&self) -> Option<bool> {
        self.checked
    }
}

impl<State, Action> MenuEntry<State, Action> for MenuItem<State, Action>
where
    State: 'static,
    Action: 'static,
{
    fn label(&self) -> Option<&str> {
        Some(&self.label)
    }

    fn is_actionable(&self) -> bool {
        true
    }

    fn is_editable(&self) -> bool {
        self.editable
    }

    fn checked(&self) -> Option<bool> {
        self.checked
    }

    fn build_widget(&self) -> NewWidget<dyn Widget> {
        let mut item = PulldownMenuItem::new(self.label.clone());
        if let Some(checked) = self.checked {
            item = item.with_checked(Some(checked));
        }
        NewWidget::new(item).erased()
    }

    fn execute(&self, state: &mut State) -> Option<Action> {
        Some((self.action)(state))
    }

    fn clone_boxed(&self) -> super::BoxedMenuEntry<State, Action> {
        Box::new(self.clone())
    }
}

/// Creates a menu item with a label and action callback.
///
/// # Example
///
/// ```ignore
/// menu_item("Open", |state: &mut AppState| state.open())
/// menu_item("Save", |state| state.save())
/// ```
pub fn menu_item<State, Action, F>(
    label: impl Into<String>,
    action: F,
) -> MenuItem<State, Action>
where
    F: Fn(&mut State) -> Action + Send + Sync + 'static,
{
    MenuItem::new(label, action)
}
