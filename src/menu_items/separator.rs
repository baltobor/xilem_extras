//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! Visual separator for menu items.

use std::marker::PhantomData;

use xilem::masonry::core::{NewWidget, Widget};

use super::entry::MenuEntry;
use crate::menu_button::MenuSeparator as MenuSeparatorWidget;

/// A visual separator line in a menu.
///
/// Created via [`separator`]. Renders as a horizontal line between menu items.
/// Does not respond to clicks or trigger actions.
pub struct SeparatorEntry<State, Action> {
    _phantom: PhantomData<fn(&mut State) -> Action>,
}

impl<State, Action> Clone for SeparatorEntry<State, Action> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<State, Action> Copy for SeparatorEntry<State, Action> {}

impl<State, Action> Default for SeparatorEntry<State, Action> {
    fn default() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<State, Action> MenuEntry<State, Action> for SeparatorEntry<State, Action>
where
    State: 'static,
    Action: 'static,
{
    fn label(&self) -> Option<&str> {
        None
    }

    fn is_actionable(&self) -> bool {
        false
    }

    fn build_widget(&self) -> NewWidget<dyn Widget> {
        NewWidget::new(MenuSeparatorWidget::new()).erased()
    }

    fn execute(&self, _state: &mut State) -> Option<Action> {
        None
    }

    fn clone_boxed(&self) -> super::BoxedMenuEntry<State, Action> {
        Box::new(*self)
    }
}

/// Creates a visual separator line for menus.
///
/// # Example
///
/// ```ignore
/// (
///     menu_item("Cut", |s| s.cut()),
///     menu_item("Copy", |s| s.copy()),
///     separator(),
///     menu_item("Paste", |s| s.paste()),
/// )
/// ```
pub fn separator<State, Action>() -> SeparatorEntry<State, Action>
where
    State: 'static,
    Action: 'static,
{
    SeparatorEntry::default()
}
