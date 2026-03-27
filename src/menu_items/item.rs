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
pub struct MenuItem<State, Action> {
    label: String,
    action: Arc<dyn Fn(&mut State) -> Action + Send + Sync>,
    _phantom: PhantomData<fn(&mut State) -> Action>,
}

impl<State, Action> Clone for MenuItem<State, Action> {
    fn clone(&self) -> Self {
        Self {
            label: self.label.clone(),
            action: Arc::clone(&self.action),
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
            _phantom: PhantomData,
        }
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

    fn build_widget(&self) -> NewWidget<dyn Widget> {
        NewWidget::new(PulldownMenuItem::new(self.label.clone())).erased()
    }

    fn execute(&self, state: &mut State) -> Option<Action> {
        Some((self.action)(state))
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
